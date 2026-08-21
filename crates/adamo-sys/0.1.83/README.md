# adamo-sys

Raw FFI bindings to the [Adamo SDK](https://adamohq.com).

For a safe, idiomatic Rust API use the [`adamo`](https://docs.rs/adamo)
crate instead.

## Supported targets

- `aarch64-unknown-linux-gnu` — Ubuntu 22.04 arm64 / NVIDIA Jetson. The
  precompiled `libadamo.so.xz` ships inside the crate tarball and is
  unpacked during the Cargo build; no network access required (robot
  builds stay offline-friendly).
- `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` — the precompiled
  library is downloaded at build time from `install.adamohq.com`,
  pinned by version and sha256 in `prebuilt/remote.txt`. Requires
  `curl` on the build host. (crates.io caps packages at 10MB, so these
  can't ship in-crate.)

All prebuilts are video-enabled. For other targets, or for offline
builds on the download targets, use `ADAMO_LIB_DIR` (below).

## Local development

To build against a locally-compiled `libadamo` (from the in-tree
`adamo-c` crate, or any other source), point at it with
`ADAMO_LIB_DIR`:

```
ADAMO_LIB_DIR=/path/to/target/release cargo build
```

## License

Apache-2.0. The precompiled `libadamo` binary is proprietary and
covered by the Adamo SDK license.
