//! Usage history types for the ACE Usage API.
//!
//! These types correspond to the `GET /api/v1/usage/history` endpoint,
//! which returns time-bucketed usage metrics for an organization and
//! (optionally) a specific project.

use crate::errors::AceError;
use serde::{Deserialize, Serialize};

/// Window parameter for the usage history endpoint.
///
/// Selects the time range covered by the returned buckets. Windows of
/// 12 hours and below are reported at hourly granularity; longer windows
/// are reported daily.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsageWindow {
    /// 1 hour window (hourly granularity).
    #[serde(rename = "1h")]
    H1,
    /// 6 hour window (hourly granularity).
    #[serde(rename = "6h")]
    H6,
    /// 12 hour window (hourly granularity).
    #[serde(rename = "12h")]
    H12,
    /// 1 day window (hourly granularity).
    #[serde(rename = "1d")]
    D1,
    /// 7 day window (daily granularity).
    #[serde(rename = "7d")]
    D7,
    /// 14 day window (daily granularity).
    #[serde(rename = "14d")]
    D14,
    /// 30 day window (daily granularity).
    #[serde(rename = "30d")]
    D30,
}

impl UsageWindow {
    /// Return the wire-format string for this window (e.g. "1h", "7d").
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::H1 => "1h",
            Self::H6 => "6h",
            Self::H12 => "12h",
            Self::D1 => "1d",
            Self::D7 => "7d",
            Self::D14 => "14d",
            Self::D30 => "30d",
        }
    }
}

impl std::fmt::Display for UsageWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for UsageWindow {
    type Err = AceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1h" => Ok(Self::H1),
            "6h" => Ok(Self::H6),
            "12h" => Ok(Self::H12),
            "1d" => Ok(Self::D1),
            "7d" => Ok(Self::D7),
            "14d" => Ok(Self::D14),
            "30d" => Ok(Self::D30),
            other => Err(AceError::Config(format!(
                "Invalid UsageWindow value '{}': expected one of 1h|6h|12h|1d|7d|14d|30d",
                other
            ))),
        }
    }
}

/// Granularity of the buckets returned by the usage history endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UsageGranularity {
    /// One bucket per hour.
    Hourly,
    /// One bucket per day.
    Daily,
}

/// A single time-bucketed usage record returned by
/// `GET /api/v1/usage/history`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageBucket {
    /// ISO timestamp identifying the bucket. For hourly granularity this
    /// is an RFC-3339 timestamp (e.g. `2026-02-17T14:00:00Z`); for daily
    /// granularity it is a date string (e.g. `2026-02-17`).
    pub period: String,
    #[serde(default)]
    pub api_calls_total: u64,
    #[serde(default)]
    pub api_calls_patterns: u64,
    #[serde(default)]
    pub api_calls_traces: u64,
    #[serde(default)]
    pub api_calls_playbook: u64,
    #[serde(default)]
    pub patterns_created: u64,
    #[serde(default)]
    pub patterns_updated: u64,
    #[serde(default)]
    pub patterns_deleted: u64,
    #[serde(default)]
    pub patterns_searched: u64,
    #[serde(default)]
    pub traces_submitted: u64,
    #[serde(default)]
    pub bootstrap_runs: u64,
}

/// Aggregate totals across all buckets in a usage history response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageHistoryTotals {
    #[serde(default)]
    pub api_calls_total: u64,
    #[serde(default)]
    pub patterns_created: u64,
    #[serde(default)]
    pub traces_submitted: u64,
}

/// Full response body of `GET /api/v1/usage/history`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageHistoryResponse {
    pub org_id: String,
    pub project_id: Option<String>,
    pub window: UsageWindow,
    pub granularity: UsageGranularity,
    #[serde(default)]
    pub buckets: Vec<UsageBucket>,
    pub totals: UsageHistoryTotals,
}

// Backward-compat aliases for pre-0.3.0 type names.
// Other SDKs (TS/Go/Python/Kotlin) kept the old names alongside the new
// `UsageWindow`/`UsageBucket`/`UsageGranularity`; Rust matches that surface.

/// Alias for [`UsageWindow`] (kept for backward compatibility with 0.2.x).
pub type UsageHistoryWindow = UsageWindow;

/// Alias for [`UsageBucket`] (kept for backward compatibility with 0.2.x).
pub type UsageHistoryBucket = UsageBucket;

/// Alias for [`UsageGranularity`] (kept for backward compatibility with 0.2.x).
pub type UsageHistoryGranularity = UsageGranularity;

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_display_matches_wire_format() {
        assert_eq!(UsageWindow::H1.to_string(), "1h");
        assert_eq!(UsageWindow::H6.to_string(), "6h");
        assert_eq!(UsageWindow::H12.to_string(), "12h");
        assert_eq!(UsageWindow::D1.to_string(), "1d");
        assert_eq!(UsageWindow::D7.to_string(), "7d");
        assert_eq!(UsageWindow::D14.to_string(), "14d");
        assert_eq!(UsageWindow::D30.to_string(), "30d");
    }

    #[test]
    fn test_from_str_roundtrip() {
        for w in [
            UsageWindow::H1,
            UsageWindow::H6,
            UsageWindow::H12,
            UsageWindow::D1,
            UsageWindow::D7,
            UsageWindow::D14,
            UsageWindow::D30,
        ] {
            let parsed = UsageWindow::from_str(&w.to_string()).unwrap();
            assert_eq!(parsed, w);
        }
    }

    #[test]
    fn test_from_str_rejects_unknown() {
        let err = UsageWindow::from_str("2h").unwrap_err();
        matches!(err, AceError::Config(_));
    }

    #[test]
    fn test_serde_rename() {
        let json = serde_json::to_string(&UsageWindow::H12).unwrap();
        assert_eq!(json, "\"12h\"");
        let back: UsageWindow = serde_json::from_str("\"7d\"").unwrap();
        assert_eq!(back, UsageWindow::D7);
    }
}
