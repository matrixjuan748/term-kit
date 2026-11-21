<div align="center">

# term-kit

**A blazing-fast terminal UI for browsing, searching, and copying your shell command history.**

[![Latest Release](https://img.shields.io/github/v/release/matrixjuan748/term-kit?logo=github)](https://github.com/matrixjuan748/term-kit/releases/latest)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)](#-installation)

</div>

---
**term-kit** is a lightweight TUI (Terminal User Interface) application built with [ratatui](https://github.com/ratatui/ratatui) that lets you interactively explore your shell history — search through thousands of past commands, bookmark your favorites, and copy them to your clipboard in an instant.

```text
┌────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                   History Finder v0.1.105 | Bash | [B]Toggle | [/]Search | [h]Help | [q]Quit                   │
└────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
┌─ Command History (Press B to switch) ──────────────────────────────────────────────────────────────────────────┐
│   1  cargo build --release                                                                                     │
│   2  git rebase -i HEAD~3                                                                                      │
│   3  docker compose up -d --build                                                                              │
│   4  ffmpeg -i input.mp4 -c:v libx264 output.mkv                                                               │
│   5  rg "TODO" --type rust -l                                                                                  │
└────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
┌─ Search ───────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ /docker                                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
 HISTORY  B Switch  b Bookmark Copied: docker compose up -d --build
```

## Features

| Feature | Description |
|---|---|
| **Instant Search** | Case-insensitive filtering as you type, up to 5,000 entries |
| **One-key Copy** | Press `Enter` to copy any command straight to your clipboard |
| **Bookmarks** | Star frequently-used commands and keep them in a dedicated view |
| **Multi-shell** | Reads **Bash**, **Zsh**, **Fish**, and **PowerShell** history — including **Git Bash** on Windows |
| **Picks the right file** | Follows `$SHELL`, falls back to the most recently written history file, and shows which one it loaded |
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
| `PgUp` / `PgDn` | Move one screen |
| `Home` / `End` | Jump to first / last entry |
| `Enter` | Copy the selected command to clipboard |
| `/` | Enter search mode |
| `b` | Toggle bookmark on the selected command |
| `B` | Switch between **History** ↔ **Bookmark** mode |
| `d` | Delete selected bookmark *(bookmark mode only)* |
| `h` / `F1` | Show the help overlay |
| `Esc` | Close help / leave bookmark mode |
| `q` / `Ctrl+C` | Quit |

### Search Mode

| Key | Action |
|---|---|
| *any printable char* | Append to the search query (case-insensitive) |
| `Backspace` | Remove last character |
| `Enter` | Copy the selection and leave search mode |
| `Esc` | Leave search mode — the cursor stays on the command you were looking at |
| `↑` / `↓` | Move selection while the filter stays active |

## Supported Shells

term-kit reads the `$SHELL` environment variable to locate and parse the correct history file. If that file doesn't exist or is empty (for example, you launched it from **cmd** but only use **Git Bash**), it falls back to the most recently written history file and tells you which one it loaded in the status bar.

| Shell | History file |
|---|---|
| **Bash** | `~/.bash_history` |
| **Zsh** | `~/.zsh_history` |
| **Fish** | `~/.local/share/fish/fish_history` |
| **PowerShell** (Linux/macOS) | `~/.local/share/powershell/PSReadLine/ConsoleHost_history.txt` |
| **PowerShell** (Windows) | `%APPDATA%\Microsoft\Windows\PowerShell\PSReadLine\ConsoleHost_history.txt` |

> Bookmarks are persisted as JSON in `~/.term_kit_bookmarks`. If the home directory isn't writable (read-only home, sandboxed shell), it silently falls back to the platform data directory and then the temp directory. A corrupted file is moved to `~/.term_kit_bookmarks.corrupt` instead of being silently overwritten.

## Building from Source

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) **≥ 1.88** (required by ratatui 0.30)
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
