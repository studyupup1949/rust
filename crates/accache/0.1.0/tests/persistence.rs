use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use accache::format::{Entry, write_file};
use accache::{Accache, Account, Compression, KeyedAccount, Pubkey, RefreshPolicy};

fn keyed(seed: u8) -> KeyedAccount {
    KeyedAccount::new(
        Pubkey::new_from_array([seed; 32]),
        Account {
            lamports: seed as u64 * 1_000,
            data: vec![seed; 16],
            owner: Pubkey::new_from_array([0; 32]),
            executable: false,
            rent_epoch: 0,
        },
    )
}

#[test]
fn auto_persist_writes_after_each_insert() {
    let dir = tempfile::tempdir().unwrap();
    let outfile = dir.path().join("auto.acc");

    let acc = Accache::builder()
        .outfile(&outfile)
        .auto_persist(true)
        .refresh(RefreshPolicy::Offline)
        .build()
        .unwrap();

    let k1 = keyed(1);
    acc.insert(k1.key, k1.account.clone());
    assert!(outfile.exists());
    let (_, e1) = accache::format::read_file(&outfile).unwrap();
    assert_eq!(e1.len(), 1);

    let k2 = keyed(2);
    acc.insert(k2.key, k2.account.clone());
    let (_, e2) = accache::format::read_file(&outfile).unwrap();
    assert_eq!(e2.len(), 2);
}

#[test]
fn flush_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let outfile = dir.path().join("idem.acc");

    let acc = Accache::builder()
        .outfile(&outfile)
        .refresh(RefreshPolicy::Offline)
        .build()
        .unwrap();

    let k = keyed(5);
    acc.insert(k.key, k.account.clone());
    acc.flush().unwrap();
    let bytes_a = std::fs::read(&outfile).unwrap();
    acc.flush().unwrap();
    let bytes_b = std::fs::read(&outfile).unwrap();
    // Compression of identical input is deterministic at the same level for zstd.
    assert_eq!(bytes_a, bytes_b);
}

#[test]
fn write_to_explicit_path() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("explicit.acc");

    let acc = Accache::builder()
        .refresh(RefreshPolicy::Offline)
        .build()
        .unwrap();
    acc.insert(keyed(9).key, keyed(9).account);
    acc.write_to(&p).unwrap();
    assert!(p.exists());
    let (_, entries) = accache::format::read_file(&p).unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn no_outfile_flush_is_noop() {
    let acc = Accache::builder()
        .refresh(RefreshPolicy::Offline)
        .build()
        .unwrap();
    acc.flush().unwrap();
    acc.insert(keyed(0).key, keyed(0).account);
    acc.flush().unwrap(); // still a no-op
}

#[test]
fn concurrent_reads_and_writes_are_safe() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("seed.acc");
    let preloaded: Vec<KeyedAccount> = (0..32).map(keyed).collect();
    let entries: Vec<Entry> = preloaded
        .iter()
        .map(|k| Entry::from_keyed(k, None))
        .collect();
    write_file(&path, &entries, Compression::Zstd { level: 1 }).unwrap();

    let acc = Accache::builder()
        .with_files([&path])
        .refresh(RefreshPolicy::Offline)
        .build()
        .unwrap();

    let success = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for t in 0..8 {
        let acc = acc.clone();
        let success = success.clone();
        let preloaded = preloaded.clone();
        handles.push(thread::spawn(move || {
            for i in 0..200 {
                if (t + i) % 3 == 0 {
                    // writer thread: insert a new key
                    let k = keyed((t * 100 + i) as u8);
                    acc.insert(k.key, k.account.clone());
                } else {
                    // reader: pick a known key
                    let k = &preloaded[(i as usize) % preloaded.len()];
                    let got = acc.get(&k.key).unwrap();
                    assert_eq!(&got, k);
                    success.fetch_add(1, Ordering::Relaxed);
                }
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    assert!(success.load(Ordering::Relaxed) > 0);
    // Cache must contain at least the preloaded set; insertions may collide on seed-mod-256.
    assert!(acc.len() >= preloaded.len());
}

#[test]
fn remove_evicts_from_cache_and_outfile() {
    let dir = tempfile::tempdir().unwrap();
    let outfile = dir.path().join("evict.acc");

    let acc = Accache::builder()
        .outfile(&outfile)
        .auto_persist(true)
        .refresh(RefreshPolicy::Offline)
        .build()
        .unwrap();

    let k = keyed(3);
    acc.insert(k.key, k.account.clone());
    assert_eq!(acc.len(), 1);

    let removed = acc.remove(&k.key).expect("present");
    assert_eq!(removed.key, k.key);
    assert_eq!(acc.len(), 0);

    let (_, entries) = accache::format::read_file(&outfile).unwrap();
    assert_eq!(entries.len(), 0);
}
