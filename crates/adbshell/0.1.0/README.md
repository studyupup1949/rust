# adbshell

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A minimal, zero-overhead Rust wrapper around the `adb` command-line tool.

`adbshell` exposes [`AdbShell`], a stateless unit-struct whose associated
functions cover the ADB operations most commonly needed by desktop Android
companion apps: device discovery, system-property queries, file push, JAR
execution, and reverse-tunnel management.  Every call spawns a fresh `adb`
subprocess — no persistent daemon connection is maintained.

## Features

- Device discovery: serial number, connection state, screen size and orientation.
- System property queries via `getprop`.
- File transfer with `adb push`.
- Reverse-tunnel setup and teardown (`adb reverse`).
- `app_process` JAR execution for launching Android server processes.
- Optional per-call timeout support (backed by `wait-timeout`).
- No async runtime required — pure `std` threads.

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
adbshell = { path = "3rd-party/adbshell" }
```

```rust,no_run
use adbshell::{AdbResult, AdbShell};

AdbShell::verify_adb_available().expect("adb not found in PATH");

let serial = AdbShell::get_device_serial().expect("no device connected");

let sdk = AdbShell::get_prop(&serial, "ro.build.version.sdk").unwrap();
println!("Android SDK: {sdk}");

let state = AdbShell::get_device_state(&serial).unwrap();
println!("Device state: {:?}", state);
```

## API overview

| Function | Description |
|----------|-------------|
| `AdbShell::verify_adb_available()` | Verify `adb` is installed and in `PATH` |
| `AdbShell::get_device_serial()` | Return the first connected device serial |
| `AdbShell::get_device_state(serial)` | Return `Connected`, `Disconnected`, or `Unknown` |
| `AdbShell::get_prop(serial, key)` | Read an Android system property |
| `AdbShell::get_android_version(serial)` | Return the API level as `u32` |
| `AdbShell::get_platform(serial)` | Return board/hardware platform string |
| `AdbShell::get_physical_screen_size(serial)` | Return `(width, height)` in pixels |
| `AdbShell::get_screen_orientation(serial)` | Return orientation 0–3 |
| `AdbShell::get_ime_state(serial)` | Return whether the soft keyboard is visible |
| `AdbShell::push_file(serial, src, dst)` | Push a local file to the device |
| `AdbShell::execute_jar(serial, jar, …)` | Launch a JAR via `app_process` |
| `AdbShell::setup_reverse_tunnel(serial, …)` | Create an ADB reverse tunnel |
| `AdbShell::remove_reverse_tunnel(serial, …)` | Remove an ADB reverse tunnel |

## Error handling

All functions return `AdbResult<T>` (an alias for `Result<T, AdbError>`).

```rust
pub enum AdbError {
    NotFound(String),       // adb binary missing or failed to spawn
    CommandFailed(String),  // adb exited with non-zero status
    Timeout,                // command exceeded the given duration
    DeviceNotFound(String), // requested device not connected
}
```

## Requirements

- `adb` (Android platform-tools) must be installed and in `PATH`.
- Rust 1.85+

## License

Licensed under either of

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
