use std::path::PathBuf;
use std::time::Duration;

use solana_commitment_config::CommitmentConfig;

/// How `Accache` should decide when to fetch a fresh copy from RPC vs. serve from cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshPolicy {
    /// Cache only. RPC is never called even if configured. Misses return [`crate::AccacheError::Offline`].
    Offline,
    /// Serve from cache when present. Fetch from RPC on miss and insert. (Default.)
    OnMiss,
    /// Always fetch from RPC and overwrite the cache. Useful when the underlying account changes.
    Always,
    /// Serve from cache while the entry is younger than `Duration`; otherwise refresh.
    Ttl(Duration),
}

impl Default for RefreshPolicy {
    fn default() -> Self {
        Self::OnMiss
    }
}

/// On-disk compression for `.acc` payloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compression {
    None,
    Zstd { level: i32 },
}

impl Default for Compression {
    fn default() -> Self {
        Self::Zstd { level: 3 }
    }
}

#[derive(Clone, Debug)]
pub struct CacheConfig {
    /// Path that [`crate::Accache::flush`] writes to (and that drop-time auto-persist uses).
    pub outfile: Option<PathBuf>,
    /// If true and `outfile` is set, write the cache after every mutation.
    pub auto_persist: bool,
    /// Strategy for deciding cache-hit vs. RPC-fetch.
    pub refresh: RefreshPolicy,
    /// Commitment level for RPC fetches.
    pub commitment: CommitmentConfig,
    /// Compression to use when writing `.acc` files.
    pub compression: Compression,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            outfile: None,
            auto_persist: false,
            refresh: RefreshPolicy::default(),
            commitment: CommitmentConfig::confirmed(),
            compression: Compression::default(),
        }
    }
}
