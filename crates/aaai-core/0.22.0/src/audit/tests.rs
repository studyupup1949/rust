//! Unit tests for audit strategies and the audit engine.

use crate::config::definition::{
    AuditDefinition, AuditEntry, AuditStrategy, LineAction, LineRule, RegexTarget,
};
use crate::diff::entry::{DiffEntry, DiffType};
use super::engine::AuditEngine;
use super::result::AuditStatus;
use super::strategy;

// ── helpers ──────────────────────────────────────────────────────────────

fn make_diff(path: &str, diff_type: DiffType, before: Option<&str>, after: Option<&str>) -> DiffEntry {
    use sha2::{Digest, Sha256};
    let before_sha256 = before.map(|t| hex::encode(Sha256::digest(t.as_bytes())));
    let after_sha256  = after.map(|t| hex::encode(Sha256::digest(t.as_bytes())));
    let stats = match (diff_type, before, after) {
        (DiffType::Modified, Some(b), Some(a)) =>
            Some(crate::diff::entry::DiffStats::compute(b, a)),
        _ => None,
    };
    DiffEntry {
        path: path.to_string(),
        diff_type,
        is_dir: false,
        before_text: before.map(String::from),
        after_text:  after.map(String::from),
        is_binary: false,
        before_size: before.map(|t| t.len() as u64),
        after_size:  after.map(|t| t.len() as u64),
        before_sha256,
        after_sha256,
        stats,
        error_detail: None,
    }
}

