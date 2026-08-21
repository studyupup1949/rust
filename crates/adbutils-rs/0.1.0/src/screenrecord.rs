//! Screen recording. Port of `adbutils/screenrecord.py`.
//!
//! Prefers the external `scrcpy` binary (records straight to mp4); falls back to
//! the device's `screenrecord` (3-minute cap) driven by a small shell script.
//! The legacy scrcpy-server-jar path is not ported.

use std::path::{Path, PathBuf};

use tokio::process::{Child, Command};

use crate::conn::AdbConnection;
use crate::device::AdbDevice;
use crate::errors::{AdbError, Result};
use crate::utils;

/// An in-progress recording. Call [`stop`](ScreenRecorder::stop) to finalize.
pub enum ScreenRecorder {
    /// scrcpy subprocess writing directly to `filename`.
    Scrcpy { child: Child },
    /// Device-side `screenrecord` via a pushed script; pulled on stop.
    Adb {
        stream: AdbConnection,
        device: AdbDevice,
        remote_path: String,
        filename: PathBuf,
    },
}

impl AdbDevice {
    /// Start recording to `filename`, choosing scrcpy if available, else the
    /// device `screenrecord`. Mirrors `start_recording`.
    pub async fn start_recording(&self, filename: impl AsRef<Path>) -> Result<ScreenRecorder> {
        let filename = filename.as_ref().to_path_buf();
        if let Some(scrcpy) = utils::which(if cfg!(windows) { "scrcpy.exe" } else { "scrcpy" }) {
            return self.start_scrcpy(&scrcpy, &filename).await;
        }
        if self.screenrecord_available().await? {
            return self.start_adb_record(&filename).await;
        }
        Err(AdbError::adb("no valid screenrecord client"))
    }

    async fn start_scrcpy(&self, scrcpy: &Path, filename: &Path) -> Result<ScreenRecorder> {
        let serial = self
            .serial()
            .ok_or_else(|| AdbError::adb("scrcpy requires a device serial"))?;
        let child = Command::new(scrcpy)
            .args(["--no-control", "--no-window", "--no-playback", "--record"])
            .arg(filename)
            .env("ADB", utils::adb_path())
            .env("ANDROID_SERIAL", serial)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .spawn()
            .map_err(|e| AdbError::adb(format!("failed to start scrcpy: {e}")))?;
        Ok(ScreenRecorder::Scrcpy { child })
    }

    async fn screenrecord_available(&self) -> Result<bool> {
        Ok(self.shell2(["which", "screenrecord"], false).await?.returncode == 0)
    }

    async fn start_adb_record(&self, filename: &Path) -> Result<ScreenRecorder> {
        let ts = chrono::Local::now().timestamp_millis();
        let remote_path = format!("/sdcard/adbutils-tmp-video-{ts}.mp4");
        let script = "#!/system/bin/sh\n# generate by adbutils\nscreenrecord \"$1\" &\nPID=$!\nread ANY\nkill -INT $PID\nwait\n";
        let script_path = "/sdcard/adbutils-screenrecord.sh";
        self.sync()
            .push_bytes(script.as_bytes(), script_path, 0o644)
            .await?;
        let stream = self
            .shell_stream(vec!["sh".to_string(), script_path.to_string(), remote_path.clone()])
            .await?;
        Ok(ScreenRecorder::Adb {
            stream,
            device: self.clone(),
            remote_path,
            filename: filename.to_path_buf(),
        })
    }
}

impl ScreenRecorder {
    /// True while the recording is still running.
    pub fn is_recording(&self) -> bool {
        match self {
            ScreenRecorder::Scrcpy { .. } => true,
            ScreenRecorder::Adb { stream, .. } => !stream.closed(),
        }
    }

    /// Stop recording and finalize the output file.
    pub async fn stop(self) -> Result<()> {
        match self {
            ScreenRecorder::Scrcpy { mut child } => {
                // scrcpy finalizes the mp4 on SIGINT; SIGKILL would corrupt it.
                #[cfg(unix)]
                if let Some(pid) = child.id() {
                    unsafe {
                        libc::kill(pid as libc::pid_t, libc::SIGINT);
                    }
                }
                let wait = tokio::time::timeout(std::time::Duration::from_secs(3), child.wait());
                match wait.await {
                    Ok(Ok(status)) => match status.code() {
                        Some(0) | None => Ok(()),
                        Some(1) => Err(AdbError::adb("scrcpy error: start failure")),
                        Some(2) => {
                            Err(AdbError::adb("scrcpy error: device disconnected while running"))
                        }
                        Some(code) => Err(AdbError::adb(format!("scrcpy error: {code}"))),
                    },
                    Ok(Err(e)) => Err(AdbError::adb(format!("scrcpy wait failed: {e}"))),
                    Err(_) => {
                        let _ = child.start_kill();
                        Err(AdbError::adb("scrcpy not handled SIGINT, killed"))
                    }
                }
            }
            ScreenRecorder::Adb { mut stream, device, remote_path, filename } => {
                // Writing a newline triggers the script's `read`, ending recording.
                stream.send(b"\n").await?;
                stream.read_until_close_bytes().await?;
                stream.close().await;
                device.sync().pull_file(&remote_path, &filename).await?;
                Ok(())
            }
        }
    }
}
