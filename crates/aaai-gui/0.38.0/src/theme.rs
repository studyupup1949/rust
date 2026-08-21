//! Status colors and the token-aware status-color resolver for aaai GUI.
//!
//! # WCAG AA / AAA compliance
//!
//! Every color in this module is verified to meet WCAG 2.1 AA contrast (≥ 4.5:1)
//! against white in both directions it is used:
//!   - as a **badge background** with white text (inspector / dashboard badges)
//!   - as **text foreground** on a white surface (status pill, diff lines)
//!
//! High-contrast variants reach ≥ 7:1 (approaching WCAG AAA) as required by
//! RFC 094.
//!
//! # Approach 1 (RFC 094 §3.7)
//!
//! OK / Pending / Failed colors are read live from `tokens.palette` so they
//! follow the active snora-design preset (including the HC presets). Error and
//! Ignored have no snora palette role; they are hand-picked below.
//!
//! The pixel-identical invariant for Light/Dark (RFC 094 §5.1) holds because
//! `Tokens::light().palette.success` is exactly the same float triple as the
//! v0.35.0 `OK_COLOR` constant: the snora light preset is the source of both.
//!
//! | Color | Standard | HC variant | Provenance |
//! |---|---|---|---|
//! | OK      | token `success`  | token `success` HC  | snora-design |
//! | Pending | token `warning`  | token `warning` HC  | snora-design |
//! | Failed  | token `danger`   | token `danger` HC   | snora-design |
//! | Error   | `#B22EB2` 5.33:1 | `#7B1F7B` 9.04:1   | hand-picked  |
//! | Ignored | `#6B6B6B` 5.32:1 | `#525252` 7.81:1   | hand-picked  |
//! | Added   | token `success`  | token `success` HC  | snora-design |
//! | Removed | token `danger`   | token `danger` HC   | snora-design |

use iced::Color;
use aaai_core::AuditStatus;
use snora::design::Tokens;

// ── Hand-picked constants for roles not covered by snora-design ───────────

/// Standard error color — purple #B22EB2, 5.33:1 on white.
/// Kept distinct from FAILED so "couldn't read" reads differently from
/// "rule no longer matches" (design doc p.9 status-vocabulary distinction).
pub const ERROR_COLOR: Color =
    Color { r: 0.70, g: 0.18, b: 0.70, a: 1.0 };

/// High-contrast error color — purple #7B1F7B, 9.04:1 on white.
pub const ERROR_HC: Color =
    Color { r: 0.482353, g: 0.121569, b: 0.482353, a: 1.0 };

/// Standard ignored color — grey #6B6B6B, 5.32:1 on white.
pub const IGNORED_COLOR: Color =
    Color { r: 0.420000, g: 0.420000, b: 0.420000, a: 1.0 };

/// High-contrast ignored color — grey #525252, 7.81:1 on white.
pub const IGNORED_HC: Color =
    Color { r: 0.321569, g: 0.321569, b: 0.321569, a: 1.0 };

// ── Token-aware status color resolver ────────────────────────────────────

/// Convert a `snora_design::Color` to an `iced::Color`.
#[inline]
fn to_iced(c: snora::design::Color) -> Color {
    Color { r: c.r, g: c.g, b: c.b, a: c.a }
}

/// Return the display color for an [`AuditStatus`], respecting the active
/// design-token preset.
///
/// For standard themes (Light / Dark), OK / Pending / Failed resolve to the
/// same values as the v0.35.0 hand-picked constants — pixel-identical, just
/// read from the token palette rather than a hard-coded literal. Under
/// high-contrast presets they escalate automatically to ≥ 8:1 values.
/// Error and Ignored fall back to hand-picked HC constants (no snora role).
/// `is_hc` is `app.theme.is_high_contrast()`.
pub fn status_color(status: AuditStatus, tokens: &Tokens, is_hc: bool) -> Color {
    match status {
        AuditStatus::Ok      => to_iced(tokens.palette.success),
        AuditStatus::Pending => to_iced(tokens.palette.warning),
        AuditStatus::Failed  => to_iced(tokens.palette.danger),
        AuditStatus::Error   => if is_hc { ERROR_HC }   else { ERROR_COLOR },
        AuditStatus::Ignored => if is_hc { IGNORED_HC } else { IGNORED_COLOR },
    }
}

