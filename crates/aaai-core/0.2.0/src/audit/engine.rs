//! Audit engine: matches DiffEntries against AuditDefinition → AuditResult.

use crate::config::definition::AuditDefinition;
use crate::diff::entry::{DiffEntry, DiffType};
use super::result::{AuditResult, AuditStatus, FileAuditResult};
use super::strategy;

pub struct AuditEngine;

impl AuditEngine {
    /// Judge every DiffEntry against the AuditDefinition.
    pub fn evaluate(diffs: &[DiffEntry], definition: &AuditDefinition) -> AuditResult {
        let mut results = Vec::new();

        for diff in diffs {
            let result = judge(diff, definition);
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
        };
    }

    // Unchanged entries have no diff to audit — auto-OK regardless of rules.
    if diff.diff_type == DiffType::Unchanged {
        return FileAuditResult {
            diff: diff.clone(),
            entry: definition.find_entry(&diff.path).cloned(),
            status: AuditStatus::Ok,
            detail: None,
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
        };
    }

    // Content strategy check.
    match strategy::evaluate(&entry.strategy, diff) {
        Ok(()) => FileAuditResult {
            diff: diff.clone(),
            entry: Some(entry.clone()),
            status: AuditStatus::Ok,
            detail: None,
        },
        Err(msg) => FileAuditResult {
            diff: diff.clone(),
            entry: Some(entry.clone()),
            status: AuditStatus::Failed,
            detail: Some(msg),
        },
    }
}
