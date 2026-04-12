# process-manager

[![CI](https://github.com/rustybyte-x/process-manager/actions/workflows/build-release.yml/badge.svg)](https://github.com/rustybyte-x/process-manager/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)

A modern terminal-based process manager written in Rust using Ratatui.

---

## ✨ Features

- 🔍 View running processes
- 🧭 Navigate using keyboard (vim-style + arrows)
- 🔎 Filter by process name or PID
- 📊 Sort by name, PID, CPU, or memory usage
- ❌ Kill processes with confirmation dialog
- 🚀 Start new processes via command input
- 🔄 Auto-refresh process list
- 📋 Detail panel for selected process
- 🎨 Colored CPU usage and process states

---

## 🖼️ Preview

> Screenshot or GIF coming soon

---

## 🏗️ Architecture

This project is structured into two crates:

- `process_manager_core` – core logic, state, controller
- `tui` – terminal UI (Ratatui)

---

## ⚙️ Requirements

- Rust (Edition 2024)
- Ratatui 0.30+
- Crossterm
- Sysinfo

---

## 🚀 Installation

### Install globally via Cargo

```
cargo install --path crates/tui
```

Binary will be installed to:

```
~/.cargo/bin
```

Make sure it is in your PATH.

---

## 📦 Download (Prebuilt Binaries)

Download from GitHub Releases:

- Windows x64 / x86
- macOS (Apple Silicon / Intel)

Each release contains ready-to-use ZIP files.

---

## ▶️ Usage

```
pm
```

---

## 🍎 macOS Security Notice

The macOS builds are currently **not code signed** and **not notarized**.

Because of this, macOS may block the binary.

### Run anyway

#### Option 1 (recommended)

1. Try to open the binary
2. Open **System Settings > Privacy & Security**
3. Click **Open Anyway**

#### Option 2 (terminal)

```
xattr -d com.apple.quarantine ./pm
chmod +x ./pm
./pm
```

Only bypass this warning if you trust the binary.

---

## 🛣️ Roadmap

- [ ] Code signing + notarization (macOS)
- [ ] GitHub Releases automation
- [ ] Process tree view
- [ ] CPU graphs
- [ ] Advanced filtering

---

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or pull requests.

---

## 📜 License

MIT OR Apache-2.0
