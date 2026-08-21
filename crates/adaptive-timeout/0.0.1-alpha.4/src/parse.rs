//! Parser for duration-range strings into [`BackoffInterval`].
//!
//! Format: `<min>..<max>`
//!
//! Each duration is a number (integer or fractional) followed by a unit suffix.
//! Spaces between the number and suffix are allowed.
//!
//! Unit designators are compatible with [jiff's friendly duration format][jiff],
//! which is what restate uses for configuration parsing.
//!
//! [jiff]: https://docs.rs/jiff/latest/jiff/fmt/friendly/index.html
//!
//! # Examples
//!
//! ```
//! use adaptive_timeout::BackoffInterval;
//!
//! let b: BackoffInterval = "10ms..1s".parse().unwrap();
//! assert_eq!(b.min_ms.get(), 10);
//! assert_eq!(b.max_ms.get(), 1_000);
//!
//! let b: BackoffInterval = "0.5s..1min".parse().unwrap();
//! assert_eq!(b.min_ms.get(), 500);
//! assert_eq!(b.max_ms.get(), 60_000);
//! ```

use std::fmt;
use std::str::FromStr;

use crate::config::{MillisNonZero, TimeoutConfig};

/// Timeout floor and ceiling parsed from a duration-range string.
///
/// Format: `<min>..<max>` (e.g., `10ms..60s`).
///
/// Both bounds use [`MillisNonZero`] — they are always positive.
///
/// Unit designators are compatible with [jiff's friendly duration format][jiff],
/// which is what restate uses for configuration parsing.
///
/// [jiff]: https://docs.rs/jiff/latest/jiff/fmt/friendly/index.html
///
/// # Parsing
///
/// ```
/// use adaptive_timeout::BackoffInterval;
///
/// let b: BackoffInterval = "10ms..60s".parse().unwrap();
/// assert_eq!(b.min_ms.get(), 10);
/// assert_eq!(b.max_ms.get(), 60_000);
///
/// // Jiff-style compact designators work:
/// let b: BackoffInterval = "250ms..1m".parse().unwrap();
/// assert_eq!(b.min_ms.get(), 250);
/// assert_eq!(b.max_ms.get(), 60_000);
///
/// // Verbose designators too:
/// let b: BackoffInterval = "500 milliseconds..2 minutes".parse().unwrap();
/// assert_eq!(b.min_ms.get(), 500);
/// assert_eq!(b.max_ms.get(), 120_000);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackoffInterval {
    /// Floor for computed timeout in milliseconds. Default: 250ms.
    pub min_ms: MillisNonZero,
    /// Ceiling for computed timeout in milliseconds. Default: 60,000ms (1min).
    pub max_ms: MillisNonZero,
}

impl Default for BackoffInterval {
    fn default() -> Self {
        Self {
            min_ms: MillisNonZero::new(250).unwrap(),
            max_ms: MillisNonZero::new(60_000).unwrap(),
        }
    }
}

impl fmt::Display for BackoffInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let min = format_ms(self.min_ms.get() as u64);
        let max = format_ms(self.max_ms.get() as u64);
        write!(f, "{min}..{max}")
    }
}

/// Format milliseconds into the most natural unit.
///
/// Uses jiff's compact designators (`d`, `h`, `m`, `s`, `ms`) so that
/// output is directly parseable by both this crate and jiff/restate.
fn format_ms(ms: u64) -> String {
    if ms == 0 {
        return "0ms".to_string();
    }
    if ms.is_multiple_of(86_400_000) {
        format!("{}d", ms / 86_400_000)
    } else if ms.is_multiple_of(3_600_000) {
        format!("{}h", ms / 3_600_000)
    } else if ms.is_multiple_of(60_000) {
        format!("{}m", ms / 60_000)
    } else if ms.is_multiple_of(1_000) {
        format!("{}s", ms / 1_000)
    } else {
        format!("{ms}ms")
    }
}

/// Convert a [`BackoffInterval`] into a [`TimeoutConfig`].
///
/// Quantile and safety factor remain at defaults.
impl From<BackoffInterval> for TimeoutConfig {
    fn from(backoff: BackoffInterval) -> Self {
        TimeoutConfig {
            backoff,
            ..TimeoutConfig::default()
        }
    }
}

