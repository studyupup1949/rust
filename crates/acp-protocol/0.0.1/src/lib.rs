//! # ACP - AI Context Protocol
//!
//! Core library for the AI Context Protocol.
//!
//! **⚠️ This is a placeholder release. Full functionality coming soon.**
//!
//! ## What is ACP?
//!
//! The AI Context Protocol (ACP) is an open standard for embedding machine-readable
//! context in codebases for AI-assisted development. This crate provides the core
//! functionality for:
//!
//! - **Parsing**: Extract ACP annotations from source code
//! - **Indexing**: Build a structured cache of codebase metadata
//! - **Querying**: Search and filter indexed symbols and files
//! - **Constraints**: Evaluate and enforce code constraints
//! - **Variables**: Expand token-efficient variable references
//!
//! ## Usage
//!
//! ```rust,ignore
//! use acp-protocol::{Cache, Config};
//!
//! // Coming soon...
//! let config = Config::load(".acp-protocol.config.json")?;
//! let cache = Cache::build(&config)?;
//! ```
//!
//! ## Links
//!
//! - [GitHub Repository](https://github.com/acp-protocol/acp-spec)
//! - [Specification](https://github.com/acp-protocol/acp-spec/tree/main/spec)
//! - [CLI Tool](https://crates.io/crates/acp-cli)
//!
//! ## License
//!
//! MIT License - see repository for details.

/// ACP specification version supported by this library.
pub const SPEC_VERSION: &str = "1.0.0";

/// Library version (placeholder).
pub const VERSION: &str = "0.0.1-placeholder";

/// Placeholder for ACP core functionality.
///
/// Full implementation coming soon.
pub fn version() -> &'static str {
    VERSION
}

/// Placeholder for spec version.
pub fn spec_version() -> &'static str {
    SPEC_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.0.1-placeholder");
    }

    #[test]
    fn test_spec_version() {
        assert_eq!(spec_version(), "1.0.0");
    }
}
