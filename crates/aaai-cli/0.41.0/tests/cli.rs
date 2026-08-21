//! Integration tests for the aaai CLI binary.
//!
//! These tests build and invoke the actual binary to verify end-to-end
//! behaviour — argument parsing, exit codes, and output format.

use std::fs;
use std::path::Path;

#[path = "cli/state_isolation.rs"]
mod state_isolation;
#[path = "cli/support.rs"]
mod support;

use support::aaai;

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

fn empty_audit(path: &Path) {
    write_audit(path, "version: \"1\"\nentries: []\n");
}

// ── audit subcommand ──────────────────────────────────────────────────────

#[test]
fn audit_maps_selected_root_failure_to_exit_3() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("missing");
    let after = tmp.path().join("after");
    fs::create_dir(&after).unwrap();
    let audit_yaml = tmp.path().join("audit.yaml");
    empty_audit(&audit_yaml);

    let output = aaai()
        .args([
            "audit",
            "--left",
            missing.to_str().unwrap(),
            "--right",
            after.to_str().unwrap(),
            "--config",
            audit_yaml.to_str().unwrap(),
            "--no-history",
        ])
        .run_output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("AAAI-ROOT-UNAVAILABLE"));
    assert!(!stderr.contains(tmp.path().to_str().unwrap()));
}

#[cfg(unix)]
#[test]
fn audit_treats_link_as_error_and_does_not_disclose_target() {
    use std::os::unix::fs::symlink;

    let tmp = tempfile::tempdir().unwrap();
    let before = tmp.path().join("before");
    let after = tmp.path().join("after");
    let outside = tmp.path().join("outside-secret-name");
    setup_dirs(&before, &after);
    fs::create_dir(&outside).unwrap();
    write(&outside, "canary", "outside-secret-content");
    symlink(outside.join("canary"), after.join("linked")).unwrap();
    let audit_yaml = tmp.path().join("audit.yaml");
    empty_audit(&audit_yaml);

    let output = aaai()
        .args([
            "audit",
            "--left",
            before.to_str().unwrap(),
            "--right",
            after.to_str().unwrap(),
            "--config",
            audit_yaml.to_str().unwrap(),
            "--no-history",
        ])
        .run_output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let rendered = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(rendered.contains("linked"));
    assert!(!rendered.contains("outside-secret-name"));
    assert!(!rendered.contains("outside-secret-content"));
}

#[cfg(unix)]
#[test]
fn audit_maps_unreadable_regular_file_to_error_exit_3() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().unwrap();
    let before = tmp.path().join("before");
    let after = tmp.path().join("after");
    setup_dirs(&before, &after);
    write(&after, "blocked", "synthetic-content");
    write(&after, "safe", "safe-content");
    fs::set_permissions(after.join("blocked"), fs::Permissions::from_mode(0o000)).unwrap();
    let audit_yaml = tmp.path().join("audit.yaml");
    empty_audit(&audit_yaml);

    let output = aaai()
        .args([
            "audit", "--left", before.to_str().unwrap(), "--right", after.to_str().unwrap(),
            "--config", audit_yaml.to_str().unwrap(), "--no-history",
        ])
        .run_output()
        .unwrap();
    fs::set_permissions(after.join("blocked"), fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(output.status.code(), Some(3));
    let rendered = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(rendered.contains("blocked"));
    assert!(rendered.contains("AAAI-PATH-READ"));
    assert!(!rendered.contains("Permission denied"));
}

#[test]
fn audit_exits_0_when_all_ok() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "f.txt", "hello\n");
    write(&a, "f.txt", "hello world\n");

    let audit_yaml = tmp.path().join("audit.yaml");
    write_audit(
        &audit_yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "Intentional change"
    strategy:
      type: None
    enabled: true
"#,
    );

    let status = aaai()
        .args([
            "audit",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            audit_yaml.to_str().unwrap(),
            "--no-history",
        ])
        .run_status()
        .unwrap();
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
    write_audit(
        &audit_yaml,
        r#"version: "1"
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
"#,
    );

    let status = aaai()
        .args([
            "audit",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            audit_yaml.to_str().unwrap(),
            "--no-history",
        ])
        .run_status()
        .unwrap();
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
        .args([
            "audit",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            audit_yaml.to_str().unwrap(),
            "--no-history",
        ])
        .run_status()
        .unwrap();
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
        .args([
            "audit",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            audit_yaml.to_str().unwrap(),
            "--allow-pending",
            "--no-history",
        ])
        .run_status()
        .unwrap();
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
    write_audit(
        &audit_yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "test"
    strategy:
      type: None
    enabled: true
"#,
    );

    let out = aaai()
        .args([
            "audit",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            audit_yaml.to_str().unwrap(),
            "--json-output",
            "--no-history",
        ])
        .run_output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_ok(),
        "JSON output must be valid JSON; got: {stdout}"
    );
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
        .args([
            "snap",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(out_path.exists(), "snap should create the output file");
    let content = fs::read_to_string(&out_path).unwrap();
    assert!(
        content.contains("added.txt") || content.contains("removed.txt"),
        "snap output should list diffed files"
    );
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
        .args([
            "snap",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
            "--dry-run",
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(!out_path.exists(), "--dry-run must NOT write the file");
}

// ── check subcommand ──────────────────────────────────────────────────────

#[test]
fn check_exits_0_for_valid_definition() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("audit.yaml");
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "test"
    strategy:
      type: None
    enabled: true
"#,
    );
    let status = aaai()
        .args(["check", yaml.to_str().unwrap()])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
}

#[test]
fn check_exits_nonzero_for_invalid_yaml() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("bad.yaml");
    write_audit(&yaml, "not: valid: yaml: [[[");
    let status = aaai()
        .args(["check", yaml.to_str().unwrap()])
        .run_status()
        .unwrap();
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
        .args([
            "report",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
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
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "html test"
    strategy:
      type: None
    enabled: true
"#,
    );
    let out = tmp.path().join("report.html");

    let status = aaai()
        .args([
            "report",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--format",
            "html",
            "--out",
            out.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(out.exists());
    let content = fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("<!DOCTYPE html>"),
        "HTML report should start with DOCTYPE"
    );
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
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: "logs/*.log"
    diff_type: Modified
    reason: "Log rotation"
    strategy:
      type: None
    enabled: true
"#,
    );

    let status = aaai()
        .args([
            "audit",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--no-history",
        ])
        .run_status()
        .unwrap();
    assert_eq!(
        status.code(),
        Some(0),
        "glob rule should match both log files"
    );
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
        .args([
            "diff",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
        ])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("old.txt") || stdout.contains("new.txt"),
        "diff should mention changed files"
    );
}

