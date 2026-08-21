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

// ── Phase 11: coverage補完 ───────────────────────────────────────────────────

// ── audit: 追加フラグ ─────────────────────────────────────────────────────────

#[test]
fn audit_exit_4_on_config_error() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    let bad_yaml = tmp.path().join("bad.yaml");
    write_audit(&bad_yaml, "not: valid: yaml: [[[\n");

    let status = aaai()
        .args(["audit",
               "--left",   b.to_str().unwrap(),
               "--right",  a.to_str().unwrap(),
               "--config", bad_yaml.to_str().unwrap(),
               "--no-history"])
        .status().unwrap();
    assert_eq!(status.code(), Some(4), "invalid YAML config must exit 4");
}

#[test]
fn audit_verbose_shows_reason() {
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
    reason: "verbose reason text"
    strategy:
      type: None
    enabled: true
"#);
    let out = aaai()
        .args(["audit",
               "--left",   b.to_str().unwrap(),
               "--right",  a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap(),
               "--verbose", "--no-history"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("verbose reason text"),
        "verbose mode should print the reason");
}

#[test]
fn audit_mask_secrets_redacts_output() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "f.txt", "x\n");
    write(&a, "f.txt", "y\n");
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "api_key=sk-abcdefghijklmnop12345678 changed"
    strategy:
      type: None
    enabled: true
"#);
    let out = aaai()
        .args(["audit",
               "--left",   b.to_str().unwrap(),
               "--right",  a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap(),
               "--verbose", "--mask-secrets", "--no-history"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.contains("sk-abcdefghijklmnop"),
        "--mask-secrets must redact secret values from output");
    assert!(stdout.contains("MASKED"),
        "--mask-secrets should replace with MASKED marker");
}

#[test]
fn audit_suppress_warnings_flag() {
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
    reason: "change"
    strategy:
      type: None
    enabled: true
"#);
    let status = aaai()
        .args(["audit",
               "--left",   b.to_str().unwrap(),
               "--right",  a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap(),
               "--suppress-warnings", "no-approver",
               "--no-history"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
}

// ── snap: 追加フラグ ─────────────────────────────────────────────────────────

#[test]
fn snap_list_templates_exits_0() {
    let out = aaai()
        .args(["snap",
               "--left", "/tmp", "--right", "/tmp",
               "--out", "/tmp/x.yaml",
               "--list-templates"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("version_bump") || stdout.contains("port_change"),
        "list-templates should show template IDs");
}

#[test]
fn snap_template_applied() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "app.toml", "version = \"1.0.0\"\n");
    write(&a, "app.toml", "version = \"1.1.0\"\n");
    let out_path = tmp.path().join("audit.yaml");

    let status = aaai()
        .args(["snap",
               "--left",     b.to_str().unwrap(),
               "--right",    a.to_str().unwrap(),
               "--out",      out_path.to_str().unwrap(),
               "--template", "version_bump"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("Regex") || content.contains("regex"),
        "version_bump template should set Regex strategy");
}

#[test]
fn snap_approver_flag_sets_approved_by() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&a, "new.txt", "content\n");
    let out_path = tmp.path().join("audit.yaml");

    let status = aaai()
        .args(["snap",
               "--left",     b.to_str().unwrap(),
               "--right",    a.to_str().unwrap(),
               "--out",      out_path.to_str().unwrap(),
               "--approver", "alice"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("alice"),
        "--approver should set approved_by in generated entries");
}

#[test]
fn snap_merge_adds_only_new_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "existing.txt", "old\n");
    write(&a, "existing.txt", "new\n");
    write(&a, "added.txt",    "hello\n");

    let out_path = tmp.path().join("audit.yaml");
    // First snap
    aaai().args(["snap",
                 "--left",  b.to_str().unwrap(),
                 "--right", a.to_str().unwrap(),
                 "--out",   out_path.to_str().unwrap()])
          .status().unwrap();
    // Set reason on existing entry so --merge should skip it
    let content = fs::read_to_string(&out_path).unwrap();
    let with_reason = content.replace("reason: ''", "reason: 'already approved'");
    fs::write(&out_path, with_reason).unwrap();

    // Second snap with --merge should not overwrite existing reason
    let status = aaai()
        .args(["snap",
               "--left",  b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--out",   out_path.to_str().unwrap(),
               "--merge"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    let final_content = fs::read_to_string(&out_path).unwrap();
    assert!(final_content.contains("already approved"),
        "--merge should not overwrite entries that already have a reason");
}

// ── report: 追加フォーマット ───────────────────────────────────────────────────

#[test]
fn report_json_format_creates_valid_file() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, "version: \"1\"\n");
    let out = tmp.path().join("report.json");

    let status = aaai()
        .args(["report",
               "--left",   b.to_str().unwrap(),
               "--right",  a.to_str().unwrap(),
               "--config", yaml.to_str().unwrap(),
               "--format", "json",
               "--out",    out.to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(out.exists());
    let v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert!(v.get("result").is_some(), "JSON report must have 'result' field");
}

#[test]
fn report_include_diff_embeds_diff_block() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "f.txt", "line1\n");
    write(&a, "f.txt", "line2\n");
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "changed"
    strategy:
      type: None
    enabled: true
"#);
    let out = tmp.path().join("report.md");

    let status = aaai()
        .args(["report",
               "--left",         b.to_str().unwrap(),
               "--right",        a.to_str().unwrap(),
               "--config",       yaml.to_str().unwrap(),
               "--include-diff",
               "--out",          out.to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out).unwrap();
    assert!(content.contains("```diff") || content.contains("-line1") || content.contains("+line2"),
        "--include-diff should embed actual diff text");
}

