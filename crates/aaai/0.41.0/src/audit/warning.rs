//! Audit warnings — non-fatal issues surfaced alongside results.
//!
//! Unlike [`AuditStatus::Error`], warnings do not change a file's verdict.
//! They are advisory: the auditor should review them but the audit is not
//! automatically failed.
//!
//! # Phase 7 warnings
//! * [`AuditWarning::LargeFileStrategy`] — Exact or LineMatch strategy applied
//!   to a file larger than [`LARGE_FILE_THRESHOLD`].

use crate::diff::entry::{DiffEntry, LARGE_FILE_THRESHOLD, fmt_size};
use crate::config::definition::{AuditEntry, AuditStrategy};

/// A non-fatal advisory raised during audit evaluation.
#[derive(Debug, Clone)]
pub enum AuditWarning {
    /// An expensive content strategy was applied to a large file.
    LargeFileStrategy {
        path: String,
        strategy: &'static str,
        size_bytes: u64,
    },
    /// An entry is using the `None` strategy on a Modified file —
    /// may be intentional, but worth reviewing.
    NoStrategyOnModified { path: String },
    /// An entry exists but has no approved_by field.
    NoApprover { path: String },
}

impl AuditWarning {
    /// Human-readable description.
    pub fn message(&self) -> String {
        match self {
            AuditWarning::LargeFileStrategy { path, strategy, size_bytes } =>
                format!(
                    "{path}: {strategy} strategy applied to a large file ({}). \
                     Consider using Checksum instead.",
                    fmt_size(*size_bytes)
                ),
            AuditWarning::NoStrategyOnModified { path } =>
                format!("{path}: Modified file uses `None` strategy — content not verified."),
            AuditWarning::NoApprover { path } =>
                format!("{path}: Entry has no `approved_by` field — approver unknown."),
        }
    }

    /// Severity label for CLI / report display.
    pub fn kind(&self) -> &'static str {
        match self {
            AuditWarning::LargeFileStrategy { .. }  => "large-file",
            AuditWarning::NoStrategyOnModified { .. } => "no-strategy",
            AuditWarning::NoApprover { .. }          => "no-approver",
        }
    }
}

/// Collect warnings for a single (diff, entry) pair.
///
/// Returns an empty Vec when there are no advisory issues.
pub fn collect(diff: &DiffEntry, entry: &AuditEntry) -> Vec<AuditWarning> {
    let mut warns = Vec::new();

    // Large-file strategy check.
    let size = diff.after_size.or(diff.before_size).unwrap_or(0);
    if size > LARGE_FILE_THRESHOLD {
        match &entry.strategy {
            AuditStrategy::Exact { .. } | AuditStrategy::LineMatch { .. } => {
                warns.push(AuditWarning::LargeFileStrategy {
                    path: diff.path.clone(),
                    strategy: entry.strategy.label(),
                    size_bytes: size,
                });
            }
            _ => {}
        }
    }

    // No-strategy on Modified.
    if diff.diff_type == crate::diff::entry::DiffType::Modified {
        if let AuditStrategy::None = entry.strategy {
            warns.push(AuditWarning::NoStrategyOnModified { path: diff.path.clone() });
        }
    }

    // No approver.
    if entry.approved_by.is_none() && !entry.reason.trim().is_empty() {
        warns.push(AuditWarning::NoApprover { path: diff.path.clone() });
    }

    warns
}

#[cfg(test)]
mod tests;
