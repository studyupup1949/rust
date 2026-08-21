Anti-AFK application for **Sober** (Roblox on Linux). Fork of [AntiAFK-RBX-Sober](https://github.com/Agzes/AntiAFK-RBX-Sober) with **Niri** support.

Supports **Hyprland**, **KDE Plasma 6**, and **Niri**.

## Features

- Smart window management — switches focus, performs action, returns cursor
- AFK Actions — Jump (Space), Walk (W/S), Camera Zoom (I/O)
- Auto-Start/Stop — detects Sober automatically
- User-Safe — pauses on user activity
- CPU Quota Limiter — limits background CPU via `systemd CPUQuota`
- Auto Reconnect — clicks "Reconnect" on disconnect [BETA]
- Stealth Mode — hides game during actions (Hyprland, Niri)
- Tray icon control

## Installation

### Cargo
```bash
cargo install AntiAFK-RBX-Sober
```

### AppImage
Download from [Releases](https://github.com/aneek0/Sober-AntiAFK/releases):
```bash
chmod +x AntiAFK-RBX-Sober-*.AppImage
./AntiAFK-RBX-Sober-*.AppImage
```

### Build from source
```bash
git clone https://github.com/aneek0/Sober-AntiAFK.git
cd Sober-AntiAFK
cargo build --release
./target/release/AntiAFK-RBX-Sober
```

Build deps (Arch): `sudo pacman -S --needed base-devel gtk4 pkg-config`
Build deps (Ubuntu): `sudo apt install build-essential libgtk-4-dev libdbus-1-dev pkg-config`
Build deps (Fedora): `sudo dnf install gtk4-devel dbus-devel gcc pkg-config`

## Setup

### 1. uinput permissions
The app needs write access to `/dev/uinput`. Use the **FIX** button in Compatibility settings, or manually:
```bash
echo 'KERNEL=="uinput", MODE="0666"' | sudo tee /etc/udev/rules.d/99-uinput-antiafk.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

### 2. Register desktop entry
```bash
./target/release/AntiAFK-RBX-Sober --install
```

### 3. Runtime dependencies
- **General**: `pgrep`, `systemd`
- **Hyprland**: `grim`
- **KDE Plasma**: `spectacle`, `qdbus`/`qdbus6`
- **Niri**: `niri` (pre-installed)

## Supported environments
| Desktop | Mode | Status |
|---------|------|--------|
| Hyprland | Swapper | Full |
| KDE Plasma 6 | Plasma | Full |
| Niri | Niri | Full |

GNOME, X11, COSMIC — planned.

---

Fork of [Agzes/AntiAFK-RBX-Sober](https://github.com/Agzes/AntiAFK-RBX-Sober)
