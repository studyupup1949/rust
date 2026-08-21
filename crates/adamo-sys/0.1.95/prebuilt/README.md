# Prebuilt libadamo binaries

`build.rs` links a precompiled `libadamo` for the active `TARGET`. It
resolves one in this order: `$ADAMO_LIB_DIR` → `prebuilt/$TARGET/`
(in-crate) → a version+sha256-pinned download from
`install.adamohq.com` (listed in `remote.txt`).

All published prebuilts are **video-enabled** (built with GStreamer), so
the host needs the GStreamer runtime present to link and run — see
*Host prerequisites* below.

## Supported hosts

| Target triple                  | How it's obtained        | Notes |
|--------------------------------|--------------------------|-------|
| `aarch64-unknown-linux-gnu`    | in-crate `prebuilt/`     | Built for Ubuntu 22.04 arm64 / NVIDIA Jetson with `--features jetson`. Ships in the crate tarball, so robot builds stay offline. Links + runs on a generic (non-Jetson) arm64 host too — only the NVENC/`nvv4l2*` *video* paths need the Tegra/L4T libs at runtime; everything else works. |
| `x86_64-unknown-linux-gnu`     | download (`remote.txt`)  | Built on Ubuntu 22.04 (glibc 2.35). |
| `aarch64-apple-darwin`         | download (`remote.txt`)  | Built on macOS 14 (Apple Silicon). |

Only the in-crate aarch64 lib ships inside the package — crates.io caps
packages at 10MB, which fits exactly one library. The desktop targets are
downloaded at build time and verified against `remote.txt`.

Not provided: `x86_64-apple-darwin` (Intel Mac) and Windows. Use
`ADAMO_LIB_DIR` on those, or see *Adding a target* below.

## Host prerequisites

The prebuilts dynamically link GStreamer (`libgstreamer-1.0`,
`-base`/`-app`/`-video`, `glib`/`gobject`/`gio`). Install it before
building anything that depends on `adamo-sys`:

- **Debian/Ubuntu:** `libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libv4l-dev`
  (the `-dev` packages for building; the runtime `libgstreamer1.0-0` etc.
  are pulled in as deps and must also be present where the binary runs).
- **macOS (Apple Silicon):** `brew install gstreamer glib`. The dylib
  records absolute `/opt/homebrew/opt/...` load paths, so GStreamer must
  live at Homebrew's default prefix.

The download path also needs `curl` on the build host. For offline or
out-of-tree builds, set `ADAMO_LIB_DIR=/path/to/libs` pointing at a
directory containing `libadamo.so` (or `libadamo.dylib`).

## How the pin stays current

`remote.txt` is **auto-maintained by CI**: the *Pin remote libadamo
artifacts* step in `.github/workflows/publish-crates.yml` runs
`scripts/pin-remote.sh <version>` on every `v*.*.*` tag and packages
adamo-sys with `--allow-dirty`, so the published crate always pins
libadamo for its own release. To regenerate by hand:

```
adamo-rs/scripts/pin-remote.sh <version>
```

To stage the in-crate aarch64 binary before publishing, run
`scripts/stage-prebuilt.sh` from the `adamo-rs/` workspace root — it
builds, strips, and xz-compresses the library so the crate stays under
the crates.io limit.

## Adding a target

1. Have `release.yml`'s `c-sdk` job build and upload
   `libadamo-<triple>.{so,dylib}.xz` to `s3://adamo-downloads/sdk/v<VER>/`
   (add a matrix entry + a case in *Upload Rust SDK sys artifact*).
2. Add the triple to `pin-remote.sh`'s default `TARGETS` so CI pins it.
3. Document its host prerequisites above.
