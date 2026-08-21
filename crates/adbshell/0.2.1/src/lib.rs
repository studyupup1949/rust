// SPDX-License-Identifier: MIT OR Apache-2.0

//! `adbshell` — a reusable Rust crate for interacting with Android devices via ADB.
//!
//! Provides [`AdbShell`], a stateful handle that maintains a **persistent
//! `adb shell` session** for low-latency device queries while delegating
//! file-transfer and JAR-execution to individual `adb` subprocesses.
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
//! // Open a persistent shell session
//! let shell = AdbShell::new(&serial).expect("failed to connect");
//!
//! // Query a system property (reuses the persistent session)
//! let sdk = shell.get_prop("ro.build.version.sdk").unwrap();
//! println!("SDK: {sdk}");
//! ```

use {
    std::{
        ffi::OsStr,
        io::{BufRead, BufReader, BufWriter, Write},
        process::{Child, ChildStdin, Command, Stdio},
        sync::{Mutex, mpsc},
        thread,
        time::{Duration, Instant},
    },
    thiserror::Error,
    tracing::trace,
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

// ── Persistent shell session ──────────────────────────────────────────────────

/// Default timeout for shell commands that do not specify one.
const DEFAULT_SHELL_TIMEOUT: Duration = Duration::from_secs(5);

/// Sentinel prefix written by the device to signal command completion.
const SENTINEL_PREFIX: &str = "__ADBSH__";

/// An active `adb shell` session.
///
/// Writes commands to stdin and reads lines from a background reader thread.
/// Every command is terminated with a sentinel line so the reader knows exactly
/// when output ends.
struct ShellSession {
    /// The spawned `adb shell` child process.
    child: Child,
    /// Buffered writer around the child's stdin.
    stdin: BufWriter<ChildStdin>,
    /// Receiver end of the channel fed by the reader thread.
    line_rx: mpsc::Receiver<AdbResult<String>>,
    /// Monotonically-increasing counter used to generate unique sentinels.
    seq: u64,
}

impl ShellSession {
    /// Spawn `adb -s <serial> shell` and start the background reader thread.
    fn new(serial: &str) -> AdbResult<Self> {
        let mut child = adb_command()
            .args(["-s", serial, "shell"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| AdbError::NotFound(format!("Failed to spawn adb shell: {e}")))?;

        let stdin = BufWriter::new(
            child
                .stdin
                .take()
                .ok_or_else(|| AdbError::NotFound("Failed to open adb shell stdin".to_string()))?,
        );

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AdbError::NotFound("Failed to open adb shell stdout".to_string()))?;

        // The channel is bounded to avoid unbounded memory growth if the caller
        // is slow to consume lines, while still providing enough headroom for
        // typical command outputs.
        let (line_tx, line_rx) = mpsc::sync_channel::<AdbResult<String>>(256);

        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        if line_tx.send(Ok(l)).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = line_tx.send(Err(AdbError::CommandFailed(e.to_string())));
                        break;
                    }
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            line_rx,
            seq: 0,
        })
    }

    /// Execute `cmd` and collect output until the sentinel line is received.
    ///
    /// Returns the trimmed output lines joined by `\n`, or an error if the
    /// command fails, the sentinel is not received within `timeout`, or the
    /// shell session dies.
    fn run(&mut self, cmd: &str, timeout: Duration) -> AdbResult<String> {
        self.seq += 1;
        let seq = self.seq;

        // Write the command followed by a sentinel that embeds the sequence
        // number and exit code: __ADBSH__<seq>__<rc>.  Including the sequence
        // number allows stale sentinels from previously-interrupted commands to
        // be discarded rather than misinterpreted.
        writeln!(
            self.stdin,
            "{cmd}; __adbsh_rc=$?; printf '__ADBSH__{seq}__%d\\n' $__adbsh_rc"
        )
        .map_err(|e| AdbError::CommandFailed(format!("Failed to write to adb shell: {e}")))?;
        self.stdin.flush().map_err(|e| {
            AdbError::CommandFailed(format!("Failed to flush adb shell stdin: {e}"))
        })?;

        let deadline = Instant::now() + timeout;
        let mut output_lines: Vec<String> = Vec::new();

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(AdbError::Timeout);
            }

            let line = match self.line_rx.recv_timeout(remaining) {
                Ok(Ok(l)) => l,
                Ok(Err(e)) => return Err(e),
                Err(mpsc::RecvTimeoutError::Timeout) => return Err(AdbError::Timeout),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(AdbError::CommandFailed(
                        "adb shell session died unexpectedly".to_string(),
                    ));
                }
            };

            // Strip Windows-style CR that adb sometimes inserts.
            let line = line.trim_end_matches('\r').to_string();

            if let Some(rest) = line.strip_prefix(SENTINEL_PREFIX)
                && let Some((seq_str, rc_str)) = rest.split_once("__")
            {
                let line_seq: u64 = seq_str.trim().parse().unwrap_or(0);
                if line_seq == seq {
                    let rc: i32 = rc_str.trim().parse().unwrap_or(1);
                    if rc != 0 {
                        let msg = output_lines.join("\n");
                        return Err(AdbError::CommandFailed(if msg.is_empty() {
                            format!("Shell command exited with code {rc}: {cmd}")
                        } else {
                            format!("Shell command exited with code {rc}: {msg}")
                        }));
                    }
                    return Ok(output_lines.join("\n"));
                }
                // Stale sentinel from a previous interrupted command — discard.
                trace!("Discarding stale sentinel seq={line_seq} (current={seq})");
                continue;
            }

            output_lines.push(line);
        }
    }
}

