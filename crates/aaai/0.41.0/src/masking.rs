pub mod engine;
pub mod patterns;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod prop_tests;

use engine::MaskingEngine;

/// Explicit, non-optional masking choice for an output surface.
///
/// Replaces `Option<&MaskingEngine>`, under which `None` was a legal, silent,
/// unmarked choice to emit unmasked output — the root cause of RFC 103's
/// F3/F4/F5 (SARIF, CSV/TSV export, and JSON never masked, because nothing
/// forced a caller to decide). See
/// `rfcs/proposed/103-safe-output-surfaces.md` §5.1.
///
/// Deliberately carries no `From<Option<&MaskingEngine>>`, `Default`, or
/// other source-compatibility shim: any of those would let an existing
/// `None` call site keep compiling, which would silently preserve the
/// defect this type exists to close. Every caller must be edited by hand.
#[derive(Clone, Copy)]
pub enum Masking<'a> {
    Enabled(&'a MaskingEngine),
    /// Caller asserts the sink is trusted. Must be justified at the call
    /// site with a comment; if you cannot write that justification
    /// honestly, the answer is `Enabled`.
    Disabled,
}

impl Masking<'_> {
    /// Mask `text` if enabled; return it unchanged if disabled.
    pub fn mask(&self, text: &str) -> String {
        match self {
            Masking::Enabled(engine) => engine.mask(text),
            Masking::Disabled => text.to_string(),
        }
    }
}
