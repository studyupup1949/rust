use std::collections::BTreeMap;

use crate::error::{Error, Result};
use crate::robot::{Robot, VideoOptions};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraInfo {
    pub vendor: String,
    pub model: String,
    pub serial: String,
    pub handle: String,
    pub has_depth: bool,
    pub has_imu: bool,
    pub has_onboard_encode: bool,
    pub extra: BTreeMap<String, String>,
}

/// Preferences for the camera discovery + auto-attach helpers
/// (`attach_camera` / `attach_all`).
///
/// `video` is the full per-track config, so every `VideoOptions` field — codec,
/// encoder, backend, bitrate, `passthrough`, … — flows through unchanged; new
/// options never need re-plumbing here. The remaining fields are discovery-only.
#[derive(Debug, Clone)]
pub struct AttachOptions {
    /// Full per-track video config. `video.width` / `video.height` set the
    /// resolution the track is attached at.
    pub video: VideoOptions,
    /// Also attach the camera's depth stream, when it has one.
    pub enable_depth: bool,
    /// Also attach the camera's IMU stream, when it has one.
    pub enable_imu: bool,
}

impl Default for AttachOptions {
    fn default() -> Self {
        Self {
            video: VideoOptions {
                width: 1920,
                height: 1080,
                bitrate_kbps: 4000,
                fps: 60,
                ..Default::default()
            },
            enable_depth: false,
            enable_imu: false,
        }
    }
}

/// Discover cameras across adapters available in the Rust SDK.
///
/// Today this covers generic V4L2 capture devices. Vendor SDK adapters remain
/// Python-only optional dependencies.
pub fn discover() -> Vec<CameraInfo> {
    discover_v4l2()
}

/// List V4L2 capture nodes on Linux. Other platforms return an empty list.
pub fn discover_v4l2() -> Vec<CameraInfo> {
    #[cfg(target_os = "linux")]
    {
        discover_v4l2_linux()
    }

    #[cfg(not(target_os = "linux"))]
    {
        Vec::new()
    }
}

/// Attach a discovered camera as an Adamo video track.
///
/// Only `vendor == "v4l2"` cameras are handled by the Rust SDK adapter.
pub fn attach_camera(
    robot: &mut Robot,
    info: &CameraInfo,
    name: Option<&str>,
    options: &AttachOptions,
) -> Result<()> {
    if info.vendor != "v4l2" {
        return Err(Error::Ffi(format!(
            "camera adapter '{}' is not available in the Rust SDK",
            info.vendor
        )));
    }

    let track_name = name.map(str::to_owned).unwrap_or_else(|| name_from_info(info));
    // Forward the full track config as-is — every VideoOptions field (codec,
    // encoder, backend, passthrough, …) propagates with no per-field plumbing.
    robot.attach_v4l2_with_options(&track_name, &info.handle, &options.video)
}

/// Discover and attach all cameras supported by the Rust SDK.
///
/// Returns the cameras that were successfully attached.
pub fn attach_all(robot: &mut Robot, options: &AttachOptions) -> Vec<CameraInfo> {
    let mut attached = Vec::new();
    for info in discover() {
        if attach_camera(robot, &info, None, options).is_ok() {
            attached.push(info);
        }
    }
    attached
}

fn name_from_info(info: &CameraInfo) -> String {
    let base = info
        .handle
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("camera");
    let clean = clean_name(&info.model);
    if clean.is_empty() {
        base.to_owned()
    } else {
        format!("{clean}_{base}")
    }
}

fn clean_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut last_was_sep = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_was_sep = false;
        } else if !last_was_sep && !out.is_empty() {
            out.push('_');
            last_was_sep = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    out
}

#[cfg(target_os = "linux")]
fn discover_v4l2_linux() -> Vec<CameraInfo> {
    use std::fs;

    const ZED_NAMES: &[&str] = &["ZED", "STEREOLABS"];
    const REALSENSE_NAMES: &[&str] = &["REALSENSE"];

    let mut paths = Vec::new();
    let Ok(entries) = fs::read_dir("/dev") else {
        return Vec::new();
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        let Some(suffix) = name.strip_prefix("video") else {
            continue;
        };
        if !suffix.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        paths.push(format!("/dev/{name}"));
    }
    paths.sort();

    let mut out = Vec::new();
    for path in paths {
        if !is_capture_node(&path) {
            continue;
        }
        let sysfs_name = read_sysfs_name(&path);
        let upper = sysfs_name.to_ascii_uppercase();
        let looks_like_zed = ZED_NAMES.iter().any(|name| upper.contains(name));
        let looks_like_realsense = REALSENSE_NAMES.iter().any(|name| upper.contains(name));

        let mut extra = BTreeMap::new();
        extra.insert("sysfs_name".to_owned(), sysfs_name.clone());
        extra.insert("looks_like_zed".to_owned(), looks_like_zed.to_string());
        extra.insert(
            "looks_like_realsense".to_owned(),
            looks_like_realsense.to_string(),
        );

        out.push(CameraInfo {
            vendor: "v4l2".to_owned(),
            model: if sysfs_name.is_empty() {
                "v4l2".to_owned()
            } else {
                sysfs_name
            },
            serial: path.clone(),
            handle: path,
            has_depth: false,
            has_imu: false,
            has_onboard_encode: false,
            extra,
        });
    }
    out
}

#[cfg(target_os = "linux")]
fn read_sysfs_name(devpath: &str) -> String {
    let Some(base) = devpath.rsplit('/').next() else {
        return String::new();
    };
    std::fs::read_to_string(format!("/sys/class/video4linux/{base}/name"))
        .map(|s| s.trim().to_owned())
        .unwrap_or_default()
}

#[cfg(target_os = "linux")]
fn is_capture_node(devpath: &str) -> bool {
    let Some(base) = devpath.rsplit('/').next() else {
        return false;
    };
    let caps_path = format!("/sys/class/video4linux/{base}/capabilities");
    let Ok(raw) = std::fs::read_to_string(caps_path) else {
        return true;
    };
    let raw = raw.trim().trim_start_matches("0x");
    let Ok(caps) = u32::from_str_radix(raw, 16) else {
        return true;
    };
    const V4L2_CAP_VIDEO_CAPTURE: u32 = 0x0000_0001;
    caps & V4L2_CAP_VIDEO_CAPTURE != 0
}
