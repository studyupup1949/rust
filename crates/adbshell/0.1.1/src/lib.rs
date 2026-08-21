// SPDX-License-Identifier: MIT OR Apache-2.0

//! `adbshell` — a reusable Rust crate for interacting with Android devices via ADB.
//!
//! Provides [`AdbShell`], a concrete implementation of all common ADB operations:
//! device info queries, file transfers, JAR execution, and reverse-tunnel management.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use adbshell::{AdbResult, AdbShell};
//!
//! // Verify adb is installed and in PATH
//! AdbShell::verify_adb_available().expect("adb not found");
//!
//! // Get the first connected device serial
//! let serial = AdbShell::get_device_serial().expect("no device connected");
//!
//! // Query a system property
//! let sdk = AdbShell::get_prop(&serial, "ro.build.version.sdk").unwrap();
//! println!("SDK: {sdk}");
//! ```

use {
    std::{
        ffi::OsStr,
        fmt,
        io::Read,
        process::{Child, Command, ExitStatus, Stdio},
        thread,
        time::Duration,
    },
    thiserror::Error,
    tracing::trace,
    wait_timeout::ChildExt,
};

// ── Error type ──────────────────────────────────────────────────────────────

/// Errors produced by ADB operations.
#[derive(Debug, Error)]
pub enum AdbError {
    /// The `adb` binary could not be found or spawned.
    #[error("ADB not found: {0}")]
    NotFound(String),

    /// An ADB command was launched but exited with a non-zero status.
    #[error("ADB command failed: {0}")]
    CommandFailed(String),

    /// An ADB command timed out.
    #[error("ADB command timed out")]
    Timeout,

    /// No device (or the requested device) could be found.
    #[error("ADB device not found: {0}")]
    DeviceNotFound(String),
}

/// Convenience result alias for ADB operations.
pub type AdbResult<T> = std::result::Result<T, AdbError>;

// ── Device state ─────────────────────────────────────────────────────────────

/// Connection state of an Android device as reported by `adb get-state`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceState {
    /// Device is online and ready (`device`).
    Connected,
    /// Device is not connected or unauthorized.
    Disconnected,
    /// Device is present but in an unexpected state.
    Unknown,
}

// ── AdbShell ─────────────────────────────────────────────────────────────────

/// A unit-struct that provides stateless ADB operations as associated functions.
///
/// Every method issues a fresh `adb` subprocess call; no persistent connection
/// is maintained.  Pass a device `serial` obtained from
/// [`AdbShell::get_device_serial`] to methods that require one.
pub struct AdbShell;

