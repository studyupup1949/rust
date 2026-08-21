pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod models;

pub use auth::{AuthMethod, SignatureGenerator};
pub use client::AbpilotClient;
#[cfg(feature = "mp")]
pub use client::MpClient;
#[cfg(feature = "app")]
pub use client::AppClient;
pub use config::Config;
pub use error::{AbpilotError, Result};

// Re-export commonly used models
#[cfg(feature = "mp")]
pub use models::{ApiKey, App, AuthToken, User, World};
#[cfg(feature = "app")]
pub use models::{Asset, Device, DeviceToken, WorldNode, WorldNodeInfo};
