use std::{
    ffi::OsStr,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde::Serialize;

pub type Result<T> = std::result::Result<T, AdbError>;

#[derive(Debug, thiserror::Error)]
pub enum AdbError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("command `{program}` timed out after {timeout:?}")]
    Timeout { program: String, timeout: Duration },
    #[error("command `{program}` failed with exit code {code:?}: {stderr}")]
    CommandFailed {
        program: String,
        code: Option<i32>,
        stderr: String,
    },
    #[error("parse error: {0}")]
    Parse(String),
    #[error("ADB error: {0}")]
    Adb(String),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdbDevice {
    pub serial: String,
    pub state: String,
}

/// Android SDK Platform-Tools version embedded by this crate.
///
/// The embedded binary is currently provided for Windows only. Non-Windows
/// builds use `ADB` or `adb` from `PATH`.
pub const BUNDLED_PLATFORM_TOOLS_VERSION: &str = "37.0.0";

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone)]
pub struct Adb {
    serial: Option<String>,
    timeout: Duration,
}

impl Adb {
    pub fn new(serial: impl Into<Option<String>>) -> Self {
        Self {
            serial: normalize_serial(serial.into()),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn default_device() -> Self {
        Self::new(None)
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }

    pub fn run<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        run_adb(self.serial(), args, self.timeout)
    }

    pub fn shell<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        adb_shell(self.serial(), args, self.timeout)
    }

    pub fn shell_text<I, S>(&self, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        adb_shell_text(self.serial(), args, self.timeout)
    }

    pub fn exec_out<I, S>(&self, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        adb_exec_out(self.serial(), args, self.timeout)
    }

    pub fn forward(&self, local: &str, remote: &str) -> Result<()> {
        adb_forward(self.serial(), local, remote, self.timeout)
    }

    pub fn forward_tcp0(&self, remote: &str) -> Result<u16> {
        adb_forward_tcp0(self.serial(), remote, self.timeout)
    }

    pub fn forward_remove(&self, local: &str) -> Result<()> {
        adb_forward_remove(self.serial(), local, self.timeout)
    }

    pub fn push(&self, local: impl AsRef<Path>, remote: &str) -> Result<()> {
        adb_push(self.serial(), local, remote, Duration::from_secs(30))
    }

    pub fn install(&self, apk: impl AsRef<Path>, reinstall: bool) -> Result<()> {
        adb_install(self.serial(), apk, reinstall, Duration::from_secs(60))
    }

    pub fn screencap_png(&self) -> Result<Vec<u8>> {
        adb_screencap_png(self.serial(), self.timeout)
    }

    pub fn window_size(&self) -> Result<(u32, u32)> {
        adb_window_size(self.serial())
    }

    pub fn display_size(&self) -> Result<(u32, u32)> {
        adb_display_size(self.serial())
    }

    pub fn shell_prop(&self, prop: &str) -> Result<String> {
        let serial = self
            .serial()
            .ok_or_else(|| AdbError::Adb("shell_prop requires an explicit serial".to_string()))?;
        adb_shell_prop(serial, prop)
    }

    pub fn sdk_abi(&self) -> Result<(u32, String)> {
        let serial = self
            .serial()
            .ok_or_else(|| AdbError::Adb("sdk_abi requires an explicit serial".to_string()))?;
        adb_sdk_abi(serial)
    }
}

pub fn adb_program() -> String {
    if let Some(adb) = std::env::var_os("ADB").filter(|value| !value.is_empty()) {
        return adb.to_string_lossy().into_owned();
    }

    bundled_adb_program()
        .ok()
        .flatten()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|| "adb".to_string())
}

/// Extracts the bundled ADB binary, when this platform has one, and returns its
/// executable path.
///
/// On unsupported platforms this returns `Ok(None)` so callers can fall back to
/// `adb` from `PATH`.
pub fn bundled_adb_program() -> Result<Option<PathBuf>> {
    #[cfg(windows)]
    {
        bundled_windows::ensure_bundled_adb()
            .map(Some)
            .map_err(AdbError::Io)
    }

    #[cfg(not(windows))]
    {
        Ok(None)
    }
}

/// Extracts bundled ADB, if available, and sets the process `ADB` environment
/// variable to it.
pub fn configure_bundled_adb() -> Result<Option<PathBuf>> {
    let adb = bundled_adb_program()?;
    if let Some(path) = &adb {
        std::env::set_var("ADB", path);
    }
    Ok(adb)
}

pub fn run_command<I, S>(program: &str, args: I, timeout: Duration) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| AdbError::Adb(format!("failed to capture stdout for `{program}`")))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| AdbError::Adb(format!("failed to capture stderr for `{program}`")))?;

    let stdout_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });

    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let stdout = stdout_reader.join().map_err(|_| {
                AdbError::Adb(format!("stdout reader thread panicked for `{program}`"))
            })??;
            let stderr = stderr_reader.join().map_err(|_| {
                AdbError::Adb(format!("stderr reader thread panicked for `{program}`"))
            })??;

            if status.success() {
                return Ok(stdout);
            }

            return Err(AdbError::CommandFailed {
                program: program.to_string(),
                code: status.code(),
                stderr: String::from_utf8_lossy(&stderr).trim().to_string(),
            });
        }

        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(AdbError::Timeout {
                program: program.to_string(),
                timeout,
            });
        }

        thread::sleep(Duration::from_millis(20));
    }
}

