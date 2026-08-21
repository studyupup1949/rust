pub mod worker;

use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use croner::Cron;
use croner::parser::{CronParser, Seconds};

/// Parse a 5-field cron expression (no seconds field).
///
/// # Errors
/// Returns an error if the expression is not a valid cron pattern.
pub fn parse_cron(pattern: &str) -> Result<Cron> {
    CronParser::builder()
        .seconds(Seconds::Disallowed)
        .build()
        .parse(pattern)
        .with_context(|| format!("failed to parse cron expression: {pattern}"))
}

/// Parse an IANA timezone name.
///
/// # Errors
/// Returns an error if the timezone name is not recognized.
pub fn parse_timezone(name: &str) -> Result<Tz> {
    Tz::from_str(name).with_context(|| format!("failed to parse timezone: {name}"))
}

/// Compute the next run time in UTC for a cron pattern and timezone.
///
/// # Errors
/// Returns an error if the cron pattern or timezone is invalid, or no next occurrence exists.
pub fn next_run_utc(cron: &Cron, timezone: Tz, after: DateTime<Utc>) -> Result<DateTime<Utc>> {
    let after_in_tz = after.with_timezone(&timezone);
    let next_in_tz = cron
        .find_next_occurrence(&after_in_tz, false)
        .with_context(|| "failed to find next occurrence for cron pattern")?;
    Ok(next_in_tz.with_timezone(&Utc))
}

/// Generate a human-readable description of a cron pattern.
#[must_use]
pub fn describe_schedule(cron: &Cron) -> String {
    cron.describe()
}

#[must_use]
pub fn format_remaining_with_now(until: DateTime<Utc>, now: DateTime<Utc>) -> String {
    if until <= now {
        return "overdue".to_string();
    }
    let diff = until - now;
    let days = diff.num_days();
    let hours = diff.num_hours() % 24;
    let minutes = diff.num_minutes() % 60;
    match (days, hours, minutes) {
        (0, 0, 0) => "in less than a minute".to_string(),
        (0, 0, m) => format!("in {m} minute{}", if m == 1 { "" } else { "s" }),
        (0, h, 0) => format!("in {h} hour{}", if h == 1 { "" } else { "s" }),
        (0, h, m) => format!(
            "in {h} hour{} and {m} minute{}",
            if h == 1 { "" } else { "s" },
            if m == 1 { "" } else { "s" }
        ),
        (d, 0, 0) => format!("in {d} day{}", if d == 1 { "" } else { "s" }),
        (d, h, 0) => format!(
            "in {d} day{} and {h} hour{}",
            if d == 1 { "" } else { "s" },
            if h == 1 { "" } else { "s" }
        ),
        (d, 0, m) => format!(
            "in {d} day{} and {m} minute{}",
            if d == 1 { "" } else { "s" },
            if m == 1 { "" } else { "s" }
        ),
        (d, h, m) => format!(
            "in {d} day{}, {h} hour{}, and {m} minute{}",
            if d == 1 { "" } else { "s" },
            if h == 1 { "" } else { "s" },
            if m == 1 { "" } else { "s" }
        ),
    }
}

/// Format the remaining time until a future UTC timestamp as a human-readable
/// English string (e.g. "in 2 hours 15 minutes").
#[must_use]
pub fn format_remaining(until: DateTime<Utc>) -> String {
    format_remaining_with_now(until, Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_cron() {
        let cron = parse_cron("0 9 * * MON-FRI").expect("valid cron");
        let desc = describe_schedule(&cron);
        assert!(desc.contains("09:00"), "desc should contain time: {}", desc);
        assert!(
            desc.contains("Monday") || desc.contains("Mon"),
            "desc should indicate weekday: {}",
            desc
        );
    }

    #[test]
    fn parse_invalid_cron_fails() {
        assert!(parse_cron("invalid").is_err());
    }

    #[test]
    fn parse_seconds_field_fails() {
        // Seconds field should be disallowed.
        assert!(parse_cron("0 0 9 * * MON-FRI").is_err());
    }

    #[test]
    fn next_run_with_timezone() {
        let cron = parse_cron("0 9 * * *").expect("valid cron");
        let tz = parse_timezone("Europe/Moscow").expect("valid timezone");
        let after = Utc::now();
        let next = next_run_utc(&cron, tz, after).expect("next run exists");
        assert!(next > after);
    }

    #[test]
    fn parse_timezone_invalid() {
        assert!(parse_timezone("Mars/Colony").is_err());
    }

    #[test]
    fn next_run_at_fixed_time() {
        let cron = parse_cron("0 9 * * *").expect("valid cron");
        let tz = parse_timezone("UTC").expect("valid timezone");
        // 2024-01-01 08:00 UTC -> next run is 2024-01-01 09:00 UTC
        let after = DateTime::parse_from_rfc3339("2024-01-01T08:00:00Z")
            .unwrap()
            .to_utc();
        let next = next_run_utc(&cron, tz, after).expect("next run exists");
        assert_eq!(
            next,
            DateTime::parse_from_rfc3339("2024-01-01T09:00:00Z")
                .unwrap()
                .to_utc()
        );
    }

    #[test]
    fn next_run_with_timezone_offset() {
        let cron = parse_cron("0 9 * * *").expect("valid cron");
        let tz = parse_timezone("Europe/Moscow").expect("valid timezone");
        // 2024-01-01 08:00 UTC -> 2024-01-01 11:00 MSK, so next MSK 09:00 is 2024-01-01 06:00 UTC
        let after = DateTime::parse_from_rfc3339("2024-01-01T05:00:00Z")
            .unwrap()
            .to_utc();
        let next = next_run_utc(&cron, tz, after).expect("next run exists");
        assert_eq!(
            next,
            DateTime::parse_from_rfc3339("2024-01-01T06:00:00Z")
                .unwrap()
                .to_utc()
        );
    }

    #[test]
    fn describe_daily_schedule() {
        let cron = parse_cron("30 14 * * *").expect("valid cron");
        let desc = describe_schedule(&cron);
        assert!(desc.contains("14:30"), "desc should contain time: {}", desc);
        assert!(
            desc.contains("daily") || desc.contains("At 14:30"),
            "desc should indicate daily: {}",
            desc
        );
    }

    #[test]
    fn format_remaining_overdue() {
        let now = DateTime::parse_from_rfc3339("2026-01-01T12:00:00Z")
            .unwrap()
            .to_utc();
        let past = now - chrono::Duration::seconds(1);
        assert_eq!(format_remaining_with_now(past, now), "overdue");
    }

    #[test]
    fn format_remaining_minutes() {
        let now = DateTime::parse_from_rfc3339("2026-01-01T12:00:00Z")
            .unwrap()
            .to_utc();
        let future = now + chrono::Duration::minutes(5);
        assert_eq!(format_remaining_with_now(future, now), "in 5 minutes");
    }

    #[test]
    fn format_remaining_composite() {
        let now = DateTime::parse_from_rfc3339("2026-01-01T12:00:00Z")
            .unwrap()
            .to_utc();
        let future = now
            + chrono::Duration::days(2)
            + chrono::Duration::hours(3)
            + chrono::Duration::minutes(4);
        assert_eq!(
            format_remaining_with_now(future, now),
            "in 2 days, 3 hours, and 4 minutes"
        );
    }
}
