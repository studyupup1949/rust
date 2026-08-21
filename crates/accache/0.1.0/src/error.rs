use std::io;
use std::path::PathBuf;

use solana_pubkey::Pubkey;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AccacheError>;

#[derive(Debug, Error)]
pub enum AccacheError {
    #[error("account {0} not found in cache or source")]
    NotFound(Pubkey),

    #[error("cache is offline; account {0} is not present (RefreshPolicy::Offline)")]
    Offline(Pubkey),

    #[error("no RPC source configured but one was required")]
    NoRpcConfigured,

    #[error("file {path}: {reason}")]
    InvalidFile { path: PathBuf, reason: String },

    #[error("unsupported .acc format version {0} (expected {expected})", expected = crate::format::FORMAT_VERSION)]
    UnsupportedFormatVersion(u16),

    #[error("bad magic in .acc file: expected {expected:?}, got {got:?}", expected = crate::format::MAGIC)]
    BadMagic { got: [u8; 4] },

    #[error("payload truncated or corrupt: {0}")]
    Corrupt(String),

    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("bincode error: {0}")]
    Bincode(#[from] Box<bincode::ErrorKind>),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[cfg(feature = "rpc")]
    #[error("rpc error: {0}")]
    Rpc(#[from] solana_rpc_client_api::client_error::Error),

    #[error("invalid pubkey: {0}")]
    InvalidPubkey(String),

    #[error("invalid commitment level: {0}")]
    InvalidCommitment(String),
}
