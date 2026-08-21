//! Async ADB wire protocol client over TCP.
//!
//! Talks directly to the ADB server (`localhost:5037`) using the
//! [ADB protocol](https://android.googlesource.com/platform/packages/modules/adb/+/refs/heads/main/OVERVIEW.TXT)
//! without spawning any child processes.
//!
//! # Quick start
//!
//! ```no_run
//! use adb_wire::AdbWire;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     let adb = AdbWire::new("emulator-5554");
//!
//!     // Run a command — returns stdout, stderr, and exit code
//!     let result = adb.shell("getprop ro.build.version.sdk").await.unwrap();
//!     println!("SDK version: {}", result.stdout_str());
//!
//!     // Check exit status
//!     let result = adb.shell("ls /nonexistent").await.unwrap();
//!     if !result.success() {
//!         eprintln!("exit {}: {}", result.exit_code, result.stderr_str());
//!     }
//!
//!     // Fire-and-forget (returns immediately, command runs on device)
//!     adb.shell_detach("input tap 500 500").await.unwrap();
//!
//!     // Stream output incrementally (e.g. logcat)
//!     use tokio::io::{AsyncBufReadExt, BufReader};
//!     let stream = adb.shell_stream("logcat -d").await.unwrap();
//!     let mut lines = BufReader::new(stream).lines();
//!     while let Some(line) = lines.next_line().await.unwrap() {
//!         println!("{line}");
//!     }
//!
//!     // File transfer
//!     adb.push_file("local.txt", "/sdcard/remote.txt").await.unwrap();
//!     adb.pull_file("/sdcard/remote.txt", "downloaded.txt").await.unwrap();
//!
//!     // Check if a remote file exists
//!     let st = adb.stat("/sdcard/remote.txt").await.unwrap();
//!     println!("exists={} size={}", st.exists(), st.size);
//!
//!     // Install an APK
//!     adb.install("app.apk").await.unwrap();
//! }
//! ```
//!
//! # Host commands
//!
//! Commands that target the ADB server (not a specific device) are free
//! functions:
//!
//! ```no_run
//! use adb_wire::{list_devices, DEFAULT_SERVER};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     for (serial, state) in list_devices(DEFAULT_SERVER).await.unwrap() {
//!         println!("{serial}\t{state}");
//!     }
//! }
//! ```

mod error;
mod shell;
mod sync;
mod wire;

pub use error::{Error, Result};
pub use shell::{ShellOutput, ShellStream};
pub use sync::{DirEntry, RemoteStat};

use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpStream;

/// Default ADB server address (`127.0.0.1:5037`).
pub const DEFAULT_SERVER: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)), 5037);

// -- Host-level (device-independent) functions --------------------------------

/// Send a host command to an ADB server and read the length-prefixed response.
///
/// These commands don't target a specific device. Examples:
/// `"host:version"`, `"host:devices"`, `"host:track-devices"`.
pub async fn host_command(server: SocketAddr, cmd: &str) -> Result<String> {
    use tokio::io::AsyncReadExt;
    let mut stream = tokio::time::timeout(
        Duration::from_secs(2),
        TcpStream::connect(server),
    )
    .await
    .map_err(|_| Error::timed_out("connect timed out"))??;
    stream.set_nodelay(true)?;

    wire::send(&mut stream, cmd).await?;
    wire::read_okay(&mut stream).await?;

    // Response length is bounded by read_hex_len (max 1 MiB).
    let len = wire::read_hex_len(&mut stream).await?;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    String::from_utf8(buf)
        .map_err(|e| Error::Protocol(format!("invalid utf-8 in response: {e}")))
}

/// List connected devices. Returns `(serial, state)` pairs.
///
/// `state` is typically `"device"`, `"offline"`, or `"unauthorized"`.
pub async fn list_devices(server: SocketAddr) -> Result<Vec<(String, String)>> {
    let raw = host_command(server, "host:devices").await?;
    Ok(parse_device_list(&raw))
}

fn parse_device_list(raw: &str) -> Vec<(String, String)> {
    raw.lines()
        .filter_map(|line| {
            let (serial, state) = line.split_once('\t')?;
            Some((serial.to_string(), state.to_string()))
        })
        .collect()
}