/// Parse a [`TimeoutConfig`] from a duration-range string.
///
/// The string is parsed as a [`BackoffInterval`]; `quantile` and
/// `safety_factor` are left at their defaults.
///
/// # Example
///
/// ```
/// use adaptive_timeout::TimeoutConfig;
///
/// let cfg: TimeoutConfig = "10ms..60s".parse().unwrap();
/// assert_eq!(cfg.backoff.min_ms.get(), 10);
/// assert_eq!(cfg.backoff.max_ms.get(), 60_000);
/// assert_eq!(cfg.quantile, 0.9999);
/// assert_eq!(cfg.safety_factor, 2.0);
/// ```
impl std::str::FromStr for TimeoutConfig {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<BackoffInterval>()?.into())
    }
}

// ---------------------------------------------------------------------------
// Serde support for TimeoutConfig (feature = "serde")
// ---------------------------------------------------------------------------

/// Serializes as the backoff interval string, e.g. `"250ms..1m"`.
///
/// `quantile` and `safety_factor` are not included; they are restored from
/// defaults on deserialization.
#[cfg(feature = "serde")]
impl serde::Serialize for TimeoutConfig {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.backoff.serialize(serializer)
    }
}

/// Deserializes from a `"<min>..<max>"` string, setting `quantile` and
/// `safety_factor` to their defaults.
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TimeoutConfig {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(BackoffInterval::deserialize(deserializer)?.into())
    }
}

// ---------------------------------------------------------------------------
// Schemars support (feature = "schemars")
// ---------------------------------------------------------------------------

/// The JSON Schema pattern that matches the `<min>..<max>` duration-range
/// format accepted by [`BackoffInterval`]'s parser.
///
/// Matches strings like `"250ms..1m"`, `"10ms..60s"`, `"0.5s..2 minutes"`.
#[cfg(feature = "schemars")]
const BACKOFF_INTERVAL_PATTERN: &str = r"^\d+(\.\d+)?\s*(ns|us|ms|s|m|h|d|nanoseconds?|microseconds?|milliseconds?|seconds?|minutes?|hours?|days?)\s*\.\.\s*\d+(\.\d+)?\s*(ns|us|ms|s|m|h|d|nanoseconds?|microseconds?|milliseconds?|seconds?|minutes?|hours?|days?)$";

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for BackoffInterval {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        "BackoffInterval".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        concat!(module_path!(), "::BackoffInterval").into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "pattern": BACKOFF_INTERVAL_PATTERN,
            "examples": ["250ms..1m", "10ms..60s", "0.5s..5m", "1s..1h"],
            "description": concat!(
                "Timeout floor and ceiling expressed as a duration range: \"<min>..<max>\".\n",
                "\n",
                "Each bound is a non-negative number followed by a unit. ",
                "Fractional values are allowed (e.g. \"0.5s\"). ",
                "Spaces between the number and the unit are permitted (e.g. \"10 ms\").\n",
                "\n",
                "Supported units:\n",
                "  ns / nanoseconds\n",
                "  us / microseconds\n",
                "  ms / milliseconds\n",
                "   s / seconds\n",
                "   m / minutes\n",
                "   h / hours\n",
                "   d / days\n",
                "\n",
                "Both compact (\"ms\", \"s\", \"m\") and verbose (\"milliseconds\", \"seconds\", \"minutes\") ",
                "forms are accepted.\n",
                "\n",
                "The minimum must be greater than zero and must not exceed the maximum.\n",
                "\n",
                "Examples: \"250ms..1m\", \"10ms..60s\", \"0.5s..5m\", \"1s..1h\"."
            )
        })
    }
}