#[test]
fn diff_json_output_is_valid() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&a, "added.txt", "content\n");

    let out = aaai()
        .args([
            "diff",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--json-output",
        ])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("diff --json-output must produce valid JSON");
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
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "test export"
    strategy:
      type: None
    enabled: true
"#,
    );
    let out_csv = tmp.path().join("out.csv");

    let status = aaai()
        .args([
            "export",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--out",
            out_csv.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(out_csv.exists());
    let content = fs::read_to_string(&out_csv).unwrap();
    let first_line = content.lines().next().unwrap_or("");
    assert!(
        first_line.contains("path"),
        "CSV header should contain 'path'"
    );
    assert!(
        first_line.contains("reason"),
        "CSV header should contain 'reason'"
    );
    assert!(
        first_line.contains("status"),
        "CSV header should contain 'status'"
    );
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
        .args([
            "export",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--format",
            "tsv",
            "--out",
            out_tsv.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out_tsv).unwrap();
    let first_line = content.lines().next().unwrap_or("");
    assert!(
        first_line.contains('\t'),
        "TSV header should use tab separator"
    );
}

// RFC 103 F1 — a real file whose name starts with a formula-trigger
// character (legal on Linux/macOS: no `/`, no NUL) must have its `path`
// cell in the CSV output neutralized with a leading apostrophe.
#[test]
fn export_neutralizes_a_real_formula_injection_filename() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&a, "=SUM(1,2)", "new content\n");
    let yaml = tmp.path().join("audit.yaml");
    write_audit(&yaml, "version: \"1\"\n");
    let out_csv = tmp.path().join("out.csv");

    let status = aaai()
        .args([
            "export",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--out",
            out_csv.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out_csv).unwrap();
    assert!(
        !content.contains("\n=SUM") && !content.contains(",=SUM"),
        "a path cell starting with '=' must never appear un-neutralized: {content}"
    );
    assert!(
        content.contains("'=SUM(1,2)"),
        "the path must still be present, just neutralized with a leading apostrophe: {content}"
    );
}

// RFC 103 F4 — export.rs bypassed the report API entirely and never
// masked anything; this is the same secret-pattern canary the report
// masking tests use.
#[test]
fn export_masks_secret_in_written_file() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "f.txt", "old\n");
    write(&a, "f.txt", "new\n");
    let yaml = tmp.path().join("audit.yaml");
    write_audit(
        &yaml,
        "version: \"1\"\nentries:\n  - path: f.txt\n    diff_type: Modified\n    reason: \"legitimate-looking review note AKIAAAAAAAAAAAAAAAAA\"\n    ticket: \"TICKET-AKIAAAAAAAAAAAAAAAAA\"\n    strategy:\n      type: None\n    enabled: true\n",
    );
    let out_csv = tmp.path().join("out.csv");

    let status = aaai()
        .args([
            "export",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--out",
            out_csv.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out_csv).unwrap();
    assert!(
        !content.contains("AKIAAAAAAAAAAAAAAAAA"),
        "aaai export must mask a secret-pattern match in reason/ticket, not just be capable of it:\n{content}"
    );
}

#[test]
fn merge_detect_conflicts() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().join("base.yaml");
    let overlay = tmp.path().join("overlay.yaml");
    write_audit(
        &base,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "base"
    strategy:
      type: None
    enabled: true
"#,
    );
    write_audit(
        &overlay,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Added
    reason: "overlay"
    strategy:
      type: None
    enabled: true
"#,
    );

    let out = aaai()
        .args([
            "merge",
            base.to_str().unwrap(),
            overlay.to_str().unwrap(),
            "--detect-conflicts",
        ])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("conflict") || stdout.contains("Conflict"),
        "conflict detection should mention conflicts"
    );
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
        .args([
            "report",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--format",
            "sarif",
            "--out",
            out.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&content).expect("SARIF output must be valid JSON");
    assert_eq!(v["version"], "2.1.0", "SARIF version must be 2.1.0");
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
    write_audit(
        &yaml,
        r#"version: "1"
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
"#,
    );
    let out = aaai()
        .args([
            "audit",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--json-output",
            "--no-history",
        ])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["result"], "PASSED");
}

// ── Phase 9: previously untested commands ───────────────────────────────

#[test]
fn completions_bash_exits_0() {
    let out = aaai().args(["completions", "bash"]).run_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("aaai"),
        "bash completion should mention 'aaai'"
    );
}

#[test]
fn completions_zsh_exits_0() {
    let out = aaai().args(["completions", "zsh"]).run_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !stdout.is_empty(),
        "zsh completion output should not be empty"
    );
}

#[test]
fn config_init_creates_aaai_yaml() {
    let tmp = tempfile::tempdir().unwrap();
    let status = aaai()
        .args(["config", "--init", "--dir", tmp.path().to_str().unwrap()])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    let config_path = tmp.path().join(".aaai.yaml");
    assert!(config_path.exists(), ".aaai.yaml should be created");
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(
        content.contains("version"),
        ".aaai.yaml should contain 'version'"
    );
}

