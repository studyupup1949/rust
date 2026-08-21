//! # ACP CLI
//!
//! Command-line interface for the AI Context Protocol.
//!
//! **⚠️ This is a placeholder release. Full functionality coming soon.**
//!
//! ## What is ACP?
//!
//! The AI Context Protocol (ACP) is an open standard for embedding machine-readable
//! context in codebases for AI-assisted development. It enables:
//!
//! - **Annotations**: Document code structure and intent for AI consumption
//! - **Constraints**: Protect critical code from unintended AI modifications
//! - **Variables**: Token-efficient references to code elements
//! - **Indexing**: Generate queryable cache of codebase structure
//!
//! ## Coming Soon
//!
//! - `acp-protocol init` - Initialize ACP in your project
//! - `acp-protocol index` - Index your codebase
//! - `acp-protocol query` - Query the index
//! - `acp-protocol constraints` - View file constraints
//! - `acp-protocol vars` - Generate variables
//!
//! ## Links
//!
//! - [GitHub Repository](https://github.com/acp-protocol/acp-spec)
//! - [Specification](https://github.com/acp-protocol/acp-spec/tree/main/spec)
//!
//! ## License
//!
//! MIT License - see repository for details.

/// Placeholder for ACP CLI functionality.
///
/// Full implementation coming soon.
pub fn version() -> &'static str {
    "0.0.1-placeholder"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.0.1-placeholder");
    }
}
