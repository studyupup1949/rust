//! Error types. Port of `adbutils/errors.py`.

use std::sync::LazyLock;

use regex::Regex;

/// Result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, AdbError>;

/// All adb errors. Mirrors the Python exception hierarchy, which is flat under
/// `AdbError`; here the variants distinguish the former subclasses.
#[derive(Debug, thiserror::Error)]
pub enum AdbError {
    /// Generic adb error carrying the server-reported message (`AdbError`).
    #[error("{0}")]
    Adb(String),

    /// Timeout communicating with the adb server (`AdbTimeout`).
    #[error("adb timeout: {0}")]
    Timeout(String),

    /// Connection error talking to the adb server (`AdbConnectionError`).
    #[error("adb connection error: {0}")]
    Connection(String),

    /// APK install failure (`AdbInstallError`). `reason` is parsed from the
    /// `Failure [REASON]` marker in the `pm install` output.
    #[error("{output}")]
    Install { reason: String, output: String },

    /// Sync sub-protocol error (`AdbSyncError`).
    #[error("adb sync error: {0}")]
    Sync(String),

    /// Underlying IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl AdbError {
    /// Build an [`AdbError::Adb`] from anything stringy.
    pub fn adb(msg: impl Into<String>) -> Self {
        AdbError::Adb(msg.into())
    }

    /// Parse `pm install` output into an [`AdbError::Install`], extracting the
    /// bracketed reason from a `Failure [REASON]` line (default `"Unknown"`).
    /// Mirrors `AdbInstallError.__init__` (`errors.py:24`).
    pub fn install(output: impl Into<String>) -> Self {
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"Failure \[([A-Z_]+).*\]").unwrap());
        let output = output.into();
        let reason = RE
            .captures(&output)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        AdbError::Install { reason, output }
    }
}
