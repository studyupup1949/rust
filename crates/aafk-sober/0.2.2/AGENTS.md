# AGENTS.md

## Project

Sober-AntiAFK — fork of AntiAFK-RBX-Sober. Anti-AFK application for Roblox (Sober) on Linux.
Written in Rust with GTK4. Supports Hyprland, KDE Plasma 6, and Niri.
Repo: https://github.com/aneek0/Sober-AntiAFK

## Build & Run

```bash
cargo build --release
./target/release/AntiAFK-RBX-Sober
```

No test suite is present in this project.

## Project Structure

- `src/main.rs` — entry point, CLI arg parsing, desktop detection, main loop
- `src/ui.rs` — GTK4 UI: settings window, tray, compatibility checks
- `src/backend.rs` — core logic: process detection, focus management, window hiding, input dispatch
- `src/state.rs` — app state: save/load config, desktop environment detection (`is_hyprland`, `is_plasma`, `is_niri`)
- `src/input.rs` — low-level input simulation via evdev + `/dev/uinput`
- `src/inputs/` — per-desktop input strategies:
  - `swapper.rs` — Hyprland-only "Swapper" mode (uses hyprctl)
  - `plasma.rs` — KDE Plasma mode (uses qdbus + KWin scripting)
  - `niri.rs` — Niri mode (uses `niri msg` CLI)

## Conventions

- Desktop detection uses `XDG_CURRENT_DESKTOP` env var (state.rs)
- Hyprland also detected via `HYPRLAND_INSTANCE_SIGNATURE`
- UI strings referencing supported desktops must list all three: Hyprland, KDE Plasma 6, Niri
- The `version` file and `Cargo.toml` [package] version must stay in sync
- No comments in code unless explicitly requested