/// Append a single-quoted, shell-safe version of `s` to `out`.
///
/// Wraps the value in single quotes, escaping any embedded single quotes
/// with the `'\''` idiom. This prevents shell metacharacter injection.
fn shell_quote_into(out: &mut String, s: &str) {
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
}

// -- Device client ------------------------------------------------------------

/// Async ADB wire protocol client bound to a specific device.
///
/// Each method opens a fresh TCP connection to the ADB server, so this type
/// holds no open connections. All fields are plain data, making `AdbWire`
/// [`Send`], [`Sync`], and [`Clone`].
#[derive(Clone, Debug)]
pub struct AdbWire {
    serial: String,
    server: SocketAddr,
    connect_timeout: Duration,
    read_timeout: Duration,
}

impl AdbWire {
    /// Create a client for the given device serial (e.g. `"emulator-5554"`,
    /// `"127.0.0.1:5555"`, `"R5CR1234567"`).
    pub fn new(serial: &str) -> Self {
        Self {
            serial: serial.to_string(),
            server: DEFAULT_SERVER,
            connect_timeout: Duration::from_secs(2),
            read_timeout: Duration::from_secs(30),
        }
    }

    /// Use a custom ADB server address (default: `127.0.0.1:5037`).
    pub fn server(mut self, addr: SocketAddr) -> Self {
        self.server = addr;
        self
    }

    /// Set the TCP connect timeout (default: 2s).
    pub fn connect_timeout(mut self, dur: Duration) -> Self {
        self.connect_timeout = dur;
        self
    }

    /// Set the read timeout for buffered methods like [`shell`](Self::shell)
    /// (default: 30s). For streaming methods, manage timeouts on the returned
    /// stream directly.
    pub fn read_timeout(mut self, dur: Duration) -> Self {
        self.read_timeout = dur;
        self
    }

    /// Get the device serial.
    pub fn serial(&self) -> &str {
        &self.serial
    }

    // -- Shell ----------------------------------------------------------------

    /// Run a shell command, returning separated stdout, stderr, and exit code.
    ///
    /// ```no_run
    /// # use adb_wire::AdbWire;
    /// # async fn example() {
    /// let adb = AdbWire::new("emulator-5554");
    /// let result = adb.shell("ls /sdcard").await.unwrap();
    /// if result.success() {
    ///     println!("{}", result.stdout_str());
    /// } else {
    ///     eprintln!("exit {}: {}", result.exit_code, result.stderr_str());
    /// }
    /// # }
    /// ```
    pub async fn shell(&self, cmd: &str) -> Result<ShellOutput> {
        let stream = self.connect_shell(cmd).await?;
        self.with_timeout(shell::read_shell(stream)).await?
    }

    /// Run a shell command and return a streaming reader.
    ///
    /// The returned [`ShellStream`] implements [`AsyncRead`](tokio::io::AsyncRead)
    /// yielding stdout bytes. Use [`collect_output`](ShellStream::collect_output)
    /// to buffer everything, or read incrementally for long-running commands.
    ///
    /// ```no_run
    /// # use adb_wire::AdbWire;
    /// # async fn example() {
    /// use tokio::io::{AsyncBufReadExt, BufReader};
    ///
    /// let adb = AdbWire::new("emulator-5554");
    /// let stream = adb.shell_stream("logcat").await.unwrap();
    /// let mut lines = BufReader::new(stream).lines();
    /// while let Some(line) = lines.next_line().await.unwrap() {
    ///     println!("{line}");
    /// }
    /// # }
    /// ```
    pub async fn shell_stream(&self, cmd: &str) -> Result<ShellStream> {
        Ok(ShellStream::new(self.connect_shell(cmd).await?))
    }

