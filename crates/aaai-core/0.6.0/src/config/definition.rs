//! Audit definition — the "expected values" YAML document (version 1).
//!
//! # Phase 3 additions
//!
//! Each [`AuditEntry`] now carries optional metadata for traceability:
//!
//! ```yaml
//! - path: "config/server.toml"
//!   diff_type: Modified
//!   reason: "Port change — INF-42"
//!   ticket: "INF-42"
//!   approved_by: "alice"
//!   approved_at: "2025-01-15T09:23:00Z"
//!   expires_at: "2025-07-01"
//!   strategy:
//!     type: LineMatch
//!     rules:
//!       - action: Removed
//!         line: "port = 80"
//!       - action: Added
//!         line: "port = 8080"
//!   enabled: true
//! ```

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::diff::entry::DiffType;

// ── Top-level document ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDefinition {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<AuditMeta>,
    #[serde(default)]
    pub entries: Vec<AuditEntry>,
}

impl AuditDefinition {
    pub fn new_empty() -> Self {
        Self { version: "1".into(), meta: None, entries: Vec::new() }
    }

    /// Find by exact path first, then by first matching glob.
    pub fn find_entry(&self, path: &str) -> Option<&AuditEntry> {
        self.entries.iter().find(|e| !e.is_glob() && e.path == path)
            .or_else(|| self.entries.iter().find(|e| e.is_glob() && e.glob_matches(path)))
    }

    pub fn find_entry_mut(&mut self, path: &str) -> Option<&mut AuditEntry> {
        self.entries.iter_mut().find(|e| !e.is_glob() && e.path == path)
    }

    pub fn upsert_entry(&mut self, entry: AuditEntry) {
        if let Some(existing) = self.find_entry_mut(&entry.path.clone()) {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
    }

    /// Return all entries whose `expires_at` is today or in the past.
    pub fn expired_entries(&self) -> Vec<&AuditEntry> {
        let today = Utc::now().date_naive();
        self.entries.iter()
            .filter(|e| e.expires_at.map_or(false, |d| d <= today))
            .collect()
    }

    /// Return entries expiring within `days` days from today.
    pub fn expiring_soon(&self, days: i64) -> Vec<&AuditEntry> {
        let today = Utc::now().date_naive();
        let threshold = today + chrono::Duration::days(days);
        self.entries.iter()
            .filter(|e| e.expires_at.map_or(false, |d| d > today && d <= threshold))
            .collect()
    }
}

// ── Metadata ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ── Per-file entry ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Root-relative path or glob pattern.
    pub path: String,
    pub diff_type: DiffType,
    /// Mandatory human-readable justification.
    pub reason: String,
    #[serde(default)]
    pub strategy: AuditStrategy,
    #[serde(default = "default_true")]
    pub enabled: bool,

    // ── Phase 3: traceability fields ────────────────────────────────────
    /// Ticket or issue reference (e.g. "JIRA-123", "INF-42").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket: Option<String>,

    /// Identity of the person who approved this entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,

    /// UTC timestamp when approval was recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<DateTime<Utc>>,

    /// Date after which this entry should be re-reviewed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<NaiveDate>,

    /// Free-form note (stored but not used for judgement).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    // ── Phase 6: versioning ──────────────────────────────────────────
    /// UTC timestamp when this entry was first created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// UTC timestamp when this entry was last modified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

fn default_true() -> bool { true }

impl AuditEntry {
    pub fn is_glob(&self) -> bool {
        self.path.contains('*') || self.path.contains('?') || self.path.contains('[')
    }

    pub fn glob_matches(&self, candidate: &str) -> bool {
        if !self.is_glob() { return self.path == candidate; }
        glob::Pattern::new(&self.path)
            .map(|p| p.matches(candidate))
            .unwrap_or(false)
    }

