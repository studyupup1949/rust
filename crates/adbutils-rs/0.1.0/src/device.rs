//! Device operations. Port of `BaseDevice` (`_device_base.py`).
//!
//! M1 slice: transport establishment, `shell`, and the `get-*` host-serial
//! commands. Forward/reverse, `shell2`, sync, framebuffer, and `create_connection`
//! land in M2; the shell helpers (`app_*`, battery, …) in M3.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::client::AdbClient;
use crate::conn::AdbConnection;
use crate::errors::{AdbError, Result};
use crate::proto::{ForwardItem, Network, ReverseItem, ShellReturn, ShellReturnRaw};
use crate::utils::{self, CmdArgs};

/// Default per-operation socket timeout (`_DEFAULT_SOCKET_TIMEOUT = 600`).
pub const DEFAULT_SOCKET_TIMEOUT: Duration = Duration::from_secs(600);

/// A handle to a single device. Addressed by serial or transport id.
#[derive(Debug, Clone)]
pub struct AdbDevice {
    client: AdbClient,
    serial: Option<String>,
    transport_id: Option<i64>,
    /// Cached feature set (`get_features`), shared across clones.
    features: Arc<Mutex<Option<HashSet<String>>>>,
}

impl AdbDevice {
    /// Construct for an explicit serial.
    pub fn with_serial(client: AdbClient, serial: String) -> Self {
        Self {
            client,
            serial: Some(serial),
            transport_id: None,
            features: Arc::new(Mutex::new(None)),
        }
    }

    /// Construct addressed by transport id.
    pub fn with_transport_id(client: AdbClient, transport_id: i64) -> Self {
        Self {
            client,
            serial: None,
            transport_id: Some(transport_id),
            features: Arc::new(Mutex::new(None)),
        }
    }

    /// The serial this handle was created with, if any.
    pub fn serial(&self) -> Option<&str> {
        self.serial.as_deref()
    }

    /// The owning client.
    pub fn client(&self) -> &AdbClient {
        &self.client
    }

