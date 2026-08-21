//! Source configuration types.
//!
//! Defines configuration structures for each source type,
//! enabling source-agnostic configuration and refresh logic.

use std::collections::HashMap;
use std::path::PathBuf;

/// File source manager configuration.
///
/// Holds configuration for all .env files managed by the FileSourceManager.
#[derive(Debug, Clone, Default)]
pub struct FileSourceConfig {
    /// Active file patterns (None = auto-discovery)
    pub active_files: Option<Vec<String>>,
    /// Directory-scoped overrides
    pub directory_overrides: HashMap<PathBuf, Vec<String>>,
}

/// Shell source configuration.
#[derive(Debug, Clone)]
pub struct ShellSourceConfig {
    /// Whether shell source is enabled
    pub enabled: bool,
    /// Optional filter for which shell vars to include
    pub include_patterns: Option<Vec<String>>,
}

impl Default for ShellSourceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_patterns: None,
        }
    }
}

/// Remote source configuration (prepared for future).
#[derive(Debug, Clone, Default)]
pub struct RemoteSourceConfig {
    pub endpoint: Option<String>,
    pub auth_token: Option<String>,
    pub timeout_ms: Option<u64>,
    pub retry_count: Option<u32>,
}

/// Memory source configuration.
#[derive(Debug, Clone, Default)]
pub struct MemorySourceConfig {
    // Currently no runtime config needed
}

/// Options for source refresh operations.
#[derive(Debug, Clone, Default)]
pub struct SourceRefreshOptions {
    /// Whether to preserve the source's configuration during refresh
    pub preserve_config: bool,
}

impl SourceRefreshOptions {
    /// Create options that preserve configuration
    pub fn preserve() -> Self {
        Self {
            preserve_config: true,
        }
    }

    /// Create options that reset configuration
    pub fn reset() -> Self {
        Self {
            preserve_config: false,
        }
    }
}
