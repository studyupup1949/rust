# adbutils (Rust)

Async Rust client for the Android Debug Bridge (adb) **server smartsocket
protocol**. A port of [adbutils-python](https://github.com/openatx/adbutils),
built to underpin a Rust port of uiautomator2.

Talks to the local `adb server` (`127.0.0.1:5037`) directly over the wire
protocol; the only time it shells out to an `adb` binary is `start-server`.

## Usage

```rust
#[tokio::main]
async fn main() -> adbutils::Result<()> {
    let adb = adbutils::adb();                 // 127.0.0.1:5037
    println!("server v{}", adb.server_version().await?);
    for d in adb.device_list().await? {
        println!("{}: {}", d.serial().unwrap(), d.shell("getprop ro.product.model").await?);
        let img = d.screenshot(None, true).await?;
        img.save("shot.png").unwrap();
    }
    Ok(())
}
```

## What's implemented

- **Transport** (`conn`, `client`): smartsocket framing (4-hex length + OKAY/FAIL),
  `host:*` commands, `devices`/`devices-l`, `connect`/`disconnect`, `wait-for`,
  `track-devices` (as a `Stream`), auto `start-server`.
- **Device** (`device`): transport switch (incl. `host:tport:serial:` + discard-8
  on adb ≥ 41), `shell`, `shell2` (v1 `X4EXIT` + v2 5-byte framing), forward,
  reverse (double-OKAY), `create_connection`, `root`, `tcpip`, `framebuffer`.
- **Sync** (`sync`): `STAT`/`LIST`/`SEND`/`RECV` with little-endian framing;
  `push`, `pull`, `pull_dir`, `read_bytes`/`read_text`, `stat`, `list`.
- **Shell helpers** (`shell`): `app_*`, `battery`, `brightness`, `wlan_ip`,
  `rotation`, `window_size`, `swipe`/`click`/`drag`, `list_packages`,
  `dump_hierarchy`, and more — same regexes as the Python original.
- **Screenshot** (`screenshot`): `screencap -p` → `image` crate.
- **Extras**: APK install (`install` + a minimal AXML `apk` manifest parser),
  screen recording (`screenrecord`: scrcpy or device `screenrecord`), a compact
  logcat colorizer (`pidcat`), and an `adbutils` CLI.

## Features

| feature | default | pulls in | enables |
|---|---|---|---|
| `image` | ✓ | `image` | `screenshot`, `framebuffer` |
| `apk` | ✓ | `reqwest`, `zip` | `install`, `apk` manifest parse |
| `screenrecord` | ✓ | `libc` | `start_recording` |
| `cli` | ✓ | `clap` | `adbutils` binary |

Disable defaults for a minimal transport-only build:
`adbutils = { version = "0.1", default-features = false }`.

## Bundled adb binary

`start-server` uses (in order): `ADBUTILS_ADB_PATH` → a binary staged at
`assets/binaries/adb` (`adb.exe` on Windows), wired up by `build.rs` → `adb` on
`PATH`. Drop a platform binary into `assets/binaries/` to bundle it.

## Testing

Unit/integration tests run against an in-process mock adb server (no device
needed) and assert the framing byte-exact: host commands, sync STAT/SEND/RECV,
shell v2 frames, tport discard-8, and shell-output parsing.

```
cargo test
cargo clippy --all-features --all-targets
```

End-to-end tests against a real device/emulator (screenshot decode, push/pull
round-trip, screenrecord, install) are **not** wired into CI yet — they need
connected hardware. The `examples/smoke.rs` example exercises the live server:

```
cargo run --example smoke
```
