#![cfg(feature = "nonblocking")]

use accache::format::{Entry, write_file};
use accache::nonblocking::Accache;
use accache::{Account, Compression, KeyedAccount, Pubkey, RefreshPolicy};

fn keyed(seed: u8) -> KeyedAccount {
    KeyedAccount::new(
        Pubkey::new_from_array([seed; 32]),
        Account {
            lamports: 100,
            data: vec![seed; 4],
            owner: Pubkey::new_from_array([0; 32]),
            executable: false,
            rent_epoch: 0,
        },
    )
}

#[tokio::test]
async fn async_offline_get_from_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("async.acc");
    let ka = keyed(8);
    let entries = vec![Entry::from_keyed(&ka, None)];
    write_file(&path, &entries, Compression::Zstd { level: 3 }).unwrap();

    let acc = Accache::builder()
        .with_files([&path])
        .refresh(RefreshPolicy::Offline)
        .build_nonblocking()
        .unwrap();

    let got = acc.get(&ka.key).await.unwrap();
    assert_eq!(got, ka);
}

#[tokio::test]
async fn async_get_multiple_offline() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("async_multi.acc");
    let kas: Vec<KeyedAccount> = (1..=5).map(keyed).collect();
    let entries: Vec<Entry> = kas.iter().map(|k| Entry::from_keyed(k, None)).collect();
    write_file(&path, &entries, Compression::None).unwrap();

    let acc = Accache::builder()
        .with_files([&path])
        .refresh(RefreshPolicy::Offline)
        .build_nonblocking()
        .unwrap();

    let pks: Vec<Pubkey> = kas.iter().map(|k| k.key).collect();
    let got = acc.get_multiple(&pks).await.unwrap();
    assert_eq!(got.len(), 5);
}