    /// True if `expires_at` is today or in the past.
    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |d| d <= Utc::now().date_naive())
    }

    /// True if expiring within `days` days but not yet expired.
    pub fn expires_soon(&self, days: i64) -> bool {
        let today = Utc::now().date_naive();
        let threshold = today + chrono::Duration::days(days);
        self.expires_at.map_or(false, |d| d > today && d <= threshold)
    }

    /// Stamp created_at (first time) and updated_at (always) with the current UTC time.
    pub fn stamp_now(&mut self) {
        let now = Utc::now();
        if self.created_at.is_none() {
            self.created_at = Some(now);
        }
        self.updated_at = Some(now);
    }

    pub fn is_approvable(&self) -> Result<(), String> {
        if self.path.trim().is_empty() {
            return Err("Path must not be empty.".into());
        }
        if self.reason.trim().is_empty() {
            return Err("Reason must not be empty.".into());
        }
        self.strategy.validate()?;
        Ok(())
    }
}

// ── Content-audit strategy ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuditStrategy {
    None,
    Checksum { expected_sha256: String },
    LineMatch { rules: Vec<LineRule> },
    Regex { pattern: String, #[serde(default)] target: RegexTarget },
    Exact { expected_content: String },
}

impl Default for AuditStrategy {
    fn default() -> Self { AuditStrategy::None }
}

impl AuditStrategy {
    pub fn label(&self) -> &'static str {
        match self {
            AuditStrategy::None           => "None",
            AuditStrategy::Checksum { .. } => "Checksum",
            AuditStrategy::LineMatch { .. } => "LineMatch",
            AuditStrategy::Regex { .. }    => "Regex",
            AuditStrategy::Exact { .. }    => "Exact",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AuditStrategy::None =>
                "Checks only that the expected change type occurred. No content inspection.",
            AuditStrategy::Checksum { .. } =>
                "Verifies the file's SHA-256 digest. Best for binaries, images, archives.",
            AuditStrategy::LineMatch { .. } =>
                "Verifies specific lines were added or removed. Primary strategy for config changes.",
            AuditStrategy::Regex { .. } =>
                "Verifies changed lines match a regular expression. Good for environment-dependent values.",
            AuditStrategy::Exact { .. } =>
                "Verifies the file's full content exactly matches expected text. Avoid for large files.",
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        match self {
            AuditStrategy::None => Ok(()),
            AuditStrategy::Checksum { expected_sha256 } => {
                if expected_sha256.trim().is_empty() {
                    return Err("Checksum: expected_sha256 must not be empty.".into());
                }
                if !expected_sha256.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err("Checksum: expected_sha256 must be a valid hex string.".into());
                }
                if expected_sha256.len() != 64 {
                    return Err("Checksum: must be a 64-character SHA-256 hex digest.".into());
                }
                Ok(())
            }
            AuditStrategy::LineMatch { rules } => {
                if rules.is_empty() {
                    return Err("LineMatch: at least one rule is required.".into());
                }
                for (i, r) in rules.iter().enumerate() {
                    if r.line.trim().is_empty() {
                        return Err(format!("LineMatch rule {}: line must not be empty.", i + 1));
                    }
                }
                Ok(())
            }
            AuditStrategy::Regex { pattern, .. } => {
                if pattern.trim().is_empty() {
                    return Err("Regex: pattern must not be empty.".into());
                }
                regex::Regex::new(pattern)
                    .map(|_| ())
                    .map_err(|e| format!("Regex: invalid pattern — {e}"))
            }
            AuditStrategy::Exact { expected_content } => {
                if expected_content.is_empty() {
                    return Err("Exact: expected_content must not be empty.".into());
                }
                Ok(())
            }
        }
    }
}

// ── LineMatch rule ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineRule {
    pub action: LineAction,
    pub line: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineAction { Added, Removed }

impl std::fmt::Display for LineAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineAction::Added   => write!(f, "Added"),
            LineAction::Removed => write!(f, "Removed"),
        }
    }
}

// ── Regex target ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegexTarget {
    #[default] AddedLines,
    RemovedLines,
    AllChangedLines,
}

impl std::fmt::Display for RegexTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexTarget::AddedLines      => write!(f, "Added lines"),
            RegexTarget::RemovedLines    => write!(f, "Removed lines"),
            RegexTarget::AllChangedLines => write!(f, "All changed lines"),
        }
    }
}
