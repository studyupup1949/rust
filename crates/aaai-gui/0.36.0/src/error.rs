//! User-facing error type for the GUI.
//!
//! RFC 020 (§2.2, §3.3) requires every error surfaced to a human carry both
//! a `message` (what went wrong) and a `hint` (the next thing to do). This
//! struct is the canonical carrier of that pair, replacing the older
//! pattern of stashing a single pre-formatted `String` in app state.
//!
//! Use [`UserError::new`] for ad-hoc errors where both halves are computed
//! locally, and [`UserError::from_i18n`] for the common case where both
//! halves live as adjacent keys in a `locales/*.yaml` file under the same
//! `error.<context>.<short_id>.{message,hint}` parent — see
//! `locales/en.yaml` for the established convention.
//!
//! The render path lives in `crate::views::*` — see the banner inside
//! `views::opening::view` for the canonical two-line layout.

/// An error to show to the user, in the RFC 020 "message + hint" form.
///
/// Both fields are already-resolved display strings (i18n already applied,
/// interpolation already done). They are NOT keys.
#[derive(Debug, Clone)]
pub struct UserError {
    /// What went wrong, including any concrete details (paths, codes,
    /// inner-error rendering). Shown prominently.
    pub message: String,
    /// What the user should do next. Shown beneath `message` in a quieter
    /// style. Plain prose, no jargon.
    pub hint: String,
}

impl UserError {
    /// Construct from already-resolved strings.
    pub fn new(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            hint: hint.into(),
        }
    }

    /// Construct by reading two adjacent i18n keys: `{prefix}.message`
    /// and `{prefix}.hint`. The convention is established in
    /// `locales/en.yaml` under `error.<context>.<short_id>` —
    /// e.g. `prefix = "error.save.failed"` reads `error.save.failed.message`
    /// and `error.save.failed.hint`.
    ///
    /// If either key is missing, the returned `UserError` will contain the
    /// raw key string in that field. This makes missing keys obvious during
    /// development without panicking in production.
    ///
    /// **Note for the i18n audit script:** `check-i18n-keys.py` recognises
    /// the `UserError::from_i18n("prefix")` call shape and treats the two
    /// derived keys (`prefix.message`, `prefix.hint`) as referenced. Don't
    /// rename this method without updating the script.
    pub fn from_i18n(prefix: &str) -> Self {
        let msg_key = format!("{prefix}.message");
        let hint_key = format!("{prefix}.hint");
        Self {
            message: rust_i18n::t!(&msg_key).to_string(),
            hint: rust_i18n::t!(&hint_key).to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_holds_both_fields() {
        let e = UserError::new("what", "what next");
        assert_eq!(e.message, "what");
        assert_eq!(e.hint, "what next");
    }

    #[test]
    fn from_i18n_resolves_message_and_hint() {
        // Set the locale to a known value to avoid test inter-dependency.
        rust_i18n::set_locale("en");
        // Use a prefix whose two derived keys exist in locales/en.yaml.
        // The literal prefix is intentionally split across `let` bindings
        // so the audit script doesn't treat the prefix itself as a
        // referenced key — only the two derived keys count.
        let prefix = ["error", "save", "failed"].join(".");
        let e = UserError::from_i18n(&prefix);
        // Fields are populated and not the literal key (which would
        // indicate a missing translation lookup).
        assert!(!e.message.contains(&prefix));
        assert!(!e.hint.contains(&prefix));
        assert!(!e.message.is_empty());
        assert!(!e.hint.is_empty());
    }
}
