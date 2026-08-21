// SPDX-License-Identifier: Apache-2.0

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use tempfile::TempDir;

struct DaemonFixture {
    child: Child,
    home: PathBuf,
    owned_home: Option<TempDir>, // kept alive if we own it
    name: String,
}

impl DaemonFixture {
    /// Start a daemon with its own isolated HOME.
    fn start(name: &str) -> Self {
        let home = tempfile::tempdir().unwrap();
        let path = home.path().to_path_buf();
        let mut f = Self::start_in(name, &path);
        f.owned_home = Some(home);
        f
    }

    /// Start a daemon in an existing HOME directory (for multi-agent tests).
    fn start_in(name: &str, home: &Path) -> Self {
        let bin = assert_cmd::cargo::cargo_bin("achat");
        let child = std::process::Command::new(&bin)
            .args(["daemon", "--name", name])
            .env("HOME", home)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn daemon");

        let sock = home
            .join(".achat")
            .join("agents")
            .join(name)
            .join("daemon.sock");
        for _ in 0..50 {
            if sock.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert!(sock.exists(), "daemon socket never appeared for {name}");

        Self {
            child,
            home: home.to_path_buf(),
            owned_home: None,
            name: name.into(),
        }
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("achat").unwrap();
        cmd.env("HOME", &self.home);
        cmd.arg("--as").arg(&self.name);
        cmd
    }
}

impl Drop for DaemonFixture {
    fn drop(&mut self) {
        let bin = assert_cmd::cargo::cargo_bin("achat");
        let _ = std::process::Command::new(&bin)
            .args(["--as", &self.name, "down"])
            .env("HOME", &self.home)
            .output();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn daemon_status() {
    let f = DaemonFixture::start("test-status");
    f.cmd()
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-status"));
}

#[test]
fn daemon_ls_empty() {
    let f = DaemonFixture::start("test-ls");
    f.cmd().arg("ls").assert().success();
}

#[test]
fn daemon_send_and_inbox() {
    let home = tempfile::tempdir().unwrap();
    let alice = DaemonFixture::start_in("alice", home.path());
    let bob = DaemonFixture::start_in("bob", home.path());

    // Wait for peer discovery (local registry scan every 2s)
    std::thread::sleep(std::time::Duration::from_secs(3));

    alice
        .cmd()
        .args(["send", "@bob", "integration test"])
        .assert()
        .success();
    std::thread::sleep(std::time::Duration::from_millis(500));

    bob.cmd()
        .args(["inbox", "--pretty"])
        .assert()
        .success()
        .stdout(predicate::str::contains("integration test"));
}
