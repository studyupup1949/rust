//! Stage a bundled `adb` binary if one is present under `assets/binaries/`.
//!
//! The Python library ships per-platform adb binaries for the `start-server`
//! fallback. Here we keep the same convention: drop `assets/binaries/adb`
//! (`adb.exe` on Windows) into the crate and it becomes the fallback when neither
//! `ADBUTILS_ADB_PATH` nor a `PATH` adb is available. When no binary is present,
//! nothing is emitted and the runtime falls back to `PATH`.

use std::path::Path;

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let bin_name = if target_os == "windows" { "adb.exe" } else { "adb" };
    let candidate = Path::new(&manifest).join("assets/binaries").join(bin_name);

    println!("cargo:rerun-if-changed=assets/binaries");
    if candidate.exists() {
        // Absolute path is embedded via option_env!("ADBUTILS_BUNDLED_ADB").
        println!("cargo:rustc-env=ADBUTILS_BUNDLED_ADB={}", candidate.display());
    }
}