impl AdbShell {
    /// Verify that the `adb` binary is available and functional.
    ///
    /// Call once at application startup to fail fast if ADB is not installed
    /// or not in `PATH`.
    pub fn verify_adb_available() -> AdbResult<()> {
        Command::new("adb")
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| AdbError::NotFound(e.to_string()))?
            .success()
            .then_some(())
            .ok_or_else(|| AdbError::NotFound("adb returned non-zero exit status".to_string()))
    }

    /// Return the serial number of the first connected device.
    ///
    /// Equivalent to `adb get-serialno`.  Note that if multiple devices are
    /// connected, `adb get-serialno` returns an error; use `adb devices` to
    /// list them and select one explicitly.
    pub fn get_device_serial() -> AdbResult<String> {
        run_adb_command(["get-serialno"], None, |status, output, stderr| {
            if !status.success() {
                return Err(AdbError::DeviceNotFound(if stderr.is_empty() {
                    format!("adb exited with status: {}", status)
                } else {
                    format!(
                        "adb exited with status: {}; stderr: {}",
                        status,
                        stderr.trim()
                    )
                }));
            }
            // `adb get-serialno` prints "unknown" (with exit 0) when no device is attached
            // and the empty string when the daemon starts but no device is connected.
            let serial = output.trim().to_string();
            if serial.is_empty() || serial == "unknown" {
                return Err(AdbError::DeviceNotFound(
                    "no device connected (adb returned \"unknown\")".to_string(),
                ));
            }
            Ok(serial)
        })
    }

    /// Return the connection state of a device (`Connected`, `Disconnected`, or `Unknown`).
    pub fn get_device_state(serial: &str) -> AdbResult<DeviceState> {
        check_serial(serial)?;
        run_adb_command(
            ["-s", serial, "get-state"],
            None,
            |status, output, _stderr| {
                if !status.success() {
                    return Ok(DeviceState::Disconnected);
                }
                match output.trim() {
                    "device" => Ok(DeviceState::Connected),
                    _ => Ok(DeviceState::Unknown),
                }
            },
        )
    }

    /// Return the physical screen size in pixels as `(width, height)`.
    pub fn get_physical_screen_size(serial: &str) -> AdbResult<(u32, u32)> {
        check_serial(serial)?;
        run_adb_command(
            ["-s", serial, "shell", "wm", "size"],
            None,
            |status, output, stderr| {
                status
                    .success()
                    .then(|| {
                        output
                            .lines()
                            .find(|line| line.contains("Physical size:"))
                            .and_then(|line| line.split(':').nth(1))
                            .and_then(|size_part| {
                                let size_str = size_part.trim();
                                let mut parts = size_str.split('x');
                                let width = parts.next().and_then(|w| w.trim().parse::<u32>().ok());
                                let height =
                                    parts.next().and_then(|h| h.trim().parse::<u32>().ok());
                                match (width, height) {
                                    (Some(w), Some(h)) => Some((w, h)),
                                    _ => None,
                                }
                            })
                    })
                    .flatten()
                    .ok_or_else(|| {
                        AdbError::CommandFailed(if stderr.is_empty() {
                            format!("Failed to parse screen size from output: {}", output.trim())
                        } else {
                            format!("Failed to parse screen size; stderr: {}", stderr.trim())
                        })
                    })
            },
        )
    }

    /// Return the current screen orientation (0–3, where 1 = 90°, 2 = 180°, 3 = 270°).
    pub fn get_screen_orientation(serial: &str) -> AdbResult<u32> {
        check_serial(serial)?;
        run_adb_command(
            ["-s", serial, "shell", "dumpsys", "window", "displays"],
            Some(Duration::from_secs(3)),
            |status, output, stderr| {
                status
                    .success()
                    .then(|| {
                        output
                            .lines()
                            .find(|line| line.contains("mCurrentRotation"))
                            .and_then(|line| line.split('=').nth(1))
                            .and_then(|s| match s.trim() {
                                "ROTATION_0" | "0" => Some(0),
                                "ROTATION_90" | "1" => Some(1),
                                "ROTATION_180" | "2" => Some(2),
                                "ROTATION_270" | "3" => Some(3),
                                _ => None,
                            })
                    })
                    .flatten()
                    .ok_or_else(|| {
                        AdbError::CommandFailed(if stderr.is_empty() {
                            "Failed to parse screen orientation in dumpsys output".to_string()
                        } else {
                            format!(
                                "Failed to parse screen orientation; stderr: {}",
                                stderr.trim()
                            )
                        })
                    })
            },
        )
    }

    /// Return `true` if the soft keyboard (IME) is currently visible.
    pub fn get_ime_state(serial: &str) -> AdbResult<bool> {
        check_serial(serial)?;
        run_adb_command(
            ["-s", serial, "shell", "dumpsys", "window", "InputMethod"],
            Some(Duration::from_secs(3)),
            |status, output, stderr| {
                if !status.success() {
                    return Err(AdbError::CommandFailed(if stderr.is_empty() {
                        format!(
                            "Failed to query IME state on [{}], adb exited with status: {}",
                            serial, status
                        )
                    } else {
                        format!(
                            "Failed to query IME state on [{}], adb exited with status: {}; \
                             stderr: {}",
                            serial,
                            status,
                            stderr.trim()
                        )
                    }));
                }
                Ok(output.lines().any(|line| line.contains("isVisible=true")))
            },
        )
    }

    /// Return the Android API level (e.g. `30` for Android 11).
    pub fn get_android_version(serial: &str) -> AdbResult<u32> {
        let output = Self::get_prop(serial, "ro.build.version.sdk")?;
        output
            .trim()
            .parse()
            .map_err(|e| AdbError::CommandFailed(format!("Failed to parse Android version: {}", e)))
    }

    /// Return the device platform string (board or hardware name).
    pub fn get_platform(serial: &str) -> AdbResult<String> {
        let platform = Self::get_prop(serial, "ro.board.platform")?;
        if !platform.is_empty() && platform != "unknown" {
            return Ok(platform);
        }
        Self::get_prop(serial, "ro.hardware")
    }

    /// Return the value of an Android system property (`getprop <prop_name>`).
    pub fn get_prop(serial: &str, prop_name: &str) -> AdbResult<String> {
        check_serial(serial)?;
        run_adb_command(
            ["-s", serial, "shell", "getprop", prop_name],
            None,
            |status, output, stderr| {
                status
                    .success()
                    .then(|| output.trim().to_string())
                    .ok_or_else(|| {
                        AdbError::CommandFailed(if stderr.is_empty() {
                            format!(
                                "Failed to get property [{}], adb exited with status: {}",
                                prop_name, status
                            )
                        } else {
                            format!(
                                "Failed to get property [{}], adb exited with status: {}; stderr: {}",
                                prop_name, status, stderr.trim()
                            )
                        })
                    })
            },
        )
    }

    /// Push a local file to the device at the given path.
    pub fn push_file(serial: &str, file: &str, path: &str) -> AdbResult<()> {
        check_serial(serial)?;
        let out = Command::new("adb")
            .args(["-s", serial, "push", file, path])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AdbError::NotFound(format!("Failed to spawn adb: {}", e)))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(AdbError::CommandFailed(if stderr.trim().is_empty() {
                format!(
                    "Failed to push file [{}] to [{}], exit: {}",
                    file, path, out.status
                )
            } else {
                format!(
                    "Failed to push file [{}] to [{}], exit: {}; stderr: {}",
                    file,
                    path,
                    out.status,
                    stderr.trim()
                )
            }));
        }
        Ok(())
    }

    /// Execute a JAR file on the device via `app_process` and return the spawned [`Child`].
    pub fn execute_jar<I, S>(
        serial: &str,
        jar: &str,
        running_dir: &str,
        class_name: &str,
        version: &str,
        args: I,
    ) -> AdbResult<Child>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        check_serial(serial)?;
        let child = Command::new("adb")
            .args(["-s", serial])
            .args([
                "shell".to_string(),
                format!("CLASSPATH={}", jar),
                "app_process".to_string(),
                running_dir.to_string(),
                class_name.to_string(),
                version.to_string(),
            ])
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                AdbError::CommandFailed(format!(
                    "Failed to execute JAR [{}] on device [{}]: {}",
                    jar, serial, e
                ))
            })?;
        Ok(child)
    }

    /// Set up an ADB reverse tunnel: `localabstract:<socket_name>` → `tcp:<local_port>`.
    pub fn setup_reverse_tunnel(serial: &str, socket_name: &str, local_port: u16) -> AdbResult<()> {
        check_serial(serial)?;
        let status = Command::new("adb")
            .args([
                "-s",
                serial,
                "reverse",
                &format!("localabstract:{}", socket_name),
                &format!("tcp:{}", local_port),
            ])
            .status()
            .map_err(|e| {
                AdbError::CommandFailed(format!(
                    "Failed to setup reverse tunnel for [{}]: {}",
                    serial, e
                ))
            })?;
        if !status.success() {
            return Err(AdbError::CommandFailed(format!(
                "Failed to setup reverse tunnel for [{}], exit: {}",
                serial, status
            )));
        }
        Ok(())
    }

    /// Remove an ADB reverse tunnel.  Silently succeeds if the tunnel is already gone.
    pub fn remove_reverse_tunnel(serial: &str, socket_name: &str) -> AdbResult<()> {
        check_serial(serial)?;
        let output = Command::new("adb")
            .args([
                "-s",
                serial,
                "reverse",
                "--remove",
                &format!("localabstract:{}", socket_name),
            ])
            .output();
        match output {
            Ok(out) if !out.status.success() => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                if stderr.contains("not found") || stderr.contains("No such reverse") {
                    return Ok(());
                }
                Err(AdbError::CommandFailed(if stderr.is_empty() {
                    format!(
                        "Failed to remove reverse tunnel for [{}], exit: {}",
                        serial, out.status
                    )
                } else {
                    format!(
                        "Failed to remove reverse tunnel for [{}], exit: {}, stderr: {}",
                        serial,
                        out.status,
                        stderr.trim()
                    )
                }))
            }
            Ok(_) => Ok(()),
            Err(e) => Err(AdbError::CommandFailed(format!(
                "Failed to remove reverse tunnel for [{}]: {}",
                serial, e
            ))),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Spawn `adb <args>`, wait (with optional timeout), then parse the output.
///
/// When no timeout is given, uses `Command::output()` which reads both stdout
/// and stderr concurrently before waiting — this prevents the pipe-buffer
/// deadlock that can occur if `wait()` is called before the output pipes are
/// drained.
///
/// When a timeout is given, two threads are spawned to drain stdout and stderr
/// concurrently while the main thread waits with a timeout.
fn run_adb_command<I, S, F, R>(args: I, timeout: Option<Duration>, parse: F) -> AdbResult<R>
where
    I: IntoIterator<Item = S> + fmt::Debug,
    S: AsRef<OsStr>,
    F: FnOnce(ExitStatus, &str, &str) -> AdbResult<R>,
{
    trace!("Running adb command with args: {:?}", args);

    if timeout.is_none() {
        // Fast path: no timeout — use output() which drains both pipes safely.
        let out = Command::new("adb")
            .args(args)
            .output()
            .map_err(|e| AdbError::NotFound(format!("Failed to spawn adb: {e}")))?;
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        return parse(out.status, &stdout, &stderr);
    }

    // Timeout path: spawn and drain stdout/stderr in two background threads.
    let mut child = Command::new("adb")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AdbError::NotFound(format!("Failed to spawn adb: {e}")))?;
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();

    // Drain stdout in a background thread.
    let stdout_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut pipe) = stdout_pipe {
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    });
    // Drain stderr in a background thread.
    let stderr_thread = thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(mut pipe) = stderr_pipe {
            let _ = pipe.read_to_end(&mut buf);
        }
        buf
    });

    let to = timeout.unwrap();
    let status = match child
        .wait_timeout(to)
        .map_err(|e| AdbError::CommandFailed(format!("Failed to wait with timeout for adb: {e}")))?
    {
        Some(s) => s,
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(AdbError::Timeout);
        }
    };

    let stdout_bytes = stdout_thread.join().unwrap_or_default();
    let stderr_bytes = stderr_thread.join().unwrap_or_default();
    let stdout = String::from_utf8_lossy(&stdout_bytes).into_owned();
    let stderr = String::from_utf8_lossy(&stderr_bytes).into_owned();
    parse(status, &stdout, &stderr)
}

fn check_serial(serial: &str) -> AdbResult<()> {
    if serial.is_empty() {
        return Err(AdbError::CommandFailed(
            "Device serial cannot be empty".to_string(),
        ));
    }
    Ok(())
}
