# aaai-gui

Desktop GUI for **aaai** (audit for asset integrity), built with [iced](https://github.com/iced-rs/iced).

```sh
aaai-gui
```

## Features

- **3-pane resizable layout** — file tree / diff viewer / inspector
- **Dark & light theme** — toggle in the footer, persisted to `~/.aaai/prefs.yaml`
- **Directory collapse** — fold/unfold directory entries in the file tree
- **Dashboard** — summary cards (OK / Pending / Failed / Error) before selecting a file
- **Keyboard shortcuts** — `Ctrl+S` save · `Ctrl+R` re-run · `Ctrl+Z` undo · `↑↓` navigate
- **Advisory warnings** — `AuditWarning` badges in the file tree and inspector
- **Profile manager** — save/load before+after+definition path combinations

## Full Documentation

- [GUI Guide](https://github.com/nabbisen/aaai/blob/main/docs/src/gui.md)
- [Getting Started](https://github.com/nabbisen/aaai/blob/main/docs/src/getting-started.md)