pub fn run_adb<I, S>(serial: Option<&str>, args: I, timeout: Duration) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut full_args: Vec<std::ffi::OsString> = Vec::new();
    if let Some(serial) = serial.filter(|s| !s.is_empty()) {
        full_args.push("-s".into());
        full_args.push(serial.into());
    }
    full_args.extend(args.into_iter().map(|arg| arg.as_ref().to_os_string()));
    run_command(&adb_program(), full_args, timeout)
}

pub fn adb_devices() -> Result<Vec<AdbDevice>> {
    let out = run_adb(None, ["devices"], Duration::from_secs(5))?;
    if out.is_empty() {
        return Ok(Vec::new());
    }
    parse_devices_output(&String::from_utf8(out)?)
}

pub fn adb_shell<I, S>(serial: Option<&str>, args: I, timeout: Duration) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut full_args: Vec<std::ffi::OsString> = vec!["shell".into()];
    full_args.extend(args.into_iter().map(|arg| arg.as_ref().to_os_string()));
    run_adb(serial, full_args, timeout)
}

pub fn adb_shell_text<I, S>(serial: Option<&str>, args: I, timeout: Duration) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Ok(String::from_utf8(adb_shell(serial, args, timeout)?)?
        .trim()
        .to_string())
}

pub fn adb_exec_out<I, S>(serial: Option<&str>, args: I, timeout: Duration) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut full_args: Vec<std::ffi::OsString> = vec!["exec-out".into()];
    full_args.extend(args.into_iter().map(|arg| arg.as_ref().to_os_string()));
    run_adb(serial, full_args, timeout)
}

pub fn adb_forward(
    serial: Option<&str>,
    local: &str,
    remote: &str,
    timeout: Duration,
) -> Result<()> {
    run_adb(serial, ["forward", local, remote], timeout)?;
    Ok(())
}

pub fn adb_forward_tcp0(serial: Option<&str>, remote: &str, timeout: Duration) -> Result<u16> {
    let out = run_adb(serial, ["forward", "tcp:0", remote], timeout)?;
    let text = String::from_utf8(out)?;
    text.trim()
        .parse::<u16>()
        .map_err(|err| AdbError::Parse(format!("unable to parse forwarded port `{text}`: {err}")))
}

pub fn adb_forward_remove(serial: Option<&str>, local: &str, timeout: Duration) -> Result<()> {
    run_adb(serial, ["forward", "--remove", local], timeout)?;
    Ok(())
}

pub fn adb_push(
    serial: Option<&str>,
    local: impl AsRef<Path>,
    remote: &str,
    timeout: Duration,
) -> Result<()> {
    let local = local.as_ref().display().to_string();
    run_adb(serial, ["push", &local, remote], timeout)?;
    Ok(())
}

pub fn adb_install(
    serial: Option<&str>,
    apk: impl AsRef<Path>,
    reinstall: bool,
    timeout: Duration,
) -> Result<()> {
    let apk = apk.as_ref().display().to_string();
    if reinstall {
        run_adb(serial, ["install", "-r", &apk], timeout)?;
    } else {
        run_adb(serial, ["install", &apk], timeout)?;
    }
    Ok(())
}

pub fn adb_screencap_png(serial: Option<&str>, timeout: Duration) -> Result<Vec<u8>> {
    adb_exec_out(serial, ["screencap", "-p"], timeout)
}

pub fn adb_window_size(serial: Option<&str>) -> Result<(u32, u32)> {
    let out = run_adb(serial, ["shell", "wm", "size"], Duration::from_secs(5))?;
    if out.is_empty() {
        return Err(AdbError::Adb(
            "adb shell wm size returned no output".to_string(),
        ));
    }
    parse_wm_size(&String::from_utf8(out)?)
}

pub fn adb_display_size(serial: Option<&str>) -> Result<(u32, u32)> {
    let out = run_adb(
        serial,
        ["shell", "dumpsys", "display"],
        Duration::from_secs(5),
    )?;
    let text = String::from_utf8(out)?;
    for marker in [
        "mOverrideDisplayInfo=DisplayInfo",
        "mBaseDisplayInfo=DisplayInfo",
    ] {
        if let Some(size) = parse_display_info_size(&text, marker) {
            return Ok(size);
        }
    }

    adb_window_size(serial)
}

pub fn adb_shell_prop(serial: &str, prop: &str) -> Result<String> {
    adb_shell_text(Some(serial), ["getprop", prop], Duration::from_secs(5))
}

pub fn adb_sdk_abi(serial: &str) -> Result<(u32, String)> {
    let sdk = adb_shell_prop(serial, "ro.build.version.sdk")?;
    let abi = adb_shell_prop(serial, "ro.product.cpu.abi")?;
    let sdk = sdk
        .parse::<u32>()
        .map_err(|err| AdbError::Parse(format!("unable to parse Android SDK `{sdk}`: {err}")))?;
    Ok((sdk, abi))
}

