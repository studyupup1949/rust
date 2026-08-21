//! Wire data types. Port of `adbutils/_proto.py`.

use std::collections::HashMap;

use chrono::{DateTime, Local};

/// Local service network kinds for `create_connection`. The `Display` value is
/// the on-wire token used in the local-service string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Tcp,
    Unix,
    Dev,
    Local,
    LocalReserved,
    LocalFilesystem,
    /// Same as `Unix`.
    LocalAbstract,
}

impl Network {
    /// On-wire token (matches the Python enum values).
    pub fn as_str(self) -> &'static str {
        match self {
            Network::Tcp => "tcp",
            Network::Unix => "unix",
            Network::Dev => "dev",
            Network::Local => "local",
            Network::LocalReserved => "localreserved",
            Network::LocalFilesystem => "localfilesystem",
            Network::LocalAbstract => "localabstract",
        }
    }
}

/// Screen brightness mode (`settings ... screen_brightness_mode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrightnessMode {
    Auto = 1,
    Manual = 0,
}

/// A single device-state transition from `track-devices`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceEvent {
    pub present: bool,
    pub serial: String,
    pub status: String,
}

/// One `adb forward` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardItem {
    pub serial: String,
    pub local: String,
    pub remote: String,
}

/// One `adb reverse` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReverseItem {
    pub remote: String,
    pub local: String,
}

/// Result of a sync `STAT` / directory `LIST` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInfo {
    pub mode: u32,
    pub size: u32,
    /// `None` when the file does not exist (mtime == 0).
    pub mtime: Option<DateTime<Local>>,
    pub path: String,
}

/// Parsed application info (`pm path` + `dumpsys package`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppInfo {
    pub package_name: String,
    pub version_name: Option<String>,
    pub version_code: Option<i64>,
    pub flags: Vec<String>,
    pub first_install_time: Option<DateTime<Local>>,
    pub last_update_time: Option<DateTime<Local>>,
    pub signature: Option<String>,
    pub path: Option<String>,
    pub sub_apk_paths: Vec<String>,
}

/// Parsed `dumpsys battery`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BatteryInfo {
    pub ac_powered: bool,
    pub usb_powered: bool,
    pub wireless_powered: Option<bool>,
    pub dock_powered: Option<bool>,
    pub max_charging_current: Option<i64>,
    pub max_charging_voltage: Option<i64>,
    pub charge_counter: Option<i64>,
    pub status: Option<i64>,
    pub health: Option<i64>,
    pub present: Option<bool>,
    pub level: Option<i64>,
    pub scale: Option<i64>,
    /// millivolts
    pub voltage: Option<i64>,
    pub temperature: Option<f64>,
    pub technology: Option<String>,
}

/// Screen dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

/// Foreground app info (`app_current`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningAppInfo {
    pub package: String,
    pub activity: String,
    pub pid: i64,
}

/// Structured shell result with decoded text (`shell2`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellReturn {
    pub command: String,
    pub returncode: i32,
    pub output: String,
    pub stderr: String,
    pub stdout: String,
}

/// Structured shell result with raw bytes (`shell2`, `encoding=None`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellReturnRaw {
    pub command: String,
    pub returncode: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub output: Vec<u8>,
}

/// A device row from `host:devices` / `host:devices-l`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdbDeviceInfo {
    pub serial: String,
    pub state: String,
    pub tags: HashMap<String, String>,
}

impl AdbDeviceInfo {
    /// Transport id parsed from the extended `transport_id` tag, if present.
    pub fn transport_id(&self) -> Option<i64> {
        self.tags.get("transport_id").and_then(|v| v.parse().ok())
    }
}
