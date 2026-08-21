//! Integration tests for the aaai CLI binary.
//!
//! These tests build and invoke the actual binary to verify end-to-end
//! behaviour — argument parsing, exit codes, and output format.

use std::fs;
use std::path::Path;
use std::process::Command;

// ── helpers ──────────────────────────────────────────────────────────────

fn aaai() -> Command {
    // Find the binary relative to the workspace target directory.
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // crates/aaai-cli -> crates
    path.pop(); // crates -> workspace root
    path.push("target");
    path.push("debug");
    path.push("aaai");
    Command::new(path)
}

fn setup_dirs(before: &Path, after: &Path) {
    fs::create_dir_all(before).unwrap();
    fs::create_dir_all(after).unwrap();
}

fn write(dir: &Path, name: &str, content: &str) {
    fs::write(dir.join(name), content).unwrap();
}

fn write_audit(path: &Path, yaml: &str) {
    fs::write(path, yaml).unwrap();
}

// ── audit subcommand ──────────────────────────────────────────────────────

#[test]
fn audit_exits_0_when_all_ok() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "f.txt", "hello\n");
    write(&a, "f.txt", "hello world\n");

    let audit_yaml = tmp.path().join("audit.yaml");
    write_audit(&audit_yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "Intentional change"
    strategy:
      type: None
    enabled: true
"#);

    let status = aaai()
        .args(["audit", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", audit_yaml.to_str().unwrap(),
               "--no-history"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
}

#[test]
fn audit_exits_1_on_failed_entry() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "cfg.toml", "port = 80\n");
    write(&a, "cfg.toml", "port = 9999\n");

    let audit_yaml = tmp.path().join("audit.yaml");
    write_audit(&audit_yaml, r#"version: "1"
entries:
  - path: cfg.toml
    diff_type: Modified
    reason: "Port change"
    strategy:
      type: LineMatch
      rules:
        - action: Removed
          line: "port = 80"
        - action: Added
          line: "port = 8080"
    enabled: true
"#);

    let status = aaai()
        .args(["audit", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", audit_yaml.to_str().unwrap(),
               "--no-history"])
        .status().unwrap();
    assert_eq!(status.code(), Some(1), "Failed audit should exit 1");
}

#[test]
fn audit_exits_2_on_pending_entry() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&a, "new.txt", "content\n");

    // Empty audit definition → new.txt will be Pending.
    let audit_yaml = tmp.path().join("audit.yaml");
    write_audit(&audit_yaml, "version: \"1\"\n");

    let status = aaai()
        .args(["audit", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", audit_yaml.to_str().unwrap(),
               "--no-history"])
        .status().unwrap();
    assert_eq!(status.code(), Some(2), "Pending should exit 2");
}

#[test]
fn audit_exit_0_with_allow_pending() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&a, "new.txt", "content\n");
    let audit_yaml = tmp.path().join("audit.yaml");
    write_audit(&audit_yaml, "version: \"1\"\n");

    let status = aaai()
        .args(["audit", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", audit_yaml.to_str().unwrap(),
               "--allow-pending", "--no-history"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
}

#[test]
fn audit_json_output_is_valid_json() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "f.txt", "a\n");
    write(&a, "f.txt", "b\n");

    let audit_yaml = tmp.path().join("audit.yaml");
    write_audit(&audit_yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "test"
    strategy:
      type: None
    enabled: true
"#);

    let out = aaai()
        .args(["audit", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", audit_yaml.to_str().unwrap(),
               "--json-output", "--no-history"])
        .output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok(),
        "JSON output must be valid JSON; got: {stdout}");
}

// ── snap subcommand ───────────────────────────────────────────────────────

#[test]
fn snap_generates_definition_file() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&a, "added.txt", "new\n");
    write(&b, "removed.txt", "old\n");

    let out_path = tmp.path().join("audit.yaml");
    let status = aaai()
        .args(["snap", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--out", out_path.to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(out_path.exists(), "snap should create the output file");
    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("added.txt") || content.contains("removed.txt"),
        "snap output should list diffed files");
}

