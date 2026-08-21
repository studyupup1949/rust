//! Async Rust client for the Android Debug Bridge (adb) server smartsocket
//! protocol. A port of [adbutils-python](https://github.com/openatx/adbutils).
//!
//! ```no_run
//! # async fn demo() -> adbutils::Result<()> {
//! let adb = adbutils::AdbClient::default();
//! for device in adb.device_list().await? {
//!     println!("{}: {}", device.serial().unwrap_or("?"), device.shell("echo hi").await?);
//! }
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "apk")]
pub mod apk;
pub mod client;
pub mod conn;
pub mod device;
pub mod errors;
#[cfg(feature = "apk")]
pub mod install;
pub mod pidcat;
pub mod proto;
#[cfg(feature = "screenrecord")]
pub mod screenrecord;
#[cfg(feature = "image")]
pub mod screenshot;
pub mod shell;
pub mod sync;
pub mod utils;

pub use client::AdbClient;
pub use device::AdbDevice;
pub use sync::Sync;
pub use errors::{AdbError, Result};
pub use proto::*;

/// Construct a client pointed at the default adb server (`127.0.0.1:5037`,
/// overridable via `ANDROID_ADB_SERVER_HOST` / `ANDROID_ADB_SERVER_PORT`).
/// Equivalent to the module-level `adb` singleton in the Python library.
pub fn adb() -> AdbClient {
    AdbClient::default()
}
