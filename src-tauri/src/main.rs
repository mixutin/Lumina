#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    ffi::OsString,
    fs,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

const UMU_RELEASE_API: &str =
    "https://api.github.com/repos/Open-Wine-Components/umu-launcher/releases/latest";
const UMU_DOWNLOAD_PREFIX: &str =
    "https://github.com/Open-Wine-Components/umu-launcher/releases/download/";
const MAX_UMU_ARCHIVE_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeStatus {
    platform: String,
    arch: String,
    session: String,
    data_dir: String,
    umu_available: bool,
    umu_managed: bool,
    umu_version: Option<String>,
    python_version: Option<String>,
    python_compatible: bool,
    hoyoplay_installed: bool,
    vulkan_available: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapResult {
    version: String,
    path: String,
    updated: bool,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    digest: Option<String>,
    size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct UmuState {
    version: String,
    sha256: String,
    source_url: String,
}

fn home_dir() -> Result<PathBuf, String> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".to_string())
}

fn data_root() -> Result<PathBuf, String> {
    if let Some(base) = env::var_os("XDG_DATA_HOME") {
        return Ok(PathBuf::from(base).join("lumina"));
    }

    Ok(home_dir()?.join(".local/share/lumina"))
}

fn prefix_dir() -> Result<PathBuf, String> {
    Ok(data_root()?.join("prefix"))
}

fn managed_umu_path() -> Result<PathBuf, String> {
    Ok(data_root()?.join("runtime/umu/current/umu-run"))
}

fn umu_state_path() -> Result<PathBuf, String> {
    Ok(data_root()?.join("state/umu.json"))
}

fn executable_in_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
        .or_else(|| {
            home_dir()
                .ok()
                .map(|home| home.join(".local/bin").join(name))
                .filter(|candidate| candidate.is_file())
        })
}

fn umu_executable() -> Option<PathBuf> {
    managed_umu_path()
        .ok()
        .filter(|path| path.is_file())
        .or_else(|| executable_in_path("umu-run"))
}

fn read_umu_state() -> Option<UmuState> {
    let path = umu_state_path().ok()?;
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn python_version() -> Option<String> {
    let output = Command::new("python3").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let line = if stdout.is_empty() { stderr } else { stdout };

    line.strip_prefix("Python ")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn python_compatible(version: Option<&str>) -> bool {
    let Some(version) = version else {
        return false;
    };

    let mut parts = version.split('.');
    let major = parts.next().and_then(|part| part.parse::<u32>().ok());
    let minor = parts.next().and_then(|part| part.parse::<u32>().ok());

    matches!((major, minor), (Some(major), Some(minor)) if major > 3 || (major == 3 && minor >= 10))
}

fn directory_has_entries(path: &Path) -> bool {
    fs::read_dir(path)
        .ok()
        .and_then(|mut entries| entries.next())
        .is_some()
}

fn vulkan_available() -> bool {
    if let Some(vulkaninfo) = executable_in_path("vulkaninfo") {
        if Command::new(vulkaninfo)
            .arg("--summary")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
        {
            return true;
        }
    }

    [
        Path::new("/usr/share/vulkan/icd.d"),
        Path::new("/etc/vulkan/icd.d"),
    ]
    .iter()
    .any(|path| directory_has_entries(path))
}

fn is_hoyoplay_executable(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if file_name != "launcher.exe" && file_name != "hoyoplay.exe" {
        return false;
    }

    path.to_string_lossy()
        .to_ascii_lowercase()
        .contains("hoyoplay")
}

fn find_hoyoplay_in(directory: &Path, depth: usize) -> Option<PathBuf> {
    if depth == 0 || !directory.is_dir() {
        return None;
    }

    let entries = fs::read_dir(directory).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() && is_hoyoplay_executable(&path) {
            return Some(path);
        }

        if path.is_dir() {
            if let Some(found) = find_hoyoplay_in(&path, depth - 1) {
                return Some(found);
            }
        }
    }

    None
}

fn hoyoplay_executable() -> Option<PathBuf> {
    let prefix = prefix_dir().ok()?;
    let drive_c = prefix.join("drive_c");

    [
        drive_c.join("Program Files"),
        drive_c.join("Program Files (x86)"),
        drive_c.join("users"),
    ]
    .into_iter()
    .find_map(|root| find_hoyoplay_in(&root, 6))
}

fn ensure_layout_inner() -> Result<PathBuf, String> {
    let root = data_root()?;

    for dir in [
        "prefix",
        "runtime",
        "runtime/umu/current",
        "downloads",
        "logs",
        "state",
    ] {
        fs::create_dir_all(root.join(dir))
            .map_err(|error| format!("failed to create {dir}: {error}"))?;
    }

    Ok(root)
}

fn http_client() -> Result<Client, String> {
    Client::builder()
        .user_agent(format!("Lumina/{}", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(90))
        .build()
        .map_err(|error| format!("failed to initialize HTTP client: {error}"))
}

fn resolve_latest_umu(client: &Client) -> Result<(String, GithubAsset), String> {
    let release = client
        .get(UMU_RELEASE_API)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("failed to query the official UMU release: {error}"))?
        .json::<GithubRelease>()
        .map_err(|error| format!("failed to parse the official UMU release metadata: {error}"))?;

    if release.tag_name.is_empty()
        || !release
            .tag_name
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
    {
        return Err("official UMU release returned an unexpected version tag".to_string());
    }

    let expected_name = format!("umu-launcher-{}-zipapp.tar", release.tag_name);
    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == expected_name)
        .ok_or_else(|| format!("official UMU release is missing {expected_name}"))?;

    if !asset.browser_download_url.starts_with(UMU_DOWNLOAD_PREFIX) {
        return Err("official UMU release returned an unexpected download URL".to_string());
    }

    if asset.size == 0 || asset.size > MAX_UMU_ARCHIVE_BYTES {
        return Err(format!(
            "official UMU archive has an unexpected size: {} bytes",
            asset.size
        ));
    }

    Ok((release.tag_name, asset))
}