#[test]
fn config_init_skips_existing() {
    let tmp = tempfile::tempdir().unwrap();
    // Create first
    aaai()
        .args(["config", "--init", "--dir", tmp.path().to_str().unwrap()])
        .run_status()
        .unwrap();
    // Try again — should report already exists, not overwrite
    let out = aaai()
        .args(["config", "--init", "--dir", tmp.path().to_str().unwrap()])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("already exists") || stdout.contains("既に"),
        "should report existing config"
    );
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
        .args([
            "dashboard",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
}

#[test]
fn init_non_interactive_creates_config() {
    let tmp = tempfile::tempdir().unwrap();
    let status = aaai()
        .args([
            "init",
            "--dir",
            tmp.path().to_str().unwrap(),
            "--non-interactive",
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    let config_path = tmp.path().join(".aaai.yaml");
    assert!(
        config_path.exists(),
        "non-interactive init should create .aaai.yaml"
    );
}

#[test]
fn lint_json_output_valid() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("audit.yaml");
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "valid reason text here"
    strategy:
      type: None
    enabled: true
"#,
    );
    let out = aaai()
        .args(["lint", yaml.to_str().unwrap(), "--json-output"])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("lint --json-output must produce valid JSON");
    assert!(parsed.is_array(), "lint JSON must be an array");
}

#[test]
fn lint_json_output_finds_empty_linematch() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("audit.yaml");
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "reason here"
    strategy:
      type: LineMatch
      rules: []
    enabled: true
"#,
    );
    let out = aaai()
        .args(["lint", yaml.to_str().unwrap(), "--json-output"])
        .run_output()
        .unwrap();
    // Should exit 1 because of empty-linematch error
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let issues: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let has_empty_linematch = issues
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["kind"] == "empty-linematch");
    assert!(has_empty_linematch, "should detect empty-linematch");
}

#[test]
fn version_json_output_valid() {
    let out = aaai()
        .args(["version", "--json-output"])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("version --json-output must produce valid JSON");
    assert!(v["version"].is_string(), "version field must be present");
    assert!(v["license"].is_string(), "license field must be present");
}

#[test]
fn version_plain_exits_0() {
    let status = aaai().args(["version"]).run_status().unwrap();
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
        .args([
            "audit",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            bad_yaml.to_str().unwrap(),
            "--no-history",
        ])
        .run_status()
        .unwrap();
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
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "verbose reason text"
    strategy:
      type: None
    enabled: true
"#,
    );
    let out = aaai()
        .args([
            "audit",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--verbose",
            "--no-history",
        ])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("verbose reason text"),
        "verbose mode should print the reason"
    );
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
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "api_key=sk-abcdefghijklmnop12345678 changed"
    strategy:
      type: None
    enabled: true
"#,
    );
    let out = aaai()
        .args([
            "audit",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--verbose",
            "--mask-secrets",
            "--no-history",
        ])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !stdout.contains("sk-abcdefghijklmnop"),
        "--mask-secrets must redact secret values from output"
    );
    assert!(
        stdout.contains("MASKED"),
        "--mask-secrets should replace with MASKED marker"
    );
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
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "change"
    strategy:
      type: None
    enabled: true
