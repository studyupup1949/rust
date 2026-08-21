//! Small utility helpers for view layers.

use chrono::{DateTime, Utc};
use rust_i18n::t;

/// Render a UTC timestamp as a short human-readable "time ago" string,
/// suitable for the Recent-projects list per RFC 023 §3.4.
///
/// Buckets:
///   - `< 60 seconds`  → "Just now"
///   - `< 60 minutes`  → "N min ago"
///   - `< 24 hours`    → "N h ago"
///   - `< 7 days`      → "N d ago"
///   - older           → absolute date `YYYY-MM-DD`
///
/// The relative buckets resolve through i18n (`relative.*`); the absolute
/// date is locale-independent (ISO-style is unambiguous and stays inside
/// the chrono `format!` macro, so no i18n key is needed).
///
/// `now` is taken from the system clock at call time. For unit-testing
/// without a clock dependency, use [`humanize_since_at`].
pub fn humanize_since(t: DateTime<Utc>) -> String {
    humanize_since_at(t, Utc::now())
}

/// Inner form of [`humanize_since`] that takes an explicit "now" so tests
/// don't depend on the real clock.
pub fn humanize_since_at(t: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let delta = now - t;

    // Future timestamps (clock skew or test data) fall through to "Just now"
    // rather than emitting a confusing negative count.
    if delta.num_seconds() < 60 {
        return t!("relative.just_now").to_string();
    }
    if delta.num_minutes() < 60 {
        return t!("relative.minutes_ago", n = delta.num_minutes().to_string()).to_string();
    }
    if delta.num_hours() < 24 {
        return t!("relative.hours_ago", n = delta.num_hours().to_string()).to_string();
    }
    if delta.num_days() < 7 {
        return t!("relative.days_ago", n = delta.num_days().to_string()).to_string();
    }
    t.format("%Y-%m-%d").to_string()
}

// ── LocalizedOption (RFC 033) ─────────────────────────────────────────────

/// Pairs a Rust enum variant with its localized display label for use as a
/// `pick_list` option. The variant is the canonical identity; the label is
/// the human-readable form rendered for the current locale.
///
/// The `PartialEq` implementation compares **by `value` only**, not by
/// label. This is the key trick that makes pick_list selection work
/// across locales: the picker uses equality to identify "the currently
/// selected option," and selecting by enum value rather than label
/// means changing the locale (or the label text) doesn't break selection
/// identity.
///
/// Use this in two places per picker:
/// 1. Build a `Vec<LocalizedOption<T>>` of options with localized labels
/// 2. Send the `LocalizedOption<T>` to `pick_list`; in the callback,
///    extract `o.value` and dispatch to a Message variant carrying `T`.
#[derive(Debug, Clone)]
pub struct LocalizedOption<T: Clone + PartialEq> {
    pub value: T,
    pub label: String,
}

impl<T: Clone + PartialEq> std::fmt::Display for LocalizedOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

impl<T: Clone + PartialEq> PartialEq for LocalizedOption<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Clone + PartialEq> Eq for LocalizedOption<T> {}

// ── StrategyKind (RFC 035) ────────────────────────────────────────────────

use aaai_core::config::definition::{AuditStrategy, RegexTarget};

/// Discriminator for `AuditStrategy` variants without their associated data.
/// Used as the value type for the strategy picker via
/// `LocalizedOption<StrategyKind>`.
///
/// This is a GUI-layer concern (display + Message protocol identity).
/// `aaai-core` continues to expose only `AuditStrategy`; this discriminator
/// stays inside the GUI's display layer.
///
/// The variants mirror `AuditStrategy`'s variants one-for-one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyKind {
    None,
    Checksum,
    LineMatch,
    Regex,
    Exact,
}

impl StrategyKind {
    /// Construct a zero-value `AuditStrategy` for this kind.
    /// Used when the picker selects a new kind — the inspector
    /// state's strategy is replaced with a fresh default of
    /// that variant (e.g. an empty `expected_sha256`, no rules).
    pub fn to_default_strategy(self) -> AuditStrategy {
        match self {
            StrategyKind::None      => AuditStrategy::None,
            StrategyKind::Checksum  => AuditStrategy::Checksum { expected_sha256: String::new() },
            StrategyKind::LineMatch => AuditStrategy::LineMatch { rules: Vec::new() },
            StrategyKind::Regex     => AuditStrategy::Regex { pattern: String::new(), target: RegexTarget::AddedLines },
            StrategyKind::Exact     => AuditStrategy::Exact { expected_content: String::new() },
        }
    }

    /// Read the kind from an existing strategy.
    pub fn from_strategy(s: &AuditStrategy) -> StrategyKind {
        match s {
            AuditStrategy::None          => StrategyKind::None,
            AuditStrategy::Checksum {..} => StrategyKind::Checksum,
            AuditStrategy::LineMatch{..} => StrategyKind::LineMatch,
            AuditStrategy::Regex {..}    => StrategyKind::Regex,
            AuditStrategy::Exact {..}    => StrategyKind::Exact,
        }
    }