impl Drop for ShellSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ── AdbShell ─────────────────────────────────────────────────────────────────

/// A stateful ADB handle that maintains a **persistent `adb shell` session**
/// for low-latency device queries.
///
/// Create one instance per connected device with [`AdbShell::new`].  Shell
/// commands (`get_prop`, `get_screen_orientation`, etc.) reuse the persistent
/// session; the session is automatically re-established on failure.
/// Operations that require a separate `adb` invocation (`push_file`,
/// `execute_jar`, `setup_reverse_tunnel`, `remove_reverse_tunnel`) always
/// spawn a fresh subprocess.
///
/// # Static helpers
///
/// [`AdbShell::verify_adb_available`], [`AdbShell::get_device_serial`], and
/// [`AdbShell::get_device_state`] do not require an instance and can be called
/// before creating one.
pub struct AdbShell {
    serial: String,
    /// `None` means the session has not been opened yet or was invalidated.
    session: Mutex<Option<ShellSession>>,
}

impl AdbShell {
    // ── Constructor ──────────────────────────────────────────────────────────

    /// Create a new `AdbShell` for the given device serial.
    ///
    /// Opens a persistent `adb shell` session immediately.  If the session
    /// cannot be opened (e.g. device not yet ready), this returns an error;
    /// callers should ensure the device is connected before calling `new`.
    pub fn new(serial: &str) -> AdbResult<Self> {
        check_serial(serial)?;
        let session = ShellSession::new(serial)?;
        Ok(Self {
            serial: serial.to_string(),
            session: Mutex::new(Some(session)),
        })
    }

    /// Return the device serial associated with this handle.
    pub fn serial(&self) -> &str { &self.serial }

    // ── Static helpers (no instance required) ───────────────────────────────

    /// Verify that the `adb` binary is available and functional.
    ///
    /// Call once at application startup to fail fast if ADB is not installed
    /// or not in `PATH`.
    pub fn verify_adb_available() -> AdbResult<()> {
        adb_command()
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
        let out = adb_command()
            .arg("get-serialno")
            .output()
            .map_err(|e| AdbError::NotFound(format!("Failed to spawn adb: {e}")))?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(AdbError::DeviceNotFound(if stderr.trim().is_empty() {
                format!("adb exited with status: {}", out.status)
            } else {
                format!(
                    "adb exited with status: {}; stderr: {}",
                    out.status,
                    stderr.trim()
                )
            }));
        }

