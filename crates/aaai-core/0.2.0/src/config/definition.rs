//! Audit definition — the "expected values" YAML document.
//!
//! # File shape (version 1)
//!
//! ```yaml
//! version: "1"
//! meta:
//!   description: "Release v2.3.0 audit"
//! entries:
//!   - path: "config/server.toml"          # exact path OR glob pattern
//!     diff_type: Modified
//!     reason: "ポート番号の仕様変更 — INF-42"
//!     strategy:
//!       type: LineMatch
//!       rules:
//!         - action: Removed
//!           line: "port = 80"
//!         - action: Added
//!           line: "port = 8080"
//!     enabled: true
//!     note: "本番環境への適用に伴う変更"
//! ```

use serde::{Deserialize, Serialize};

use crate::diff::entry::DiffType;

// ── Top-level document ────────────────────────────────────────────────────

/// The root of an audit definition file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDefinition {
    /// Schema version. Currently `"1"`.
    pub version: String,

    /// Optional human-readable metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<AuditMeta>,

    /// Ordered list of expected-value entries.
    #[serde(default)]
    pub entries: Vec<AuditEntry>,
}

impl AuditDefinition {
    pub fn new_empty() -> Self {
        Self { version: "1".to_string(), meta: None, entries: Vec::new() }
    }

    /// Find an entry that matches `path` by exact path or glob pattern.
    /// Exact-path entries take priority over glob entries.
    pub fn find_entry(&self, path: &str) -> Option<&AuditEntry> {
        // 1. Exact match first.
        if let Some(e) = self.entries.iter().find(|e| !e.is_glob() && e.path == path) {
            return Some(e);
        }
        // 2. Glob match (first winning entry).
        self.entries.iter().find(|e| e.is_glob() && e.glob_matches(path))
    }

    pub fn find_entry_mut(&mut self, path: &str) -> Option<&mut AuditEntry> {
        // Only exact-path entries are mutable via this API.
        self.entries.iter_mut().find(|e| !e.is_glob() && e.path == path)
    }

    /// Upsert an entry: replace the existing exact-path entry or append.
    pub fn upsert_entry(&mut self, entry: AuditEntry) {
        if let Some(existing) = self.find_entry_mut(&entry.path.clone()) {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
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
    /// Root-relative path with forward slashes, OR a glob pattern.
    /// Glob patterns may contain `*`, `**`, and `?`.
    pub path: String,

    pub diff_type: DiffType,

    /// Mandatory human-readable justification. Must not be empty for approval.
    pub reason: String,

    #[serde(default)]
    pub strategy: AuditStrategy,

    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn default_true() -> bool { true }

impl AuditEntry {
    /// Returns `true` if `path` contains a glob metacharacter.
    pub fn is_glob(&self) -> bool {
        self.path.contains('*') || self.path.contains('?') || self.path.contains('[')
    }

    /// Whether this glob pattern matches `candidate`.
    pub fn glob_matches(&self, candidate: &str) -> bool {
        if !self.is_glob() {
            return self.path == candidate;
        }
        match glob::Pattern::new(&self.path) {
            Ok(pat) => pat.matches(candidate),
            Err(_) => false,
        }
    }

    /// Validate that the entry is complete enough for approval.
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
            AuditStrategy::None       => "None",
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
pub enum LineAction {
    Added,
    Removed,
}

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
    #[default]
    AddedLines,
    RemovedLines,
    AllChangedLines,
}

impl std::fmt::Display for RegexTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexTarget::AddedLines     => write!(f, "Added lines"),
            RegexTarget::RemovedLines   => write!(f, "Removed lines"),
            RegexTarget::AllChangedLines => write!(f, "All changed lines"),
        }
    }
}
