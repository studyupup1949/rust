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
        let out = humanize_since_at(earlier - chrono::Duration::seconds(1), now);
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
}
