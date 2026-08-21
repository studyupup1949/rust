//! Rust conformance runner (Phase 7).
//!
//! Executes the shared, language-agnostic fixtures in `reference-tests/`
//! against the Rust SDK:
//!   - all canonicalization vectors (byte-for-byte + SHA-256)
//!   - hashchain_verify tests (raw-JSON chain verification per C-3)
//!   - canonical_hash tests
//!
//! Test types not implemented by this runner (schema_validation,
//! lifecycle_order, witness_verify — Python-runner-first) are counted and
//! skipped, mirroring G-3's ignore-unknown discipline.

use aae::hashchain::{canonicalize, verify_chain_values, HASH_PREFIX};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = <repo>/sdks/rust
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn load_jsonl(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .expect("read jsonl")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse event"))
        .collect()
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{HASH_PREFIX}{:x}", hasher.finalize())
}

#[test]
fn canonicalization_vectors() {
    let path = repo_root().join("reference-tests/test-vectors/canonicalization.json");
    let doc: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    let vectors = doc["vectors"].as_array().expect("vectors array");
    assert!(!vectors.is_empty());

    let mut failures = Vec::new();
    for v in vectors {
        let id = v["id"].as_str().unwrap();
        let input = &v["input"];
        let expected_hex = v["expected_canonical_hex"].as_str().unwrap();
        let expected_sha = v["expected_sha256"].as_str().unwrap();

        let canonical = match canonicalize(input) {
            Ok(b) => b,
            Err(e) => {
                failures.push(format!("{id}: canonicalize error: {e}"));
                continue;
            }
        };
        let got_hex = hex::encode(&canonical);
        if got_hex != expected_hex {
            failures.push(format!(
                "{id}: bytes mismatch\n  expected: {expected_hex}\n  got:      {got_hex}"
            ));
            continue;
        }
        let got_sha = sha256_prefixed(&canonical);
        if got_sha != expected_sha {
            failures.push(format!("{id}: sha mismatch: {got_sha} != {expected_sha}"));
        }
    }
    assert!(
        failures.is_empty(),
        "canonicalization vector failures:\n{}",
        failures.join("\n")
    );
    println!(
        "canonicalization: {}/{} vectors passed",
        vectors.len(),
        vectors.len()
    );
}

#[test]
fn conformance_descriptor() {
    let root = repo_root();
    let ref_dir = root.join("reference-tests");
    let doc: Value =
        serde_json::from_str(&fs::read_to_string(ref_dir.join("conformance.json")).unwrap())
            .unwrap();
    let tests = doc["tests"].as_array().expect("tests array");

    let (mut passed, mut skipped) = (0usize, 0usize);
    let mut failures = Vec::new();

    for t in tests {
        let id = t["id"].as_str().unwrap();
        match t["type"].as_str().unwrap() {
            "hashchain_verify" => {
                let fixture = ref_dir.join(t["fixture"].as_str().unwrap());
                let raw = fs::read_to_string(&fixture).unwrap();
                let events: Vec<Value> = raw
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|l| serde_json::from_str(l).unwrap())
                    .collect();
                match (verify_chain_values(&events), t["expect"].as_str()) {
                    (Ok(tip), Some("valid")) => {
                        if let Some(expected_tip) = t["expected_tip"].as_str() {
                            if tip != expected_tip {
                                failures
                                    .push(format!("{id}: tip mismatch: {tip} != {expected_tip}"));
                                continue;
                            }
                        }
                        passed += 1;
                    }
                    (Err(_), Some("invalid")) => passed += 1,
                    (Ok(_), Some("invalid")) => {
                        failures.push(format!("{id}: chain verified but expected invalid"))
                    }
                    (Err(e), _) => failures.push(format!("{id}: verify failed: {e}")),
                    (_, other) => failures.push(format!("{id}: unknown expect {other:?}")),
                }
            }
            "canonical_hash" => {
                let fixture = ref_dir.join(t["fixture"].as_str().unwrap());
                let value: Value =
                    serde_json::from_str(&fs::read_to_string(&fixture).unwrap()).unwrap();
                let expected = t["expected_hash"].as_str().unwrap();
                match canonicalize(&value) {
                    Ok(bytes) => {
                        let got = sha256_prefixed(&bytes);
                        if got == expected {
                            passed += 1;
                        } else {
                            failures.push(format!("{id}: {got} != {expected}"));
                        }
                    }
                    Err(e) => failures.push(format!("{id}: canonicalize error: {e}")),
                }
            }
            "signature_verify" => {
                let events = load_jsonl(&ref_dir.join(t["fixture"].as_str().unwrap()));
                let mut keys = std::collections::HashMap::new();
                if let Some(map) = t["public_keys"].as_object() {
                    for (kid, path) in map {
                        let pem = fs::read_to_string(ref_dir.join(path.as_str().unwrap())).unwrap();
                        keys.insert(kid.clone(), pem);
                    }
                }
                let result = aae::signing::verify_chain_signatures(&events, &keys, true);
                match (result, t["expect"].as_str()) {
                    (Ok(count), Some("valid")) => {
                        if let Some(expected) = t["expected_signed_count"].as_u64() {
                            if count as u64 != expected {
                                failures.push(format!(
                                    "{id}: signed count {count} != expected {expected}"
                                ));
                                continue;
                            }
                        }
                        passed += 1;
                    }
                    (Err(_), Some("invalid")) => passed += 1,
                    (Ok(_), Some("invalid")) => {
                        failures.push(format!("{id}: signatures verified but expected invalid"));
                    }
                    (Err(e), _) => failures.push(format!("{id}: signature verify failed: {e}")),
                    (_, other) => failures.push(format!("{id}: unknown expect {other:?}")),
                }
            }
            // Python-runner-only (schema models) or not-yet-ported test
            // types are skipped, mirroring G-3's ignore-unknown discipline:
            // a new descriptor test type must not break existing runners.
            _other => skipped += 1,
        }
    }

    assert!(
        failures.is_empty(),
        "conformance failures:\n{}",
        failures.join("\n")
    );
    println!(
        "conformance: {passed} passed, {skipped} skipped (schema_validation: Python runner only)"
    );
}
