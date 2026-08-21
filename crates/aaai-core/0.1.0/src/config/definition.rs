//! Audit definition — the "expected values" YAML document.
//!
//! The on-disk format is versioned YAML.  Each [`AuditEntry`] describes one
//! expected file-level difference plus the content-audit [`AuditStrategy`]
//! and the mandatory human-readable `reason`.
//!
//! # File shape
//!
//! ```yaml
//! version: "1"
//! meta:
//!   description: "Release v2.3.0 audit"
//! entries:
//!   - path: "config/server.toml"
//!     diff_type: Modified
//!     reason: "ポート番号の仕様変更"
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
    /// Schema version.  Currently `"1"`.
    pub version: String,

    /// Optional human-readable metadata for the definition file itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<AuditMeta>,

    /// Ordered list of expected-value entries.
    #[serde(default)]
    pub entries: Vec<AuditEntry>,
}

impl AuditDefinition {
    /// Construct a new, empty definition at the current schema version.
    pub fn new_empty() -> Self {
        Self {
            version: "1".to_string(),
            meta: None,
            entries: Vec::new(),
        }
    }

    /// Return the entry for `path`, if any.  Comparison is
    /// case-sensitive and uses Unix-style forward-slash separators.
    pub fn find_entry(&self, path: &str) -> Option<&AuditEntry> {
        self.entries.iter().find(|e| e.path == path)
    }

    /// Mutable variant of [`find_entry`].
    pub fn find_entry_mut(&mut self, path: &str) -> Option<&mut AuditEntry> {
        self.entries.iter_mut().find(|e| e.path == path)
    }

    /// Upsert an entry: replace the existing entry for the same path or
    /// append a new one.  Preserves the sort-stable order of existing
    /// entries so that repeated saves do not produce spurious diffs.
    pub fn upsert_entry(&mut self, entry: AuditEntry) {
        if let Some(existing) = self.find_entry_mut(&entry.path.clone()) {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
    }
}

// ── Metadata ─────────────────────────────────────────────────────────────

/// Free-form metadata attached to the definition file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditMeta {
    /// A short description of what this audit covers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ── Per-file entry ────────────────────────────────────────────────────────

/// One expected-value record for a single path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Root-relative path using forward slashes (`config/server.toml`).
    pub path: String,

    /// The difference type this entry expects to see.
    pub diff_type: DiffType,

    /// Mandatory human-readable justification.  Empty strings are
    /// rejected at validation time.
    pub reason: String,

    /// Content-audit strategy.
    #[serde(default)]
    pub strategy: AuditStrategy,

    /// Whether this entry participates in auditing.
    /// Disabled entries produce [`crate::audit::result::AuditStatus::Ignored`].
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Optional free-form note (stored but not used for judgement).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn default_true() -> bool {
    true
}

impl AuditEntry {
    /// Return `true` when the entry is complete enough for a valid approval:
    /// non-empty path, non-empty reason, and a strategy that passes its own
    /// validation.
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

/// The content-level audit strategy to apply to a matched entry.
///
/// Each variant carries its own parameters.  The engine dispatches on this
/// enum to produce a per-file content verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuditStrategy {
    /// Check only that the expected diff kind occurred.  No content
    /// inspection.
    None,

    /// Verify that the file's SHA-256 digest matches `expected_sha256`.
    Checksum {
        /// Expected lowercase hex SHA-256 digest.
        expected_sha256: String,
    },

    /// Verify that specific lines were added and/or removed.
    LineMatch {
        /// Ordered list of expected line changes.
        rules: Vec<LineRule>,
    },

    /// Verify that added/removed lines match a regular expression.
    Regex {
        /// The regular expression pattern (applied per changed line).
        pattern: String,
        /// Which side of the diff to apply the pattern to.
        #[serde(default)]
        target: RegexTarget,
    },

    /// Verify that the *after* file's full content exactly matches
    /// `expected_content`.
    Exact {
        /// Expected full file content.
        expected_content: String,
    },
}

impl Default for AuditStrategy {
    fn default() -> Self {
        AuditStrategy::None
    }
}

impl AuditStrategy {
    /// Human-readable short label for UI display.
    pub fn label(&self) -> &'static str {
        match self {
            AuditStrategy::None => "None",
            AuditStrategy::Checksum { .. } => "Checksum",
            AuditStrategy::LineMatch { .. } => "LineMatch",
            AuditStrategy::Regex { .. } => "Regex",
            AuditStrategy::Exact { .. } => "Exact",
        }
    }

    /// Brief description shown in the inspector UI.
    pub fn description(&self) -> &'static str {
        match self {
            AuditStrategy::None =>
                "Checks only that the expected change type occurred. \
                 No content inspection is performed.",
            AuditStrategy::Checksum { .. } =>
                "Verifies the file's SHA-256 digest. \
                 Suitable for binaries, images, and archives.",
            AuditStrategy::LineMatch { .. } =>
                "Verifies that specific lines were added or removed. \
                 The primary strategy for config-value changes.",
            AuditStrategy::Regex { .. } =>
                "Verifies that changed lines match a regular expression. \
                 Useful when the exact value is environment-dependent.",
            AuditStrategy::Exact { .. } =>
                "Verifies that the file's full content exactly matches \
                 the expected text. Avoid for large files.",
        }
    }

    /// Validate strategy-specific parameters.  Returns `Err` with a
    /// human-readable message when the strategy is misconfigured.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            AuditStrategy::None => Ok(()),

            AuditStrategy::Checksum { expected_sha256 } => {
                if expected_sha256.trim().is_empty() {
                    return Err("Checksum: expected_sha256 must not be empty.".into());
                }
                if !expected_sha256.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(
                        "Checksum: expected_sha256 must be a valid hex string.".into()
                    );
                }
                if expected_sha256.len() != 64 {
                    return Err(
                        "Checksum: expected_sha256 must be a 64-character SHA-256 hex digest."
                            .into(),
                    );
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

/// One expected line change in a [`AuditStrategy::LineMatch`] strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineRule {
    /// Whether this line is expected to have been added or removed.
    pub action: LineAction,
    /// The exact line content (without a trailing newline).
    pub line: String,
}

/// Direction of a line change in [`LineRule`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineAction {
    /// Line is expected to be present only in the *after* folder (added).
    Added,
    /// Line is expected to be present only in the *before* folder (removed).
    Removed,
}

impl std::fmt::Display for LineAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineAction::Added => write!(f, "Added"),
            LineAction::Removed => write!(f, "Removed"),
        }
    }
}

// ── Regex target ──────────────────────────────────────────────────────────

/// Which lines the [`AuditStrategy::Regex`] pattern is applied to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegexTarget {
    /// Apply to added lines (present in *after* but not *before*).
    #[default]
    AddedLines,
    /// Apply to removed lines (present in *before* but not *after*).
    RemovedLines,
    /// Apply to all changed lines (union of added and removed).
    AllChangedLines,
}

impl std::fmt::Display for RegexTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegexTarget::AddedLines => write!(f, "Added lines"),
            RegexTarget::RemovedLines => write!(f, "Removed lines"),
            RegexTarget::AllChangedLines => write!(f, "All changed lines"),
        }
    }
}
