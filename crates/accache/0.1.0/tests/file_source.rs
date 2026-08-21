use std::path::PathBuf;

use accache::format::{Entry, write_file};
use accache::{Accache, AccacheError, Account, Compression, KeyedAccount, Pubkey, RefreshPolicy};

fn keyed(seed: u8) -> KeyedAccount {
    KeyedAccount::new(
        Pubkey::new_from_array([seed; 32]),
        Account {
            lamports: 1_000_000 * (seed as u64 + 1),
            data: vec![seed; 64],
            owner: Pubkey::new_from_array([255 - seed; 32]),
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
fn loads_known_keys_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fixture.acc");
    let ka = keyed(1);
    write_fixture(&path, &[ka.clone()]);

    let acc = Accache::builder()
        .with_files([&path])
        .refresh(RefreshPolicy::Offline)
        .build()
        .unwrap();

    assert_eq!(acc.len(), 1);
    let got = acc.get(ka.key()).unwrap();
    assert_eq!(got, ka);
}

#[test]
fn offline_returns_typed_error_on_miss() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fixture.acc");
    write_fixture(&path, &[keyed(1)]);

    let acc = Accache::builder()
        .with_files([&path])
        .refresh(RefreshPolicy::Offline)
        .build()
        .unwrap();

    let unknown = Pubkey::new_from_array([42; 32]);
    match acc.get(&unknown) {
        Err(AccacheError::Offline(p)) => assert_eq!(p, unknown),
        other => panic!("expected Offline, got {other:?}"),
    }
}

#[test]
fn no_rpc_configured_falls_through_for_non_offline_policy() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fixture.acc");
    write_fixture(&path, &[keyed(1)]);

    // OnMiss but no RPC source: misses must error with NoRpcConfigured.
    let acc = Accache::builder()
        .with_files([&path])
        .refresh(RefreshPolicy::OnMiss)
        .build()
        .unwrap();

    let unknown = Pubkey::new_from_array([42; 32]);
    match acc.get(&unknown) {
        Err(AccacheError::NoRpcConfigured) => {}
        other => panic!("expected NoRpcConfigured, got {other:?}"),
    }
}

#[test]
fn multiple_files_last_wins() {
    let dir = tempfile::tempdir().unwrap();
    let p1 = dir.path().join("a.acc");
    let p2 = dir.path().join("b.acc");

    let mut shared = keyed(7);
    shared.account.lamports = 100;
    write_fixture(&p1, &[shared.clone()]);
    shared.account.lamports = 999;
    write_fixture(&p2, &[shared.clone()]);

    let acc = Accache::builder()
        .with_files([&p1, &p2])
        .refresh(RefreshPolicy::Offline)
        .build()
        .unwrap();

    let got = acc.get(shared.key()).unwrap();
    assert_eq!(got.account.lamports, 999);
}

#[test]
fn try_get_returns_none_for_missing_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fixture.acc");
    write_fixture(&path, &[keyed(1)]);

    let acc = Accache::builder()
        .with_files([&path])
        .refresh(RefreshPolicy::Offline)
        .build()
        .unwrap();

    assert!(acc.try_get(&Pubkey::new_from_array([99; 32])).is_none());
    assert!(acc.try_get(keyed(1).key()).is_some());
}

#[test]
fn get_multiple_returns_in_input_order() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fixture.acc");
    let kas: Vec<_> = (1..=4).map(keyed).collect();
    write_fixture(&path, &kas);

    let acc = Accache::builder()
        .with_files([&path])
        .refresh(RefreshPolicy::Offline)
        .build()
        .unwrap();

    let keys: Vec<Pubkey> = vec![kas[2].key, kas[0].key, kas[3].key];
    let got = acc.get_multiple(&keys).unwrap();
    assert_eq!(got.len(), 3);
    assert_eq!(got[0].key, kas[2].key);
    assert_eq!(got[1].key, kas[0].key);
    assert_eq!(got[2].key, kas[3].key);
}
