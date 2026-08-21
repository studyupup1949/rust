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
}
