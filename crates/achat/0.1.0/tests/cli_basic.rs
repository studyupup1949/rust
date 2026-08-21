// SPDX-License-Identifier: Apache-2.0

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_flag() {
    Command::cargo_bin("achat")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("agent-to-agent"));
}

#[test]
fn version_flag() {
    Command::cargo_bin("achat")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("achat"));
}

#[test]
fn help_json_works_without_daemon() {
    Command::cargo_bin("achat")
        .unwrap()
        .arg("help-json")
        .assert()
        .success()
        .stdout(predicate::str::contains("commands"));
}

#[test]
fn unknown_subcommand_fails() {
    Command::cargo_bin("achat")
        .unwrap()
        .arg("nonexistent")
        .assert()
        .failure();
}

#[test]
fn send_without_daemon_fails() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("achat")
        .unwrap()
        .env("HOME", dir.path())
        .args(["--as", "nobody", "send", "@bob", "hello"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("connect"));
}
