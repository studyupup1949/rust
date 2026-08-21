//! Error type for the core library.

use crate::types::{Host, ValidationReport};

/// Result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Library error enum. Specific variants let callers match on failure mode.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid card JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("card is not an AdaptiveCard (missing or invalid 'type' field)")]
    NotAnAdaptiveCard,

    #[error("schema validation failed: {errors} error(s)")]
    SchemaInvalid {
        errors: usize,
        report: Box<ValidationReport>,
    },

    #[error("unsupported card version: {0}")]
    UnsupportedVersion(String),

    #[error("host {host:?} incompatible: {reason}")]
    HostIncompatible { host: Host, reason: String },

    #[error("transform would lose data (strict mode): {0}")]
    TransformLossy(String),

    #[error("data shape not recognized for data_to_card")]
    UnrecognizedDataShape,

    #[error("knowledge base entry not found: {id}")]
    KnowledgeEntryNotFound { id: String },

    #[error("knowledge base load failed: {0}")]
    KnowledgeLoad(String),

    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_formats() {
        let e = Error::NotAnAdaptiveCard;
        assert_eq!(
            e.to_string(),
            "card is not an AdaptiveCard (missing or invalid 'type' field)"
        );

        let e = Error::UnsupportedVersion("2.0".to_string());
        assert_eq!(e.to_string(), "unsupported card version: 2.0");
    }

    #[test]
    fn error_from_serde_json() {
        let bad: Result<serde_json::Value> = serde_json::from_str("not json").map_err(Error::from);
        assert!(matches!(bad, Err(Error::InvalidJson(_))));
    }
}
