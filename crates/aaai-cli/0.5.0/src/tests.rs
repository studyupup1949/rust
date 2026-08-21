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