fn make_entry(path: &str, diff_type: DiffType, strategy: AuditStrategy) -> AuditEntry {
    AuditEntry {
        path: path.to_string(),
        diff_type,
        reason: "test reason".to_string(),
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

fn make_def(entries: Vec<AuditEntry>) -> AuditDefinition {
    let mut def = AuditDefinition::new_empty();
    def.entries = entries;
    def
}

// ── strategy::None ────────────────────────────────────────────────────────

#[test]
fn strategy_none_always_ok() {
    let diff = make_diff("f.txt", DiffType::Added, None, Some("x"));
    assert!(strategy::evaluate(&AuditStrategy::None, &diff).is_ok());
}

// ── strategy::Checksum ────────────────────────────────────────────────────

#[test]
fn checksum_matches() {
    use sha2::{Digest, Sha256};
    let content = "hello";
    let sha = hex::encode(Sha256::digest(content.as_bytes()));
    let diff = make_diff("f", DiffType::Modified, Some("old"), Some(content));
    let strat = AuditStrategy::Checksum { expected_sha256: sha };
    assert!(strategy::evaluate(&strat, &diff).is_ok());
}

#[test]
fn checksum_mismatch_fails() {
    let diff = make_diff("f", DiffType::Modified, Some("old"), Some("hello"));
    let strat = AuditStrategy::Checksum {
        expected_sha256: "a".repeat(64),
    };
    assert!(strategy::evaluate(&strat, &diff).is_err());
}

// ── strategy::LineMatch ───────────────────────────────────────────────────

#[test]
fn linematch_added_line_ok() {
    let diff = make_diff("cfg.toml", DiffType::Modified,
        Some("port = 80\n"),
        Some("port = 8080\n"));
    let strat = AuditStrategy::LineMatch {
        rules: vec![
            LineRule { action: LineAction::Removed, line: "port = 80".to_string() },
            LineRule { action: LineAction::Added,   line: "port = 8080".to_string() },
        ],
    };
    assert!(strategy::evaluate(&strat, &diff).is_ok());
}

#[test]
fn linematch_missing_line_fails() {
    let diff = make_diff("cfg.toml", DiffType::Modified,
        Some("port = 80\n"),
        Some("port = 9999\n"));
    let strat = AuditStrategy::LineMatch {
        rules: vec![
            LineRule { action: LineAction::Added, line: "port = 8080".to_string() },
        ],
    };
    let result = strategy::evaluate(&strat, &diff);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("port = 8080"));
}

// ── strategy::Regex ───────────────────────────────────────────────────────

#[test]
fn regex_added_lines_match() {
    let diff = make_diff("ver.txt", DiffType::Modified,
        Some("version = 1.0\n"),
        Some("version = 2.0\n"));
    let strat = AuditStrategy::Regex {
        pattern: r"^version = \d+\.\d+$".to_string(),
        target: RegexTarget::AddedLines,
    };
    assert!(strategy::evaluate(&strat, &diff).is_ok());
}

#[test]
fn regex_invalid_pattern_errors() {
    let diff = make_diff("f", DiffType::Modified, Some("a\n"), Some("b\n"));
    let strat = AuditStrategy::Regex {
        pattern: "[invalid".to_string(),
        target: RegexTarget::AddedLines,
    };
    assert!(strategy::evaluate(&strat, &diff).is_err());
}

// ── strategy::Exact ───────────────────────────────────────────────────────

#[test]
fn exact_match_ok() {
    let expected = "exact content\n";
    let diff = make_diff("f", DiffType::Modified, Some("old\n"), Some(expected));
    let strat = AuditStrategy::Exact { expected_content: expected.to_string() };
    assert!(strategy::evaluate(&strat, &diff).is_ok());
}

#[test]
fn exact_mismatch_fails() {
    let diff = make_diff("f", DiffType::Modified, Some("old\n"), Some("actual\n"));
    let strat = AuditStrategy::Exact { expected_content: "expected\n".to_string() };
    assert!(strategy::evaluate(&strat, &diff).is_err());
}

// ── AuditEngine ───────────────────────────────────────────────────────────

#[test]
fn pending_when_no_entry() {
    let diff = make_diff("unknown.txt", DiffType::Added, None, Some("x"));
    let def = make_def(vec![]);
    let result = AuditEngine::evaluate(&[diff], &def);
    assert_eq!(result.results[0].status, AuditStatus::Pending);
}

#[test]
fn ok_when_diff_type_matches_and_strategy_passes() {
    let diff = make_diff("f.txt", DiffType::Added, None, Some("x"));
    let entry = make_entry("f.txt", DiffType::Added, AuditStrategy::None);
    let def = make_def(vec![entry]);
    let result = AuditEngine::evaluate(&[diff], &def);
    assert_eq!(result.results[0].status, AuditStatus::Ok);
}

#[test]
fn failed_when_diff_type_mismatch() {
    let diff = make_diff("f.txt", DiffType::Modified, Some("a\n"), Some("b\n"));
    let entry = make_entry("f.txt", DiffType::Added, AuditStrategy::None);
    let def = make_def(vec![entry]);
    let result = AuditEngine::evaluate(&[diff], &def);
    assert_eq!(result.results[0].status, AuditStatus::Failed);
}

#[test]
fn ignored_when_entry_disabled() {
    let diff = make_diff("f.txt", DiffType::Added, None, Some("x"));
    let mut entry = make_entry("f.txt", DiffType::Added, AuditStrategy::None);
    entry.enabled = false;
    let def = make_def(vec![entry]);
    let result = AuditEngine::evaluate(&[diff], &def);
    assert_eq!(result.results[0].status, AuditStatus::Ignored);
}

#[test]
fn error_for_unreadable_diff() {
    let mut diff = make_diff("f.txt", DiffType::Unreadable, None, None);
    diff.error_detail = Some("Permission denied".into());
    let def = make_def(vec![]);
    let result = AuditEngine::evaluate(&[diff], &def);
    assert_eq!(result.results[0].status, AuditStatus::Error);
}

#[test]
fn summary_counts_are_correct() {
    let diffs = vec![
        make_diff("ok.txt",      DiffType::Added,    None,          Some("x")),
        make_diff("pending.txt", DiffType::Added,    None,          Some("y")),
        make_diff("failed.txt",  DiffType::Modified, Some("a\n"),   Some("b\n")),
    ];
    let def = make_def(vec![
        make_entry("ok.txt", DiffType::Added, AuditStrategy::None),
        // pending.txt has no entry
        make_entry("failed.txt", DiffType::Added, AuditStrategy::None), // type mismatch
    ]);
    let result = AuditEngine::evaluate(&diffs, &def);
    assert_eq!(result.summary.ok,      1);
    assert_eq!(result.summary.pending, 1);
    assert_eq!(result.summary.failed,  1);
}

// ── Phase 3 behaviours ───────────────────────────────────────────────────

#[test]
fn empty_reason_is_pending_not_ok() {
    let diff = make_diff("f.txt", DiffType::Added, None, Some("x"));
    let mut entry = make_entry("f.txt", DiffType::Added, AuditStrategy::None);
    entry.reason = "   ".to_string(); // whitespace only
    let def = make_def(vec![entry]);
    let result = AuditEngine::evaluate(&[diff], &def);
    assert_eq!(result.results[0].status, AuditStatus::Pending,
        "empty reason must produce Pending, not OK");
}

#[test]
fn ok_requires_non_empty_reason() {
    let diff = make_diff("f.txt", DiffType::Added, None, Some("x"));
    let mut entry = make_entry("f.txt", DiffType::Added, AuditStrategy::None);
    entry.reason = "Intentionally added".to_string();
    let def = make_def(vec![entry]);
    let result = AuditEngine::evaluate(&[diff], &def);
    assert_eq!(result.results[0].status, AuditStatus::Ok);
}

#[test]
fn unchanged_is_auto_ok_without_entry() {
    let diff = make_diff("same.txt", DiffType::Unchanged, Some("x"), Some("x"));
    let def = make_def(vec![]);
    let result = AuditEngine::evaluate(&[diff], &def);
    assert_eq!(result.results[0].status, AuditStatus::Ok,
        "Unchanged entries should be auto-OK even without a rule");
}

