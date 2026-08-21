use accache::format::{AccReader, AccWriter, Entry, FORMAT_VERSION, MAGIC, read_file, write_file};
use accache::{Account, Compression, KeyedAccount, Pubkey};

fn make_keyed(seed: u8, data_len: usize) -> KeyedAccount {
    let key = Pubkey::new_from_array([seed; 32]);
    let owner = Pubkey::new_from_array([seed.wrapping_add(1); 32]);
    let mut data = vec![0u8; data_len];
    for (i, b) in data.iter_mut().enumerate() {
        *b = ((i as u32 ^ seed as u32) & 0xff) as u8;
    }
    KeyedAccount::new(
        key,
        Account {
            lamports: 1_000_000 + seed as u64,
            data,
            owner,
            executable: seed % 2 == 0,
            rent_epoch: seed as u64,
        },
    )
}

fn roundtrip_n(n: usize, data_len: usize, compression: Compression) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.acc");

    let entries: Vec<Entry> = (0..n)
        .map(|i| Entry::from_keyed(&make_keyed(i as u8, data_len), Some(i as u64 * 1_000)))
        .collect();

    write_file(&path, &entries, compression).unwrap();
    let (header, decoded) = read_file(&path).unwrap();

    assert_eq!(header.magic, MAGIC);
    assert_eq!(header.version, FORMAT_VERSION);
    assert_eq!(header.count as usize, n);
    assert_eq!(decoded.len(), n);
    assert_eq!(decoded, entries);
}

#[test]
fn empty_file_roundtrip_zstd() {
    roundtrip_n(0, 0, Compression::Zstd { level: 3 });
}

#[test]
fn empty_file_roundtrip_uncompressed() {
    roundtrip_n(0, 0, Compression::None);
}

#[test]
fn one_entry_roundtrip() {
    roundtrip_n(1, 128, Compression::Zstd { level: 3 });
    roundtrip_n(1, 128, Compression::None);
}

#[test]
fn many_entries_roundtrip() {
    roundtrip_n(1024, 64, Compression::Zstd { level: 3 });
}

#[test]
fn large_data_streams_through() {
    // 4 MiB account, just one entry.
    roundtrip_n(1, 4 * 1024 * 1024, Compression::Zstd { level: 1 });
}

#[test]
fn rejects_corrupt_payload() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("corrupt.acc");

    let entries = vec![Entry::from_keyed(&make_keyed(7, 32), Some(1))];
    write_file(&path, &entries, Compression::None).unwrap();

    // Corrupt the last byte (which lives inside the bincode payload, not the header).
    let mut bytes = std::fs::read(&path).unwrap();
    let n = bytes.len();
    // Truncate aggressively so deserialization fails.
    bytes.truncate(n.saturating_sub(8).max(16));
    std::fs::write(&path, &bytes).unwrap();

    let result = read_file(&path);
    assert!(result.is_err(), "expected an error for truncated payload");
}

#[test]
fn rejects_bad_magic_at_file_level() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.acc");
    std::fs::write(&path, b"NOTACC1\0\0\0\0\0\0\0\0\0").unwrap();
    let result = read_file(&path);
    assert!(matches!(
        result,
        Err(accache::AccacheError::BadMagic { .. })
    ));
}

#[test]
fn writer_then_reader_via_in_memory_buffer() {
    let entries: Vec<Entry> = (0..16)
        .map(|i| Entry::from_keyed(&make_keyed(i, 256), Some(i as u64)))
        .collect();
    let mut buf = Vec::new();
    let mut w = AccWriter::new(&mut buf, Compression::Zstd { level: 3 }).unwrap();
    w.extend(entries.clone());
    let _ = w.finish().unwrap();

    let reader = AccReader::new(buf.as_slice()).unwrap();
    assert_eq!(reader.header.count as usize, entries.len());
    assert!(reader.header.is_zstd());
    let decoded = reader.read_entries().unwrap();
    assert_eq!(decoded, entries);
}