        let serial = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if serial.is_empty() || serial == "unknown" {
            return Err(AdbError::DeviceNotFound(
                "no device connected (adb returned \"unknown\")".to_string(),
            ));
        }
        Ok(serial)
    }

    /// Return the connection state of a device (`Connected`, `Disconnected`, or `Unknown`).
    ///
    /// This is a **static** helper that spawns a fresh `adb` subprocess.  It
    /// is intentionally kept static so that callers can poll device readiness
    /// before constructing an [`AdbShell`] instance.
    pub fn get_device_state(serial: &str) -> AdbResult<DeviceState> {
        check_serial(serial)?;
        let out = adb_command()
            .args(["-s", serial, "get-state"])
            .output()
            .map_err(|e| AdbError::NotFound(format!("Failed to spawn adb: {e}")))?;

        if !out.status.success() {
            return Ok(DeviceState::Disconnected);
        }
        match String::from_utf8_lossy(&out.stdout).trim() {
            "device" => Ok(DeviceState::Connected),
            _ => Ok(DeviceState::Unknown),
        }
    }

    // ── Instance shell queries (reuse persistent session) ───────────────────

    /// Return the value of an Android system property (`getprop <prop_name>`).
    pub fn get_prop(&self, prop_name: &str) -> AdbResult<String> {
        let output = self.run_shell_cmd(&format!("getprop {prop_name}"), DEFAULT_SHELL_TIMEOUT)?;
        Ok(output.trim().to_string())
    }

    /// Return the Android API level (e.g. `30` for Android 11).
    pub fn get_android_version(&self) -> AdbResult<u32> {
        let output = self.get_prop("ro.build.version.sdk")?;
        output
            .trim()
            .parse()
            .map_err(|e| AdbError::CommandFailed(format!("Failed to parse Android version: {e}")))
    }

    /// Return the device platform string (board or hardware name).
    pub fn get_platform(&self) -> AdbResult<String> {
        let platform = self.get_prop("ro.board.platform")?;
        if !platform.is_empty() && platform != "unknown" {
            return Ok(platform);
        }
        self.get_prop("ro.hardware")
    }

    /// Return the physical screen size in pixels as `(width, height)`.
    pub fn get_physical_screen_size(&self) -> AdbResult<(u32, u32)> {
        let output = self.run_shell_cmd("wm size", DEFAULT_SHELL_TIMEOUT)?;
        output
            .lines()
            .find(|line| line.contains("Physical size:"))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|size_part| {
                let size_str = size_part.trim();
                let mut parts = size_str.split('x');
                let width = parts.next().and_then(|w| w.trim().parse::<u32>().ok());
                let height = parts.next().and_then(|h| h.trim().parse::<u32>().ok());
                match (width, height) {
                    (Some(w), Some(h)) => Some((w, h)),
                    _ => None,
                }
            })
            .ok_or_else(|| {
                AdbError::CommandFailed(format!(
                    "Failed to parse screen size from output: {}",
                    output.trim()
                ))
            })
    }

    /// Return the current screen orientation (0–3, where 1 = 90°, 2 = 180°, 3 = 270°).
    pub fn get_screen_orientation(&self) -> AdbResult<u32> {
        let output = self.run_shell_cmd("dumpsys window displays", Duration::from_secs(3))?;
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
            .ok_or_else(|| {
                AdbError::CommandFailed(
                    "Failed to parse screen orientation in dumpsys output".to_string(),
                )
            })
    }

    /// Return `true` if the soft keyboard (IME) is currently visible.
    pub fn get_ime_state(&self) -> AdbResult<bool> {
        let output = self.run_shell_cmd("dumpsys window InputMethod", Duration::from_secs(3))?;
        Ok(output.lines().any(|line| line.contains("isVisible=true")))
    }

    // ── Instance process-spawn operations ───────────────────────────────────

    /// Push a local file to the device at the given path.
    pub fn push_file(&self, file: &str, path: &str) -> AdbResult<()> {
        let out = adb_command()
            .args(["-s", &self.serial, "push", file, path])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| AdbError::NotFound(format!("Failed to spawn adb: {e}")))?;
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
        &self,
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
        let child = adb_command()
            .args(["-s", &self.serial])
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
                    jar, self.serial, e
                ))
            })?;
        Ok(child)
    }

    /// Set up an ADB reverse tunnel: `localabstract:<socket_name>` → `tcp:<local_port>`.
    pub fn setup_reverse_tunnel(&self, socket_name: &str, local_port: u16) -> AdbResult<()> {
        let status = adb_command()
            .args([
                "-s",
                &self.serial,
                "reverse",
                &format!("localabstract:{socket_name}"),
                &format!("tcp:{local_port}"),
            ])
            .status()
            .map_err(|e| {
                AdbError::CommandFailed(format!(
                    "Failed to setup reverse tunnel for [{}]: {}",
                    self.serial, e
                ))
            })?;
        if !status.success() {
            return Err(AdbError::CommandFailed(format!(
                "Failed to setup reverse tunnel for [{}], exit: {}",
                self.serial, status
            )));
        }
        Ok(())
    }

    /// Remove an ADB reverse tunnel.  Silently succeeds if the tunnel is already gone.
    pub fn remove_reverse_tunnel(&self, socket_name: &str) -> AdbResult<()> {
        let output = adb_command()
            .args([
                "-s",
                &self.serial,
                "reverse",
                "--remove",
                &format!("localabstract:{socket_name}"),
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
                        self.serial, out.status
                    )
                } else {
                    format!(
                        "Failed to remove reverse tunnel for [{}], exit: {}, stderr: {}",
                        self.serial,
                        out.status,
                        stderr.trim()
                    )
                }))
            }
            Ok(_) => Ok(()),
            Err(e) => Err(AdbError::CommandFailed(format!(
                "Failed to remove reverse tunnel for [{}]: {}",
                self.serial, e
            ))),
        }
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    /// Run a shell command on the persistent session, auto-reconnecting on failure.
    fn run_shell_cmd(&self, cmd: &str, timeout: Duration) -> AdbResult<String> {
        trace!("Running shell command: {}", cmd);

        let mut guard = self
            .session
            .lock()
            .map_err(|_| AdbError::CommandFailed("Shell session mutex poisoned".to_string()))?;

        // Attempt on the existing session (or a freshly-opened one).
        if let Some(ref mut session) = *guard {
            match session.run(cmd, timeout) {
                Ok(output) => return Ok(output),
                Err(e) => {
                    trace!("Shell session error (will reconnect): {}", e);
                    // Invalidate the broken session.
                    *guard = None;
                }
            }
        }

        // Reconnect once.
        let mut new_session = ShellSession::new(&self.serial)?;
        let result = new_session.run(cmd, timeout);
        *guard = Some(new_session);
        result
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn check_serial(serial: &str) -> AdbResult<()> {
    if serial.is_empty() {
        return Err(AdbError::CommandFailed(
            "Device serial cannot be empty".to_string(),
        ));
    }
    Ok(())
}

/// Returns a [`Command`] for `adb` pre-configured to suppress the console
/// window that Windows would otherwise pop up for every subprocess.
///
/// On non-Windows platforms this is identical to `Command::new("adb")`.
fn adb_command() -> Command {
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("adb")
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("adb");

        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW — prevents a cmd.exe console from appearing.
        cmd.creation_flags(0x0800_0000);

        cmd
    }
}