    /// Open a connection to the device.
    ///
    /// With `command`: a host-serial / host-transport-id one-shot that stays in
    /// host context. Without: a transport switch into device context for a
    /// following local service (`shell:`, `sync:`, …). On adb server ≥ 41 the
    /// `host:tport:serial:` path discards the trailing 8-byte transport id.
    /// Mirrors `open_transport` (`_device_base.py:68`).
    pub async fn open_transport(
        &self,
        command: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<AdbConnection> {
        let mut c = self.client.make_connection(timeout).await?;
        match command {
            Some(cmd) => {
                if let Some(tid) = self.transport_id {
                    c.send_command(&format!("host-transport-id:{tid}:{cmd}")).await?;
                    c.check_okay().await?;
                } else if let Some(serial) = &self.serial {
                    c.send_command(&format!("host-serial:{serial}:{cmd}")).await?;
                    c.check_okay().await?;
                } else {
                    return Err(AdbError::adb("serial or transport_id must be set"));
                }
            }
            None => {
                if let Some(tid) = self.transport_id {
                    c.send_command(&format!("host:transport-id:{tid}")).await?;
                    c.check_okay().await?;
                } else if let Some(serial) = &self.serial {
                    if self.client.server_version().await? >= 41 {
                        c.send_command(&format!("host:tport:serial:{serial}")).await?;
                        c.check_okay().await?;
                        c.read(8).await?; // discard transport id
                    } else {
                        c.send_command(&format!("host:transport:{serial}")).await?;
                        c.check_okay().await?;
                    }
                } else {
                    return Err(AdbError::adb("serial or transport_id must be set"));
                }
            }
        }
        Ok(c)
    }

    /// Run a host-serial command and read its reply block (`_get_with_command`).
    async fn get_with_command(&self, cmd: &str) -> Result<String> {
        let mut c = self.open_transport(Some(cmd), Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        c.read_string_block().await
    }

    /// Device state: `offline` / `bootloader` / `device`.
    pub async fn get_state(&self) -> Result<String> {
        self.get_with_command("get-state").await
    }

    /// The device's real serial (`get-serialno`).
    pub async fn get_serialno(&self) -> Result<String> {
        self.get_with_command("get-serialno").await
    }

    /// The device path (`get-devpath`).
    pub async fn get_devpath(&self) -> Result<String> {
        self.get_with_command("get-devpath").await
    }

    /// Comma-separated feature list (`features`). Also refreshes the cache.
    pub async fn get_features(&self) -> Result<String> {
        let features = self.get_with_command("features").await?;
        let set: HashSet<String> = features.split(',').map(str::to_string).collect();
        *self.features.lock().await = Some(set);
        Ok(features)
    }

    /// True if the device advertises `feature`, fetching the set once and caching.
    async fn has_feature(&self, feature: &str) -> Result<bool> {
        {
            let guard = self.features.lock().await;
            if let Some(set) = guard.as_ref() {
                return Ok(set.contains(feature));
            }
        }
        self.get_features().await?;
        let guard = self.features.lock().await;
        Ok(guard.as_ref().map(|s| s.contains(feature)).unwrap_or(false))
    }

    /// Open a live shell connection (`open_shell`): transport switch then
    /// `shell:<cmd>`.
    pub async fn open_shell(&self, cmdargs: impl Into<CmdArgs>) -> Result<AdbConnection> {
        let cmdline = cmdargs.into().to_cmdline();
        let mut c = self.open_transport(None, Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        c.send_command(&format!("shell:{cmdline}")).await?;
        c.check_okay().await?;
        Ok(c)
    }

    /// Run a shell command and return decoded stdout, right-stripped (`shell`,
    /// default `encoding="utf-8"`, `rstrip=True`).
    pub async fn shell(&self, cmdargs: impl Into<CmdArgs>) -> Result<String> {
        let mut c = self.open_shell(cmdargs).await?;
        let out = c.read_until_close().await?;
        Ok(out.trim_end().to_string())
    }

    /// Run a shell command and return raw stdout bytes (`shell(encoding=None)`).
    pub async fn shell_bytes(&self, cmdargs: impl Into<CmdArgs>) -> Result<Vec<u8>> {
        let mut c = self.open_shell(cmdargs).await?;
        c.read_until_close_bytes().await
    }

    /// Open a shell and hand back the live connection to stream output
    /// (`shell(stream=True)`).
    pub async fn shell_stream(&self, cmdargs: impl Into<CmdArgs>) -> Result<AdbConnection> {
        self.open_shell(cmdargs).await
    }

    /// Run a shell command and return a structured result with exit code
    /// (`shell2`). Uses shell v2 when `v2` is requested and supported, else the
    /// v1 `X4EXIT:` marker trick. Output is decoded UTF-8 (lossy).
    pub async fn shell2(&self, cmdargs: impl Into<CmdArgs>, v2: bool) -> Result<ShellReturn> {
        let cmdline = cmdargs.into().to_cmdline();
        let raw = self.shell2_raw(&cmdline, v2).await?;
        let decode = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
        Ok(ShellReturn {
            command: raw.command,
            returncode: raw.returncode,
            output: decode(&raw.output),
            stderr: decode(&raw.stderr),
            stdout: decode(&raw.stdout),
        })
    }

    /// Like [`shell2`](Self::shell2) but returns raw bytes
    /// (`shell2(encoding=None)`).
    pub async fn shell2_raw(&self, cmdargs: impl Into<CmdArgs>, mut v2: bool) -> Result<ShellReturnRaw> {
        let cmdline = cmdargs.into().to_cmdline();
        if v2 && !self.has_feature("shell_v2").await? {
            log::warn!("shell_v2 specified but not supported by device");
            v2 = false;
        }
        if v2 {
            self.shell_v2(&cmdline).await
        } else {
            self.shell_v1(&cmdline).await
        }
    }

    /// v1 exit-code trick: append `; echo X4EXIT:$?` and parse the trailing
    /// marker (`_shell_v1`). Low-level; prefer [`shell2`](Self::shell2).
    pub async fn shell_v1(&self, cmdline: &str) -> Result<ShellReturnRaw> {
        const MAGIC: &str = "X4EXIT:";
        let newcmd = format!("{cmdline}; echo {MAGIC}$?");
        let output = self.shell_bytes(newcmd).await?;
        let rindex = find_last(&output, MAGIC.as_bytes())
            .ok_or_else(|| AdbError::adb("shell output invalid"))?;
        let code_bytes = &output[rindex + MAGIC.len()..];
        let returncode: i32 = String::from_utf8_lossy(code_bytes)
            .trim()
            .parse()
            .map_err(|_| AdbError::adb("shell output invalid"))?;
        let output = output[..rindex].to_vec();
        Ok(ShellReturnRaw {
            command: cmdline.to_string(),
            returncode,
            stdout: Vec::new(),
            stderr: Vec::new(),
            output,
        })
    }

    /// v2 binary framing: 5-byte header (1-byte id + 4-byte LE length) + payload;
    /// ids 1/2/3 = stdout/stderr/exit (`_shell_v2`). Low-level; prefer
    /// [`shell2`](Self::shell2), which negotiates support first.
    pub async fn shell_v2(&self, cmdline: &str) -> Result<ShellReturnRaw> {
        let mut c = self.open_transport(None, Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        c.send_command(&format!("shell,v2:{cmdline}")).await?;
        c.check_okay().await?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut output = Vec::new();
        loop {
            let header = c.read_exact(5).await?;
            let msg_id = header[0];
            let length = u32::from_le_bytes([header[1], header[2], header[3], header[4]]) as usize;
            if length == 0 {
                continue;
            }
            let data = c.read_exact(length).await?;
            match msg_id {
                1 => {
                    stdout.extend_from_slice(&data);
                    output.extend_from_slice(&data);
                }
                2 => {
                    stderr.extend_from_slice(&data);
                    output.extend_from_slice(&data);
                }
                3 => {
                    return Ok(ShellReturnRaw {
                        command: cmdline.to_string(),
                        returncode: data[0] as i32,
                        stdout,
                        stderr,
                        output,
                    });
                }
                _ => {}
            }
        }
    }

    // ---- forward -------------------------------------------------------------

    /// `forward[:norebind]:<local>;<remote>`.
    pub async fn forward(&self, local: &str, remote: &str, norebind: bool) -> Result<()> {
        let mut cmd = String::from("forward");
        if norebind {
            cmd.push_str(":norebind");
        }
        cmd.push_str(&format!(":{local};{remote}"));
        self.open_transport(Some(&cmd), Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        Ok(())
    }

    /// Forward a random free local tcp port to `remote`, reusing an existing
    /// forward if present. Returns the local port (`forward_port`).
    pub async fn forward_port(&self, remote: impl Into<String>) -> Result<u16> {
        let remote = remote.into();
        let remote = if remote.parse::<u16>().is_ok() {
            format!("tcp:{remote}")
        } else {
            remote
        };
        for f in self.forward_list().await? {
            if Some(f.serial.as_str()) == self.serial.as_deref()
                && f.remote == remote
                && f.local.starts_with("tcp:")
            {
                if let Ok(p) = f.local["tcp:".len()..].parse::<u16>() {
                    return Ok(p);
                }
            }
        }
        let local_port = utils::get_free_port()?;
        self.forward(&format!("tcp:{local_port}"), &remote, false).await?;
        Ok(local_port)
    }

    /// This device's `list-forward` entries.
    pub async fn forward_list(&self) -> Result<Vec<ForwardItem>> {
        let mut c = self.open_transport(Some("list-forward"), Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        let content = c.read_string_block().await?;
        let mut items = Vec::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 3 {
                continue;
            }
            items.push(ForwardItem {
                serial: parts[0].to_string(),
                local: parts[1].to_string(),
                remote: parts[2].to_string(),
            });
        }
        Ok(items)
    }

    /// Remove a forward by local address (`killforward:<local>`).
    pub async fn forward_remove(&self, local: &str, raise_non_found: bool) -> Result<()> {
        match self
            .open_transport(Some(&format!("killforward:{local}")), Some(DEFAULT_SOCKET_TIMEOUT))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) if raise_non_found => Err(e),
            Err(_) => Ok(()),
        }
    }

    /// Remove all forwards (`killforward-all`).
    pub async fn forward_remove_all(&self) -> Result<()> {
        self.open_transport(Some("killforward-all"), Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        Ok(())
    }

    // ---- reverse (device-context local service, double OKAY) -----------------

    /// `reverse:forward[:norebind]:<remote>;<local>`.
    pub async fn reverse(&self, remote: &str, local: &str, norebind: bool) -> Result<()> {
        let mut c = self.open_transport(None, Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        let mut cmd = String::from("reverse:forward");
        if norebind {
            cmd.push_str(":norebind");
        }
        cmd.push_str(&format!(":{remote};{local}"));
        c.send_command(&cmd).await?;
        c.check_okay().await?; // receipt
        c.check_okay().await // response
    }

    /// Remove all reverse connections (`reverse:killforward-all`).
    pub async fn reverse_remove_all(&self) -> Result<()> {
        let mut c = self.open_transport(None, Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        c.send_command("reverse:killforward-all").await?;
        c.check_okay().await?;
        c.check_okay().await
    }

    /// Remove a reverse by remote address (`reverse:killforward:<remote>`).
    pub async fn reverse_remove(&self, remote: &str) -> Result<()> {
        let mut c = self.open_transport(None, Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        c.send_command(&format!("reverse:killforward:{remote}")).await?;
        c.check_okay().await?;
        c.check_okay().await
    }

    /// `reverse:list-forward` entries.
    pub async fn reverse_list(&self) -> Result<Vec<ReverseItem>> {
        let mut c = self.open_transport(None, Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        c.send_command("reverse:list-forward").await?;
        c.check_okay().await?;
        let content = c.read_string_block().await?;
        let mut items = Vec::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 3 {
                continue;
            }
            // parts[0] is the (ignored) serial; remote, local follow.
            items.push(ReverseItem {
                remote: parts[1].to_string(),
                local: parts[2].to_string(),
            });
        }
        Ok(items)
    }

    // ---- raw socket / adbd control -------------------------------------------

    /// Open a socket to a device-side service and hand back the raw stream
    /// (`create_connection`). The smartsocket becomes a transparent pipe.
    pub async fn create_connection(
        &self,
        network: Network,
        address: &str,
    ) -> Result<TcpStream> {
        let mut c = self.open_transport(None, Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        let service = match network {
            Network::Tcp => format!("tcp:{address}"),
            Network::Unix | Network::LocalAbstract => format!("localabstract:{address}"),
            Network::LocalFilesystem
            | Network::Local
            | Network::Dev
            | Network::LocalReserved => format!("{}:{address}", network.as_str()),
        };
        c.send_command(&service).await?;
        c.check_okay().await?;
        c.into_stream()
    }

    /// Restart adbd as root (`root:`). Returns the server message.
    pub async fn root(&self) -> Result<String> {
        let mut c = self.open_transport(None, Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        c.send_command("root:").await?;
        c.check_okay().await?;
        c.read_until_close().await
    }

    /// Restart adbd listening on TCP `port` (`tcpip:<port>`).
    pub async fn tcpip(&self, port: u16) -> Result<String> {
        let mut c = self.open_transport(None, Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        c.send_command(&format!("tcpip:{port}")).await?;
        c.check_okay().await?;
        c.read_until_close().await
    }

    /// Capture the screen via the raw `framebuffer:` protocol (not very stable;
    /// prefer [`screenshot`](Self::screenshot)). Reads the framebuffer header of
    /// LE-u32 fields, then the raw pixel buffer. Port of `framebuffer`.
    #[cfg(feature = "image")]
    pub async fn framebuffer(&self) -> Result<image::RgbaImage> {
        let mut c = self.open_transport(None, Some(DEFAULT_SOCKET_TIMEOUT)).await?;
        c.send_command("framebuffer:").await?;
        c.check_okay().await?;

        let version = c.read_u32_le().await?;
        if version == 16 {
            return Err(AdbError::adb("Unsupported framebuffer version 16"));
        }
        let bpp = c.read_u32_le().await?;
        if bpp != 24 && bpp != 32 {
            return Err(AdbError::adb(format!("Unsupported bpp: {bpp}")));
        }
        let mut size = c.read_u32_le().await?;
        if size == 1 {
            size = c.read_u32_le().await?;
        }
        let width = c.read_u32_le().await?;
        let height = c.read_u32_le().await?;
        let _red_offset = c.read_u32_le().await?;
        let _red_length = c.read_u32_le().await?;
        let blue_offset = c.read_u32_le().await?;
        let _blue_length = c.read_u32_le().await?;
        let _green_offset = c.read_u32_le().await?;
        let _green_length = c.read_u32_le().await?;
        let _alpha_offset = c.read_u32_le().await?;
        let alpha_length = c.read_u32_le().await?;

        let has_alpha = bpp == 32 || alpha_length != 0;
        let is_bgr = blue_offset == 0;
        let buffer = c.read_exact(size as usize).await?;

        // Normalize to RGBA regardless of source channel order.
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        let px = if has_alpha { 4 } else { 3 };
        for chunk in buffer.chunks_exact(px) {
            let (r, g, b) = if is_bgr {
                (chunk[2], chunk[1], chunk[0])
            } else {
                (chunk[0], chunk[1], chunk[2])
            };
            let a = if has_alpha { chunk[3] } else { 255 };
            rgba.extend_from_slice(&[r, g, b, a]);
        }
        image::RgbaImage::from_raw(width, height, rgba)
            .ok_or_else(|| AdbError::adb("framebuffer size not match"))
    }
}

/// Find the last occurrence of `needle` in `haystack`.
fn find_last(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    (0..=haystack.len() - needle.len())
        .rev()
        .find(|&i| &haystack[i..i + needle.len()] == needle)
}
