use super::*;
use crate::diff::entry::{DiffEntry, DiffType};
use crate::config::definition::{AuditEntry, AuditStrategy, LineRule, LineAction};

fn make_diff(path: &str, size: u64, diff_type: DiffType) -> DiffEntry {
    DiffEntry {
        path: path.to_string(), diff_type, is_dir: false,
        before_text: None, after_text: None,
        is_binary: false,
        before_size: Some(size), after_size: Some(size),
        before_sha256: None, after_sha256: None,
        stats: None, error_detail: None,
    }
}

fn make_entry(strategy: AuditStrategy) -> AuditEntry {
    AuditEntry {
        path: "f.txt".to_string(),
        diff_type: DiffType::Modified,
        reason: "test".to_string(),
        strategy,
        enabled: true,
        ticket: None,
        approved_by: None,
        approved_at: None,
        expires_at: None,
        note: None,
        created_at: None,
        updated_at: None,
    }
}

#[test]
fn large_file_with_linematch_warns() {
    let diff  = make_diff("big.txt", 2 * 1024 * 1024, DiffType::Modified);
    let entry = make_entry(AuditStrategy::LineMatch {
        rules: vec![LineRule { action: LineAction::Added, line: "x".into() }],
    });
    let warns = collect(&diff, &entry);
    assert!(warns.iter().any(|w| matches!(w, AuditWarning::LargeFileStrategy { .. })),
        "large LineMatch should warn");
}

#[test]
fn large_file_with_checksum_no_warn() {
    let diff  = make_diff("big.bin", 2 * 1024 * 1024, DiffType::Modified);
    let entry = make_entry(AuditStrategy::Checksum { expected_sha256: "a".repeat(64) });
    let warns = collect(&diff, &entry);
    assert!(!warns.iter().any(|w| matches!(w, AuditWarning::LargeFileStrategy { .. })),
        "Checksum on large file should not warn");
}

#[test]
fn none_strategy_on_modified_warns() {
    let diff  = make_diff("cfg.toml", 100, DiffType::Modified);
    let entry = make_entry(AuditStrategy::None);
    let warns = collect(&diff, &entry);
    assert!(warns.iter().any(|w| matches!(w, AuditWarning::NoStrategyOnModified { .. })));
}

#[test]
fn none_strategy_on_added_no_warn() {
    let diff  = make_diff("new.txt", 100, DiffType::Added);
    let mut entry = make_entry(AuditStrategy::None);
    entry.diff_type = DiffType::Added;
    let warns = collect(&diff, &entry);
    assert!(!warns.iter().any(|w| matches!(w, AuditWarning::NoStrategyOnModified { .. })));
}

#[test]
fn no_approver_warns() {
    let diff  = make_diff("f.txt", 100, DiffType::Modified);
    let entry = make_entry(AuditStrategy::None);
    let warns = collect(&diff, &entry);
    assert!(warns.iter().any(|w| matches!(w, AuditWarning::NoApprover { .. })));
}

#[test]
fn with_approver_no_warn() {
    let diff  = make_diff("f.txt", 100, DiffType::Modified);
    let mut entry = make_entry(AuditStrategy::None);
    entry.approved_by = Some("alice".into());
    let warns = collect(&diff, &entry);
    assert!(!warns.iter().any(|w| matches!(w, AuditWarning::NoApprover { .. })));
}
