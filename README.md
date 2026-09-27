# ✦ Lumina

<p align="center">
  <strong>A beautiful, Steam-free Linux launcher experience for Zenless Zone Zero.</strong>
</p>

<p align="center">
  Lumina is an unofficial open-source Linux launcher that runs the <strong>official HoYoPlay launcher</strong> through a managed compatibility environment powered by UMU/Proton.
</p>

<p align="center">
  <a href="https://mixutin.github.io/Lumina/"><img alt="Website" src="https://img.shields.io/badge/Website-Lumina-8b5cf6?style=for-the-badge"></a>
  <a href="https://github.com/mixutin/Lumina/actions"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/mixutin/Lumina/ci.yml?style=for-the-badge&label=CI"></a>
  <a href="https://github.com/mixutin/Lumina/blob/main/LICENSE"><img alt="License" src="https://img.shields.io/github/license/mixutin/Lumina?style=for-the-badge"></a>
  <img alt="Linux" src="https://img.shields.io/badge/Linux-first-111827?style=for-the-badge&logo=linux&logoColor=white">
</p>

---

## 🌙 What is Lumina?

Lumina aims to make the official Zenless Zone Zero PC launcher feel at home on Linux.

Instead of shipping game files, replacing HoYoPlay, or patching anti-cheat components, Lumina manages the Linux compatibility layer around the official launcher:

- ✦ Managed UMU/Proton runtime
- ✦ Dedicated Wine prefix
- ✦ Official HoYoPlay installer flow
- ✦ GPU, Vulkan and runtime diagnostics
- ✦ Repairable launcher environment
- ✦ Human-readable launch logs
- ✦ Native-feeling Tauri desktop UI
- ✦ No Steam dependency

> **Project status:** pre-alpha. The UI and launcher architecture are being built now. Do not expect a stable daily-driver release yet.

## ⚡ Design goals

| Goal | Direction |
| --- | --- |
| **Steam-free** | HoYoPlay stays the source of truth for installation and updates. |
| **Safe by default** | Never patch, disable or bypass game anti-cheat systems. |
| **Distro friendly** | Keep host assumptions minimal and isolate runtime state. |
| **Recoverable** | Prefix/runtime problems should be diagnosable and repairable. |
| **Fast** | Native Rust backend with a lightweight Tauri frontend. |
| **Beautiful** | A polished neon-noir interface inspired by late-night city lights. |

## 🧱 Architecture

```text
Lumina
├── Tauri desktop shell
│   └── React + TypeScript UI
├── Rust launcher service
│   ├── system diagnostics
│   ├── prefix management
│   ├── HoYoPlay discovery
│   └── process/log management
└── Runtime layer
    └── UMU
        └── Proton
            └── official HoYoPlay
                └── Zenless Zone Zero
```

Lumina does **not** redistribute Zenless Zone Zero, HoYoPlay, Proton, or UMU.

## 🛣️ Roadmap

### Milestone 0 — Foundation
- [x] Project identity and repository
- [x] Tauri + React architecture
- [x] GitHub Pages foundation
- [ ] Local development bootstrap
- [x] Runtime state model

### Milestone 1 — First launch
- [ ] Detect host architecture and desktop session
- [ ] Detect Vulkan-capable GPU
- [ ] Detect/install UMU
- [ ] Create managed prefix
- [ ] Import or select the official HoYoPlay installer
- [ ] Launch HoYoPlay
- [ ] Capture logs and exit state

### Milestone 2 — Daily-driver basics
- [ ] Runtime repair
- [ ] Prefix backup/reset
- [ ] Installed-game detection
- [ ] Launch profiles
- [ ] GameMode integration
- [ ] NVIDIA / AMD / Intel diagnostics

### Milestone 3 — Distribution
- [ ] AppImage
- [ ] Flatpak
- [ ] .deb / .rpm
- [ ] Arch package
- [ ] Signed release artifacts
- [ ] Automatic update checks

## 🧑‍💻 Development

Prerequisites:

- Node.js 20+
- Rust stable
- Tauri 2 system dependencies for your distribution

```bash
git clone https://github.com/mixutin/Lumina.git
cd Lumina
npm install
npm run tauri dev
```

Frontend-only development:

```bash
npm run dev
```

## 🔐 Security philosophy

Lumina treats game and anti-cheat files as immutable external software.

The project will not include functionality intended to:

- disable or bypass anti-cheat;
- patch protected game binaries;
- spoof game services;
- distribute proprietary HoYoverse assets or executables.

Compatibility fixes belong in the runtime environment around the official software.

## 🤝 Contributing

Issues, testing reports and pull requests are welcome. During pre-alpha, reports are especially useful when they include:

- distribution and version;
- desktop session (X11/Wayland);
- GPU and driver;
- Lumina diagnostics output;
- relevant launcher logs with personal information removed.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow.

## ⚖️ Disclaimer

Lumina is an independent community project. It is not affiliated with, endorsed by, sponsored by, or otherwise associated with HoYoverse. Zenless Zone Zero, HoYoPlay and related names and marks belong to their respective owners.

## 📄 License

Lumina is released under the [MIT License](LICENSE).

---

<p align="center">
  <strong>Enter the city. Keep your Linux install.</strong><br>
  <sub>Built for Linux, around the official launcher.</sub>
</p>
