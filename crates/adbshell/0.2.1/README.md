# adbshell

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A minimal Rust wrapper around the `adb` command-line tool with a **persistent shell session**.

`adbshell` exposes [`AdbShell`], a stateful handle that maintains a long-lived
`adb shell` connection for low-latency device queries.  Each shell command
(getprop, dumpsys, wm) reuses the same session instead of spawning a fresh
`adb` subprocess, reducing per-call overhead.  File transfers and JAR execution
still use individual `adb` subprocesses.

## Features

- Persistent `adb shell` session with automatic reconnect on failure.
- Sentinel-based output framing — reliable output capture without per-process overhead.
- Device discovery: serial number, connection state, screen size and orientation.
- System property queries via `getprop`.
- File transfer with `adb push`.
- Reverse-tunnel setup and teardown (`adb reverse`).
- `app_process` JAR execution for launching Android server processes.
- No async runtime required — pure `std` threads.

## Quick start

Add to your `Cargo.toml`:

```toml
[dependencies]
adbshell = "0.2"
```

```rust,no_run
use adbshell::{AdbResult, AdbShell};

AdbShell::verify_adb_available().expect("adb not found in PATH");

let serial = AdbShell::get_device_serial().expect("no device connected");

// Open a persistent shell session
let shell = AdbShell::new(&serial).expect("failed to open shell");

let sdk = shell.get_prop("ro.build.version.sdk").unwrap();
println!("Android SDK: {sdk}");

let state = AdbShell::get_device_state(&serial).unwrap();
println!("Device state: {:?}", state);
```

## API overview

### Static helpers (no instance required)

| Function                             | Description                                      |
| ------------------------------------ | ------------------------------------------------ |
| `AdbShell::verify_adb_available()`   | Verify `adb` is installed and in `PATH`          |
| `AdbShell::get_device_serial()`      | Return the first connected device serial         |
| `AdbShell::get_device_state(serial)` | Return `Connected`, `Disconnected`, or `Unknown` |

### Instance methods (persistent shell session)

| Method                            | Description                                 |
| --------------------------------- | ------------------------------------------- |
| `AdbShell::new(serial)`           | Open a persistent `adb shell` session       |
| `shell.serial()`                  | Return the device serial                    |
| `shell.get_prop(key)`             | Read an Android system property             |
| `shell.get_android_version()`     | Return the API level as `u32`               |
| `shell.get_platform()`            | Return board/hardware platform string       |
| `shell.get_physical_screen_size()`| Return `(width, height)` in pixels          |
| `shell.get_screen_orientation()`  | Return orientation 0–3                      |
| `shell.get_ime_state()`           | Return whether the soft keyboard is visible |

### Instance methods (individual subprocesses)

| Method                                   | Description                          |
| ---------------------------------------- | ------------------------------------ |
| `shell.push_file(src, dst)`              | Push a local file to the device      |
| `shell.execute_jar(jar, …)`              | Launch a JAR via `app_process`       |
| `shell.setup_reverse_tunnel(name, port)` | Create an ADB reverse tunnel         |
| `shell.remove_reverse_tunnel(name)`      | Remove an ADB reverse tunnel         |

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
