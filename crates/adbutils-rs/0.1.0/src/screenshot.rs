//! Screen capture. Port of `adbutils/screenshot.py` (`ScreenshotExtesion`).
//!
//! Uses `screencap -p` over the plain shell channel and decodes the PNG with the
//! `image` crate. The framebuffer protocol is intentionally avoided (unstable),
//! matching the Python default; the raw framebuffer path lives on
//! [`AdbDevice::framebuffer`].

use std::sync::LazyLock;

use image::RgbImage;
use regex::Regex;

use crate::device::AdbDevice;
use crate::errors::{AdbError, Result};

impl AdbDevice {
    /// Capture the screen as an RGB image via `screencap -p`.
    ///
    /// `display_id` selects a display (see `dumpsys SurfaceFlinger --display-id`).
    /// When `error_ok` and decoding fails, returns a black image sized to the
    /// current window (mirrors the Python fallback); otherwise errors.
    pub async fn screenshot(&self, display_id: Option<i32>, error_ok: bool) -> Result<RgbImage> {
        match self.screencap(display_id).await {
            Ok(img) => Ok(img),
            Err(e) => {
                if error_ok {
                    let ws = self.window_size(None).await?;
                    Ok(RgbImage::new(ws.width, ws.height))
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn screencap(&self, display_id: Option<i32>) -> Result<RgbImage> {
        let mut args: Vec<String> = vec!["screencap".into(), "-p".into()];
        if let Some(d) = display_id {
            let real = self.real_display_id(d).await?;
            args.push("-d".into());
            args.push(real);
        }
        let png = self.shell_bytes(args).await?;
        let img = image::load_from_memory(&png)
            .map_err(|e| AdbError::adb(format!("cannot decode screencap png: {e}")))?;
        Ok(img.to_rgb8())
    }

    /// Map a logical display index to the real display id reported by
    /// `dumpsys SurfaceFlinger --display-id`.
    async fn real_display_id(&self, display_id: i32) -> Result<String> {
        static DISPLAY: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"Display (\d+) ").unwrap());
        let output = self
            .shell(["dumpsys", "SurfaceFlinger", "--display-id"])
            .await?;
        let ids: Vec<&str> = DISPLAY
            .captures_iter(&output)
            .filter_map(|c| c.get(1).map(|m| m.as_str()))
            .collect();
        ids.get(display_id as usize)
            .map(|s| s.to_string())
            .ok_or_else(|| AdbError::adb(format!("display_id {display_id} not found")))
    }
}
