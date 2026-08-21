# Prebuilt libadamo binaries

Each supported target triple gets its own subdirectory. `build.rs`
picks `prebuilt/$TARGET/` based on the active `TARGET` env var.

Currently supported:

```
prebuilt/
└── aarch64-unknown-linux-gnu/
    └── libadamo.so.xz
```

(Built for Ubuntu 22.04 arm64 / Jetson.)

Desktop targets (`x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`)
don't ship in-crate — the 10MB crates.io package cap only fits one
library. They are downloaded at build time instead, pinned by version
and sha256 in `remote.txt` (maintained by `scripts/pin-remote.sh`).

To stage the in-crate binary before publishing, run
`scripts/stage-prebuilt.sh` from the `adamo-rs/` workspace root. The
script strips and xz-compresses the shared library so the crate stays
below the crates.io package limit.

For offline or out-of-tree builds, set `ADAMO_LIB_DIR=/path/to/libs`
to point at a directory containing `libadamo.so` (or `libadamo.dylib`
on macOS hosts) directly.