    /// Send a shell command without waiting for output.
    ///
    /// Opens a shell connection and immediately drops it. The ADB server
    /// will still deliver the command to the device, but the command may
    /// not have *started* on the device by the time this method returns.
    ///
    /// **Warning:** Because the connection is dropped immediately, there is
    /// no guarantee the command will execute. If the device has not received
    /// the command before the TCP connection closes, it will be lost. Use
    /// [`shell`](Self::shell) if you need confirmation that the command ran.
    ///
    /// Best suited for input commands (tap, swipe, keyevent) where occasional
    /// drops are acceptable.
    pub async fn shell_detach(&self, cmd: &str) -> Result<()> {
        let mut stream = self.connect_shell(cmd).await?;
        // Wait briefly for the server to acknowledge, giving it time to
        // forward the command to the device before we drop the connection.
        let mut header = [0u8; 5];
        let _ = tokio::time::timeout(
            Duration::from_millis(50),
            tokio::io::AsyncReadExt::read_exact(&mut stream, &mut header),
        )
        .await;
        Ok(())
    }

    // -- File transfer (sync protocol) ----------------------------------------

    /// Query file metadata on the device.
    ///
    /// Returns a [`RemoteStat`] with mode, size, and modification time.
    /// If the path does not exist, all fields will be zero —
    /// check with [`RemoteStat::exists()`].
    ///
    /// ```no_run
    /// # use adb_wire::AdbWire;
    /// # async fn example() {
    /// let adb = AdbWire::new("emulator-5554");
    /// let st = adb.stat("/sdcard/Download/test.txt").await.unwrap();
    /// if st.exists() {
    ///     println!("size: {} bytes, mode: {:o}", st.size, st.mode);
    /// }
    /// # }
    /// ```
    pub async fn stat(&self, remote_path: &str) -> Result<RemoteStat> {
        // Try STAT2 first for large file / extended metadata support.
        let mut stream = self.connect_cmd("sync:").await?;
        match sync::stat2_sync(&mut stream, remote_path).await {
            Ok(st) => Ok(st),
            Err(_) => {
                // Fall back to STAT v1 on a fresh sync connection.
                let mut stream = self.connect_cmd("sync:").await?;
                sync::stat_v1_sync(&mut stream, remote_path).await
            }
        }
    }

    /// List files in a remote directory.
    ///
    /// Returns entries excluding `.` and `..`. Uses the sync LIST protocol.
    ///
    /// ```no_run
    /// # use adb_wire::AdbWire;
    /// # async fn example() {
    /// let adb = AdbWire::new("emulator-5554");
    /// for entry in adb.list_dir("/sdcard").await.unwrap() {
    ///     println!("{}\t{} bytes", entry.name, entry.size);
    /// }
    /// # }
    /// ```
    pub async fn list_dir(&self, remote_path: &str) -> Result<Vec<DirEntry>> {
        let mut stream = self.connect_cmd("sync:").await?;
        sync::list_sync(&mut stream, remote_path).await
    }

    /// Pull a file from the device, writing its contents to `writer`.
    ///
    /// Returns the number of bytes written.
    pub async fn pull(
        &self,
        remote_path: &str,
        writer: &mut (impl tokio::io::AsyncWrite + Unpin),
    ) -> Result<u64> {
        let mut stream = self.connect_cmd("sync:").await?;
        sync::pull_sync(&mut stream, remote_path, writer).await
    }

    /// Pull a file from the device to a local path.
    pub async fn pull_file(
        &self,
        remote_path: &str,
        local_path: impl AsRef<std::path::Path>,
    ) -> Result<u64> {
        let mut f = tokio::fs::File::create(local_path.as_ref()).await?;
        self.pull(remote_path, &mut f).await
    }

    /// Push data from `reader` to a file on the device.
    ///
    /// `mode` is the Unix file permission as an octal value (e.g. `0o644`,
    /// `0o755`). `mtime` is seconds since the Unix epoch.
    pub async fn push(
        &self,
        remote_path: &str,
        mode: u32,
        mtime: u32,
        reader: &mut (impl tokio::io::AsyncRead + Unpin),
    ) -> Result<()> {
        let mut stream = self.connect_cmd("sync:").await?;
        sync::push_sync(&mut stream, remote_path, mode, mtime, reader).await
    }

    /// Push a local file to the device with mode `0o644`.
    pub async fn push_file(
        &self,
        local_path: impl AsRef<std::path::Path>,
        remote_path: &str,
    ) -> Result<()> {
        self.push_file_with_mode(local_path, remote_path, 0o644).await
    }

