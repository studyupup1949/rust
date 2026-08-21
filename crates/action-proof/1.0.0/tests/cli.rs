use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn reports_invalid_yaml_manifest() {
    let mut cmd = Command::cargo_bin("action-proof").unwrap();

    cmd.args([
        "--manifest",
        "examples/bad-unquoted-colon.yml",
        "--repo-root",
        ".",
    ])
    .assert()
    .failure()
    .stdout(predicate::str::contains("manifest.yaml"))
    .stdout(predicate::str::contains("invalid"));
}

#[test]
fn checks_valid_example_manifest() {
    let mut cmd = Command::cargo_bin("action-proof").unwrap();

    cmd.args([
        "--manifest",
        "examples/good-action.yml",
        "--repo-root",
        ".",
        "--format",
        "json",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"failed\": 0"));
}

#[test]
fn exposes_version() {
    let mut cmd = Command::cargo_bin("action-proof").unwrap();

    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("action-proof"));
}
