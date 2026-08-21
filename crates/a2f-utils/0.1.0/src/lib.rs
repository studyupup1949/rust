//! # a2f-utils
//!
//! ユーティリティ機能を提供するクレート
//!
//! ## 機能
//! - X25519鍵交換

mod error;
mod key_exchange;

pub use error::{UtilsError, UtilsResult};
pub use key_exchange::KeyExchange;