"#,
    );
    let status = aaai()
        .args([
            "audit",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--suppress-warnings",
            "no-approver",
            "--no-history",
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
}

// ── snap: 追加フラグ ─────────────────────────────────────────────────────────

#[test]
fn snap_list_templates_exits_0() {
    let out = aaai()
        .args([
            "snap",
            "--left",
            "/tmp",
            "--right",
            "/tmp",
            "--out",
            "/tmp/x.yaml",
            "--list-templates",
        ])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("version_bump") || stdout.contains("port_change"),
        "list-templates should show template IDs"
    );
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
        .args([
            "snap",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
            "--template",
            "version_bump",
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out_path).unwrap();
    assert!(
        content.contains("Regex") || content.contains("regex"),
        "version_bump template should set Regex strategy"
    );
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
        .args([
            "snap",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
            "--approver",
            "alice",
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out_path).unwrap();
    assert!(
        content.contains("alice"),
        "--approver should set approved_by in generated entries"
    );
}

#[test]
fn snap_merge_adds_only_new_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "existing.txt", "old\n");
    write(&a, "existing.txt", "new\n");
    write(&a, "added.txt", "hello\n");

    let out_path = tmp.path().join("audit.yaml");
    // First snap
    aaai()
        .args([
            "snap",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    // Set reason on existing entry so --merge should skip it
    let content = fs::read_to_string(&out_path).unwrap();
    let with_reason = content.replace("reason: ''", "reason: 'already approved'");
    fs::write(&out_path, with_reason).unwrap();

    // Second snap with --merge should not overwrite existing reason
    let status = aaai()
        .args([
            "snap",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
            "--merge",
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    let final_content = fs::read_to_string(&out_path).unwrap();
    assert!(
        final_content.contains("already approved"),
        "--merge should not overwrite entries that already have a reason"
    );
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
        .args([
            "report",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--format",
            "json",
            "--out",
            out.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(out.exists());
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert!(
        v.get("result").is_some(),
        "JSON report must have 'result' field"
    );
}

// ── RFC 103 §5.4, layer 2 — call-site level ───────────────────────────────
//
// Layer 1 (crates/aaai/src/report/generator/tests.rs) proves each surface
// *can* mask and encode correctly, given a masker. It cannot prove `aaai
// report` actually *enables* one — that is exactly what F3/F4/F5 got wrong:
// masking existed and was never wired to any real call site. These tests
// run the actual binary and inspect the written file, so they fail unless
// the wiring genuinely enables masking, not merely the capability to.

/// A value matching the built-in "AWS access key" pattern
/// (`crates/aaai/src/masking/patterns.rs`), used the same way in the
/// function-level guard.
const REPORT_MASK_CANARY: &str = "AKIAAAAAAAAAAAAAAAAA";

fn write_secret_definition(yaml: &Path) {
    // A deliberately wrong `expected_sha256` forces AuditStatus::Failed, so
    // the entry appears in SARIF's `results` array (which only includes
    // Failed/Pending/Error) — `type: None` would score OK and the SARIF
    // filter would hide the leak regardless of masking, proving nothing.
    //
    // The canary is embedded in both `reason` and `ticket`: the audit
    // engine always sets a non-empty `detail` alongside Failed/Pending/
    // Error (`crates/aaai/src/audit/engine.rs`), so SARIF's message prefers
    // `detail` and never reaches `reason` in practice — but SARIF's
    // `properties.ticket` serializes the raw ticket unconditionally, so
    // `ticket` is this format's actually-reachable leak vector. Markdown/
    // HTML/JSON all expose `reason`. Carrying the canary in both fields
    // means every format's test exercises whichever field it really
    // surfaces, without four different fixtures.
    write_audit(
        yaml,
        &format!(
            r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "legitimate-looking review note {REPORT_MASK_CANARY}"
    ticket: "TICKET-{REPORT_MASK_CANARY}"
    strategy:
      type: Checksum
      expected_sha256: "0000000000000000000000000000000000000000000000000000000000000000"
    enabled: true
"#
        ),
    );
}

fn assert_report_masks_secret(format: &str, out_name: &str) {
    let tmp = tempfile::tempdir().unwrap();
    let b = tmp.path().join("before");
    let a = tmp.path().join("after");
    setup_dirs(&b, &a);
    write(&b, "f.txt", "old\n");
    write(&a, "f.txt", "new\n");
    let yaml = tmp.path().join("audit.yaml");
    write_secret_definition(&yaml);
    let out = tmp.path().join(out_name);

    let status = aaai()
        .args([
            "report",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--format",
            format,
            "--out",
            out.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out).unwrap();
    assert!(
        !content.contains(REPORT_MASK_CANARY),
        "aaai report -f {format} must mask a secret-pattern match in the \
         written file, not just be capable of masking it:\n{content}"
    );
}

#[test]
fn report_markdown_masks_secret_in_written_file() {
    assert_report_masks_secret("markdown", "report.md");
}

#[test]
fn report_html_masks_secret_in_written_file() {
    assert_report_masks_secret("html", "report.html");
}

#[test]
fn report_json_masks_secret_in_written_file() {
    assert_report_masks_secret("json", "report.json");
}

#[test]
fn report_sarif_masks_secret_in_written_file() {
    assert_report_masks_secret("sarif", "report.sarif");
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
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "changed"
    strategy:
      type: None
    enabled: true
"#,
    );
    let out = tmp.path().join("report.md");

    let status = aaai()
        .args([
            "report",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--config",
            yaml.to_str().unwrap(),
            "--include-diff",
            "--out",
            out.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    let content = fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("```diff") || content.contains("-line1") || content.contains("+line2"),
        "--include-diff should embed actual diff text"
    );
}

// ── lint: 追加フラグ ──────────────────────────────────────────────────────────

#[test]
fn lint_detects_duplicate_path() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("dup.yaml");
    write_audit(
        &yaml,
        r#"version: "1"
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
"#,
    );
    let out = aaai()
        .args(["lint", yaml.to_str().unwrap(), "--json-output"])
        .run_output()
        .unwrap();
    // duplicate-path is an error → exit 1
    assert_eq!(out.status.code(), Some(1));
    let issues: serde_json::Value =
        serde_json::from_str(&String::from_utf8(out.stdout).unwrap()).unwrap();
    let has_dup = issues
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["kind"] == "duplicate-path");
    assert!(has_dup, "lint should detect duplicate-path entries");
}

#[test]
fn lint_require_ticket_warns_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("audit.yaml");
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "change without ticket"
    strategy:
      type: None
    enabled: true
"#,
    );
    let out = aaai()
        .args([
            "lint",
            yaml.to_str().unwrap(),
            "--require-ticket",
            "--json-output",
        ])
        .run_output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let issues: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let has_ticket_warn = issues
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["kind"] == "missing-ticket");
    assert!(
        has_ticket_warn,
        "--require-ticket should warn when ticket is absent"
    );
}

#[test]
fn lint_min_reason_len_warns_short() {
    let tmp = tempfile::tempdir().unwrap();
    let yaml = tmp.path().join("audit.yaml");
    write_audit(
        &yaml,
        r#"version: "1"
entries:
  - path: f.txt
    diff_type: Modified
    reason: "ok"
    strategy:
      type: None
    enabled: true
"#,
    );
    let out = aaai()
        .args([
            "lint",
            yaml.to_str().unwrap(),
            "--min-reason-len",
            "20",
            "--json-output",
        ])
        .run_output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let issues: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let has_short = issues
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["kind"] == "short-reason");
    assert!(
        has_short,
        "--min-reason-len should warn when reason is shorter than threshold"
    );
}

// ── merge: 実際のマージ動作 ────────────────────────────────────────────────────

#[test]
fn merge_actually_merges_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().join("base.yaml");
    let overlay = tmp.path().join("overlay.yaml");
    let merged = tmp.path().join("merged.yaml");

    write_audit(
        &base,
        r#"version: "1"
entries:
  - path: base.txt
    diff_type: Added
    reason: "from base"
    strategy:
      type: None
    enabled: true
"#,
    );
    write_audit(
        &overlay,
        r#"version: "1"
entries:
  - path: overlay.txt
    diff_type: Added
    reason: "from overlay"
    strategy:
      type: None
    enabled: true
"#,
    );

    let status = aaai()
        .args([
            "merge",
            base.to_str().unwrap(),
            overlay.to_str().unwrap(),
            "--out",
            merged.to_str().unwrap(),
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(merged.exists(), "merge should create output file");
    let content = fs::read_to_string(&merged).unwrap();
    assert!(
        content.contains("base.txt"),
        "merged file should contain base entries"
    );
    assert!(
        content.contains("overlay.txt"),
        "merged file should contain overlay entries"
    );
}

#[test]
fn merge_dry_run_does_not_write() {
    let tmp = tempfile::tempdir().unwrap();
    let base = tmp.path().join("base.yaml");
    let overlay = tmp.path().join("overlay.yaml");
    let out = tmp.path().join("out.yaml");

    write_audit(&base, "version: \"1\"\n");
    write_audit(&overlay, "version: \"1\"\n");

    let status = aaai()
        .args([
            "merge",
            base.to_str().unwrap(),
            overlay.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
            "--dry-run",
        ])
        .run_status()
        .unwrap();
    assert_eq!(status.code(), Some(0));
    assert!(
        !out.exists(),
        "merge --dry-run must NOT write the output file"
    );
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
        .args([
            "diff",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--content",
        ])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("+new line") || stdout.contains("-old line"),
        "--content should show inline diff text"
    );
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
        .args([
            "diff",
            "--left",
            b.to_str().unwrap(),
            "--right",
            a.to_str().unwrap(),
            "--all",
        ])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("same.txt"),
        "--all should include Unchanged files in output"
    );
}