    /// Push a local file to the device with a custom Unix file mode.
    ///
    /// ```no_run
    /// # use adb_wire::AdbWire;
    /// # async fn example() {
    /// let adb = AdbWire::new("emulator-5554");
    /// adb.push_file_with_mode("run.sh", "/data/local/tmp/run.sh", 0o755).await.unwrap();
    /// # }
    /// ```
    pub async fn push_file_with_mode(
        &self,
        local_path: impl AsRef<std::path::Path>,
        remote_path: &str,
        mode: u32,
    ) -> Result<()> {
        let path = local_path.as_ref();
        let meta = tokio::fs::metadata(path).await?;
        let mtime = meta.modified().ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_secs() as u32);
        let mut f = tokio::fs::File::open(path).await?;
        self.push(remote_path, mode, mtime, &mut f).await
    }

    // -- Install --------------------------------------------------------------

    /// Install an APK on the device. Equivalent to `adb install <path>`.
    ///
    /// ```no_run
    /// # use adb_wire::AdbWire;
    /// # async fn example() {
    /// let adb = AdbWire::new("emulator-5554");
    /// adb.install("app-debug.apk").await.unwrap();
    /// # }
    /// ```
    pub async fn install(&self, local_apk: impl AsRef<std::path::Path>) -> Result<()> {
        self.install_with_args(local_apk, &[]).await
    }

    /// Install an APK with additional `pm install` flags.
    ///
    /// ```no_run
    /// # use adb_wire::AdbWire;
    /// # async fn example() {
    /// let adb = AdbWire::new("emulator-5554");
    /// adb.install_with_args("app.apk", &["-r", "-d"]).await.unwrap();
    /// # }
    /// ```
    pub async fn install_with_args(
        &self,
        local_apk: impl AsRef<std::path::Path>,
        args: &[&str],
    ) -> Result<()> {
        let local = local_apk.as_ref();
        let filename = local
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("install.apk");

        // Reject filenames that could escape the shell or path.
        if filename.contains('/') || filename.contains('\0') || filename.contains('\'') {
            return Err(Error::Adb(format!("unsafe filename: {filename}")));
        }

        let remote = format!("/data/local/tmp/{filename}");
        self.push_file_with_mode(local, &remote, 0o644).await?;

        let mut cmd = String::from("pm install");
        for arg in args {
            cmd.push(' ');
            shell_quote_into(&mut cmd, arg);
        }
        cmd.push(' ');
        shell_quote_into(&mut cmd, &remote);

        let result = self.shell(&cmd).await?;

        // Best-effort cleanup of the temp APK.
        let cleanup_ok = self
            .shell(&format!("rm -f '{remote}'"))
            .await
            .map(|r| r.success())
            .unwrap_or(false);

        if result.success() {
            Ok(())
        } else {
            let output = if result.stderr.is_empty() {
                result.stdout_str()
            } else {
                result.stderr_str()
            };
            let mut msg = format!("pm install failed (exit {}): {output}", result.exit_code);
            if !cleanup_ok {
                msg.push_str(&format!(" (cleanup of {remote} also failed)"));
            }
            Err(Error::Adb(msg))
        }
    }

    // -- Port forwarding ------------------------------------------------------

    /// Set up port forwarding from a local socket to the device.
    ///
    /// `local` and `remote` are ADB socket specs, e.g. `"tcp:8080"`,
    /// `"localabstract:chrome_devtools_remote"`.
    pub async fn forward(&self, local: &str, remote: &str) -> Result<()> {
        let mut stream = self.connect_raw().await?;
        let cmd = format!("host-serial:{}:forward:{};{}", self.serial, local, remote);
        wire::send(&mut stream, &cmd).await?;
        wire::read_okay(&mut stream).await?;
        wire::read_okay(&mut stream).await?; // second OKAY after setup
        Ok(())
    }

    /// Set up reverse port forwarding from the device to a local socket.
    pub async fn reverse(&self, remote: &str, local: &str) -> Result<()> {
        let mut stream = self.connect_transport().await?;
        let cmd = format!("reverse:forward:{remote};{local}");
        wire::send(&mut stream, &cmd).await?;
        wire::read_okay(&mut stream).await?;
        Ok(())
    }

    // -- Connection internals -------------------------------------------------

    async fn connect_transport(&self) -> Result<TcpStream> {
        let mut stream = self.connect_raw().await?;
        wire::send(&mut stream, &format!("host:transport:{}", self.serial)).await?;
        wire::read_okay(&mut stream).await?;
        Ok(stream)
    }

    async fn connect_cmd(&self, cmd: &str) -> Result<TcpStream> {
        let mut stream = self.connect_transport().await?;
        wire::send(&mut stream, cmd).await?;
        wire::read_okay(&mut stream).await?;
        Ok(stream)
    }

    async fn connect_shell(&self, cmd: &str) -> Result<TcpStream> {
        self.connect_cmd(&format!("shell,v2,raw:{cmd}")).await
    }

    async fn with_timeout<F: std::future::Future>(&self, f: F) -> Result<F::Output> {
        tokio::time::timeout(self.read_timeout, f)
            .await
            .map_err(|_| Error::timed_out("read timed out"))
    }

    async fn connect_raw(&self) -> Result<TcpStream> {
        let stream = tokio::time::timeout(
            self.connect_timeout,
            TcpStream::connect(self.server),
        )
        .await
        .map_err(|_| Error::timed_out("connect timed out"))??;
        stream.set_nodelay(true)?;
        Ok(stream)
    }
}