fn expected_sha256(asset: &GithubAsset) -> Result<String, String> {
    let digest = asset
        .digest
        .as_deref()
        .and_then(|value| value.strip_prefix("sha256:"))
        .ok_or_else(|| "official UMU release does not publish a SHA-256 digest".to_string())?;

    if digest.len() != 64 || !digest.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err("official UMU release returned an invalid SHA-256 digest".to_string());
    }

    Ok(digest.to_ascii_lowercase())
}

fn download_verified(
    client: &Client,
    asset: &GithubAsset,
    destination: &Path,
    expected_digest: &str,
) -> Result<(), String> {
    let mut response = client
        .get(&asset.browser_download_url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| format!("failed to download UMU from the official release: {error}"))?;

    if let Some(length) = response.content_length() {
        if length > MAX_UMU_ARCHIVE_BYTES {
            return Err("UMU download exceeded the maximum expected archive size".to_string());
        }
    }

    let mut file = File::create(destination)
        .map_err(|error| format!("failed to create UMU download file: {error}"))?;
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let read = response
            .read(&mut buffer)
            .map_err(|error| format!("failed while downloading UMU: {error}"))?;

        if read == 0 {
            break;
        }

        total += read as u64;
        if total > MAX_UMU_ARCHIVE_BYTES {
            return Err("UMU download exceeded the maximum expected archive size".to_string());
        }

        file.write_all(&buffer[..read])
            .map_err(|error| format!("failed to write UMU archive: {error}"))?;
        hasher.update(&buffer[..read]);
    }

    file.sync_all()
        .map_err(|error| format!("failed to flush UMU archive: {error}"))?;

    let actual = format!("{:x}", hasher.finalize());
    if actual != expected_digest {
        return Err(format!(
            "UMU archive integrity check failed: expected {expected_digest}, got {actual}"
        ));
    }

    Ok(())
}

fn extract_umu_run(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let archive_file =
        File::open(archive_path).map_err(|error| format!("failed to open UMU archive: {error}"))?;
    let mut archive = tar::Archive::new(archive_file);
    let entries = archive
        .entries()
        .map_err(|error| format!("failed to read UMU archive: {error}"))?;
    let mut found = false;

    for entry in entries {
        let mut entry = entry.map_err(|error| format!("failed to read UMU archive entry: {error}"))?;
        let path = entry
            .path()
            .map_err(|error| format!("failed to inspect UMU archive entry: {error}"))?;

        if path.file_name().and_then(|name| name.to_str()) != Some("umu-run") {
            continue;
        }

        if !entry.header().entry_type().is_file() {
            return Err("UMU archive contains a non-file umu-run entry".to_string());
        }

        if found {
            return Err("UMU archive contains multiple umu-run entries".to_string());
        }

        let mut output = File::create(destination)
            .map_err(|error| format!("failed to create managed umu-run: {error}"))?;
        std::io::copy(&mut entry, &mut output)
            .map_err(|error| format!("failed to extract managed umu-run: {error}"))?;
        output
            .sync_all()
            .map_err(|error| format!("failed to flush managed umu-run: {error}"))?;
        found = true;
    }

    if !found {
        return Err("UMU archive did not contain umu-run".to_string());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(destination, fs::Permissions::from_mode(0o755))
            .map_err(|error| format!("failed to mark managed umu-run executable: {error}"))?;
    }

    Ok(())
}

fn validate_umu(path: &Path) -> Result<(), String> {
    let output = Command::new(path)
        .arg("--help")
        .env("UMU_LOG", "0")
        .output()
        .map_err(|error| format!("failed to validate managed UMU: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "managed UMU failed its validation check: {}",
            stderr.trim()
        ));
    }

    Ok(())
}

fn write_umu_state(state: &UmuState) -> Result<(), String> {
    let path = umu_state_path()?;
    let temporary = path.with_extension("json.tmp");
    let content = serde_json::to_vec_pretty(state)
        .map_err(|error| format!("failed to serialize UMU state: {error}"))?;

    fs::write(&temporary, content)
        .map_err(|error| format!("failed to write UMU state: {error}"))?;
    fs::rename(&temporary, &path)
        .map_err(|error| format!("failed to activate UMU state: {error}"))?;

    Ok(())
}