pub fn parse_wm_size(output: &str) -> Result<(u32, u32)> {
    output
        .lines()
        .rev()
        .filter_map(|line| line.split_once(':').map(|(_, rhs)| rhs.trim()))
        .find_map(|size| {
            let (w, h) = size.split_once('x')?;
            Some((w.trim().parse::<u32>().ok()?, h.trim().parse::<u32>().ok()?))
        })
        .ok_or_else(|| AdbError::Parse(format!("unable to parse adb window size from `{output}`")))
}

fn parse_devices_output(text: &str) -> Result<Vec<AdbDevice>> {
    Ok(text
        .lines()
        .skip_while(|line| line.starts_with("List of devices"))
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let serial = parts.next()?;
            let state = parts.next()?;
            Some(AdbDevice {
                serial: serial.to_string(),
                state: state.to_string(),
            })
        })
        .collect())
}

fn parse_display_info_size(text: &str, marker: &str) -> Option<(u32, u32)> {
    let start = text.find(marker)?;
    let tail = &text[start..text.len().min(start + 1200)];
    let real = tail.find("real ")?;
    let size = &tail[real + "real ".len()..];
    let (width, rest) = size.split_once(" x ")?;
    let height = rest
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    Some((width.trim().parse().ok()?, height.parse().ok()?))
}

fn normalize_serial(serial: Option<String>) -> Option<String> {
    serial
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(windows)]
mod bundled_windows {
    use super::*;
    use std::fs;

    struct EmbeddedFile {
        name: &'static str,
        bytes: &'static [u8],
    }

    const FILES: &[EmbeddedFile] = &[
        EmbeddedFile {
            name: "adb.exe",
            bytes: include_bytes!("../resources/adb/windows/adb.exe"),
        },
        EmbeddedFile {
            name: "AdbWinApi.dll",
            bytes: include_bytes!("../resources/adb/windows/AdbWinApi.dll"),
        },
        EmbeddedFile {
            name: "AdbWinUsbApi.dll",
            bytes: include_bytes!("../resources/adb/windows/AdbWinUsbApi.dll"),
        },
        EmbeddedFile {
            name: "libwinpthread-1.dll",
            bytes: include_bytes!("../resources/adb/windows/libwinpthread-1.dll"),
        },
        EmbeddedFile {
            name: "NOTICE.txt",
            bytes: include_bytes!("../resources/adb/windows/NOTICE.txt"),
        },
        EmbeddedFile {
            name: "source.properties",
            bytes: include_bytes!("../resources/adb/windows/source.properties"),
        },
    ];

    pub(super) fn ensure_bundled_adb() -> io::Result<PathBuf> {
        let dir = bundled_adb_dir();
        fs::create_dir_all(&dir)?;

        for file in FILES {
            write_if_needed(&dir.join(file.name), file.bytes)?;
        }

        Ok(dir.join("adb.exe"))
    }

    fn bundled_adb_dir() -> PathBuf {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        base.join("adblite-rs").join(format!(
            "platform-tools-windows-{BUNDLED_PLATFORM_TOOLS_VERSION}"
        ))
    }

    fn write_if_needed(path: &Path, bytes: &[u8]) -> io::Result<()> {
        if path
            .metadata()
            .map(|metadata| metadata.len() == bytes.len() as u64)
            .unwrap_or(false)
        {
            return Ok(());
        }

        let temp_path = path.with_extension(format!("{}.tmp", std::process::id()));
        fs::write(&temp_path, bytes)?;

        match fs::rename(&temp_path, path) {
            Ok(()) => Ok(()),
            Err(rename_err) => {
                let copy_result = fs::copy(&temp_path, path)
                    .map(|_| ())
                    .and_then(|_| fs::remove_file(&temp_path));
                copy_result.map_err(|copy_err| {
                    io::Error::new(
                        copy_err.kind(),
                        format!(
                            "failed to install bundled adb file `{}`: rename failed: {rename_err}; copy failed: {copy_err}",
                            path.display()
                        ),
                    )
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wm_size() {
        assert_eq!(
            parse_wm_size("Physical size: 1080x1920\n").unwrap(),
            (1080, 1920)
        );
        assert_eq!(
            parse_wm_size("Physical size: 1080x1920\nOverride size: 720x1280\n").unwrap(),
            (720, 1280)
        );
    }

    #[test]
    fn parses_devices_output() {
        let devices =
            parse_devices_output("List of devices attached\nemulator-5554\tdevice\n").unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].serial, "emulator-5554");
        assert_eq!(devices[0].state, "device");
    }

    #[cfg(windows)]
    #[test]
    fn extracts_bundled_adb() {
        let adb = bundled_adb_program().unwrap().unwrap();
        let dir = adb.parent().unwrap();

        assert_eq!(adb.file_name().unwrap(), "adb.exe");
        assert!(adb.is_file());
        assert!(dir.join("AdbWinApi.dll").is_file());
        assert!(dir.join("AdbWinUsbApi.dll").is_file());
        assert!(dir.join("libwinpthread-1.dll").is_file());
    }
}