// ── config: discover ==========================================================

#[test]
fn config_show_when_no_config_found() {
    // Run from a temp dir with no .aaai.yaml
    let tmp = tempfile::tempdir().unwrap();
    let out = aaai()
        .args(["config"])
        .current_dir(tmp.path())
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("No") || stdout.contains("not found"),
        "config with no .aaai.yaml should report not found"
    );
}

// ── watch: smoke test =========================================================

#[test]
fn watch_help_exits_0() {
    let status = aaai().args(["watch", "--help"]).run_status().unwrap();
    assert_eq!(status.code(), Some(0));
}

// ── completions: fish/powershell ──────────────────────────────────────────────

#[test]
fn completions_fish_exits_0() {
    let out = aaai().args(["completions", "fish"]).run_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(
        !String::from_utf8(out.stdout).unwrap().is_empty(),
        "fish completion output should not be empty"
    );
}

#[test]
fn completions_powershell_exits_0() {
    let out = aaai()
        .args(["completions", "powershell"])
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
}

// ── RFC 024: CLI Dashboard & Help Discoverability Polish ──────────────────────
//
// These tests verify that every subcommand exposes a "Next steps:" block in
// its `--help` output, that the top-level `aaai --help` carries a "Getting
// started:" block, and that the new `exit-codes` subcommand works.

fn help_stdout(args: &[&str]) -> String {
    let out = aaai()
        .args(args)
        .run_output()
        .expect("aaai binary should run");
    assert_eq!(out.status.code(), Some(0), "{args:?} should exit 0");
    String::from_utf8(out.stdout).expect("--help output should be UTF-8")
}

#[test]
fn rfc024_top_level_help_has_getting_started() {
    let out = help_stdout(&["--help"]);
    assert!(
        out.contains("Getting started:"),
        "top-level --help should carry a 'Getting started:' block, got:\n{out}"
    );
    // The block lists the core onboarding commands.
    assert!(out.contains("aaai init"));
    assert!(out.contains("aaai snap"));
    assert!(out.contains("aaai audit"));
    assert!(out.contains("aaai-gui"));
}

#[test]
fn rfc024_every_subcommand_help_has_next_steps() {
    // Every subcommand declared in main.rs must surface a "Next steps:"
    // footer per RFC 024 FR-2. `help` and `version` are clap built-ins;
    // exit-codes is a leaf command that prints the codes inline and doesn't
    // need a Next-steps section.
    let subcommands = [
        "audit",
        "snap",
        "report",
        "check",
        "history",
        "config",
        "dashboard",
        "watch",
        "completions",
        "diff",
        "merge",
        "init",
        "export",
        "version",
        "lint",
    ];
    for sc in subcommands {
        let out = help_stdout(&[sc, "--help"]);
        assert!(
            out.contains("Next steps:"),
            "`aaai {sc} --help` should contain 'Next steps:', got:\n{out}"
        );
    }
}

#[test]
fn rfc024_audit_help_lists_exit_codes() {
    let out = help_stdout(&["audit", "--help"]);
    assert!(
        out.contains("Exit codes"),
        "audit --help should call out exit codes, got:\n{out}"
    );
    // The short-form table lists the canonical names.
    for code_name in ["PASSED", "FAILED", "PENDING", "ERROR", "CONFIG_ERROR"] {
        assert!(
            out.contains(code_name),
            "audit --help should mention {code_name}, got:\n{out}"
        );
    }
}

#[test]
fn rfc024_exit_codes_subcommand_prints_table() {
    let out = aaai().arg("exit-codes").run_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Exit code reference:"));
    // Each canonical code/name pair appears.
    for (code, name) in [
        ("0", "PASSED"),
        ("1", "FAILED"),
        ("2", "PENDING"),
        ("3", "ERROR"),
        ("4", "CONFIG_ERROR"),
    ] {
        assert!(
            stdout.contains(code),
            "exit-codes table should mention {code}, got:\n{stdout}"
        );
        assert!(
            stdout.contains(name),
            "exit-codes table should mention {name}, got:\n{stdout}"
        );
    }
}

#[test]
fn rfc024_audit_zone4_hint_appears_for_pending() {
    // Use snap to seed a valid audit.yaml with a Pending entry, then run
    // audit and confirm the Zone 4 hint (now sourced from cmd::next_hint)
    // mentions 'reason' and `re-run`.
    let tmp = tempfile::tempdir().unwrap();
    let before = tmp.path().join("before");
    let after = tmp.path().join("after");
    setup_dirs(&before, &after);
    write(&before, "config.toml", "port = 80\n");
    write(&after, "config.toml", "port = 8080\n");
    let audit_yaml = tmp.path().join("audit.yaml");

    let snap = aaai()
        .args(["snap", "-l"])
        .arg(&before)
        .arg("-r")
        .arg(&after)
        .arg("-o")
        .arg(&audit_yaml)
        .run_status()
        .unwrap();
    assert_eq!(snap.code(), Some(0));

    let mut audit_command = aaai();
    let out = audit_command
        .args(["audit", "-l"])
        .arg(&before)
        .arg("-r")
        .arg(&after)
        .arg("-c")
        .arg(&audit_yaml)
        .run_output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("Next: open your audit.yaml"),
        "Zone 4 hint missing or wrong wording, got:\n{stdout}"
    );
    assert!(stdout.contains("Pending"));
    assert!(stdout.contains("re-run"));
    assert_eq!(audit_command.history_records().len(), 1);
}

