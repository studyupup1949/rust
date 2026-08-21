# Prebuilt libadamo binaries

Each supported target triple gets its own subdirectory. `build.rs`
picks `prebuilt/$TARGET/` based on the active `TARGET` env var.

Currently supported:

```
prebuilt/
└── aarch64-unknown-linux-gnu/
    └── libadamo.so
```

(Built for Ubuntu 22.04 arm64 / Jetson.)

To stage binaries before publishing, run `scripts/stage-prebuilt.sh`
from the `adamo-rs/` workspace root.

For offline or out-of-tree builds, set `ADAMO_LIB_DIR=/path/to/libs`
to point at a directory containing `libadamo.so` (or `libadamo.dylib`
on macOS hosts) directly.
