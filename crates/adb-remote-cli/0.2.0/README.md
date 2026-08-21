<div align="center">

# ADB Remote CLI

Rust TUI for controlling Android devices via ADB input keyevents. Allows connecting to the device on the local network on an exposed port.

[![Crates.io](https://img.shields.io/crates/v/adb-remote-cli?style=flat-square&color=orange)](https://crates.io/crates/adb-remote-cli)
[![Downloads](https://img.shields.io/crates/d/adb-remote-cli?style=flat-square&color=blue)](https://crates.io/crates/adb-remote-cli)
[![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/goodboyneon/adb-remote-cli/release.yml?style=flat-square&label=build)](https://github.com/yourusername/adb-remote/actions)
[![Rust](https://img.shields.io/badge/built%20with-Rust-b7410e?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey?style=flat-square)](#-installation)

</div>

## 📦 Installation

### Requirements

> ⚠️ **[Android Platform Tools (`adb`)](https://developer.android.com/tools/releases/platform-tools) must be installed and available on your `PATH`.** `adb-remote-cli` shells out to `adb` — it does not bundle it.

| Platform              | Install command                                                                                                  |
| --------------------- | ---------------------------------------------------------------------------------------------------------------- |
| macOS                 | `brew install android-platform-tools`                                                                            |
| Linux (Debian/Ubuntu) | `sudo apt install adb`                                                                                           |
| Linux (Arch)          | `sudo pacman -S android-tools`                                                                                   |
| Windows               | [Download platform-tools](https://developer.android.com/tools/releases/platform-tools) and add it to your `PATH` |

Verify it's installed:

```bash
adb version
```

### Option 1 — via Cargo (recommended)

```bash
cargo install adb-remote-cli
```

### Option 2 — download a prebuilt binary

Follow the instructions on the [**Releases**](https://github.com/goodboyneon/adb-remote-cli/releases) page.

### Option 3 — build from source

```bash
git clone https://github.com/goodboyneon/adb-remote-cli.git
cd adb-remote-cli
cargo build --release
./target/release/adb-remote-cli
```

---

## 🚀 Usage

Connect a device over Wi-Fi with `-c`, then launch:

```bash

# Wireless ADB
adb-remote-cli -c <IP:PORT>


### CLI options

| Flag                        | Description                                                      |
| --------------------------- | ---------------------------------------------------------------- |
| `-c`, `--connect <IP:PORT>` | Connect to a device over network ADB before launching            |
| `-h`, `--help`              | Print help                                                       |
| `-V`, `--version`           | Print version                                                    |

### Controls

<div align="center">
| Key | Action |
|---|---|
| `↑` `↓` `←` `→` | D-pad navigation |
| `Enter` | OK / Select |
| `Backspace` | Back |
| `h` | Home |
| `m` | Menu |
| `p` | Power |
| `+` / `-` | Volume up / down |
| `0` | Mute |
| `Space` | Play / Pause |
| `[` / `]` | Rewind / Fast-forward |
| `Esc` / `q` / `Ctrl+C` | Quit |

*Buttons are also clickable with the mouse.*

</div>
---

## 🛠️ How it works

`adb-remote-cli` renders a full-screen terminal UI ([crossterm](https://github.com/crossterm-rs/crossterm)) styled like a physical TV remote. Key presses and mouse clicks are mapped to [Android `KeyEvent`](https://developer.android.com/reference/android/view/KeyEvent) codes and sent through a single, persistent `adb shell` process via `input keyevent <code>` — avoiding the connection overhead of spawning a new `adb` process per button press.

A background thread polls `adb get-state` every couple of seconds to keep the connection indicator live without blocking input handling.

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!

1. Fork the repo
2. Create a branch (`git checkout -b feature/my-feature`)
3. Commit your changes
4. Open a pull request
---

## 📄 License

Distributed under the **MIT License**. See [`LICENSE`](LICENSE) for details.

---

<div align="center">
Made with 🦀 ;)

Consider ⭐ starring the repo!

</div>
```