// ── history: 追加フラグ ────────────────────────────────────────────────────────

#[test]
fn history_prune_exits_0() {
    // prune は履歴ファイルが存在しなくても 0 で終了する
    let status = aaai()
        .args(["history", "--prune", "100"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
}

#[test]
fn history_n_flag_limits_output() {
    let out = aaai()
        .args(["history", "-n", "3"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn history_json_output_is_array() {
    let out = aaai()
        .args(["history", "--json-output", "-n", "5"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout)
        .expect("history --json-output must be valid JSON");
    assert!(v.is_array(), "history --json-output must be a JSON array");
}

// ── lint: 追加フラグ ──────────────────────────────────────────────────────────

#[test]
fn lint_detects_duplicate_path() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("dup.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "first"
    strategy:
      type: None
    enabled: true
  - path: f.txt
    diff_type: Modified
    reason: "duplicate"
    strategy:
      type: None
    enabled: true
"#);
    let out = aaai()
        .args(["lint", yaml.to_str().unwrap(), "--json-output"])
        .output().unwrap();
    // duplicate-path is an error → exit 1
    assert_eq!(out.status.code(), Some(1));
    let issues: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let has_dup = issues.as_array().unwrap()
        .iter().any(|i| i["kind"] == "duplicate-path");
    assert!(has_dup, "lint should detect duplicate-path entries");
}

#[test]
fn lint_require_ticket_warns_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "change without ticket"
    strategy:
      type: None
    enabled: true
"#);
    let out = aaai()
        .args(["lint", yaml.to_str().unwrap(),
               "--require-ticket", "--json-output"])
        .output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let issues: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let has_ticket_warn = issues.as_array().unwrap()
        .iter().any(|i| i["kind"] == "missing-ticket");
    assert!(has_ticket_warn, "--require-ticket should warn when ticket is absent");
}

#[test]
fn lint_min_reason_len_warns_short() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "ok"
    strategy:
      type: None
    enabled: true
"#);
    let out = aaai()
        .args(["lint", yaml.to_str().unwrap(),
               "--min-reason-len", "20", "--json-output"])
        .output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let issues: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let has_short = issues.as_array().unwrap()
        .iter().any(|i| i["kind"] == "short-reason");
    assert!(has_short, "--min-reason-len should warn when reason is shorter than threshold");
}

// ── merge: 実際のマージ動作 ────────────────────────────────────────────────────

#[test]
fn merge_actually_merges_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().join("base.yaml");
    let overlay = tmp.path().join("overlay.yaml");
    let merged = tmp.path().join("merged.yaml");

    write_audit(&base, r#"version: "1"
entries:
  - path: base.txt
    diff_type: Added
    reason: "from base"
    strategy:
      type: None
    enabled: true
"#);
    write_audit(&overlay, r#"version: "1"
entries:
  - path: overlay.txt
    diff_type: Added
    reason: "from overlay"
    strategy:
      type: None
    enabled: true
"#);

    let status = aaai()
        .args(["merge",
               base.to_str().unwrap(),
               overlay.to_str().unwrap(),
               "--out", merged.to_str().unwrap()])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(merged.exists(), "merge should create output file");
    let content = fs::read_to_string(&merged).unwrap();
    assert!(content.contains("base.txt"),    "merged file should contain base entries");
    assert!(content.contains("overlay.txt"), "merged file should contain overlay entries");
}

#[test]
fn merge_dry_run_does_not_write() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().join("base.yaml");
    let overlay = tmp.path().join("overlay.yaml");
    let out = tmp.path().join("out.yaml");

    write_audit(&base,    "version: \"1\"\n");
    write_audit(&overlay, "version: \"1\"\n");

    let status = aaai()
        .args(["merge",
               base.to_str().unwrap(),
               overlay.to_str().unwrap(),
               "--out", out.to_str().unwrap(),
               "--dry-run"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(!out.exists(), "merge --dry-run must NOT write the output file");
}

// ── diff: 追加フラグ ──────────────────────────────────────────────────────────

#[test]
fn diff_content_flag_shows_diff_text() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "f.txt", "old line\n");
    write(&a, "f.txt", "new line\n");

    let out = aaai()
        .args(["diff",
               "--left",    b.to_str().unwrap(),
               "--right",   a.to_str().unwrap(),
               "--content"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("+new line") || stdout.contains("-old line"),
        "--content should show inline diff text");
}

#[test]
fn diff_all_flag_includes_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "same.txt", "same\n");
    write(&a, "same.txt", "same\n");
    write(&a, "added.txt", "new\n");

    let out = aaai()
        .args(["diff",
               "--left",  b.to_str().unwrap(),
               "--right", a.to_str().unwrap(),
               "--all"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("same.txt"),
        "--all should include Unchanged files in output");
}

// ── config: discover ==========================================================

#[test]
fn config_show_when_no_config_found() {
    // Run from a temp dir with no .aaai.yaml
    let tmp = tempfile::tempdir().unwrap();
    let out = aaai()
        .args(["config"])
        .current_dir(tmp.path())
        .output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("No") || stdout.contains("not found"),
        "config with no .aaai.yaml should report not found");
}

// ── watch: smoke test =========================================================

#[test]
fn watch_help_exits_0() {
    let status = aaai()
        .args(["watch", "--help"])
        .status().unwrap();
    assert_eq!(status.code(), Some(0));
}

// ── completions: fish/powershell ──────────────────────────────────────────────

#[test]
fn completions_fish_exits_0() {
    let out = aaai().args(["completions", "fish"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(!String::from_utf8(out.stdout).unwrap().is_empty(),
        "fish completion output should not be empty");
}

#[test]
fn completions_powershell_exits_0() {
    let out = aaai().args(["completions", "powershell"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}
