# aaai-gui

Desktop GUI for **aaai** (audit for asset integrity), built with [iced](https://github.com/iced-rs/iced).

```sh
aaai-gui
```

## Features

- **3-pane resizable layout** — file tree / diff viewer / inspector
- **Bottom action bar** — "Approve & Save" as the primary action; current file and unresolved count always visible
- **Diff view tabs** — Side by side / Unified / Changes only
- **Reason textarea** — multi-line text editor for the reason field (≈ 4 lines)
- **LineMatch colour blocks** — rules shown as red/green code blocks; click to edit
- **ABDD status icons** — status (✓ ⚠ ✗ ! —) in the file tree, diff-type tag on the right
- **Dark & light theme** — toggle in the footer, persisted to `~/.aaai/prefs.yaml`
- **Directory collapse** — fold/unfold directory entries in the file tree
- **Dashboard** — summary cards (OK / Pending / Failed / Error) before selecting a file
- **Keyboard shortcuts** — `Ctrl+S` save · `Ctrl+R` re-run · `Ctrl+Z` undo · `Ctrl+E` report · `/` search · `Enter` focus reason · `↑↓` navigate
- **Advisory warnings** — `AuditWarning` badges in the file tree and inspector
- **Profile manager** — save/load before+after+definition path combinations

## Full Documentation

- [GUI Guide](https://github.com/nabbisen/aaai/blob/main/docs/src/gui.md)
- [Getting Started](https://github.com/nabbisen/aaai/blob/main/docs/src/getting-started.md)
