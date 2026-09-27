#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeStatus {
    platform: String,
    arch: String,
    session: String,
    data_dir: String,
    umu_available: bool,
    hoyoplay_installed: bool,
    vulkan_available: bool,
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

#[tauri::command]
fn get_system_status() -> Result<RuntimeStatus, String> {
    let data = data_root()?;

    Ok(RuntimeStatus {
        platform: format!("{} {}", env::consts::OS, env::consts::FAMILY),
        arch: env::consts::ARCH.to_string(),
        session: env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unknown".into()),
        data_dir: data.display().to_string(),
        umu_available: executable_in_path("umu-run").is_some(),
        hoyoplay_installed: hoyoplay_executable().is_some(),
        vulkan_available: vulkan_available(),
    })
}

#[tauri::command]
fn ensure_layout() -> Result<String, String> {
    let root = data_root()?;

    for dir in ["prefix", "runtime", "downloads", "logs", "state"] {
        fs::create_dir_all(root.join(dir))
            .map_err(|error| format!("failed to create {dir}: {error}"))?;
    }

    Ok(format!("Lumina data directories are ready at {}", root.display()))
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
    let umu = executable_in_path("umu-run")
        .ok_or_else(|| "umu-run is not installed or is not available in PATH".to_string())?;

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
            launch_hoyoplay
        ])
        .run(tauri::generate_context!())
        .expect("error while running Lumina");
}