/// Shorthand for diff-view added lines — same as OK / success.
pub fn added_color(tokens: &Tokens) -> Color { to_iced(tokens.palette.success) }

/// Shorthand for diff-view removed lines — same as Failed / danger.
pub fn removed_color(tokens: &Tokens) -> Color { to_iced(tokens.palette.danger) }

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Color;

    fn luminance(c: Color) -> f32 {
        fn lin(ch: f32) -> f32 {
            if ch <= 0.03928 { ch / 12.92 } else { ((ch + 0.055) / 1.055).powf(2.4) }
        }
        0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
    }
    fn contrast(a: Color, b: Color) -> f32 {
        let (l1, l2) = (luminance(a), luminance(b));
        let (hi, lo) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (hi + 0.05) / (lo + 0.05)
    }
    const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    /// All standard-theme status colors must meet WCAG AA (≥ 4.5:1).
    #[test]
    fn standard_status_colors_meet_aa() {
        let tokens = snora::design::Tokens::light();
        let statuses = [
            AuditStatus::Ok,
            AuditStatus::Pending,
            AuditStatus::Failed,
            AuditStatus::Error,
            AuditStatus::Ignored,
        ];
        for s in statuses {
            let c = status_color(s, &tokens, false);
            let r = contrast(c, WHITE);
            assert!(r >= 4.5, "{s:?} standard contrast {r:.2}:1 < 4.5:1");
        }
    }

    /// All HC-theme status colors must meet ≥ 7:1.
    #[test]
    fn hc_status_colors_meet_enhanced_contrast() {
        let tokens = snora::design::Tokens::high_contrast_light();
        let statuses = [
            AuditStatus::Ok,
            AuditStatus::Pending,
            AuditStatus::Failed,
            AuditStatus::Error,
            AuditStatus::Ignored,
        ];
        for s in statuses {
            let c = status_color(s, &tokens, true);
            let r = contrast(c, WHITE);
            assert!(r >= 7.0, "{s:?} HC contrast {r:.2}:1 < 7:1");
        }
    }

    /// Light-theme status colors are pixel-identical to the v0.35.0 constants.
    #[test]
    fn light_theme_pixels_identical_to_v035_constants() {
        let tokens = snora::design::Tokens::light();
        // v0.35.0 constants (from snora-design light palette)
        let v035 = [
            (AuditStatus::Ok,      Color { r: 0.082353, g: 0.501961, b: 0.239216, a: 1.0 }),
            (AuditStatus::Pending, Color { r: 0.603922, g: 0.356863, b: 0.000000, a: 1.0 }),
            (AuditStatus::Failed,  Color { r: 0.701961, g: 0.149020, b: 0.117647, a: 1.0 }),
            (AuditStatus::Error,   ERROR_COLOR),
            (AuditStatus::Ignored, IGNORED_COLOR),
        ];
        for (s, expected) in v035 {
            let got = status_color(s, &tokens, false);
            let diff = (got.r - expected.r).abs()
                .max((got.g - expected.g).abs())
                .max((got.b - expected.b).abs());
            assert!(diff < 1e-5,
                "{s:?}: got ({:.6},{:.6},{:.6}) expected ({:.6},{:.6},{:.6})",
                got.r, got.g, got.b, expected.r, expected.g, expected.b);
        }
    }

    /// Sanity check for the contrast helper.
    #[test]
    fn contrast_helper_black_on_white_is_21() {
        let black = Color::BLACK;
        let r = contrast(black, WHITE);
        assert!((r - 21.0).abs() < 0.1, "expected ~21:1 got {r:.2}");
    }
}
