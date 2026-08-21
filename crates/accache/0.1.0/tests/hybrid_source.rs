use std::time::Duration;

use accache::format::{Entry, write_file};
use accache::{Accache, AccacheError, Account, Compression, KeyedAccount, Pubkey, RefreshPolicy};
use httpmock::Method::POST;
use httpmock::MockServer;
use serde_json::{Value, json};

fn keyed(seed: u8) -> KeyedAccount {
    KeyedAccount::new(
        Pubkey::new_from_array([seed; 32]),
        Account {
            lamports: 5_000_000 * (seed as u64 + 1),
            data: vec![seed; 32],
            owner: Pubkey::new_from_array([255 - seed; 32]),
            executable: false,
            rent_epoch: 0,
        },
    )
}

/// Build a base64 string from raw bytes (matches what RPC servers return when
/// they serialize an Account with `UiAccountEncoding::Base64`).
fn b64(bytes: &[u8]) -> String {
    use std::io::Write;
    let mut out = Vec::new();
    let alpha = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let chunks = bytes.chunks_exact(3);
    let rem = chunks.remainder().to_vec();
    for c in chunks {
        let n = ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32);
        out.push(alpha[(n >> 18 & 0x3F) as usize]);
        out.push(alpha[(n >> 12 & 0x3F) as usize]);
        out.push(alpha[(n >> 6 & 0x3F) as usize]);
        out.push(alpha[(n & 0x3F) as usize]);
    }
    match rem.len() {
        0 => {}
        1 => {
            let n = (rem[0] as u32) << 16;
            out.push(alpha[(n >> 18 & 0x3F) as usize]);
            out.push(alpha[(n >> 12 & 0x3F) as usize]);
            out.write_all(b"==").unwrap();
        }
        2 => {
            let n = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
            out.push(alpha[(n >> 18 & 0x3F) as usize]);
            out.push(alpha[(n >> 12 & 0x3F) as usize]);
            out.push(alpha[(n >> 6 & 0x3F) as usize]);
            out.write_all(b"=").unwrap();
        }
        _ => unreachable!(),
    }
    String::from_utf8(out).unwrap()
}

fn rpc_account_value(ka: &KeyedAccount) -> Value {
    json!({
        "lamports": ka.account.lamports,
        "data": [b64(&ka.account.data), "base64"],
        "owner": ka.account.owner.to_string(),
        "executable": ka.account.executable,
        "rentEpoch": ka.account.rent_epoch,
        "space": ka.account.data.len(),
    })
}

/// Mount a single `getAccountInfo` response keyed off the requested pubkey. We let httpmock
/// match on body substring (the pubkey string) so order-independent.
fn mount_get_account<'a>(server: &'a MockServer, ka: &KeyedAccount) -> httpmock::Mock<'a> {
    let pk = ka.key.to_string();
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "context": { "apiVersion": "1.18.0", "slot": 123 },
            "value": rpc_account_value(ka),
        }
    });
    server.mock(|when, then| {
        when.method(POST)
            .body_contains("\"getAccountInfo\"")
            .body_contains(&pk);
        then.status(200)
            .header("content-type", "application/json")
            .json_body(body);
    })
}

fn mount_get_multiple<'a>(server: &'a MockServer, kas: &[KeyedAccount]) -> httpmock::Mock<'a> {
    // Build a value array matching the order keys appear in the request body.
    // For our tests we always hit `get_multiple_accounts` with the same canonical order,
    // so a single mock returning every account in that order is enough.
    let values: Vec<Value> = kas.iter().map(rpc_account_value).collect();
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "context": { "apiVersion": "1.18.0", "slot": 123 },
            "value": values,
        }
    });
    server.mock(|when, then| {
        when.method(POST).body_contains("\"getMultipleAccounts\"");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(body);
    })
}

#[test]
fn rpc_only_get_caches_after_first_fetch() {
    let server = MockServer::start();
    let ka = keyed(11);
    let m = mount_get_account(&server, &ka);

    let acc = Accache::builder()
        .with_rpc(server.base_url())
        .build()
        .unwrap();

    let got = acc.get(&ka.key).unwrap();
    assert_eq!(got, ka);
    assert_eq!(acc.len(), 1);

    // Second call must hit cache, not RPC.
    let got2 = acc.get(&ka.key).unwrap();
    assert_eq!(got2, ka);
    m.assert_hits(1);
}

#[test]
fn always_policy_bypasses_cache() {
    let server = MockServer::start();
    let ka = keyed(13);
    let m = mount_get_account(&server, &ka);

    let acc = Accache::builder()
        .with_rpc(server.base_url())
        .refresh(RefreshPolicy::Always)
        .build()
        .unwrap();

    let _ = acc.get(&ka.key).unwrap();
    let _ = acc.get(&ka.key).unwrap();
    m.assert_hits(2);
}

#[test]
fn ttl_policy_refreshes_after_expiry() {
    let server = MockServer::start();
    let ka = keyed(17);
    let m = mount_get_account(&server, &ka);

    let acc = Accache::builder()
        .with_rpc(server.base_url())
        .refresh(RefreshPolicy::Ttl(Duration::from_millis(50)))
        .build()
        .unwrap();

    let _ = acc.get(&ka.key).unwrap();
    let _ = acc.get(&ka.key).unwrap(); // cached
    std::thread::sleep(Duration::from_millis(80));
    let _ = acc.get(&ka.key).unwrap(); // stale → refetch
    m.assert_hits(2);
}