    /// Localised label for the picker.
    /// Resolves through `inspector.strategy_{none,checksum,linematch,regex,exact}`.
    pub fn label(self) -> String {
        match self {
            StrategyKind::None      => t!("inspector.strategy_none"),
            StrategyKind::Checksum  => t!("inspector.strategy_checksum"),
            StrategyKind::LineMatch => t!("inspector.strategy_linematch"),
            StrategyKind::Regex     => t!("inspector.strategy_regex"),
            StrategyKind::Exact     => t!("inspector.strategy_exact"),
        }.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // rust_i18n is initialised by main.rs; tests run inside the same
    // crate so the macro registry is available. We test the bucket
    // dispatch logic by asserting which i18n key fires (presence of
    // expected words in the output), not the exact wording — that's
    // a translation concern.

    fn t(year: i32, month: u32, day: u32, h: u32, m: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(year, month, day, h, m, 0).unwrap()
    }

    #[test]
    fn within_a_minute_is_just_now() {
        let now = t(2026, 5, 13, 12, 0);
        let earlier = t(2026, 5, 13, 11, 59);  // 60 s ago — boundary
        // Move 1 s into the "just now" side of the boundary (59 s ago,
        // not 61 s ago — the original `-1s` was a Phase 12 typo).
        let out = humanize_since_at(earlier + chrono::Duration::seconds(1), now);
        // Either translation; we just check the key resolved (no "min", no "h", no "d")
        assert!(!out.contains(" min "));
        assert!(!out.contains(" h "));
        assert!(!out.contains(" d "));
    }

    #[test]
    fn minutes_bucket() {
        let now = t(2026, 5, 13, 12, 0);
        let earlier = t(2026, 5, 13, 11, 55);  // 5 min ago
        let out = humanize_since_at(earlier, now);
        assert!(out.contains("5"), "expected '5' in output: {out}");
        // contains the "min" or Japanese-equivalent fragment via key resolution
    }

    #[test]
    fn hours_bucket() {
        let now = t(2026, 5, 13, 12, 0);
        let earlier = t(2026, 5, 13, 9, 0);   // 3 h ago
        let out = humanize_since_at(earlier, now);
        assert!(out.contains("3"), "expected '3' in output: {out}");
    }

    #[test]
    fn days_bucket() {
        let now = t(2026, 5, 13, 12, 0);
        let earlier = t(2026, 5, 10, 12, 0);  // 3 d ago
        let out = humanize_since_at(earlier, now);
        assert!(out.contains("3"), "expected '3' in output: {out}");
    }

    #[test]
    fn beyond_a_week_falls_back_to_absolute_date() {
        let now = t(2026, 5, 13, 12, 0);
        let earlier = t(2026, 4, 1, 12, 0);   // 42 d ago — well past a week
        let out = humanize_since_at(earlier, now);
        assert_eq!(out, "2026-04-01",
            "beyond 7 days should be an ISO date: {out}");
    }

    #[test]
    fn exactly_seven_days_is_already_absolute() {
        let now = t(2026, 5, 13, 12, 0);
        let earlier = t(2026, 5,  6, 12, 0);  // 7 d ago exactly
        let out = humanize_since_at(earlier, now);
        assert_eq!(out, "2026-05-06");
    }

    #[test]
    fn future_timestamp_does_not_panic() {
        let now = t(2026, 5, 13, 12, 0);
        let later = t(2026, 5, 13, 12, 5);  // 5 min in the future
        let out = humanize_since_at(later, now);
        // Just confirm we get some string back rather than panic.
        assert!(!out.is_empty());
    }

    // ── LocalizedOption (RFC 033) ────────────────────────────────────

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestAction { A, B }

    /// RFC 033 — verify equality compares by `value` alone, not by
    /// `label`. This is the property that makes pick_list selection
    /// work correctly when the locale changes: the picker's
    /// "currently selected" identity is preserved across translations.
    #[test]
    fn localized_option_equality_ignores_label() {
        let a = LocalizedOption { value: TestAction::A, label: "Added".into() };
        let b = LocalizedOption { value: TestAction::A, label: "追加".into() };
        assert_eq!(a, b);  // same value, different label
    }

    /// RFC 033 — verify inequality when values differ even if labels
    /// happen to match. This is unlikely in practice (labels are
    /// usually distinct) but the contract is: value determines identity.
    #[test]
    fn localized_option_inequality_by_value() {
        let a = LocalizedOption { value: TestAction::A, label: "X".into() };
        let b = LocalizedOption { value: TestAction::B, label: "X".into() };
        assert_ne!(a, b);  // different value, same label
    }

    // ── StrategyKind (RFC 035) ───────────────────────────────────────

    /// RFC 035 — verify the discriminator round-trips through
    /// `to_default_strategy()` and back via `from_strategy()`.
    /// This is the key contract for the picker: selecting a kind
    /// produces a strategy whose kind matches.
    #[test]
    fn strategy_kind_roundtrips_through_strategy() {
        for kind in [StrategyKind::None, StrategyKind::Checksum,
                     StrategyKind::LineMatch, StrategyKind::Regex,
                     StrategyKind::Exact] {
            let strategy = kind.to_default_strategy();
            assert_eq!(StrategyKind::from_strategy(&strategy), kind,
                "round-trip failed for {kind:?}");
        }
    }

    /// RFC 035 — `AuditStrategy::default()` is `None`; verify our
    /// discriminator agrees with that.
    #[test]
    fn strategy_kind_default_is_none() {
        let default_strategy = AuditStrategy::default();
        assert_eq!(StrategyKind::from_strategy(&default_strategy), StrategyKind::None);
    }
}
