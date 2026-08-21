# adamo-sys

Raw FFI bindings to the [Adamo SDK](https://adamohq.com). The crate
ships a precompiled `libadamo.so.xz` inside the crate tarball and
unpacks it during the Cargo build; no network access is required at
build time.

For a safe, idiomatic Rust API use the [`adamo`](https://docs.rs/adamo)
crate instead.

## Supported targets

- `aarch64-unknown-linux-gnu` — Ubuntu 22.04 arm64 / NVIDIA Jetson.

More targets may be added later.

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
