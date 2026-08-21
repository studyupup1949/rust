//! Asserts every text-on-surface role pair the views apply as plain text
//! colour reaches WCAG AA (>= 4.5:1), and >= 7:1 for the two high-contrast
//! presets, using `snora::design::contrast::contrast_ratio` — the exact
//! function the built-in presets are themselves validated against.
//!
//! RFC 099's gap analysis
//! (`.git-exclude/reviewed/037-gui-uiux-gap-analysis-2026-07-28.md` §2.4)
//! measured four pairs that failed AA in light mode before token migration:
//! dashboard muted label 3.17:1, muted text on panel 3.71:1, muted text on
//! card 3.81:1, diff secondary text 4.13:1. Every one of those call sites was
//! migrated to `palette.text_secondary`, which this test now holds to the
//! same guarantee the token system already documents for that role ("still
//! meets body contrast on primary surfaces"). Before that migration this
//! assertion would have failed for the `text_secondary` / light-preset case
//! at the measured ratios above; it passes now because the call sites read
//! the token instead of a hardcoded literal.
//!
//! `text_muted` is intentionally excluded: `Palette` documents it as "exempt
//! from mandatory contrast checks", used deliberately for non-essential
//! content (hints, timestamps, decorative labels) throughout the views.
//!
//! Status-role text-on-status-background pairs (e.g. white text on a
//! `danger` badge) are not repeated here — `crate::theme::tests` already
//! covers those. This module covers the complementary case introduced by
//! RFC 099: status/accent colours used as plain foreground text directly on
//! the application's neutral surfaces.

use snora::design::contrast::contrast_ratio;
use snora::design::{Color, Tokens};

struct Preset {
    name: &'static str,
    tokens: Tokens,
    min_ratio: f32,
}

fn presets() -> [Preset; 4] {
    [
        Preset {
            name: "light",
            tokens: Tokens::light(),
            min_ratio: 4.5,
        },
        Preset {
            name: "dark",
            tokens: Tokens::dark(),
            min_ratio: 4.5,
        },
        Preset {
            name: "high_contrast_light",
            tokens: Tokens::high_contrast_light(),
            min_ratio: 7.0,
        },
        Preset {
            name: "high_contrast_dark",
            tokens: Tokens::high_contrast_dark(),
            min_ratio: 7.0,
        },
    ]
}

/// Foreground roles the views apply directly as plain text colour.
fn foreground_roles(tokens: &Tokens) -> [(&'static str, Color); 7] {
    [
        ("text_primary", tokens.palette.text_primary),
        ("text_secondary", tokens.palette.text_secondary),
        ("accent", tokens.palette.accent),
        ("success", tokens.palette.success),
        ("warning", tokens.palette.warning),
        ("danger", tokens.palette.danger),
        ("info", tokens.palette.info),
    ]
}

/// Neutral surfaces the views render that text directly.
fn background_roles(tokens: &Tokens) -> [(&'static str, Color); 3] {
    [
        ("background", tokens.palette.background),
        ("surface", tokens.palette.surface),
        ("surface_raised", tokens.palette.surface_raised),
    ]
}

/// One of the four pairs RFC 099 §2.4 measured as failing AA in light mode.
struct HistoricalPair {
    site: &'static str,
    /// The literal `Color::from_rgb` values the pre-migration source used.
    old_fg: Color,
    old_bg: Color,
    /// The measured ratio recorded in RFC 099 §2.4 for `old_fg`/`old_bg`.
    old_ratio: f32,
    /// The token role that now supplies the foreground at this site.
    new_fg: Color,
}

fn historical_pairs(tokens: &Tokens) -> [HistoricalPair; 4] {
    [
        HistoricalPair {
            site: "dashboard muted label",
            old_fg: Color::rgb(0.55, 0.55, 0.60),
            old_bg: Color::rgb(0.98, 0.98, 0.99),
            old_ratio: 3.17,
            new_fg: tokens.palette.text_secondary,
        },
        HistoricalPair {
            site: "muted text on panel",
            old_fg: Color::rgb(0.5, 0.5, 0.5),
            old_bg: Color::rgb(0.96, 0.97, 0.98),
            old_ratio: 3.71,
            new_fg: tokens.palette.text_secondary,
        },
        HistoricalPair {
            site: "muted text on card",
            old_fg: Color::rgb(0.5, 0.5, 0.5),
            old_bg: Color::rgb(0.98, 0.98, 0.99),
            old_ratio: 3.81,
            new_fg: tokens.palette.text_secondary,
        },
        HistoricalPair {
            site: "diff secondary text",
            old_fg: Color::rgb(0.45, 0.47, 0.52),
            old_bg: Color::rgb(0.96, 0.97, 0.98),
            old_ratio: 4.13,
            new_fg: tokens.palette.text_secondary,
        },
    ]
}

#[test]
fn every_text_on_surface_pair_meets_its_preset_threshold() {
    let mut failures = Vec::new();

    for preset in presets() {
        for (fg_name, fg) in foreground_roles(&preset.tokens) {
            for (bg_name, bg) in background_roles(&preset.tokens) {
                let ratio = contrast_ratio(fg, bg);
                if ratio < preset.min_ratio {
                    failures.push(format!(
                        "{}: {} on {} is {:.2}:1, need >= {:.1}:1",
                        preset.name, fg_name, bg_name, ratio, preset.min_ratio
                    ));
                }
            }
        }
    }

    // Historical anchor: reproduce the exact four pairs RFC 099 §2.4 measured
    // as failing AA in light mode using the literal hardcoded RGB values the
    // pre-T1 source actually used at each site, confirm they still fail
    // exactly as recorded, then confirm the token-derived replacement now in
    // place at that same site passes. If `text_secondary` were ever
    // hardcoded back to a value this dark, this loop would catch it failing
    // exactly as the original code did.
    let light = Tokens::light();
    for case in historical_pairs(&light) {
        let old_measured = contrast_ratio(case.old_fg, case.old_bg);
        if (old_measured - case.old_ratio).abs() >= 0.02 {
            failures.push(format!(
                "{}: recomputed old ratio {:.2}:1 does not match RFC 099 \
                 §2.4's recorded {:.2}:1 — the historical record is wrong, \
                 not the fix",
                case.site, old_measured, case.old_ratio
            ));
        }
        if old_measured >= 4.5 {
            failures.push(format!(
                "{}: the historical pair no longer fails AA at {:.2}:1 — \
                 confirm RFC 099 §2.4 before relying on this as a regression \
                 proof",
                case.site, old_measured
            ));
        }
        let new_ratio = contrast_ratio(case.new_fg, case.old_bg);
        if new_ratio < 4.5 {
            failures.push(format!(
                "{}: token replacement is {:.2}:1 on the original measured \
                 background, need >= 4.5:1",
                case.site, new_ratio
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "contrast failures:\n{}",
        failures.join("\n")
    );
}