#[tauri::command]
fn get_system_status() -> Result<RuntimeStatus, String> {
    let data = data_root()?;
    let managed_path = managed_umu_path()?;
    let state = read_umu_state();
    let python = python_version();

    Ok(RuntimeStatus {
        platform: format!("{} {}", env::consts::OS, env::consts::FAMILY),
        arch: env::consts::ARCH.to_string(),
        session: env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into()),
        data_dir: data.display().to_string(),
        umu_available: umu_executable().is_some(),
        umu_managed: managed_path.is_file(),
        umu_version: state.map(|state| state.version),
        python_compatible: python_compatible(python.as_deref()),
        python_version: python,
        hoyoplay_installed: hoyoplay_executable().is_some(),
        vulkan_available: vulkan_available(),
    })
}

#[tauri::command]
fn ensure_layout() -> Result<String, String> {
    let root = ensure_layout_inner()?;
    Ok(format!(
        "Lumina data directories are ready at {}",
        root.display()
    ))
}

#[tauri::command]
fn bootstrap_umu() -> Result<BootstrapResult, String> {
    let root = ensure_layout_inner()?;
    let python = python_version();

    if !python_compatible(python.as_deref()) {
        return Err(match python {
            Some(version) => format!(
                "UMU requires Python 3.10 or newer; this system reports Python {version}"
            ),
            None => "UMU requires Python 3.10 or newer, but python3 was not found".to_string(),
        });
    }

    let client = http_client()?;
    let (version, asset) = resolve_latest_umu(&client)?;
    let digest = expected_sha256(&asset)?;
    let target = managed_umu_path()?;

    if target.is_file() {
        if let Some(state) = read_umu_state() {
            if state.version == version && state.sha256 == digest {
                validate_umu(&target)?;
                return Ok(BootstrapResult {
                    version,
                    path: target.display().to_string(),
                    updated: false,
                });
            }
        }
    }

    let archive = root
        .join("downloads")
        .join(format!("umu-launcher-{version}-zipapp.tar.part"));
    let candidate = root
        .join("runtime/umu/current")
        .join(format!("umu-run.new-{}", std::process::id()));

    let result = (|| {
        download_verified(&client, &asset, &archive, &digest)?;
        extract_umu_run(&archive, &candidate)?;
        validate_umu(&candidate)?;

        fs::rename(&candidate, &target)
            .map_err(|error| format!("failed to activate managed UMU: {error}"))?;

        write_umu_state(&UmuState {
            version: version.clone(),
            sha256: digest,
            source_url: asset.browser_download_url,
        })?;

        Ok(BootstrapResult {
            version,
            path: target.display().to_string(),
            updated: true,
        })
    })();

    let _ = fs::remove_file(&archive);
    let _ = fs::remove_file(&candidate);

    result
}

fn log_file(name: &str) -> Result<fs::File, String> {
    let logs = data_root()?.join("logs");
    fs::create_dir_all(&logs).map_err(|error| error.to_string())?;

    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(logs.join(name))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn launch_hoyoplay() -> Result<String, String> {
    let umu = umu_executable()
        .ok_or_else(|| "UMU is not installed. Install the managed Lumina runtime first.".to_string())?;

    let launcher = hoyoplay_executable()
        .ok_or_else(|| "HoYoPlay was not found in Lumina's managed prefix".to_string())?;

    let prefix = prefix_dir()?;
    let proton: OsString = env::var_os("LUMINA_PROTON").unwrap_or_else(|| "GE-Proton".into());

    let stdout = log_file("hoyoplay.stdout.log")?;
    let stderr = log_file("hoyoplay.stderr.log")?;

    let child = Command::new(&umu)
        .arg(&launcher)
        .env("WINEPREFIX", &prefix)
        .env("GAMEID", "0")
        .env("PROTONPATH", proton)
        .env("UMU_LOG", "1")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|error| format!("failed to start UMU: {error}"))?;

    Ok(format!(
        "HoYoPlay started through UMU (process {}). Logs are being written to Lumina's log directory.",
        child.id()
    ))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_system_status,
            ensure_layout,
            bootstrap_umu,
            launch_hoyoplay
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lumina");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_supported_python_versions() {
        assert!(python_compatible(Some("3.10.0")));
        assert!(python_compatible(Some("3.12.8")));
        assert!(python_compatible(Some("4.0.0")));
    }

    #[test]
    fn rejects_old_or_invalid_python_versions() {
        assert!(!python_compatible(Some("3.9.19")));
        assert!(!python_compatible(Some("invalid")));
        assert!(!python_compatible(None));
    }

    #[test]
    fn validates_sha256_digest_shape() {
        let asset = GithubAsset {
            name: "umu.tar".to_string(),
            browser_download_url: "https://example.invalid/umu.tar".to_string(),
            digest: Some(format!("sha256:{}", "a".repeat(64))),
            size: 1,
        };

        assert_eq!(expected_sha256(&asset).unwrap(), "a".repeat(64));
    }
}
