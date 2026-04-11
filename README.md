# process-manager

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

## 🏗️ Architecture

This project is structured into two crates:

- core
- tui

Core contains logic, TUI handles rendering.

---

## ⚙️ Requirements

- Rust (Edition 2024)
- Ratatui 0.30+
- Crossterm
- Sysinfo

---

## 🚀 Installation (CLI)

### Install globally via Cargo

```bash
cargo install --path crates/tui
```

This installs the binary into:

```
~/.cargo/bin
```

Make sure this path is in your PATH environment variable.

---

### Windows PATH setup

1. Add:

```
C:\Users\YOUR_NAME\.cargo\bin
```

to your PATH.

2. Restart terminal

---

## ▶️ Usage

```bash
pm
```

---

## 🧠 Notes

- Requires Rust Edition 2024 (for let-chains)
- Uses Ratatui 0.30 APIs

---

## 📜 License

MIT OR Apache-2.0
