# Contributing to Lumina

Thanks for helping make Linux gaming a little less painful. ✦

## Before opening a PR

1. Search existing issues and pull requests.
2. Keep changes focused.
3. Do not add proprietary game assets, launcher binaries or credentials.
4. Do not add anti-cheat bypasses, game binary patches or service spoofing.
5. Test the frontend and Rust backend when your change touches both.

## Local setup

```bash
npm install
npm run dev
npm run tauri dev
```

## Commit style

Lumina uses short conventional-style commits:

```text
feat: add runtime diagnostics
fix: handle missing Vulkan tools
docs: document Fedora dependencies
chore: update CI
```

## Bug reports

Please include:

- distribution and version
- kernel version
- X11 or Wayland
- GPU and driver
- UMU/Proton version if known
- exact Lumina version/commit
- sanitized Lumina logs

Never post account tokens, cookies, private paths containing sensitive information, or other credentials.

## Compatibility work

Runtime compatibility changes should be:

- scoped to the compatibility environment;
- documented with the affected runtime/version;
- reversible;
- disabled by default when experimental.

Lumina does not modify protected game or anti-cheat files.

## Code style

- TypeScript: keep UI state typed and components small.
- Rust: return structured errors instead of panicking in command handlers.
- Shelling out: validate executable paths and pass arguments without invoking a shell.
- Logs: avoid secrets and redact user-sensitive values where practical.