// -- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn new_defaults() {
        let adb = AdbWire::new("emulator-5554");
        assert_eq!(adb.serial(), "emulator-5554");
        assert_eq!(adb.server, DEFAULT_SERVER);
        assert_eq!(adb.read_timeout, Duration::from_secs(30));
    }

    #[test]
    fn builder() {
        let addr = SocketAddr::from(([192, 168, 1, 100], 5037));
        let adb = AdbWire::new("device123")
            .server(addr)
            .connect_timeout(Duration::from_millis(500))
            .read_timeout(Duration::from_secs(60));
        assert_eq!(adb.server, addr);
        assert_eq!(adb.connect_timeout, Duration::from_millis(500));
        assert_eq!(adb.read_timeout, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn send_format() {
        let mut buf = Vec::new();
        wire::send(&mut buf, "host:version").await.unwrap();
        assert_eq!(&buf, b"000chost:version");
    }

    #[tokio::test]
    async fn okay_response() {
        let mut cur = Cursor::new(b"OKAY".to_vec());
        assert!(wire::read_okay(&mut cur).await.is_ok());
    }

    #[tokio::test]
    async fn fail_response() {
        let mut cur = Cursor::new(b"FAIL0005nope!".to_vec());
        let err = wire::read_okay(&mut cur).await.unwrap_err();
        assert!(matches!(err, Error::Adb(msg) if msg == "nope!"));
    }

    #[tokio::test]
    async fn hex_len() {
        let mut cur = Cursor::new(b"001f".to_vec());
        assert_eq!(wire::read_hex_len(&mut cur).await.unwrap(), 31);
    }

    #[tokio::test]
    async fn bad_status() {
        let mut cur = Cursor::new(b"WHAT".to_vec());
        assert!(matches!(
            wire::read_okay(&mut cur).await.unwrap_err(),
            Error::Protocol(_)
        ));
    }

    #[test]
    fn parse_devices() {
        let raw = "emulator-5554\tdevice\nR5CR1234567\toffline\n";
        let devices = parse_device_list(raw);
        assert_eq!(
            devices,
            vec![
                ("emulator-5554".into(), "device".into()),
                ("R5CR1234567".into(), "offline".into()),
            ]
        );
    }

    #[test]
    fn parse_devices_empty() {
        assert!(parse_device_list("").is_empty());
        assert!(parse_device_list("\n").is_empty());
    }

    #[test]
    fn shell_quote_simple() {
        let mut out = String::new();
        shell_quote_into(&mut out, "hello");
        assert_eq!(out, "'hello'");
    }

    #[test]
    fn shell_quote_with_spaces_and_semicolons() {
        let mut out = String::new();
        shell_quote_into(&mut out, "foo; rm -rf /");
        assert_eq!(out, "'foo; rm -rf /'");
    }

    #[test]
    fn shell_quote_with_single_quotes() {
        let mut out = String::new();
        shell_quote_into(&mut out, "it's");
        assert_eq!(out, "'it'\\''s'");
    }
}
