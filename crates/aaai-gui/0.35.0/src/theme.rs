//! Shared color constants for aaai GUI.
//!
//! # WCAG AA compliance
//!
//! Every status color below is verified to meet WCAG 2.1 AA contrast (>= 4.5:1)
//! against white in **both** directions it is used:
//!   - as a **badge background** with white text (inspector / dashboard badges)
//!   - as **text foreground** on a white surface (status pill, diff lines)
//!
//! Because contrast is symmetric, a single >= 4.5:1 figure covers both uses.
//!
//! The OK / Pending / Failed values are adopted from the snora-design light
//! preset's AA-tested `success` / `warning` / `danger` roles (snora-design
//! 0.25.1). Adopting the constant values rather than the full design-token
//! runtime keeps this fix minimal; the full design-system adoption is tracked
//! in RFC 092. Error and Ignored have no clean snora role (see RFC 092) and
//! are hand-picked to clear the same AA bar.
//!
//! Contrast figures (white <-> color), measured with WCAG 2.1 relative luminance:
//!
//! | Constant | Hex       | Contrast | Provenance |
//! |----------|-----------|----------|------------|
//! | OK       | `#15803D` | 5.02:1   | snora-design `success` |
//! | PENDING  | `#9A5B00` | 5.43:1   | snora-design `warning` |
//! | FAILED   | `#B3261E` | 6.54:1   | snora-design `danger`  |
//! | ERROR    | `#B22EB2` | 5.33:1   | hand-picked (purple, kept) |
//! | IGNORED  | `#6B6B6B` | 5.32:1   | hand-picked (darkened from #8C8C8C) |
//! | ADDED    | `#15803D` | 5.02:1   | snora-design `success` |
//! | REMOVED  | `#B3261E` | 6.54:1   | snora-design `danger`  |

use iced::Color;

/// OK / passed status. snora-design `success` (#15803D), 5.02:1 on white.
pub const OK_COLOR: Color      = Color { r: 0.082353, g: 0.501961, b: 0.239216, a: 1.0 };
/// Pending / needs-review status. snora-design `warning` (#9A5B00), 5.43:1.
///
/// Replaces the previous orange `#E0991F`, which was **2.40:1** — a WCAG AA
/// failure both as a white-on-orange badge and as orange text on white.
pub const PENDING_COLOR: Color = Color { r: 0.603922, g: 0.356863, b: 0.000000, a: 1.0 };
/// Failed status. snora-design `danger` (#B3261E), 6.54:1 on white.
pub const FAILED_COLOR: Color  = Color { r: 0.701961, g: 0.149020, b: 0.117647, a: 1.0 };
/// Error status (file unreadable). Hand-picked purple (#B22EB2), 5.33:1.
/// Kept distinct from FAILED so "couldn't read" reads differently from
/// "rule no longer matches" (design doc p.9 status-vocabulary distinction).
pub const ERROR_COLOR: Color   = Color { r: 0.70, g: 0.18, b: 0.70, a: 1.0 };
/// Ignored / skipped status. Hand-picked grey (#6B6B6B), 5.32:1.
///
/// Darkened from the previous `#8C8C8C` (3.35:1 — a white-on-grey AA failure).
pub const IGNORED_COLOR: Color = Color { r: 0.420000, g: 0.420000, b: 0.420000, a: 1.0 };

/// Added / present / OK indicator in diffs (green). Same value as OK_COLOR.
pub const ADDED_COLOR: Color   = Color { r: 0.082353, g: 0.501961, b: 0.239216, a: 1.0 };
/// Removed / missing / failed indicator in diffs (red). Same value as FAILED_COLOR.
pub const REMOVED_COLOR: Color = Color { r: 0.701961, g: 0.149020, b: 0.117647, a: 1.0 };

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Color;

    /// WCAG 2.1 relative luminance.
    fn luminance(c: Color) -> f32 {
        fn lin(ch: f32) -> f32 {
            if ch <= 0.03928 { ch / 12.92 } else { ((ch + 0.055) / 1.055).powf(2.4) }
        }
        0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
    }

    /// WCAG 2.1 contrast ratio between two colors.
    fn contrast(a: Color, b: Color) -> f32 {
        let (l1, l2) = (luminance(a), luminance(b));
        let (hi, lo) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (hi + 0.05) / (lo + 0.05)
    }

    const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    const AA_NORMAL: f32 = 4.5;

    /// Every status color must meet WCAG AA against white.
    ///
    /// Status colors serve both as badge backgrounds (white text on the color)
    /// and as text foreground on white. Contrast is symmetric, so one check
    /// covers both. This test guards against a future regression that
    /// reintroduces a low-contrast value (the orange Pending #E0991F that this
    /// module replaced was 2.40:1).
    #[test]
    fn all_status_colors_meet_wcag_aa_on_white() {
        let cases = [
            ("OK",      OK_COLOR),
            ("PENDING", PENDING_COLOR),
            ("FAILED",  FAILED_COLOR),
            ("ERROR",   ERROR_COLOR),
            ("IGNORED", IGNORED_COLOR),
            ("ADDED",   ADDED_COLOR),
            ("REMOVED", REMOVED_COLOR),
        ];
        for (name, color) in cases {
            let ratio = contrast(color, WHITE);
            assert!(
                ratio >= AA_NORMAL,
                "{name} contrast {ratio:.2}:1 is below WCAG AA {AA_NORMAL}:1",
            );
        }
    }

    /// Sanity check on the contrast helper itself: black on white is ~21:1.
    #[test]
    fn contrast_helper_matches_wcag_reference() {
        let black = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
        let ratio = contrast(black, WHITE);
        assert!((ratio - 21.0).abs() < 0.1, "expected ~21:1, got {ratio:.2}");
    }
}