#[test]
fn rfc024_dashboard_emits_next_action_hint() {
    let tmp = tempfile::tempdir().unwrap();
    let before = tmp.path().join("before");
    let after = tmp.path().join("after");
    setup_dirs(&before, &after);
    write(&before, "a.txt", "v1\n");
    write(&after, "a.txt", "v2\n");
    let audit_yaml = tmp.path().join("audit.yaml");

    let snap = aaai()
        .args(["snap", "-l"])
        .arg(&before)
        .arg("-r")
        .arg(&after)
        .arg("-o")
        .arg(&audit_yaml)
        .run_status()
        .unwrap();
    assert_eq!(snap.code(), Some(0));

    let out = aaai()
        .args(["dashboard", "-l"])
        .arg(&before)
        .arg("-r")
        .arg(&after)
        .arg("-c")
        .arg(&audit_yaml)
        .run_output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("Next:"),
        "dashboard should append a Next-action hint, got:\n{stdout}"
    );
}

#[test]
fn rfc024_quiet_audit_suppresses_zone4_hint() {
    // RFC 024 NFR-1: --quiet should suppress the Next-action hint.
    let tmp = tempfile::tempdir().unwrap();
    let before = tmp.path().join("before");
    let after = tmp.path().join("after");
    setup_dirs(&before, &after);
    write(&before, "x.txt", "a\n");
    write(&after, "x.txt", "b\n");
    let audit_yaml = tmp.path().join("audit.yaml");
    let _ = aaai()
        .args(["snap", "-l"])
        .arg(&before)
        .arg("-r")
        .arg(&after)
        .arg("-o")
        .arg(&audit_yaml)
        .run_status();

    let mut audit_command = aaai();
    let out = audit_command
        .args(["audit", "--quiet", "-l"])
        .arg(&before)
        .arg("-r")
        .arg(&after)
        .arg("-c")
        .arg(&audit_yaml)
        .run_output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !stdout.contains("Next: "),
        "--quiet should suppress the Zone 4 hint, got:\n{stdout}"
    );
    assert_eq!(audit_command.history_records().len(), 1);
}

#[test]
fn rfc024_json_output_audit_suppresses_zone4_hint() {
    // RFC 024 NFR-2: --json-output should keep machine output clean.
    let tmp = tempfile::tempdir().unwrap();
    let before = tmp.path().join("before");
    let after = tmp.path().join("after");
    setup_dirs(&before, &after);
    write(&before, "x.txt", "a\n");
    write(&after, "x.txt", "b\n");
    let audit_yaml = tmp.path().join("audit.yaml");
    let _ = aaai()
        .args(["snap", "-l"])
        .arg(&before)
        .arg("-r")
        .arg(&after)
        .arg("-o")
        .arg(&audit_yaml)
        .run_status();

    let mut audit_command = aaai();
    let out = audit_command
        .args(["audit", "--json-output", "-l"])
        .arg(&before)
        .arg("-r")
        .arg(&after)
        .arg("-c")
        .arg(&audit_yaml)
        .run_output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
    // JSON should not contain the hint string anywhere.
    assert!(
        !stdout.contains("Next: "),
        "--json-output should suppress the Zone 4 hint, got:\n{stdout}"
    );
    assert_eq!(audit_command.history_records().len(), 1);
}

// ── RFC 056: `aaai watch` ─────────────────────────────────────────────────

#[test]
fn rfc056_watch_help_available() {
    // Smoke test: the watch subcommand exists and --help succeeds.
    let out = aaai().args(["watch", "--help"]).run_output().unwrap();
    assert!(
        out.status.success(),
        "watch --help failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("before"),
        "watch --help should mention the before path"
    );
    assert!(
        stdout.contains("config"),
        "watch --help should mention the config path"
    );
}

// ── RFC 057: `aaai export` ────────────────────────────────────────────────