/// [`TimeoutConfig`] serializes and deserializes as the same `"<min>..<max>"`
/// string as [`BackoffInterval`], so its JSON Schema is identical.
#[cfg(feature = "schemars")]
impl schemars::JsonSchema for TimeoutConfig {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> std::borrow::Cow<'static, str> {
        "TimeoutConfig".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        concat!(module_path!(), "::TimeoutConfig").into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // Same wire format as BackoffInterval — delegate to keep them in sync.
        BackoffInterval::json_schema(generator)
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Errors that can occur when parsing a [`BackoffInterval`] string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input string is empty.
    Empty,
    /// Missing the `..` range separator.
    MissingRangeSeparator,
    /// Failed to parse the minimum timeout duration.
    InvalidMin(String),
    /// Failed to parse the maximum timeout duration.
    InvalidMax(String),
    /// The minimum timeout is zero.
    ZeroMin,
    /// The max timeout is zero.
    ZeroMax,
    /// The minimum timeout exceeds `u32::MAX`.
    MinOverflow(u64),
    /// The maximum timeout exceeds `u32::MAX`.
    MaxOverflow(u64),
    /// `min > max`.
    MinExceedsMax { min_ms: u64, max_ms: u64 },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty input"),
            Self::MissingRangeSeparator => write!(f, "missing '..' range separator"),
            Self::InvalidMin(s) => write!(f, "invalid min timeout: {s}"),
            Self::InvalidMax(s) => write!(f, "invalid max timeout: {s}"),
            Self::ZeroMin => write!(f, "min timeout must be > 0"),
            Self::ZeroMax => write!(f, "max timeout must be > 0"),
            Self::MinOverflow(v) => write!(f, "min timeout ({v}ms) exceeds u32::MAX"),
            Self::MaxOverflow(v) => write!(f, "max timeout ({v}ms) exceeds u32::MAX"),
            Self::MinExceedsMax { min_ms, max_ms } => {
                write!(
                    f,
                    "min timeout ({min_ms}ms) exceeds max timeout ({max_ms}ms)"
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

// ---------------------------------------------------------------------------
// Serde support (feature = "serde")
// ---------------------------------------------------------------------------

/// Serializes as the human-readable display string, e.g. `"250ms..1m"`.
#[cfg(feature = "serde")]
impl serde::Serialize for BackoffInterval {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// Deserializes from the same `"<min>..<max>"` string format accepted by
/// [`FromStr`].
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for BackoffInterval {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl FromStr for BackoffInterval {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseError::Empty);
        }

        let (min_str, max_str) = s
            .split_once("..")
            .ok_or(ParseError::MissingRangeSeparator)?;

        let min_raw =
            parse_duration_ms(min_str.trim()).map_err(|e| ParseError::InvalidMin(e.to_string()))?;
        let max_raw =
            parse_duration_ms(max_str.trim()).map_err(|e| ParseError::InvalidMax(e.to_string()))?;

        if min_raw == 0 {
            return Err(ParseError::ZeroMin);
        }
        if max_raw == 0 {
            return Err(ParseError::ZeroMax);
        }
        if min_raw > max_raw {
            return Err(ParseError::MinExceedsMax {
                min_ms: min_raw,
                max_ms: max_raw,
            });
        }

        let min_u32 = u32::try_from(min_raw).map_err(|_| ParseError::MinOverflow(min_raw))?;
        let max_u32 = u32::try_from(max_raw).map_err(|_| ParseError::MaxOverflow(max_raw))?;

        // Safety: we already checked > 0 above.
        let min_ms = MillisNonZero::new(min_u32).unwrap();
        let max_ms = MillisNonZero::new(max_u32).unwrap();

        Ok(BackoffInterval { min_ms, max_ms })
    }
}

// ---------------------------------------------------------------------------
// Duration string parser
// ---------------------------------------------------------------------------

/// Errors from parsing a single duration string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DurationParseError {
    Empty,
    InvalidNumber(String),
    UnknownUnit(String),
    /// The parsed value truncates to zero milliseconds (e.g., `1ns`).
    TruncatesToZero(String),
}

impl fmt::Display for DurationParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty duration string"),
            Self::InvalidNumber(s) => write!(f, "invalid number: {s}"),
            Self::UnknownUnit(s) => write!(f, "unknown unit: {s}"),
            Self::TruncatesToZero(s) => {
                write!(f, "duration '{s}' truncates to 0ms")
            }
        }
    }
}

