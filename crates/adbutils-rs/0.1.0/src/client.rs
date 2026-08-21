//! adb server host commands. Port of `BaseClient` (`_adb.py`) + `AdbClient`
//! (`__init__.py`).

use std::collections::HashSet;
use std::time::Duration;

use futures_core::Stream;

use crate::conn::AdbConnection;
use crate::device::AdbDevice;
use crate::errors::{AdbError, Result};
use crate::proto::{AdbDeviceInfo, DeviceEvent, ForwardItem};

/// Client handle for the local adb server. Cheap to clone (just host/port/timeout);
/// every command opens its own connection.
#[derive(Debug, Clone)]
pub struct AdbClient {
    host: String,
    port: u16,
    socket_timeout: Option<Duration>,
}

impl Default for AdbClient {
    /// Host/port from `ANDROID_ADB_SERVER_HOST` / `ANDROID_ADB_SERVER_PORT`,
    /// defaulting to `127.0.0.1:5037`.
    fn default() -> Self {
        let host = std::env::var("ANDROID_ADB_SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".into());
        let port = std::env::var("ANDROID_ADB_SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(5037);
        Self { host, port, socket_timeout: None }
    }
}

impl AdbClient {
    /// Construct with explicit host/port/timeout.
    pub fn new(host: impl Into<String>, port: u16, socket_timeout: Option<Duration>) -> Self {
        Self { host: host.into(), port, socket_timeout }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Open a fresh connection to the adb server (`make_connection`).
    pub async fn make_connection(&self, timeout: Option<Duration>) -> Result<AdbConnection> {
        let timeout = timeout.or(self.socket_timeout);
        AdbConnection::connect(&self.host, self.port, timeout).await
    }

    /// `host:version` → server version int (e.g. 41 for 1.0.41).
    pub async fn server_version(&self) -> Result<u32> {
        let mut c = self.make_connection(None).await?;
        c.send_command("host:version").await?;
        c.check_okay().await?;
        let block = c.read_string_block().await?;
        u32::from_str_radix(block.trim(), 16)
            .map_err(|_| AdbError::adb(format!("bad version block: {block:?}")))
    }

    /// `host:kill` — stop the adb server if running.
    pub async fn server_kill(&self) -> Result<()> {
        // Probe first (matches `_check_server`); if unreachable, nothing to do.
        if self.make_connection(Some(Duration::from_millis(100))).await.is_err() {
            return Ok(());
        }
        let mut c = self.make_connection(None).await?;
        c.send_command("host:kill").await?;
        c.check_okay().await
    }

    /// `wait-for-<transport>-<state>` (double OKAY).
    pub async fn wait_for(
        &self,
        serial: Option<&str>,
        transport: &str,
        state: &str,
        timeout: Duration,
    ) -> Result<()> {
        let mut c = self.make_connection(Some(timeout)).await?;
        let cmd = match serial {
            Some(s) => format!("host-serial:{s}:wait-for-{transport}-{state}"),
            None => format!("host:wait-for-{transport}-{state}"),
        };
        c.send_command(&cmd).await?;
        c.check_okay().await?;
        c.check_okay().await
    }

    /// `host:connect:<addr>` → server message.
    pub async fn connect(&self, addr: &str, timeout: Option<Duration>) -> Result<String> {
        let mut c = self.make_connection(timeout).await?;
        c.send_command(&format!("host:connect:{addr}")).await?;
        c.check_okay().await?;
        c.read_string_block().await
    }

    /// `host:disconnect:<addr>` → server message. Swallows errors unless
    /// `raise_error`.
    pub async fn disconnect(&self, addr: &str, raise_error: bool) -> Result<Option<String>> {
        let res = async {
            let mut c = self.make_connection(None).await?;
            c.send_command(&format!("host:disconnect:{addr}")).await?;
            c.check_okay().await?;
            c.read_string_block().await
        }
        .await;
        match res {
            Ok(s) => Ok(Some(s)),
            Err(e) if raise_error => Err(e),
            Err(_) => Ok(None),
        }
    }

    /// `host:list-forward` (or per-serial) → forward entries.
    pub async fn forward_list(&self, serial: Option<&str>) -> Result<Vec<ForwardItem>> {
        let mut c = self.make_connection(None).await?;
        let cmd = match serial {
            Some(s) => format!("host-serial:{s}:list-forward"),
            None => "host:list-forward".to_string(),
        };
        c.send_command(&cmd).await?;
        c.check_okay().await?;
        let content = c.read_string_block().await?;
        let mut items = Vec::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 3 {
                continue;
            }
            if let Some(s) = serial {
                if parts[0] != s {
                    continue;
                }
            }
            items.push(ForwardItem {
                serial: parts[0].to_string(),
                local: parts[1].to_string(),
                remote: parts[2].to_string(),
            });
        }
        Ok(items)
    }