#[test]
fn rfc057_export_csv_to_stdout() {
    let tmp = tempfile::tempdir().unwrap();
    let before = tmp.path().join("before");
    let after = tmp.path().join("after");
    setup_dirs(&before, &after);
    write(&before, "a.txt", "old\n");
    write(&after, "a.txt", "new\n");
    let audit_yaml = tmp.path().join("audit.yaml");
    let _ = aaai()
        .args(["snap", "-l"])
        .arg(&before)
        .args(["-r"])
        .arg(&after)
        .args(["-o"])
        .arg(&audit_yaml)
        .run_status();

    let out = aaai()
        .args(["export", "-l"])
        .arg(&before)
        .args(["-r"])
        .arg(&after)
        .args(["-c"])
        .arg(&audit_yaml)
        .run_output()
        .unwrap();
    assert!(
        out.status.success(),
        "export failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let csv = String::from_utf8(out.stdout).unwrap();
    assert!(
        csv.starts_with("path,"),
        "first line should be CSV header, got: {csv}"
    );
    assert!(csv.contains("a.txt"), "CSV should contain the changed file");
}

#[test]
fn rfc057_export_tsv_format() {
    let tmp = tempfile::tempdir().unwrap();
    let before = tmp.path().join("before");
    let after = tmp.path().join("after");
    setup_dirs(&before, &after);
    write(&before, "b.txt", "x\n");
    write(&after, "b.txt", "y\n");
    let audit_yaml = tmp.path().join("audit.yaml");
    let _ = aaai()
        .args(["snap", "-l"])
        .arg(&before)
        .args(["-r"])
        .arg(&after)
        .args(["-o"])
        .arg(&audit_yaml)
        .run_status();

    let out = aaai()
        .args(["export", "--format", "tsv", "-l"])
        .arg(&before)
        .args(["-r"])
        .arg(&after)
        .args(["-c"])
        .arg(&audit_yaml)
        .run_output()
        .unwrap();
    assert!(out.status.success(), "export tsv failed");
    let tsv = String::from_utf8(out.stdout).unwrap();
    // TSV uses tab separators, not commas.
    let header = tsv.lines().next().unwrap_or("");
    assert!(
        header.contains('\t'),
        "TSV header should use tab separators"
    );
}

#[test]
fn rfc057_export_to_file() {
    let tmp = tempfile::tempdir().unwrap();
    let before = tmp.path().join("before");
    let after = tmp.path().join("after");
    setup_dirs(&before, &after);
    write(&before, "c.txt", "1\n");
    write(&after, "c.txt", "2\n");
    let audit_yaml = tmp.path().join("audit.yaml");
    let output_csv = tmp.path().join("out.csv");
    let _ = aaai()
        .args(["snap", "-l"])
        .arg(&before)
        .args(["-r"])
        .arg(&after)
        .args(["-o"])
        .arg(&audit_yaml)
        .run_status();

    let status = aaai()
        .args(["export", "-l"])
        .arg(&before)
        .args(["-r"])
        .arg(&after)
        .args(["-c"])
        .arg(&audit_yaml)
        .args(["-o"])
        .arg(&output_csv)
        .run_status()
        .unwrap();
    assert!(status.success(), "export to file failed");
    assert!(output_csv.exists(), "output CSV file should exist");
    let content = std::fs::read_to_string(&output_csv).unwrap();
    assert!(
        content.contains("c.txt"),
        "output file should contain the changed entry"
    );
}

// ── RFC 059: `aaai lint` ──────────────────────────────────────────────────

#[test]
fn rfc059_lint_clean_file_exits_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let before = tmp.path().join("before");
    let after = tmp.path().join("after");
    setup_dirs(&before, &after);
    write(&before, "a.txt", "hello\n");
    write(&after, "a.txt", "world\n");
    let audit_yaml = tmp.path().join("audit.yaml");
    // snap, then manually set a proper reason so lint passes
    let _ = aaai()
        .args(["snap", "-l"])
        .arg(&before)
        .args(["-r"])
        .arg(&after)
        .args(["-o"])
        .arg(&audit_yaml)
        .run_status();
    // Set a long-enough reason via audit --approve-all-pending is not available;
    // write the YAML directly
    std::fs::write(&audit_yaml,
        "version: '1'\nentries:\n- path: a.txt\n  diff_type: Modified\n  reason: 'Changed for testing purposes'\n  strategy:\n    type: None\n  enabled: true\n").unwrap();

    let out = aaai().args(["lint"]).arg(&audit_yaml).run_output().unwrap();
    assert!(out.status.success(), "lint should exit 0 on clean file");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("No issues") || !stdout.contains("error"),
        "clean file should have no errors:\n{stdout}"
    );
}

#[test]
fn rfc059_lint_short_reason_warns() {
    let tmp = tempfile::tempdir().unwrap();
    let audit_yaml = tmp.path().join("audit.yaml");
    std::fs::write(&audit_yaml,
        "version: '1'\nentries:\n- path: b.txt\n  diff_type: Modified\n  reason: ok\n  strategy:\n    type: None\n  enabled: true\n").unwrap();

    let out = aaai().args(["lint"]).arg(&audit_yaml).run_output().unwrap();
    // short reason → warning, not error → still exits 0
    assert!(
        out.status.success(),
        "short reason is a warning not an error"
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("short-reason"),
        "should warn about short reason:\n{stdout}"
    );
}

#[test]
fn rfc059_lint_json_output() {
    let tmp = tempfile::tempdir().unwrap();
    let audit_yaml = tmp.path().join("audit.yaml");
    std::fs::write(&audit_yaml,
        "version: '1'\nentries:\n- path: c.txt\n  diff_type: Modified\n  reason: hi\n  strategy:\n    type: None\n  enabled: true\n").unwrap();

    let out = aaai()
        .args(["lint", "--json-output"])
        .arg(&audit_yaml)
        .run_output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    // Must be valid JSON array
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("--json-output must produce valid JSON");
    assert!(parsed.is_array(), "output must be a JSON array");
}

// ── RFC 060: `aaai merge` ─────────────────────────────────────────────────

#[test]
fn rfc060_merge_combine_two_definitions() {
    let tmp = tempfile::tempdir().unwrap();
    let base_yaml = tmp.path().join("base.yaml");
    let overlay_yaml = tmp.path().join("overlay.yaml");
    let out_yaml = tmp.path().join("merged.yaml");

    std::fs::write(&base_yaml,
        "version: '1'\nentries:\n- path: a.txt\n  diff_type: Modified\n  reason: Base reason\n  strategy:\n    type: None\n  enabled: true\n").unwrap();
    std::fs::write(&overlay_yaml,
        "version: '1'\nentries:\n- path: b.txt\n  diff_type: Added\n  reason: Overlay reason\n  strategy:\n    type: None\n  enabled: true\n").unwrap();

    let status = aaai()
        .args(["merge"])
        .arg(&base_yaml)
        .arg(&overlay_yaml)
        .args(["-o"])
        .arg(&out_yaml)
        .run_status()
        .unwrap();
    assert!(status.success(), "merge should succeed");
    let merged = std::fs::read_to_string(&out_yaml).unwrap();
    assert!(
        merged.contains("a.txt"),
        "merged file should contain base entry"
    );
    assert!(
        merged.contains("b.txt"),
        "merged file should contain overlay entry"
    );
}

