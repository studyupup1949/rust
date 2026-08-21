//! Tests for config round-trip serialization, validation, and glob matching.

use super::definition::{
    AuditDefinition, AuditEntry, AuditStrategy, LineAction, LineRule,
};
use crate::diff::entry::DiffType;

fn sample_entry() -> AuditEntry {
    AuditEntry {
        path: "config/server.toml".to_string(),
        diff_type: DiffType::Modified,
        reason: "Port change".to_string(),
        strategy: AuditStrategy::LineMatch {
            rules: vec![
                LineRule { action: LineAction::Removed, line: "port = 80".to_string() },
                LineRule { action: LineAction::Added,   line: "port = 8080".to_string() },
            ],
        },
        enabled: true,
        ticket: Some("INF-42".to_string()),
        approved_by: None,
        approved_at: None,
        expires_at: None,
        note: Some("INF-42".to_string()),
        created_at: None,
        updated_at: None,
    }
}

#[test]
fn round_trip_yaml() {
    let mut def = AuditDefinition::new_empty();
    def.entries.push(sample_entry());
    let yaml = serde_yaml::to_string(&def).unwrap();
    let restored: AuditDefinition = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(restored.version, "1");
    assert_eq!(restored.entries.len(), 1);
    assert_eq!(restored.entries[0].note.as_deref(), Some("INF-42"));
}

#[test]
fn upsert_replaces_existing() {
    let mut def = AuditDefinition::new_empty();
    def.upsert_entry(sample_entry());
    let mut updated = sample_entry();
    updated.reason = "Updated reason".to_string();
    def.upsert_entry(updated);
    assert_eq!(def.entries.len(), 1, "should not duplicate");
    assert_eq!(def.entries[0].reason, "Updated reason");
}

#[test]
fn upsert_appends_new_path() {
    let mut def = AuditDefinition::new_empty();
    def.upsert_entry(sample_entry());
    let mut other = sample_entry();
    other.path = "other.txt".to_string();
    def.upsert_entry(other);
    assert_eq!(def.entries.len(), 2);
}

#[test]
fn approvable_fails_empty_reason() {
    let mut e = sample_entry();
    e.reason = "   ".to_string();
    assert!(e.is_approvable().is_err());
}

#[test]
fn approvable_fails_invalid_regex() {
    let mut e = sample_entry();
    e.strategy = AuditStrategy::Regex { pattern: "[invalid".to_string(), target: Default::default() };
    assert!(e.is_approvable().is_err());
}

#[test]
fn approvable_ok_for_valid_entry() {
    assert!(sample_entry().is_approvable().is_ok());
}

#[test]
fn strategy_validate_checksum_requires_64_hex() {
    assert!(AuditStrategy::Checksum { expected_sha256: "a".repeat(64) }.validate().is_ok());
    assert!(AuditStrategy::Checksum { expected_sha256: "abc".to_string() }.validate().is_err());
    assert!(AuditStrategy::Checksum { expected_sha256: "z".repeat(64) }.validate().is_err());
}

#[test]
fn strategy_validate_linematch_requires_rules() {
    assert!(AuditStrategy::LineMatch { rules: vec![] }.validate().is_err());
}

// ── Glob pattern matching ────────────────────────────────────────────────

#[test]
fn is_glob_detects_star() {
    let mut e = sample_entry();
    e.path = "logs/*.log".to_string();
    assert!(e.is_glob());
}

#[test]
fn is_glob_false_for_exact() {
    assert!(!sample_entry().is_glob());
}

#[test]
fn glob_matches_wildcard() {
    let mut e = sample_entry();
    e.path = "logs/*.log".to_string();
    assert!(e.glob_matches("logs/app.log"));
    assert!(e.glob_matches("logs/error.log"));
    assert!(!e.glob_matches("logs/app.txt"));
    assert!(!e.glob_matches("other/app.log"));
}

#[test]
fn glob_double_star_matches_subdirs() {
    let mut e = sample_entry();
    e.path = "build/**".to_string();
    assert!(e.glob_matches("build/output.js"));
    assert!(e.glob_matches("build/assets/main.css"));
}

#[test]
fn find_entry_exact_before_glob() {
    let mut def = AuditDefinition::new_empty();
    // Glob entry
    let mut glob_entry = sample_entry();
    glob_entry.path = "config/*.toml".to_string();
    glob_entry.reason = "glob".to_string();
    def.entries.push(glob_entry);
    // Exact entry
    let mut exact_entry = sample_entry();
    exact_entry.reason = "exact".to_string();
    def.entries.push(exact_entry);

    // Exact path should win
    let found = def.find_entry("config/server.toml").unwrap();
    assert_eq!(found.reason, "exact");
}

#[test]
fn find_entry_falls_back_to_glob() {
    let mut def = AuditDefinition::new_empty();
    let mut glob_entry = sample_entry();
    glob_entry.path = "config/*.toml".to_string();
    glob_entry.reason = "glob match".to_string();
    def.entries.push(glob_entry);

    let found = def.find_entry("config/database.toml").unwrap();
    assert_eq!(found.reason, "glob match");
}
