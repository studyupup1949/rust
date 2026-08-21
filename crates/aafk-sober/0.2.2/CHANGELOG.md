# Changelog

## [0.2.2] Niri Pin Window + Rename - 2026-07-12

### Added
- **Niri: Pin Window** setting — adds a Niri window rule to open tiled with fixed size (467×690).
- Niri window rule file: `~/.config/niri/config.d/antiafk-rbx-sober.kdl`.

### Changed
- Package renamed to `aafk-sober` on crates.io.

---

## [0.2.1] Niri Support - 2026-07-11

### Added
- **Niri Support**: Full support for Niri compositor.
  - New input method: **"Niri"** using `niri msg` CLI for window and workspace management.
  - Automatic desktop environment detection extended to Niri.

### Changed
- **Renamed**: "FPS Capper" → "CPU Quota Limiter" (reflects actual functionality).
- Updated UI strings and documentation to reflect Niri as a supported desktop.

---

## [0.2.0] KDE Plasma 6 Support - 2026-03-28

### Added
- **KDE Plasma 6 (Wayland) Support**: Full support for KDE Plasma 6 environment.
  - New input method: **"Plasma (preview)"** using `qdbus6` and `KWin` for window management.
  - Non-interactive focus detection for Plasma using KWin scripting and journalctl.
  - Automatic desktop environment detection (Hyprland & Plasma).
- **UI Enhancements & Reliability**:
  - **Custom Icon Fallback**: New `get_safe_icon` system ensures icons display correctly even if standard symbolic icons are missing in the current theme.
  - **Theme fix & update**: Now fully using system gtk theme. 
  - **Styles update**: Updated styling for a more nice look.

### Changed
- **Major Architectural Refactoring**: 
  - Moved input simulation logic to a new module structure (`src/inputs/swapper.rs`, `src/inputs/plasma.rs`).
  - Decoupled `backend.rs` from specific input implementations for better maintainability.
- **Enhanced Process & Window Detection**: 
  - Improved reliability of identifying Sober windows.

*And other minor fixes...*

---

## [0.1.0] Init Release - 2026-03-15
init release :D