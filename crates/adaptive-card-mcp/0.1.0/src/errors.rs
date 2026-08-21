//! Convert `adaptive_card_core::Error` into user-facing text suitable for
//! MCP tool results.
//!
//! The vast majority of failures are surfaced via the tool result (with
//! `isError = true`) so the calling LLM can inspect the message and retry.
//! Only truly internal errors are escalated as JSON-RPC protocol errors.

#![allow(
    dead_code,
    reason = "consumed by tools:: handlers and wired into the server in Task 35"
)]

use adaptive_card_core::Error as CoreError;

/// Convert a core error into a user-facing message for tool results.
#[must_use]
pub fn to_tool_error_text(e: &CoreError) -> String {
    match e {
        CoreError::InvalidJson(_)
        | CoreError::NotAnAdaptiveCard
        | CoreError::UnsupportedVersion(_)
        | CoreError::HostIncompatible { .. }
        | CoreError::TransformLossy(_)
        | CoreError::UnrecognizedDataShape
        | CoreError::KnowledgeEntryNotFound { .. } => e.to_string(),
        CoreError::SchemaInvalid { errors, .. } => {
            format!("schema validation failed: {errors} error(s)")
        }
        CoreError::KnowledgeLoad(m) | CoreError::Internal(m) => format!("internal: {m}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_not_an_adaptive_card() {
        let e = CoreError::NotAnAdaptiveCard;
        let text = to_tool_error_text(&e);
        assert!(text.contains("AdaptiveCard"));
    }

    #[test]
    fn formats_unsupported_version() {
        let e = CoreError::UnsupportedVersion("2.0".into());
        assert_eq!(to_tool_error_text(&e), "unsupported card version: 2.0");
    }

    #[test]
    fn formats_internal_error_with_prefix() {
        let e = CoreError::Internal("oops".into());
        assert_eq!(to_tool_error_text(&e), "internal: oops");
    }
}
