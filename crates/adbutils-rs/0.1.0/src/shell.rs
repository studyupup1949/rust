//! High-level shell helpers. Port of `adbutils/shell.py` (`ShellExtension`).
//!
//! Every method here drives plain `shell` (text output) and parses the result
//! with the same regexes as the Python library.

use std::sync::LazyLock;
use std::time::Duration;

use chrono::{Local, NaiveDateTime, TimeZone};
use regex::Regex;

use crate::device::AdbDevice;
use crate::errors::{AdbError, Result};
use crate::proto::{AppInfo, BatteryInfo, BrightnessMode, RunningAppInfo, WindowSize};

/// True if `v` is a fractional coordinate (`<= 1.0`), matching `is_percent`.
fn is_percent(v: f64) -> bool {
    v <= 1.0
}

impl AdbDevice {
    /// `getprop <prop>`.
    pub async fn getprop(&self, prop: &str) -> Result<String> {
        Ok(self.shell(["getprop", prop]).await?.trim().to_string())
    }

    /// `input keyevent <key_code>`.
    pub async fn keyevent(&self, key_code: &str) -> Result<()> {
        self.shell(["input", "keyevent", key_code]).await?;
        Ok(())
    }

    /// Increase volume `times` steps (waking the volume bar).
    pub async fn volume_up(&self, times: u32) -> Result<()> {
        for _ in 0..times {
            self.shell("input keyevent VOLUME_UP").await?;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Ok(())
    }

    /// Decrease volume `times` steps.
    pub async fn volume_down(&self, times: u32) -> Result<()> {
        for _ in 0..times {
            self.shell("input keyevent VOLUME_DOWN").await?;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Ok(())
    }

    /// Mute volume.
    pub async fn volume_mute(&self) -> Result<()> {
        self.shell("input keyevent VOLUME_MUTE").await?;
        Ok(())
    }

    /// `reboot`.
    pub async fn reboot(&self) -> Result<()> {
        self.shell("reboot").await?;
        Ok(())
    }

    /// Turn the screen on (`true`) or off (`false`).
    pub async fn switch_screen(&self, enable: bool) -> Result<()> {
        self.keyevent(if enable { "224" } else { "223" }).await
    }

    /// Screen brightness value `[0, 255]`.
    pub async fn brightness_value(&self) -> Result<i32> {
        let v = self.shell("settings get system screen_brightness").await?;
        v.trim()
            .parse()
            .map_err(|_| AdbError::adb(format!("bad brightness: {v:?}")))
    }

    /// Set screen brightness value `[0, 255]`.
    pub async fn set_brightness_value(&self, value: i32) -> Result<()> {
        if !(0..=255).contains(&value) {
            return Err(AdbError::adb("Brightness value must be between 0 and 255"));
        }
        self.shell(format!("settings put system screen_brightness {value}"))
            .await?;
        Ok(())
    }

    /// Screen brightness mode (auto/manual).
    pub async fn brightness_mode(&self) -> Result<BrightnessMode> {
        let v = self.shell("settings get system screen_brightness_mode").await?;
        match v.trim() {
            "1" => Ok(BrightnessMode::Auto),
            "0" => Ok(BrightnessMode::Manual),
            other => Err(AdbError::adb(format!("bad brightness mode: {other:?}"))),
        }
    }

    /// Set screen brightness mode.
    pub async fn set_brightness_mode(&self, mode: BrightnessMode) -> Result<()> {
        let v = mode as i32;
        self.shell(format!("settings put system screen_brightness_mode {v}"))
            .await?;
        Ok(())
    }

    /// Toggle airplane mode.
    pub async fn switch_airplane(&self, enable: bool) -> Result<()> {
        let (setting, state) = if enable { ("1", "true") } else { ("0", "false") };
        self.shell(["settings", "put", "global", "airplane_mode_on", setting])
            .await?;
        self.shell([
            "am", "broadcast", "-a", "android.intent.action.AIRPLANE_MODE", "--ez", "state", state,
        ])
        .await?;
        Ok(())
    }

    /// Toggle WiFi.
    pub async fn switch_wifi(&self, enable: bool) -> Result<()> {
        let arg = if enable { "enable" } else { "disable" };
        self.shell(["svc", "wifi", arg]).await?;
        Ok(())
    }

    /// Screen size in pixels; width/height are swapped when landscape.
    /// `landscape = None` derives orientation from `rotation()`.
    pub async fn window_size(&self, landscape: Option<bool>) -> Result<WindowSize> {
        let wsize = self.wm_size().await?;
        let landscape = match landscape {
            Some(v) => v,
            None => self.rotation().await? % 2 == 1,
        };
        Ok(if landscape {
            WindowSize { width: wsize.height, height: wsize.width }
        } else {
            wsize
        })
    }

    async fn wm_size(&self) -> Result<WindowSize> {
        static OVERRIDE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"Override size: (\d+)x(\d+)").unwrap());
        static PHYSICAL: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"Physical size: (\d+)x(\d+)").unwrap());
        let output = self.shell("wm size").await?;
        for re in [&*OVERRIDE, &*PHYSICAL] {
            if let Some(c) = re.captures(&output) {
                let w = c[1].parse().unwrap_or(0);
                let h = c[2].parse().unwrap_or(0);
                return Ok(WindowSize { width: w, height: h });
            }
        }
        Err(AdbError::adb(format!("wm size output unexpected: {output:?}")))
    }

    /// Resolve possibly-fractional (x, y) against the current window size.
    async fn resolve_point(&self, x: f64, y: f64) -> Result<(i64, i64)> {
        if is_percent(x) || is_percent(y) {
            let ws = self.window_size(None).await?;
            let rx = if is_percent(x) { (x * ws.width as f64) as i64 } else { x as i64 };
            let ry = if is_percent(y) { (y * ws.height as f64) as i64 } else { y as i64 };
            Ok((rx, ry))
        } else {
            Ok((x as i64, y as i64))
        }
    }

    /// Swipe from (sx, sy) to (ex, ey); coordinates `<= 1.0` are treated as
    /// fractions of the screen. `duration` in seconds.
    pub async fn swipe(&self, sx: f64, sy: f64, ex: f64, ey: f64, duration: f64) -> Result<()> {
        let (x1, y1) = self.resolve_point(sx, sy).await?;
        let (x2, y2) = self.resolve_point(ex, ey).await?;
        let ms = (duration * 1000.0) as i64;
        self.shell([
            "input", "swipe", &x1.to_string(), &y1.to_string(), &x2.to_string(), &y2.to_string(),
            &ms.to_string(),
        ])
        .await?;
        Ok(())
    }

    /// Drag-and-drop from (sx, sy) to (ex, ey).
    pub async fn drag(&self, sx: f64, sy: f64, ex: f64, ey: f64, duration: f64) -> Result<()> {
        let (x1, y1) = self.resolve_point(sx, sy).await?;
        let (x2, y2) = self.resolve_point(ex, ey).await?;
        let ms = (duration * 1000.0) as i64;
        self.shell([
            "input", "draganddrop", &x1.to_string(), &y1.to_string(), &x2.to_string(),
            &y2.to_string(), &ms.to_string(),
        ])
        .await?;
        Ok(())
    }

    /// Tap at (x, y). `display_id` targets a specific display.
    pub async fn click(&self, x: f64, y: f64, display_id: Option<i32>) -> Result<()> {
        let (px, py) = self.resolve_point(x, y).await?;
        let mut args: Vec<String> = vec!["input".into()];
        if let Some(d) = display_id {
            args.push("-d".into());
            args.push(d.to_string());
        }
        args.push("tap".into());
        args.push(px.to_string());
        args.push(py.to_string());
        self.shell(args).await?;
        Ok(())
    }

    /// Type `text` (`input text`).
    pub async fn send_keys(&self, text: &str) -> Result<()> {
        self.shell(["input", "text", text]).await?;
        Ok(())
    }

    /// Device WLAN IP address, with `ifconfig`/`ip`/`eth0` fallbacks.
    pub async fn wlan_ip(&self) -> Result<String> {
        static INET_ADDR: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"inet\s*addr:(.*?)\s").unwrap());
        static IP_ADDR: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"inet (\d+.*?)/\d+").unwrap());

        let result = self.shell(["ifconfig", "wlan0"]).await?;
        if let Some(c) = INET_ADDR.captures(&result) {
            return Ok(c[1].to_string());
        }
        // Huawei P30 has no ifconfig
        let result = self.shell(["ip", "addr", "show", "dev", "wlan0"]).await?;
        if let Some(c) = IP_ADDR.captures(&result) {
            return Ok(c[1].to_string());
        }
        // Virtual devices may use eth0
        let result = self.shell(["ifconfig", "eth0"]).await?;
        if let Some(c) = INET_ADDR.captures(&result) {
            return Ok(c[1].to_string());
        }
        Err(AdbError::adb("fail to parse wlan ip"))
    }

    /// Display rotation `[0, 1, 2, 3]`.
    pub async fn rotation(&self) -> Result<u32> {
        static ORIENT: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r".*?orientation=(\d+)").unwrap());
        let output = self.shell("dumpsys display").await?;
        for line in output.lines() {
            if let Some(c) = ORIENT.captures(line) {
                if let Ok(o) = c[1].parse() {
                    return Ok(o);
                }
            }
        }
        Err(AdbError::adb("rotation get failed"))
    }

    /// `rm <path>`.
    pub async fn remove(&self, path: &str) -> Result<()> {
        self.shell(["rm", path]).await?;
        Ok(())
    }

    /// `rm -r <path>`.
    pub async fn rmtree(&self, path: &str) -> Result<()> {
        self.shell(["rm", "-r", path]).await?;
        Ok(())
    }

    /// Whether the screen is on (`dumpsys power`).
    pub async fn is_screen_on(&self) -> Result<bool> {
        let output = self.shell(["dumpsys", "power"]).await?;
        Ok(output.contains("mHoldingDisplaySuspendBlocker=true"))
    }

    /// Open `url` in the browser (adds `https://` if scheme-less).
    pub async fn open_browser(&self, url: &str) -> Result<()> {
        static SCHEME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^https?://").unwrap());
        let url = if SCHEME.is_match(url) {
            url.to_string()
        } else {
            format!("https://{url}")
        };
        self.shell(["am", "start", "-a", "android.intent.action.VIEW", "-d", &url])
            .await?;
        Ok(())
    }

    /// Installed package names, sorted (`pm list packages`). `filters` are pm
    /// flags such as `-3` (third-party) or `-s` (system).
    pub async fn list_packages(&self, filters: &[&str]) -> Result<Vec<String>> {
        static PKG: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"(?m)^package:(\S+)\r?$").unwrap());
        let mut cmd: Vec<String> = vec!["pm".into(), "list".into(), "packages".into()];
        cmd.extend(filters.iter().map(|s| s.to_string()));
        let output = self.shell(cmd).await?;
        let mut result: Vec<String> = PKG
            .captures_iter(&output)
            .map(|c| c[1].to_string())
            .collect();
        result.sort();
        Ok(result)
    }

    /// Uninstall by package name (`pm uninstall`).
    pub async fn uninstall(&self, pkg_name: &str) -> Result<String> {
        self.shell(["pm", "uninstall", pkg_name]).await
    }

    /// Install an APK already present on the device (`pm install`). Raises
    /// [`AdbError::Install`] if the output lacks `Success`.
    pub async fn install_remote(&self, remote_path: &str, clean: bool, flags: &[&str]) -> Result<()> {
        let mut args: Vec<String> = vec!["pm".into(), "install".into()];
        args.extend(flags.iter().map(|s| s.to_string()));
        args.push(remote_path.to_string());
        let output = self.shell(args).await?;
        if !output.contains("Success") {
            return Err(AdbError::install(output));
        }
        if clean {
            self.shell(["rm", remote_path]).await?;
        }
        Ok(())
    }

    /// Launch an app via `am start -n pkg/activity`, or `monkey` LAUNCHER when
    /// no activity is given.
    pub async fn app_start(&self, package_name: &str, activity: Option<&str>) -> Result<()> {
        match activity {
            Some(act) => {
                self.shell(["am", "start", "-n", &format!("{package_name}/{act}")])
                    .await?;
            }
            None => {
                self.shell([
                    "monkey", "-p", package_name, "-c", "android.intent.category.LAUNCHER", "1",
                ])
                .await?;
            }
        }
        Ok(())
    }

    /// Force-stop an app (`am force-stop`).
    pub async fn app_stop(&self, package_name: &str) -> Result<()> {
        self.shell(["am", "force-stop", package_name]).await?;
        Ok(())
    }

    /// Clear app data (`pm clear`).
    pub async fn app_clear(&self, package_name: &str) -> Result<()> {
        self.shell(["pm", "clear", package_name]).await?;
        Ok(())
    }

    /// Detailed app info (`pm path` + `dumpsys package`), or `None` if not found.
    pub async fn app_info(&self, package_name: &str) -> Result<Option<AppInfo>> {
        static VERSION_NAME: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"versionName=(\S+)").unwrap());
        static VERSION_CODE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"versionCode=(\d+)").unwrap());
        static SIGNATURE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"PackageSignatures\{.*?\[(.*)\]\}").unwrap());
        static FLAGS: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"pkgFlags=\[\s*(.*)\s*\]").unwrap());
        static FIRST_INSTALL: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"firstInstallTime=([-\d]+\s+[:\d]+)").unwrap());
        static LAST_UPDATE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"lastUpdateTime=([-\d]+\s+[:\d]+)").unwrap());

        let output = self.shell(["pm", "path", package_name]).await?;
        if !output.contains("package:") {
            return Ok(None);
        }
        let apk_paths: Vec<&str> = output.lines().collect();
        let apk_path = apk_paths[0]
            .split_once(':')
            .map(|(_, v)| v)
            .unwrap_or("")
            .trim()
            .to_string();
        let sub_apk_paths: Vec<String> = apk_paths[1..]
            .iter()
            .map(|p| p.replacen("package:", "", 1))
            .collect();

        let output = self.shell(["dumpsys", "package", package_name]).await?;
        let mut version_name = VERSION_NAME
            .captures(&output)
            .map(|c| c[1].to_string())
            .unwrap_or_default();
        if version_name == "null" {
            version_name.clear();
        }
        let version_code = VERSION_CODE
            .captures(&output)
            .and_then(|c| c[1].parse::<i64>().ok());
        let signature = SIGNATURE.captures(&output).map(|c| c[1].to_string());
        if version_name.is_empty() && signature.is_none() {
            return Ok(None);
        }
        let flags: Vec<String> = FLAGS
            .captures(&output)
            .map(|c| c[1].split_whitespace().map(str::to_string).collect())
            .unwrap_or_default();
        let parse_time = |re: &Regex| -> Option<chrono::DateTime<Local>> {
            re.captures(&output).and_then(|c| {
                NaiveDateTime::parse_from_str(c[1].trim(), "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .and_then(|ndt| Local.from_local_datetime(&ndt).single())
            })
        };

        Ok(Some(AppInfo {
            package_name: package_name.to_string(),
            version_name: if version_name.is_empty() { None } else { Some(version_name) },
            version_code,
            flags,
            first_install_time: parse_time(&FIRST_INSTALL),
            last_update_time: parse_time(&LAST_UPDATE),
            signature,
            path: if apk_path.is_empty() { None } else { Some(apk_path) },
            sub_apk_paths,
        }))
    }

    /// Foreground app (`dumpsys window/activity`). Retries up to 3× (0.5s delay),
    /// mirroring the Python `@retry`.
    pub async fn app_current(&self) -> Result<RunningAppInfo> {
        let mut last_err = AdbError::adb("Couldn't get focused app");
        for attempt in 0..3 {
            match self.app_current_once().await {
                Ok(info) => return Ok(info),
                Err(e) => {
                    last_err = e;
                    if attempt < 2 {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        }
        Err(last_err)
    }

    async fn app_current_once(&self) -> Result<RunningAppInfo> {
        static FOCUSED: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"mCurrentFocus=Window\{.*\s+(\S+)/(\S+)\}").unwrap()
        });
        static RESUMED: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"mResumedActivity: ActivityRecord\{.*?\s+(\S+)/(\S+)\s.*?\}").unwrap()
        });
        static ACTIVITY: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"ACTIVITY (\S+)/([^/\s]+) \w+ pid=(\d+)").unwrap()
        });

        let out = self.shell(["dumpsys", "window", "windows"]).await?;
        if let Some(c) = FOCUSED.captures(&out) {
            return Ok(RunningAppInfo {
                package: c[1].to_string(),
                activity: c[2].to_string(),
                pid: 0,
            });
        }

        let output = self.shell(["dumpsys", "activity", "activities"]).await?;
        let package = RESUMED.captures(&output).map(|c| c[1].to_string());

        let output = self.shell(["dumpsys", "activity", "top"]).await?;
        let mut ret: Option<RunningAppInfo> = None;
        for c in ACTIVITY.captures_iter(&output) {
            let info = RunningAppInfo {
                package: c[1].to_string(),
                activity: c[2].to_string(),
                pid: c[3].parse().unwrap_or(0),
            };
            if Some(&info.package) == package.as_ref() {
                return Ok(info);
            }
            ret = Some(info);
        }
        ret.ok_or_else(|| AdbError::adb("Couldn't get focused app"))
    }

    /// Dump the UI hierarchy XML (`uiautomator dump`).
    pub async fn dump_hierarchy(&self) -> Result<String> {
        let target = "/data/local/tmp/uidump.xml";
        let output = self
            .shell(format!("rm -f {target}; uiautomator dump {target} && echo success"))
            .await?;
        if output.contains("ERROR") || !output.contains("success") {
            return Err(AdbError::adb(format!("uiautomator dump failed: {output}")));
        }
        let buf = self.sync().read_bytes(target).await?;
        let xml = String::from_utf8_lossy(&buf).into_owned();
        if !xml.starts_with("<?xml") {
            return Err(AdbError::adb("dump output is not xml"));
        }
        Ok(xml)
    }

    /// Battery details (`dumpsys battery`).
    pub async fn battery(&self) -> Result<BatteryInfo> {
        let output = self.shell(["dumpsys", "battery"]).await?;
        let mut kvs = std::collections::HashMap::new();
        for line in output.lines() {
            if let Some((k, v)) = line.trim().split_once(':') {
                kvs.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        let to_bool = |k: &str| kvs.get(k).map(|v| v == "true");
        let to_int = |k: &str| kvs.get(k).and_then(|v| v.parse::<i64>().ok());
        Ok(BatteryInfo {
            ac_powered: to_bool("AC powered").unwrap_or(false),
            usb_powered: to_bool("USB powered").unwrap_or(false),
            wireless_powered: to_bool("Wireless powered"),
            dock_powered: to_bool("Dock powered"),
            max_charging_current: to_int("Max charging current"),
            max_charging_voltage: to_int("Max charging voltage"),
            charge_counter: to_int("Charge counter"),
            status: to_int("status"),
            health: to_int("health"),
            present: to_bool("present"),
            level: to_int("level"),
            scale: to_int("scale"),
            voltage: to_int("voltage"),
            temperature: to_int("temperature").map(|t| t as f64 / 10.0),
            technology: kvs.get("technology").cloned(),
        })
    }
}
