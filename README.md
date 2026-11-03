<div align="center">

# term-kit

**A blazing-fast terminal UI for browsing, searching, and copying your shell command history.**

[![Latest Release](https://img.shields.io/github/v/release/matrixjuan748/term-kit?logo=github)](https://github.com/matrixjuan748/term-kit/releases/latest)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)](#-installation)

</div>

---
**term-kit** is a lightweight TUI (Terminal User Interface) application built with [ratatui](https://github.com/ratatui/ratatui) that lets you interactively explore your shell history — search through thousands of past commands, bookmark your favorites, and copy them to your clipboard in an instant.

```bash
┌───────────────────────────────────────────────────────────────────┐
│ History Finder v0.1 │ Mode: HISTORY │ [B]Toggle [/]Search [q]Quit │
├───────────────────────────────────────────────────────────────────┤
│ Command History                                                   │
│   1  cargo build --release                                        │
│   2  git rebase -i HEAD~3                                         │
│ ▸ 3  docker compose up -d --build                                 │
│   4  ffmpeg -i input.mp4 -c:v libx264 output.mkv                  │
│   5  rg "TODO" --type rust -l                                     │
├───────────────────────────────────────────────────────────────────┤
│ /docker                                                           │
├───────────────────────────────────────────────────────────────────┤
│ HISTORY  B Switch  b Bookmark  Copied: docker compose up -d...    │
└───────────────────────────────────────────────────────────────────┘
```

## Features

| Feature | Description |
|---|---|
| **Instant Search** | Filter thousands of history entries as you type |
| **One-key Copy** | Press `Enter` to copy any command straight to your clipboard |
| **Bookmarks** | Star frequently-used commands and keep them in a dedicated view |
| **Multi-shell** | Auto-detects and parses **Bash**, **Zsh**, **Fish**, and **PowerShell** history |
| **Cross-platform** | Native clipboard support on Linux (X11 / Wayland), macOS, and Windows |
| **Zero config** | Runs out of the box — no configuration files needed |
| **Clean TUI** | Smooth rendering powered by ratatui + crossterm |

## Installation

### Prebuilt binaries (recommended)

**Linux / macOS** — install via shell script:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/matrixjuan748/term-kit/releases/latest/download/term-kit-installer.sh | sh
```

**Windows** — install via PowerShell script:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/matrixjuan748/term-kit/releases/latest/download/term-kit-installer.ps1 | iex"
```

> The URLs above always point to the **latest release**. Need a pinned version? Grab it from the [Releases page](https://github.com/matrixjuan748/term-kit/releases).

### Direct download

| Platform | Architecture | Download |
|---|---|---|
| 🍎 macOS | Apple Silicon | [`term-kit-aarch64-apple-darwin.tar.xz`](https://github.com/matrixjuan748/term-kit/releases/latest/download/term-kit-aarch64-apple-darwin.tar.xz) |
| 🍎 macOS | Intel | [`term-kit-x86_64-apple-darwin.tar.xz`](https://github.com/matrixjuan748/term-kit/releases/latest/download/term-kit-x86_64-apple-darwin.tar.xz) |
| 🪟 Windows | x64 | [`term-kit-x86_64-pc-windows-msvc.zip`](https://github.com/matrixjuan748/term-kit/releases/latest/download/term-kit-x86_64-pc-windows-msvc.zip) |
| 🐧 Linux | ARM64 | [`term-kit-aarch64-unknown-linux-gnu.tar.xz`](https://github.com/matrixjuan748/term-kit/releases/latest/download/term-kit-aarch64-unknown-linux-gnu.tar.xz) |
| 🐧 Linux | x64 | [`term-kit-x86_64-unknown-linux-gnu.tar.xz`](https://github.com/matrixjuan748/term-kit/releases/latest/download/term-kit-x86_64-unknown-linux-gnu.tar.xz) |

### From source

```bash
git clone https://github.com/matrixjuan748/term-kit.git
cd term-kit
cargo install --path .
```

### Run

```bash
term-kit
```

That's it. term-kit will detect your current shell, load its history file, and drop you into the interactive UI.

## Keybindings

### General

| Key | Action |
|---|---|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Enter` | Copy the selected command to clipboard |
| `/` | Enter search mode |
| `b` | Toggle bookmark on the selected command |
| `B` | Switch between **History** ↔ **Bookmark** mode |
| `d` | Delete selected bookmark *(bookmark mode only)* |
| `h` | Show / hide the help overlay |
| `q` | Quit |

### Search Mode

| Key | Action |
|---|---|
| *any printable char* | Append to the search query |
| `Backspace` | Remove last character |
| `Esc` | Cancel search and restore full list |

## Supported Shells

term-kit reads the `$SHELL` environment variable (or defaults on Windows) to locate and parse the correct history file:

| Shell | History file |
|---|---|
| **Bash** | `~/.bash_history` |
| **Zsh** | `~/.zsh_history` |
| **Fish** | `~/.local/share/fish/fish_history` |
| **PowerShell** (Linux/macOS) | `~/.local/share/powershell/PSReadLine/ConsoleHost_history.txt` |
| **PowerShell** (Windows) | `%AppData%\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt` |

> Bookmarks are persisted as JSON in `~/.term_kit_bookmarks`.

## Building from Source

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) **≥ 1.75**
- On Linux (optional, for clipboard): `xclip` (X11) or `wl-clipboard` (Wayland)

### Build

```bash
cargo build --release
# binary → target/release/term-kit
```

### Run tests

```bash
cargo test
```

> Official release artifacts are built and published automatically via [cargo-dist](https://github.com/axodotdev/cargo-dist) (see `dist-workspace.toml`).

## Project Structure

```
src/
├── main.rs      # Entry point – terminal setup / teardown
├── app.rs       # Core state: history parsing, bookmarks, clipboard, search
├── events.rs    # Input handling & event loop
└── ui.rs        # All ratatui rendering (layout, widgets, help overlay)
```

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a PR.

1. Fork the repository
2. Create your feature branch (`git checkout -b feat/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feat/amazing-feature`)
5. Open a Pull Request

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).

## 📄 License

Distributed under the **Apache License 2.0** — see [LICENSE](LICENSE) for details.

---

<div align="center">

Built with 🦀 and [ratatui](https://github.com/ratatui/ratatui)

</div>