    /// `host-serial:<serial>:forward[:norebind]:<local>;<remote>`.
    pub async fn forward(
        &self,
        serial: &str,
        local: &str,
        remote: &str,
        norebind: bool,
    ) -> Result<()> {
        let mut c = self.make_connection(None).await?;
        let rebind = if norebind { "forward:norebind:" } else { "forward:" };
        c.send_command(&format!("host-serial:{serial}:{rebind}{local};{remote}"))
            .await?;
        c.check_okay().await
    }

    /// `host:devices` / `host:devices-l` → all device rows (including offline).
    pub async fn list(&self, extended: bool) -> Result<Vec<AdbDeviceInfo>> {
        let mut c = self.make_connection(None).await?;
        c.send_command(if extended { "host:devices-l" } else { "host:devices" })
            .await?;
        c.check_okay().await?;
        let output = c.read_string_block().await?;
        let mut infos = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }
            let mut tags = std::collections::HashMap::new();
            if extended {
                for part in &parts[2..] {
                    if let Some((k, v)) = part.split_once(':') {
                        tags.insert(k.to_string(), v.to_string());
                    }
                }
            }
            infos.push(AdbDeviceInfo {
                serial: parts[0].to_string(),
                state: parts[1].to_string(),
                tags,
            });
        }
        Ok(infos)
    }

    /// Devices in state `device`, as [`AdbDevice`] handles (`device_list`).
    pub async fn device_list(&self) -> Result<Vec<AdbDevice>> {
        let infos = self.list(false).await?;
        Ok(infos
            .into_iter()
            .filter(|i| i.state == "device")
            .map(|i| AdbDevice::with_serial(self.clone(), i.serial))
            .collect())
    }

    /// Device handle for an explicit serial (no IO).
    pub fn device(&self, serial: impl Into<String>) -> AdbDevice {
        AdbDevice::with_serial(self.clone(), serial.into())
    }

    /// Device handle addressed by transport id (no IO).
    pub fn device_with_transport_id(&self, transport_id: i64) -> AdbDevice {
        AdbDevice::with_transport_id(self.clone(), transport_id)
    }

    /// Resolve the default device: `ANDROID_SERIAL`, else the sole connected
    /// device (error if none / more than one). Mirrors `device(None)`.
    pub async fn any_device(&self) -> Result<AdbDevice> {
        if let Ok(serial) = std::env::var("ANDROID_SERIAL") {
            if !serial.is_empty() {
                return Ok(self.device(serial));
            }
        }
        let mut ds = self.device_list().await?;
        match ds.len() {
            0 => Err(AdbError::adb("Can't find any android device/emulator")),
            1 => Ok(ds.pop().unwrap()),
            _ => Err(AdbError::adb(
                "more than one device/emulator, please specify the serial number",
            )),
        }
    }

    /// `host:track-devices` → a stream of state transitions. Ends (with an error
    /// item) when the adb server dies.
    pub fn track_devices(&self) -> impl Stream<Item = Result<DeviceEvent>> + '_ {
        async_stream::try_stream! {
            let mut c = self.make_connection(None).await?;
            c.send_command("host:track-devices").await?;
            c.check_okay().await?;
            let mut orig: Vec<DeviceEvent> = Vec::new();
            loop {
                let output = c.read_string_block().await?;
                let curr = output2devices(&output);
                for ev in diff_devices(&orig, &curr) {
                    yield ev;
                }
                orig = curr;
            }
        }
    }
}

/// Parse a `track-devices` block: `serial\tstatus` lines.
fn output2devices(output: &str) -> Vec<DeviceEvent> {
    let mut devices = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if let Some((serial, status)) = line.split_once('\t') {
            devices.push(DeviceEvent {
                present: true,
                serial: serial.to_string(),
                status: status.to_string(),
            });
        }
    }
    devices
}

/// Set-difference between snapshots → present/absent events (`_diff_devices`).
fn diff_devices(orig: &[DeviceEvent], curr: &[DeviceEvent]) -> Vec<DeviceEvent> {
    let key = |d: &DeviceEvent| (d.serial.clone(), d.status.clone());
    let orig_set: HashSet<_> = orig.iter().map(key).collect();
    let curr_set: HashSet<_> = curr.iter().map(key).collect();
    let mut events = Vec::new();
    for d in orig {
        if !curr_set.contains(&key(d)) {
            events.push(DeviceEvent {
                present: false,
                serial: d.serial.clone(),
                status: "absent".to_string(),
            });
        }
    }
    for d in curr {
        if !orig_set.contains(&key(d)) {
            events.push(DeviceEvent {
                present: true,
                serial: d.serial.clone(),
                status: d.status.clone(),
            });
        }
    }
    events
}
