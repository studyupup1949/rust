use std::path::PathBuf;

use accache::format::{Entry, write_file};
use accache::{Account, Compression, KeyedAccount, Pubkey};
use assert_cmd::Command;
use httpmock::Method::POST;
use httpmock::MockServer;
use predicates::prelude::*;
use serde_json::json;

fn keyed(seed: u8) -> KeyedAccount {
    KeyedAccount::new(
        Pubkey::new_from_array([seed; 32]),
        Account {
            lamports: 100 + seed as u64,
            data: vec![seed; 8],
            owner: Pubkey::new_from_array([0xAA; 32]),
            executable: false,
            rent_epoch: 0,
        },
    )
}

fn write_fixture(path: &PathBuf, kas: &[KeyedAccount]) {
    let entries: Vec<Entry> = kas.iter().map(|k| Entry::from_keyed(k, None)).collect();
    write_file(path, &entries, Compression::Zstd { level: 3 }).unwrap();
}

#[test]
fn cli_list_shows_entries() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("list.acc");
    let kas: Vec<_> = (1..=3).map(keyed).collect();
    write_fixture(&path, &kas);

    let mut cmd = Command::cargo_bin("accache").unwrap();
    cmd.args(["list", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains(kas[0].key.to_string()))
        .stdout(predicate::str::contains(kas[1].key.to_string()))
        .stdout(predicate::str::contains(kas[2].key.to_string()))
        .stdout(predicate::str::contains("3 entries"));
}

#[test]
fn cli_inspect_shows_one_entry() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("inspect.acc");
    let ka = keyed(7);
    write_fixture(&path, &[ka.clone()]);

    let mut cmd = Command::cargo_bin("accache").unwrap();
    cmd.args(["inspect", path.to_str().unwrap(), &ka.key.to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("lamports:   {}", ka.account.lamports)))
        .stdout(predicate::str::contains("data_len:   8"));
}

#[test]
fn cli_inspect_with_hex_data() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("inspect_hex.acc");
    let ka = keyed(0xAB);
    write_fixture(&path, &[ka.clone()]);

    let expected_hex = hex::encode(&ka.account.data);
    Command::cargo_bin("accache")
        .unwrap()
        .args([
            "inspect",
            path.to_str().unwrap(),
            &ka.key.to_string(),
            "--data",
            "hex",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_hex));
}

#[test]
fn cli_inspect_unknown_key_errors() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing.acc");
    write_fixture(&path, &[keyed(1)]);
    let unknown = Pubkey::new_from_array([99; 32]);

    Command::cargo_bin("accache")
        .unwrap()
        .args(["inspect", path.to_str().unwrap(), &unknown.to_string()])
        .assert()
        .failure();
}

#[test]
fn cli_merge_dedups_last_wins() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = dir.path().join("a.acc");
    let p2 = dir.path().join("b.acc");
    let out = dir.path().join("merged.acc");

    let mut shared = keyed(5);
    shared.account.lamports = 100;
    write_fixture(&p1, &[shared.clone(), keyed(1)]);
    shared.account.lamports = 999;
    write_fixture(&p2, &[shared.clone(), keyed(2)]);

    Command::cargo_bin("accache")
        .unwrap()
        .args([
            "merge",
            p1.to_str().unwrap(),
            p2.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    let (_, entries) = accache::format::read_file(&out).unwrap();
    assert_eq!(entries.len(), 3);
    let merged = entries
        .iter()
        .find(|e| e.pubkey() == shared.key)
        .unwrap();
    assert_eq!(merged.lamports, 999, "last file must win on conflict");
}

#[test]
fn cli_export_json_dumps_full_contents() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("src.acc");
    let out = dir.path().join("dump.json");
    write_fixture(&path, &[keyed(3), keyed(4)]);

    Command::cargo_bin("accache")
        .unwrap()
        .args([
            "export",
            path.to_str().unwrap(),
            "--format",
            "json",
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(&out).unwrap()).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 2);
    assert!(v[0]["pubkey"].is_string());
    assert!(v[0]["data_base64"].is_string());
}

#[test]
fn cli_export_test_validator_format_writes_one_file_per_account() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("src.acc");
    let outdir = dir.path().join("validator");
    let kas: Vec<_> = (1..=3).map(keyed).collect();
    write_fixture(&path, &kas);

    Command::cargo_bin("accache")
        .unwrap()
        .args([
            "export",
            path.to_str().unwrap(),
            "--format",
            "test-validator",
            "--out",
            outdir.to_str().unwrap(),
        ])
        .assert()
        .success();

    for ka in &kas {
        let p = outdir.join(format!("{}.json", ka.key));
        assert!(p.exists(), "missing per-account file: {p:?}");
        let v: serde_json::Value = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        // Match the schema solana-test-validator --account expects.
        assert_eq!(v["pubkey"], ka.key.to_string());
        assert!(v["account"]["lamports"].is_u64());
        assert_eq!(v["account"]["data"][1], "base64");
        assert!(v["account"]["owner"].is_string());
    }
}

#[test]
fn cli_fetch_writes_acc_file_via_mock_rpc() {
    let server = MockServer::start();
    let ka = keyed(42);
    let body = json!({
        "jsonrpc": "2.0", "id": 1,
        "result": {
            "context": { "apiVersion": "1.18.0", "slot": 1 },
            "value": [{
                "lamports": ka.account.lamports,
                "data": [base64_encode(&ka.account.data), "base64"],
                "owner": ka.account.owner.to_string(),
                "executable": ka.account.executable,
                "rentEpoch": ka.account.rent_epoch,
                "space": ka.account.data.len(),
            }],
        }
    });
    server.mock(|when, then| {
        when.method(POST).body_contains("\"getMultipleAccounts\"");
        then.status(200).header("content-type", "application/json").json_body(body);
    });

    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("fetched.acc");

    Command::cargo_bin("accache")
        .unwrap()
        .args([
            "fetch",
            &ka.key.to_string(),
            "--rpc",
            &server.base_url(),
            "--out",
            out.to_str().unwrap(),
        ])
        .assert()
        .success();

    let (_, entries) = accache::format::read_file(&out).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].pubkey(), ka.key);
    assert_eq!(entries[0].lamports, ka.account.lamports);
}

fn base64_encode(bytes: &[u8]) -> String {
    const ALPHA: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let chunks = bytes.chunks_exact(3);
    let rem = chunks.remainder().to_vec();
    for c in chunks {
        let n = ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32);
        out.push(ALPHA[(n >> 18 & 0x3F) as usize] as char);
        out.push(ALPHA[(n >> 12 & 0x3F) as usize] as char);
        out.push(ALPHA[(n >> 6 & 0x3F) as usize] as char);
        out.push(ALPHA[(n & 0x3F) as usize] as char);
    }
    match rem.len() {
        0 => {}
        1 => {
            let n = (rem[0] as u32) << 16;
            out.push(ALPHA[(n >> 18 & 0x3F) as usize] as char);
            out.push(ALPHA[(n >> 12 & 0x3F) as usize] as char);
            out.push_str("==");
        }
        2 => {
            let n = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
            out.push(ALPHA[(n >> 18 & 0x3F) as usize] as char);
            out.push(ALPHA[(n >> 12 & 0x3F) as usize] as char);
            out.push(ALPHA[(n >> 6 & 0x3F) as usize] as char);
            out.push('=');
        }
        _ => unreachable!(),
    }
    out
}
