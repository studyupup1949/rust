mod error;
mod config;
mod device;
mod cmd;

// 功能模块
pub mod app;
pub mod transfer;
pub mod remote;
pub mod media;
pub mod forward;
pub mod resource;
pub mod parallel;
pub mod utils;

// 导出主要类型
pub use config::{ADBConfig, ADBConfigBuilder};
pub use device::{ADB, ADBDevice, DeviceStatus};
pub use error::{ADBError, ADBResult};
pub use app::PackageInfo;
pub use transfer::TransferOptions;

// 便利的预导出模块
pub mod prelude {
    pub use super::{ADB, ADBConfig, ADBConfigBuilder, ADBDevice, ADBError, ADBResult};
    pub use super::app::PackageInfo;
    pub use super::transfer::TransferOptions;
}