/// Parse a duration string like `10ms`, `1.5s`, `2min` into milliseconds.
///
/// Unit designators are compatible with [jiff's friendly duration format][jiff].
/// All jiff unit labels (compact, short, and verbose) are accepted:
///
/// | Unit         | Accepted designators                                                           |
/// |--------------|-------------------------------------------------------------------------------|
/// | nanoseconds  | `ns`, `nsec`, `nsecs`, `nano`, `nanos`, `nanosecond`, `nanoseconds`            |
/// | microseconds | `us`, `µs`/`μs`, `usec`, `usecs`, `micro`, `micros`, `microsecond`, `microseconds` |
/// | milliseconds | `ms`, `msec`, `msecs`, `milli`, `millis`, `millisecond`, `milliseconds`        |
/// | seconds      | `s`, `sec`, `secs`, `second`, `seconds`                                       |
/// | minutes      | `m`, `min`, `mins`, `minute`, `minutes`                                       |
/// | hours        | `h`, `hr`, `hrs`, `hour`, `hours`                                             |
/// | days         | `d`, `day`, `days`                                                            |
///
/// [jiff]: https://docs.rs/jiff/latest/jiff/fmt/friendly/index.html
///
/// Spaces between the number and unit are allowed: `10 ms` is valid.
/// Fractional values are supported: `0.5s` = 500ms.
fn parse_duration_ms(s: &str) -> Result<u64, DurationParseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(DurationParseError::Empty);
    }

    // Special case: bare "0"
    if s == "0" {
        return Ok(0);
    }

    // Find the boundary between number and unit.
    let num_end = s
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
        .unwrap_or(s.len());

    let num_str = s[..num_end].trim();
    let unit_str = s[num_end..].trim();

    if num_str.is_empty() {
        return Err(DurationParseError::InvalidNumber(s.to_string()));
    }
    if unit_str.is_empty() {
        return Err(DurationParseError::UnknownUnit(
            "missing unit suffix".to_string(),
        ));
    }

    let value: f64 = num_str
        .parse()
        .map_err(|_| DurationParseError::InvalidNumber(num_str.to_string()))?;

    // Unit designators match jiff's friendly format grammar exactly.
    // U+00B5 (µ MICRO SIGN) and U+03BC (μ GREEK SMALL LETTER MU) both accepted.
    let factor_ns: f64 = match unit_str {
        "nanoseconds" | "nanosecond" | "nanos" | "nano" | "nsecs" | "nsec" | "ns" => 1.0,
        "microseconds" | "microsecond" | "micros" | "micro" | "usecs" | "usec" | "us"
        | "\u{00B5}s" | "\u{00B5}secs" | "\u{00B5}sec" | "\u{03BC}s" | "\u{03BC}secs"
        | "\u{03BC}sec" => 1_000.0,
        "milliseconds" | "millisecond" | "millis" | "milli" | "msecs" | "msec" | "ms" => {
            1_000_000.0
        }
        "seconds" | "second" | "secs" | "sec" | "s" => 1_000_000_000.0,
        "minutes" | "minute" | "mins" | "min" | "m" => 60.0 * 1_000_000_000.0,
        "hours" | "hour" | "hrs" | "hr" | "h" => 3_600.0 * 1_000_000_000.0,
        "days" | "day" | "d" => 86_400.0 * 1_000_000_000.0,
        _ => return Err(DurationParseError::UnknownUnit(unit_str.to_string())),
    };

    let total_ns = value * factor_ns;
    let ms = (total_ns / 1_000_000.0).round() as u64;

    // Reject durations that round to 0ms but had a non-zero input.
    if ms == 0 && value != 0.0 {
        return Err(DurationParseError::TruncatesToZero(s.to_string()));
    }

    Ok(ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Duration parsing
    // -----------------------------------------------------------------------

    #[test]
    fn parse_milliseconds() {
        assert_eq!(parse_duration_ms("10ms").unwrap(), 10);
        assert_eq!(parse_duration_ms("100ms").unwrap(), 100);
        assert_eq!(parse_duration_ms("1ms").unwrap(), 1);
        assert_eq!(parse_duration_ms("0ms").unwrap(), 0);
    }

    #[test]
    fn parse_seconds() {
        assert_eq!(parse_duration_ms("1s").unwrap(), 1_000);
        assert_eq!(parse_duration_ms("10s").unwrap(), 10_000);
        assert_eq!(parse_duration_ms("1sec").unwrap(), 1_000);
        assert_eq!(parse_duration_ms("2secs").unwrap(), 2_000);
        assert_eq!(parse_duration_ms("1second").unwrap(), 1_000);
        assert_eq!(parse_duration_ms("3seconds").unwrap(), 3_000);
    }

    #[test]
    fn parse_fractional() {
        assert_eq!(parse_duration_ms("0.5s").unwrap(), 500);
        assert_eq!(parse_duration_ms("1.5s").unwrap(), 1_500);
        assert_eq!(parse_duration_ms("0.1s").unwrap(), 100);
        assert_eq!(parse_duration_ms("2.5ms").unwrap(), 3); // rounds
    }

    #[test]
    fn parse_minutes() {
        assert_eq!(parse_duration_ms("1min").unwrap(), 60_000);
        assert_eq!(parse_duration_ms("2mins").unwrap(), 120_000);
        assert_eq!(parse_duration_ms("1m").unwrap(), 60_000);
        assert_eq!(parse_duration_ms("1minute").unwrap(), 60_000);
        assert_eq!(parse_duration_ms("3minutes").unwrap(), 180_000);
    }

    #[test]
    fn parse_hours() {
        assert_eq!(parse_duration_ms("1h").unwrap(), 3_600_000);
        assert_eq!(parse_duration_ms("1hr").unwrap(), 3_600_000);
        assert_eq!(parse_duration_ms("2hrs").unwrap(), 7_200_000);
        assert_eq!(parse_duration_ms("1hour").unwrap(), 3_600_000);
        assert_eq!(parse_duration_ms("3hours").unwrap(), 10_800_000);
    }

    #[test]
    fn parse_days() {
        assert_eq!(parse_duration_ms("1d").unwrap(), 86_400_000);
        assert_eq!(parse_duration_ms("1day").unwrap(), 86_400_000);
        assert_eq!(parse_duration_ms("2days").unwrap(), 172_800_000);
    }

    #[test]
    fn parse_microseconds() {
        assert_eq!(parse_duration_ms("1000us").unwrap(), 1);
        assert_eq!(parse_duration_ms("500us").unwrap(), 1); // rounds to 1ms
        assert_eq!(parse_duration_ms("1000\u{03BC}s").unwrap(), 1); // Greek mu
        assert_eq!(parse_duration_ms("1000\u{00B5}s").unwrap(), 1); // Micro sign (jiff)
        assert_eq!(parse_duration_ms("1000usec").unwrap(), 1);
        assert_eq!(parse_duration_ms("1000usecs").unwrap(), 1);
        assert_eq!(parse_duration_ms("1000micro").unwrap(), 1);
        assert_eq!(parse_duration_ms("1000micros").unwrap(), 1);
        assert_eq!(parse_duration_ms("1000microsecond").unwrap(), 1);
        assert_eq!(parse_duration_ms("1000microseconds").unwrap(), 1);
    }

    #[test]
    fn parse_nanoseconds() {
        assert_eq!(parse_duration_ms("1000000ns").unwrap(), 1);
        assert_eq!(parse_duration_ms("1000000nsec").unwrap(), 1);
        assert_eq!(parse_duration_ms("1000000nsecs").unwrap(), 1);
        assert_eq!(parse_duration_ms("1000000nano").unwrap(), 1);
        assert_eq!(parse_duration_ms("1000000nanos").unwrap(), 1);
        assert_eq!(parse_duration_ms("1000000nanosecond").unwrap(), 1);
        assert_eq!(parse_duration_ms("1000000nanoseconds").unwrap(), 1);
    }

    #[test]
    fn parse_millisecond_aliases() {
        assert_eq!(parse_duration_ms("10msec").unwrap(), 10);
        assert_eq!(parse_duration_ms("10msecs").unwrap(), 10);
        assert_eq!(parse_duration_ms("10milli").unwrap(), 10);
        assert_eq!(parse_duration_ms("10millis").unwrap(), 10);
        assert_eq!(parse_duration_ms("10millisecond").unwrap(), 10);
        assert_eq!(parse_duration_ms("10milliseconds").unwrap(), 10);
    }

    #[test]
    fn parse_with_spaces() {
        assert_eq!(parse_duration_ms("10 ms").unwrap(), 10);
        assert_eq!(parse_duration_ms(" 1 s ").unwrap(), 1_000);
    }

    #[test]
    fn parse_bare_zero() {
        assert_eq!(parse_duration_ms("0").unwrap(), 0);
    }

    #[test]
    fn parse_truncates_to_zero() {
        assert!(matches!(
            parse_duration_ms("1ns"),
            Err(DurationParseError::TruncatesToZero(_))
        ));
    }

    #[test]
    fn parse_unknown_unit() {
        assert!(matches!(
            parse_duration_ms("10xyz"),
            Err(DurationParseError::UnknownUnit(_))
        ));
    }

    #[test]
    fn parse_missing_unit() {
        assert!(matches!(
            parse_duration_ms("42"),
            Err(DurationParseError::UnknownUnit(_))
        ));
    }

    #[test]
    fn parse_empty() {
        assert!(matches!(
            parse_duration_ms(""),
            Err(DurationParseError::Empty)
        ));
    }

    // -----------------------------------------------------------------------
    // BackoffInterval parsing
    // -----------------------------------------------------------------------

    #[test]
    fn parse_basic_range() {
        let b: BackoffInterval = "10ms..1s".parse().unwrap();
        assert_eq!(b.min_ms.get(), 10);
        assert_eq!(b.max_ms.get(), 1_000);
    }

    #[test]
    fn parse_fractional_range() {
        let b: BackoffInterval = "0.5s..1.5s".parse().unwrap();
        assert_eq!(b.min_ms.get(), 500);
        assert_eq!(b.max_ms.get(), 1_500);
    }

    #[test]
    fn parse_with_spaces_around() {
        let b: BackoffInterval = "  10ms .. 1s  ".parse().unwrap();
        assert_eq!(b.min_ms.get(), 10);
        assert_eq!(b.max_ms.get(), 1_000);
    }

    #[test]
    fn parse_same_min_max() {
        let b: BackoffInterval = "100ms..100ms".parse().unwrap();
        assert_eq!(b.min_ms.get(), 100);
        assert_eq!(b.max_ms.get(), 100);
    }

    #[test]
    fn parse_large_values() {
        let b: BackoffInterval = "1h..3d".parse().unwrap();
        assert_eq!(b.min_ms.get(), 3_600_000);
        assert_eq!(b.max_ms.get(), 259_200_000);
    }

    #[test]
    fn parse_mixed_units() {
        let b: BackoffInterval = "100ms..1min".parse().unwrap();
        assert_eq!(b.min_ms.get(), 100);
        assert_eq!(b.max_ms.get(), 60_000);
    }

    #[test]
    fn err_empty() {
        assert_eq!(
            "".parse::<BackoffInterval>().unwrap_err(),
            ParseError::Empty
        );
    }

    #[test]
    fn err_missing_separator() {
        assert_eq!(
            "10ms".parse::<BackoffInterval>().unwrap_err(),
            ParseError::MissingRangeSeparator
        );
    }

    #[test]
    fn err_zero_min() {
        assert_eq!(
            "0ms..1s".parse::<BackoffInterval>().unwrap_err(),
            ParseError::ZeroMin
        );
    }

    #[test]
    fn err_min_exceeds_max() {
        assert!(matches!(
            "10s..1s".parse::<BackoffInterval>(),
            Err(ParseError::MinExceedsMax { .. })
        ));
    }

    #[test]
    fn err_invalid_min() {
        assert!(matches!(
            "abc..1s".parse::<BackoffInterval>(),
            Err(ParseError::InvalidMin(_))
        ));
    }

    #[test]
    fn err_invalid_max() {
        assert!(matches!(
            "10ms..abc".parse::<BackoffInterval>(),
            Err(ParseError::InvalidMax(_))
        ));
    }

    // -----------------------------------------------------------------------
    // Display round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn display_basic() {
        let b: BackoffInterval = "10ms..1min".parse().unwrap();
        assert_eq!(b.to_string(), "10ms..1m");
    }

    #[test]
    fn display_sub_second() {
        let b: BackoffInterval = "500ms..1500ms".parse().unwrap();
        assert_eq!(b.to_string(), "500ms..1500ms");
    }

    #[test]
    fn display_round_trip() {
        for original in &["10ms..60s", "250ms..1m", "1s..1h", "100ms..1d"] {
            let b: BackoffInterval = original.parse().unwrap();
            let displayed = b.to_string();
            let reparsed: BackoffInterval = displayed.parse().unwrap();
            assert_eq!(b, reparsed, "round-trip failed for {original}");
        }
    }

    // -----------------------------------------------------------------------
    // From<BackoffInterval> for TimeoutConfig
    // -----------------------------------------------------------------------

    #[test]
    fn into_timeout_config() {
        let b: BackoffInterval = "50ms..30s".parse().unwrap();
        let cfg: TimeoutConfig = b.into();
        assert_eq!(cfg.backoff.min_ms.get(), 50);
        assert_eq!(cfg.backoff.max_ms.get(), 30_000);
        // Defaults preserved:
        assert_eq!(cfg.quantile, 0.9999);
        assert_eq!(cfg.safety_factor, 2.0);
    }

    // -----------------------------------------------------------------------
    // Serde round-trips (feature = "serde")
    // -----------------------------------------------------------------------

    #[cfg(feature = "serde")]
    mod serde_tests {
        use super::*;

        #[test]
        fn serialize_to_string() {
            let b: BackoffInterval = "250ms..1m".parse().unwrap();
            let json = serde_json::to_string(&b).unwrap();
            assert_eq!(json, r#""250ms..1m""#);
        }

        #[test]
        fn deserialize_from_string() {
            let b: BackoffInterval = serde_json::from_str(r#""10ms..60s""#).unwrap();
            assert_eq!(b.min_ms.get(), 10);
            assert_eq!(b.max_ms.get(), 60_000);
        }

        #[test]
        fn serde_round_trip() {
            for s in &["10ms..60s", "250ms..1m", "1s..1h", "100ms..1d"] {
                let original: BackoffInterval = s.parse().unwrap();
                let json = serde_json::to_string(&original).unwrap();
                let restored: BackoffInterval = serde_json::from_str(&json).unwrap();
                assert_eq!(original, restored, "round-trip failed for {s}");
            }
        }

        #[test]
        fn deserialize_error_propagated() {
            let err = serde_json::from_str::<BackoffInterval>(r#""not-a-range""#);
            assert!(err.is_err());
        }
    }

    // -----------------------------------------------------------------------
    // Schemars (feature = "schemars")
    // -----------------------------------------------------------------------

    #[cfg(feature = "schemars")]
    mod schemars_tests {
        use schemars::JsonSchema;

        use super::*;

        fn backoff_schema() -> schemars::Schema {
            let mut generator = schemars::SchemaGenerator::default();
            BackoffInterval::json_schema(&mut generator)
        }

        fn timeout_schema() -> schemars::Schema {
            let mut generator = schemars::SchemaGenerator::default();
            TimeoutConfig::json_schema(&mut generator)
        }

        #[test]
        fn backoff_interval_is_string_type() {
            let schema = backoff_schema();
            let obj = schema.as_object().unwrap();
            assert_eq!(obj["type"], "string");
        }

        #[test]
        fn backoff_interval_has_pattern() {
            let schema = backoff_schema();
            let obj = schema.as_object().unwrap();
            assert!(obj.contains_key("pattern"), "schema should have a pattern");
            // Spot-check: the pattern must mention ".." as a separator.
            assert!(
                obj["pattern"].as_str().unwrap().contains(r"\.\."),
                "pattern should include the '..' separator"
            );
        }

        #[test]
        fn backoff_interval_has_examples() {
            let schema = backoff_schema();
            let obj = schema.as_object().unwrap();
            assert!(obj.contains_key("examples"), "schema should have examples");
        }

        #[test]
        fn timeout_config_schema_matches_backoff_interval() {
            // TimeoutConfig delegates to BackoffInterval — schemas must be equal.
            assert_eq!(
                backoff_schema().as_object().unwrap()["type"],
                timeout_schema().as_object().unwrap()["type"],
            );
            assert_eq!(
                backoff_schema().as_object().unwrap()["pattern"],
                timeout_schema().as_object().unwrap()["pattern"],
            );
        }

        #[test]
        fn backoff_interval_schema_name() {
            assert_eq!(BackoffInterval::schema_name(), "BackoffInterval");
        }

        #[test]
        fn timeout_config_schema_name() {
            assert_eq!(TimeoutConfig::schema_name(), "TimeoutConfig");
        }

        #[test]
        fn both_are_inline() {
            assert!(BackoffInterval::inline_schema());
            assert!(TimeoutConfig::inline_schema());
        }
    }
}
