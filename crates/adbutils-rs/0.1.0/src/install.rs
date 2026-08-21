//! APK install. Port of `adbutils/install.py` (`InstallExtension`).
//!
//! Downloads (for URLs), pushes the apk, runs `pm install`, and retries after an
//! uninstall on the known downgrade/incompatible failures. Unlike the Python
//! version — which uses `apkutils` to find the main activity — launch here goes
//! through `monkey` (via [`app_start`](AdbDevice::app_start) with no activity),
//! so only the package name is needed.

use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;

use crate::apk;
use crate::device::AdbDevice;
use crate::errors::{AdbError, Result};

/// Reasons that warrant an uninstall-then-retry.
const RETRY_REASONS: &[&str] = &[
    "INSTALL_FAILED_PERMISSION_MODEL_DOWNGRADE",
    "INSTALL_FAILED_UPDATE_INCOMPATIBLE",
    "INSTALL_FAILED_VERSION_DOWNGRADE",
];

impl AdbDevice {
    /// Download an APK from `url` into `path` (`download_apk`).
    pub async fn download_apk(url: &str, path: &Path) -> Result<()> {
        let resp = reqwest::get(url)
            .await
            .map_err(|e| AdbError::adb(format!("download failed: {e}")))?
            .error_for_status()
            .map_err(|e| AdbError::adb(format!("download failed: {e}")))?;
        let mut file = tokio::fs::File::create(path)
            .await
            .map_err(|e| AdbError::adb(format!("cannot create {}: {e}", path.display())))?;
        let mut resp = resp;
        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|e| AdbError::adb(format!("download read failed: {e}")))?
        {
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        Ok(())
    }

    /// Install an APK from a local path or `http(s)://` URL.
    ///
    /// - `nolaunch`: skip launching after install.
    /// - `uninstall`: uninstall the package first.
    /// - `flags`: `pm install` flags (default `["-r", "-t"]`).
    pub async fn install(
        &self,
        path_or_url: &str,
        nolaunch: bool,
        uninstall: bool,
        flags: &[&str],
    ) -> Result<()> {
        // Resolve a local source path, downloading URLs to a temp file.
        let (src_path, _tmp): (PathBuf, Option<TempApk>) =
            if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
                let tmp = TempApk::new()?;
                Self::download_apk(path_or_url, &tmp.path).await?;
                (tmp.path.clone(), Some(tmp))
            } else {
                (PathBuf::from(path_or_url), None)
            };
        if !src_path.is_file() {
            return Err(AdbError::adb(format!("File or URL not found: {path_or_url}")));
        }

        let package_name = apk::parse_apk(&src_path).ok().map(|i| i.package_name);
        let device_dst = format!(
            "/data/local/tmp/{}.apk",
            package_name.as_deref().unwrap_or("unknown")
        );

        // Push and verify size.
        let sync = self.sync();
        let pushed = sync.push(&src_path, &device_dst, 0o644, false).await?;
        let apk_size = std::fs::metadata(&src_path).map(|m| m.len()).unwrap_or(0);
        if pushed != apk_size {
            return Err(AdbError::adb(format!(
                "pushed apk size not matched, expect {apk_size} got {pushed}"
            )));
        }

        if uninstall {
            if let Some(pkg) = &package_name {
                self.uninstall(pkg).await.ok();
            }
        }

        // Install, retrying once after uninstall on known downgrade failures.
        match self.install_remote(&device_dst, true, flags).await {
            Ok(()) => {}
            Err(AdbError::Install { reason, output }) => {
                let retryable = package_name.is_some() && RETRY_REASONS.contains(&reason.as_str());
                if retryable {
                    let pkg = package_name.as_ref().unwrap();
                    self.uninstall(pkg).await.ok();
                    self.install_remote(&device_dst, true, flags).await?;
                } else {
                    return Err(AdbError::Install { reason, output });
                }
            }
            Err(e) => return Err(e),
        }

        if !nolaunch {
            if let Some(pkg) = &package_name {
                self.app_start(pkg, None).await?;
            }
        }
        Ok(())
    }
}

/// A temp file with a `.apk` suffix, deleted on drop.
struct TempApk {
    path: PathBuf,
}

impl TempApk {
    fn new() -> Result<Self> {
        // Unique-enough name from the process id and address of a stack local;
        // Date/random are avoided elsewhere but fine at runtime here.
        let mut dir = std::env::temp_dir();
        let pid = std::process::id();
        let salt = &dir as *const _ as usize;
        dir.push(format!("adbutils-{pid}-{salt:x}.apk"));
        Ok(Self { path: dir })
    }
}

impl Drop for TempApk {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
