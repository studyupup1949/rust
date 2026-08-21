use std::path::PathBuf;

use solana_commitment_config::CommitmentConfig;

use crate::accache::Accache;
use crate::config::{CacheConfig, Compression, RefreshPolicy};
use crate::error::Result;

/// Where account data is sourced from.
#[derive(Clone, Debug)]
pub enum Source {
    /// Fetch from RPC on miss.
    Rpc { url: String },
    /// Load `.acc` files at build time, no RPC.
    Files(Vec<PathBuf>),
    /// Load `.acc` files at build time, then fall through to RPC on miss.
    Hybrid {
        rpc_url: String,
        files: Vec<PathBuf>,
    },
}

/// Fluent builder for [`Accache`].
#[derive(Debug, Default)]
pub struct AccacheBuilder {
    rpc_url: Option<String>,
    files: Vec<PathBuf>,
    config: CacheConfig,
}

impl AccacheBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_rpc(url: impl Into<String>) -> Self {
        Self {
            rpc_url: Some(url.into()),
            ..Self::default()
        }
    }

    pub fn from_files<I, P>(paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        Self {
            files: paths.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    pub fn with_rpc(mut self, url: impl Into<String>) -> Self {
        self.rpc_url = Some(url.into());
        self
    }

    pub fn with_files<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.files.extend(paths.into_iter().map(Into::into));
        self
    }

    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.files.push(path.into());
        self
    }

    pub fn outfile(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.outfile = Some(path.into());
        self
    }

    pub fn auto_persist(mut self, enabled: bool) -> Self {
        self.config.auto_persist = enabled;
        self
    }

    pub fn refresh(mut self, policy: RefreshPolicy) -> Self {
        self.config.refresh = policy;
        self
    }

    pub fn commitment(mut self, c: CommitmentConfig) -> Self {
        self.config.commitment = c;
        self
    }

    pub fn compression(mut self, c: Compression) -> Self {
        self.config.compression = c;
        self
    }

    pub fn config(mut self, c: CacheConfig) -> Self {
        self.config = c;
        self
    }

    pub fn source(&self) -> Source {
        match (&self.rpc_url, self.files.is_empty()) {
            (Some(url), true) => Source::Rpc { url: url.clone() },
            (Some(url), false) => Source::Hybrid {
                rpc_url: url.clone(),
                files: self.files.clone(),
            },
            (None, _) => Source::Files(self.files.clone()),
        }
    }

    pub fn build(self) -> Result<Accache> {
        Accache::from_builder(self)
    }

    #[cfg(feature = "nonblocking")]
    pub fn build_nonblocking(self) -> Result<crate::nonblocking::Accache> {
        crate::nonblocking::Accache::from_builder(self)
    }

    pub(crate) fn into_parts(self) -> (Option<String>, Vec<PathBuf>, CacheConfig) {
        (self.rpc_url, self.files, self.config)
    }
}