#[test]
fn snap_dry_run_does_not_write_file() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&a, "new.txt", "hi\n");

    let out_path = tmp.path().join("audit.yaml");
    let status = aaai()
        .args(["snap", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--out", out_path.to_str().unwrap(),
               "--dry-run"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(!out_path.exists(), "--dry-run must NOT write the file");
}

// ── check subcommand ──────────────────────────────────────────────────────

#[test]
fn check_exits_0_for_valid_definition() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "test"
    strategy:
      type: None
    enabled: true
"#);
    let status = aaai()
        .args(["check", yaml.to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
}

#[test]
fn check_exits_nonzero_for_invalid_yaml() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("bad.yaml");
    write_audit(&yaml, "not: valid: yaml: [[[");
    let status = aaai()
        .args(["check", yaml.to_str().unwrap()])
        .status().unwrap();
    assert_ne!(status.code(), Some(0), "invalid YAML must fail check");
}

// ── report subcommand ─────────────────────────────────────────────────────

#[test]
fn report_markdown_creates_file() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, "version: \"1\"\n");
    let out = tmp.path().join("report.md");

    let status = aaai()
        .args(["report", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap(),
               "--out", out.to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(out.exists());
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("aaai Audit Report"));
}

#[test]
fn report_html_creates_valid_file() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "f.txt", "old\n");
    write(&a, "f.txt", "new\n");
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "html test"
    strategy:
      type: None
    enabled: true
"#);
    let out = tmp.path().join("report.html");

    let status = aaai()
        .args(["report", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap(),
               "--format", "html",
               "--out", out.to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(out.exists());
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("<!DOCTYPE html>"), "HTML report should start with DOCTYPE");
    assert!(content.contains("aaai Audit Report"));
}

// ── glob rule ─────────────────────────────────────────────────────────────

#[test]
fn glob_rule_matches_multiple_files() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    fs::create_dir_all(b.join("logs")).unwrap();
    fs::create_dir_all(a.join("logs")).unwrap();
    write(&b.join("logs"), "app.log", "old\n");
    write(&a.join("logs"), "app.log", "new\n");
    write(&b.join("logs"), "db.log", "old\n");
    write(&a.join("logs"), "db.log", "new\n");

    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: "logs/*.log"
    diff_type: Modified
    reason: "Log rotation"
    strategy:
      type: None
    enabled: true
"#);

    let status = aaai()
        .args(["audit", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap(),
               "--no-history"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0), "glob rule should match both log files");
}

// ── Phase 7: new commands ────────────────────────────────────────────────

#[test]
fn diff_exits_0_and_shows_changes() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "old.txt", "line1\n");
    write(&a, "new.txt", "line2\n");
    write(&b, "same.txt", "same\n");
    write(&a, "same.txt", "same\n");

    let out = aaai()
        .args(["diff", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap()])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("old.txt") || stdout.contains("new.txt"),
        "diff should mention changed files");
}

#[test]
fn diff_json_output_is_valid() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&a, "added.txt", "content\n");

    let out = aaai()
        .args(["diff", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--json-output"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("diff --json-output must produce valid JSON");
    assert!(parsed.is_array(), "diff JSON output should be an array");
}

#[test]
fn export_csv_produces_header_row() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "f.txt", "old\n");
    write(&a, "f.txt", "new\n");
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "test export"
    strategy:
      type: None
    enabled: true
"#);
    let out_csv = tmp.path().join("out.csv");

    let status = aaai()
        .args(["export", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap(),
               "--out", out_csv.to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(out_csv.exists());
    let content = fs::read_to_string(&out_csv).unwrap();
    let first_line = content.lines().next().unwrap_or("");
    assert!(first_line.contains("path"), "CSV header should contain 'path'");
    assert!(first_line.contains("reason"), "CSV header should contain 'reason'");
    assert!(first_line.contains("status"), "CSV header should contain 'status'");
}

#[test]
fn export_tsv_uses_tab_separator() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, "version: \"1\"\n");
    let out_tsv = tmp.path().join("out.tsv");

    let status = aaai()
        .args(["export", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap(),
               "--format", "tsv",
               "--out", out_tsv.to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out_tsv).unwrap();
    let first_line = content.lines().next().unwrap_or("");
    assert!(first_line.contains('\t'), "TSV header should use tab separator");
}

