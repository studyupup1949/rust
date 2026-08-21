//! Audit engine: matches DiffEntries against AuditDefinition → AuditResult.

use crate::audit::warning;
use crate::config::definition::AuditDefinition;
use crate::diff::entry::{DiffEntry, DiffType};
use super::result::{AuditResult, AuditStatus, FileAuditResult};
use super::strategy;

pub struct AuditEngine;

/// Options for audit evaluation.
#[derive(Debug, Clone, Default)]
pub struct AuditOptions {
    /// Warning kind IDs to suppress (e.g. ["no-approver"]).
    pub suppress_warnings: Vec<String>,
}

impl AuditEngine {
    /// Judge every DiffEntry against the AuditDefinition.
    pub fn evaluate(diffs: &[DiffEntry], definition: &AuditDefinition) -> AuditResult {
        Self::evaluate_with_options(diffs, definition, &AuditOptions::default())
    }

    /// Evaluate with custom options (warning suppression, etc.)
    pub fn evaluate_with_options(
        diffs: &[DiffEntry],
        definition: &AuditDefinition,
        options: &AuditOptions,
    ) -> AuditResult {
        let mut results = Vec::new();

        for diff in diffs {
            let mut result = judge(diff, definition);
            // Filter suppressed warnings.
            if !options.suppress_warnings.is_empty() {
                result.warnings.retain(|w| {
                    !options.suppress_warnings.iter().any(|s| s == w.kind())
                });
            }
            results.push(result);
        }

        AuditResult::new(results)
    }
}

fn judge(diff: &DiffEntry, definition: &AuditDefinition) -> FileAuditResult {
    // Diff-level errors (Unreadable / Incomparable) → always Error.
    if diff.diff_type.is_error() {
        return FileAuditResult {
            diff: diff.clone(),
            entry: None,
            status: AuditStatus::Error,
            detail: diff.error_detail.clone().or_else(|| {
                Some("File could not be read or compared.".into())
            }),
            warnings: Vec::new(),
        };
    }

    // Unchanged entries have no diff to audit — auto-OK regardless of rules.
    if diff.diff_type == DiffType::Unchanged {
        return FileAuditResult {
            diff: diff.clone(),
            entry: definition.find_entry(&diff.path).cloned(),
            status: AuditStatus::Ok,
            detail: None,
            warnings: Vec::new(),
        };
    }

    // Look up the matching entry.
    let entry = match definition.find_entry(&diff.path) {
        Some(e) => e,
        None => {
            return FileAuditResult {
                diff: diff.clone(),
                entry: None,
                status: AuditStatus::Pending,
                detail: Some("No audit rule defined for this path.".into()),
                warnings: Vec::new(),
            };
        }
    };

    // Disabled entries → Ignored.
    if !entry.enabled {
        return FileAuditResult {
            diff: diff.clone(),
            entry: Some(entry.clone()),
            status: AuditStatus::Ignored,
            detail: Some("Entry is disabled.".into()),
            warnings: Vec::new(),
        };
    }

    // Empty reason → treat as Pending (not yet human-approved).
    if entry.reason.trim().is_empty() {
        return FileAuditResult {
            diff: diff.clone(),
            entry: Some(entry.clone()),
            status: AuditStatus::Pending,
            detail: Some("Entry exists but has no reason — human approval required.".into()),
            warnings: Vec::new(),
        };
    }

    // Diff-type mismatch → Failed.
    if entry.diff_type != diff.diff_type {
        return FileAuditResult {
            diff: diff.clone(),
            entry: Some(entry.clone()),
            status: AuditStatus::Failed,
            detail: Some(format!(
                "Expected diff type {:?} but found {:?}.",
                entry.diff_type, diff.diff_type
            )),
            warnings: Vec::new(),
        };
    }

    // Content strategy check.
    match strategy::evaluate(&entry.strategy, diff) {
        Ok(()) => {
            let warns = warning::collect(diff, entry);
            FileAuditResult {
                diff: diff.clone(),
                entry: Some(entry.clone()),
                status: AuditStatus::Ok,
                detail: None,
                warnings: warns,
            }
        }
        Err(msg) => FileAuditResult {
            diff: diff.clone(),
            entry: Some(entry.clone()),
            status: AuditStatus::Failed,
            detail: Some(msg),
            warnings: Vec::new(),
        },
    }
}