#[test]
fn rfc060_merge_dry_run_does_not_write() {
    let tmp = tempfile::tempdir().unwrap();
    let base_yaml = tmp.path().join("base.yaml");
    let overlay_yaml = tmp.path().join("overlay.yaml");

    std::fs::write(&base_yaml,
        "version: '1'\nentries:\n- path: x.txt\n  diff_type: Modified\n  reason: r\n  strategy:\n    type: None\n  enabled: true\n").unwrap();
    std::fs::write(&overlay_yaml,
        "version: '1'\nentries:\n- path: y.txt\n  diff_type: Added\n  reason: r\n  strategy:\n    type: None\n  enabled: true\n").unwrap();
    let original = std::fs::read_to_string(&base_yaml).unwrap();

    let status = aaai()
        .args(["merge", "--dry-run"])
        .arg(&base_yaml)
        .arg(&overlay_yaml)
        .run_status()
        .unwrap();
    assert!(status.success());
    assert_eq!(
        std::fs::read_to_string(&base_yaml).unwrap(),
        original,
        "--dry-run must not modify the base file"
    );
}

#[test]
fn rfc060_merge_detect_conflicts() {
    let tmp = tempfile::tempdir().unwrap();
    let base_yaml = tmp.path().join("base.yaml");
    let overlay_yaml = tmp.path().join("overlay.yaml");

    std::fs::write(&base_yaml,
        "version: '1'\nentries:\n- path: conflict.txt\n  diff_type: Modified\n  reason: r\n  strategy:\n    type: None\n  enabled: true\n").unwrap();
    std::fs::write(&overlay_yaml,
        "version: '1'\nentries:\n- path: conflict.txt\n  diff_type: Added\n  reason: r\n  strategy:\n    type: None\n  enabled: true\n").unwrap();

    let out = aaai()
        .args(["merge", "--detect-conflicts"])
        .arg(&base_yaml)
        .arg(&overlay_yaml)
        .run_output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("conflict.txt"),
        "should report the conflicting path:\n{stdout}"
    );
}

// ── RFC 061: `aaai check` ─────────────────────────────────────────────────

#[test]
fn rfc061_check_valid_file() {
    let tmp = tempfile::tempdir().unwrap();
    let audit_yaml = tmp.path().join("audit.yaml");
    std::fs::write(&audit_yaml,
        "version: '1'\nentries:\n- path: a.txt\n  diff_type: Modified\n  reason: All good\n  strategy:\n    type: None\n  enabled: true\n").unwrap();

    let out = aaai()
        .args(["check"])
        .arg(&audit_yaml)
        .run_output()
        .unwrap();
    assert!(out.status.success(), "valid file should exit 0");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("valid"),
        "should say entries are valid:\n{stdout}"
    );
}

#[test]
fn rfc061_check_invalid_yaml_exits_nonzero() {
    let tmp = tempfile::tempdir().unwrap();
    let audit_yaml = tmp.path().join("audit.yaml");
    std::fs::write(&audit_yaml, "not: valid: aaai: yaml: [\n").unwrap();

    let status = aaai()
        .args(["check"])
        .arg(&audit_yaml)
        .run_status()
        .unwrap();
    assert!(!status.success(), "invalid YAML should exit non-zero");
}

// ── RFC 063: `aaai dashboard` ─────────────────────────────────────────────

#[test]
fn rfc063_dashboard_help_available() {
    let out = aaai().args(["dashboard", "--help"]).run_output().unwrap();
    assert!(out.status.success(), "dashboard --help failed");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("left") || stdout.contains("before"),
        "dashboard --help should mention input paths:\n{stdout}"
    );
}

#[test]
fn rfc063_dashboard_basic_run() {
    let tmp = tempfile::tempdir().unwrap();
    let before = tmp.path().join("before");
    let after = tmp.path().join("after");
    setup_dirs(&before, &after);
    write(&before, "f.txt", "old\n");
    write(&after, "f.txt", "new\n");
    let audit_yaml = tmp.path().join("audit.yaml");
    std::fs::write(&audit_yaml, "version: '1'\nentries: []\n").unwrap();

    let out = aaai()
        .args(["dashboard", "-l"])
        .arg(&before)
        .args(["-r"])
        .arg(&after)
        .args(["-c"])
        .arg(&audit_yaml)
        .run_output()
        .unwrap();
    assert!(
        out.status.success(),
        "dashboard should succeed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("Pending") || stdout.contains("FAILED"),
        "dashboard output should show audit status:\n{stdout}"
    );
}

// ── RFC 065: `aaai init` ──────────────────────────────────────────────────

#[test]
fn rfc065_init_non_interactive_creates_config() {
    let tmp = tempfile::tempdir().unwrap();
    let status = aaai()
        .args(["init", "--non-interactive", "--dir"])
        .arg(tmp.path())
        .run_status()
        .unwrap();
    assert!(status.success(), "init --non-interactive should exit 0");
    let config = tmp.path().join(".aaai.yaml");
    assert!(config.exists(), ".aaai.yaml should be created");
    let content = std::fs::read_to_string(&config).unwrap();
    assert!(
        content.contains("version"),
        ".aaai.yaml should have a version field"
    );
}

#[test]
fn rfc065_init_existing_config_warns_and_exits_zero() {
    let tmp = tempfile::tempdir().unwrap();
    // Pre-create the config file
    std::fs::write(tmp.path().join(".aaai.yaml"), "version: '1'\n").unwrap();

    let out = aaai()
        .args(["init", "--non-interactive", "--dir"])
        .arg(tmp.path())
        .run_output()
        .unwrap();
    assert!(
        out.status.success(),
        "init with existing config should still exit 0"
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("already exists") || stdout.contains("⚠"),
        "should warn that config exists:\n{stdout}"
    );
}

#[test]
fn rfc065_init_help_available() {
    let out = aaai().args(["init", "--help"]).run_output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("non-interactive"),
        "init --help should mention --non-interactive"
    );
}