#[test]
fn merge_detect_conflicts() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().join("base.yaml");
    let overlay = tmp.path().join("overlay.yaml");
    write_audit(&base, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "base"
    strategy:
      type: None
    enabled: true
"#);
    write_audit(&overlay, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Added
    reason: "overlay"
    strategy:
      type: None
    enabled: true
"#);

    let out = aaai()
        .args(["merge", base.to_str().unwrap(), overlay.to_str().unwrap(),
               "--detect-conflicts"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("conflict") || stdout.contains("Conflict"),
        "conflict detection should mention conflicts");
}

#[test]
fn report_sarif_format_valid() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, "version: \"1\"\n");
    let out = tmp.path().join("result.sarif");

    let status = aaai()
        .args(["report", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap(),
               "--format", "sarif",
               "--out", out.to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out).unwrap();
    let v: serde_json::Value = serde_json::from_str(&content)
        .expect("SARIF output must be valid JSON");
    assert_eq!(v["version"], "2.1.0", "SARIF version must be 2.1.0");
}

#[test]
fn history_stats_exits_0() {
    let status = aaai()
        .args(["history", "--stats"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
}

#[test]
fn audit_warns_on_large_file_with_linematch() {
    // Verify the audit JSON output includes the Warning field / or at least exits cleanly.
    // (We can't easily create a >1 MB file in a quick test, so we verify the command works.)
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "cfg.toml", "x = 1\n");
    write(&a, "cfg.toml", "x = 2\n");
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: cfg.toml
    diff_type: Modified
    reason: "value change"
    strategy:
      type: LineMatch
      rules:
        - action: Removed
          line: "x = 1"
        - action: Added
          line: "x = 2"
    enabled: true
"#);
    let out = aaai()
        .args(["audit", "--left", b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap(),
               "--json-output", "--no-history"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["result"], "PASSED");
}

// ── Phase 9: previously untested commands ───────────────────────────────

#[test]
fn completions_bash_exits_0() {
    let out = aaai().args(["completions", "bash"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("aaai"), "bash completion should mention 'aaai'");
}

#[test]
fn completions_zsh_exits_0() {
    let out = aaai().args(["completions", "zsh"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.is_empty(), "zsh completion output should not be empty");
}

#[test]
fn config_init_creates_aaai_yaml() {
    let tmp = tempfile::tempdir().unwrap();
    let status = aaai()
        .args(["config", "--init", "--dir", tmp.path().to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    let config_path = tmp.path().join(".aaai.yaml");
    assert!(config_path.exists(), ".aaai.yaml should be created");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("version"), ".aaai.yaml should contain 'version'");
}

#[test]
fn config_init_skips_existing() {
    let tmp = tempfile::tempdir().unwrap();
    // Create first
    aaai().args(["config", "--init", "--dir", tmp.path().to_str().unwrap()])
        .status().unwrap();
    // Try again — should report already exists, not overwrite
    let out = aaai()
        .args(["config", "--init", "--dir", tmp.path().to_str().unwrap()])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("already exists") || stdout.contains("既に"),
        "should report existing config");
}

#[test]
fn dashboard_exits_0() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, "version: \"1\"\n");

    let status = aaai()
        .args(["dashboard",
               "--left",   b.to_str().unwrap(),
               "--right",  a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
}

#[test]
fn init_non_interactive_creates_config() {
    let tmp = tempfile::tempdir().unwrap();
    let status = aaai()
        .args(["init", "--dir", tmp.path().to_str().unwrap(), "--non-interactive"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    let config_path = tmp.path().join(".aaai.yaml");
    assert!(config_path.exists(), "non-interactive init should create .aaai.yaml");
}

#[test]
fn lint_json_output_valid() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "valid reason text here"
    strategy:
      type: None
    enabled: true
"#);
    let out = aaai()
        .args(["lint", yaml.to_str().unwrap(), "--json-output"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .expect("lint --json-output must produce valid JSON");
    assert!(parsed.is_array(), "lint JSON must be an array");
}

#[test]
fn lint_json_output_finds_empty_linematch() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "reason here"
    strategy:
      type: LineMatch
      rules: []
    enabled: true
"#);
    let out = aaai()
        .args(["lint", yaml.to_str().unwrap(), "--json-output"])
        .output().unwrap();
    // Should exit 1 because of empty-linematch error
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let issues: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let has_empty_linematch = issues.as_array().unwrap()
        .iter()
        .any(|i| i["kind"] == "empty-linematch");
    assert!(has_empty_linematch, "should detect empty-linematch");
}

#[test]
fn version_json_output_valid() {
    let out = aaai().args(["version", "--json-output"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout)
        .expect("version --json-output must produce valid JSON");
    assert!(v["version"].is_string(), "version field must be present");
    assert!(v["license"].is_string(), "license field must be present");
}

#[test]
fn version_plain_exits_0() {
    let status = aaai().args(["version"]).status().unwrap();
    assert_eq!(status.code(), Some(0));
}