#[test]
fn hybrid_falls_through_to_rpc_on_miss_and_persists() {
    let dir = tempfile::tempdir().unwrap();
    let fixture = dir.path().join("seed.acc");
    let outfile = dir.path().join("merged.acc");

    let preloaded = keyed(20);
    let entries = vec![Entry::from_keyed(&preloaded, None)];
    write_file(&fixture, &entries, Compression::Zstd { level: 3 }).unwrap();

    let fresh = keyed(21);
    let server = MockServer::start();
    let m = mount_get_account(&server, &fresh);

    let acc = Accache::builder()
        .with_files([&fixture])
        .with_rpc(server.base_url())
        .outfile(&outfile)
        .build()
        .unwrap();

    // Cached hit, no RPC.
    let got = acc.get(&preloaded.key).unwrap();
    assert_eq!(got, preloaded);
    m.assert_hits(0);

    // Miss → RPC.
    let got = acc.get(&fresh.key).unwrap();
    assert_eq!(got, fresh);
    m.assert_hits(1);

    acc.flush().unwrap();
    let (_, persisted) = accache::format::read_file(&outfile).unwrap();
    assert_eq!(persisted.len(), 2);
}

#[test]
fn get_multiple_chunks_to_max_batch() {
    let server = MockServer::start();
    // 250 accounts → expect 3 batches of (100, 100, 50). We encode each batch's worth
    // of values in order; httpmock will return the same payload for each call. Since
    // we just check counts, this is fine.
    let kas: Vec<KeyedAccount> = (0..250).map(|i| keyed(i as u8)).collect();

    // Mount three separate mocks: one matching a body containing the first key of
    // each batch, with the corresponding slice of values.
    let mut mocks = Vec::new();
    for (idx, batch) in kas.chunks(100).enumerate() {
        let first_key = batch[0].key.to_string();
        let values: Vec<Value> = batch.iter().map(rpc_account_value).collect();
        let body = json!({
            "jsonrpc": "2.0", "id": 1,
            "result": {
                "context": { "apiVersion": "1.18.0", "slot": 123 },
                "value": values,
            }
        });
        let m = server.mock(|when, then| {
            when.method(POST)
                .body_contains("\"getMultipleAccounts\"")
                .body_contains(&first_key);
            then.status(200)
                .header("content-type", "application/json")
                .json_body(body);
        });
        mocks.push((idx, m));
    }

    let acc = Accache::builder()
        .with_rpc(server.base_url())
        .build()
        .unwrap();
    let pks: Vec<Pubkey> = kas.iter().map(|k| k.key).collect();
    let got = acc.get_multiple(&pks).unwrap();
    assert_eq!(got.len(), 250);
    let total_hits: usize = mocks.iter().map(|(_, m)| m.hits()).sum();
    assert_eq!(total_hits, 3, "expected exactly 3 RPC calls (100+100+50)");
}

#[test]
fn rpc_account_not_found_returns_typed_error() {
    let server = MockServer::start();
    // Mount a `null` response for any getAccountInfo.
    server.mock(|when, then| {
        when.method(POST).body_contains("\"getAccountInfo\"");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "jsonrpc": "2.0", "id": 1,
                "result": {
                    "context": { "apiVersion": "1.18.0", "slot": 123 },
                    "value": null,
                }
            }));
    });

    let acc = Accache::builder()
        .with_rpc(server.base_url())
        .build()
        .unwrap();
    let k = Pubkey::new_from_array([7; 32]);
    match acc.get(&k) {
        Err(AccacheError::NotFound(p)) => assert_eq!(p, k),
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn persisted_file_preserves_request_order() {
    let server = MockServer::start();
    // Request pubkeys in a deliberately non-sorted, non-hash-sorted order. The
    // persisted .acc file should iterate back in this exact order.
    let kas: Vec<KeyedAccount> = [42, 7, 200, 1, 99, 33].iter().map(|s| keyed(*s)).collect();
    let _m = mount_get_multiple(&server, &kas);

    let dir = tempfile::tempdir().unwrap();
    let outfile = dir.path().join("ordered.acc");
    let acc = Accache::builder()
        .with_rpc(server.base_url())
        .outfile(&outfile)
        .build()
        .unwrap();
    let pks: Vec<Pubkey> = kas.iter().map(|k| k.key).collect();
    let _ = acc.get_multiple(&pks).unwrap();
    acc.flush().unwrap();

    let (_, persisted) = accache::format::read_file(&outfile).unwrap();
    let persisted_keys: Vec<Pubkey> = persisted.iter().map(|e| e.pubkey()).collect();
    assert_eq!(persisted_keys, pks);
}

#[test]
fn get_multiple_skips_rpc_when_all_cached() {
    let server = MockServer::start();
    let m = mount_get_multiple(&server, &[]);
    // Pre-populate via inserts; no RPC needed.
    let acc = Accache::builder()
        .with_rpc(server.base_url())
        .build()
        .unwrap();
    let kas: Vec<_> = (1..=5).map(keyed).collect();
    for k in &kas {
        acc.insert(k.key, k.account.clone());
    }
    let pks: Vec<Pubkey> = kas.iter().map(|k| k.key).collect();
    let got = acc.get_multiple(&pks).unwrap();
    assert_eq!(got.len(), 5);
    m.assert_hits(0);
}
