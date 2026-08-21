#![cfg(feature = "aot")]

use aam_rs::aam::AAM;
use aam_rs::aot::{AamCompiler, AamLoader};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn unique_path(name: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("aam_rs_{name}_{ts}.aam"))
}

#[test]
fn cook_and_fast_load_roundtrip() {
    let source = unique_path("roundtrip");
    fs::write(&source, "host = localhost\nport = 8080\n").expect("write source");

    let cache = AamCompiler::cook(&source).expect("cook").to_path_buf();
    assert!(cache.exists(), "cooked cache should exist");

    let mapped = AamLoader::load_fast(&source).expect("load_fast");
    assert_eq!(mapped.get("host"), Some("localhost"));
    assert_eq!(mapped.get("port"), Some("8080"));
    assert!(mapped.len() >= 2);

    let _ = fs::remove_file(&source);
    let _ = fs::remove_file(&cache);
}

#[test]
fn load_fast_rebuilds_stale_cache() {
    let source = unique_path("stale");
    fs::write(&source, "value = old\n").expect("write source");

    let cache = AamCompiler::cook(&source)
        .expect("initial cook")
        .to_path_buf();

    // Most filesystems have coarse timestamp resolution, so ensure mtime ticks.
    thread::sleep(Duration::from_millis(1100));
    fs::write(&source, "value = fresh\n").expect("rewrite source");

    let mapped = AamLoader::load_fast(&source).expect("load_fast after update");
    assert_eq!(mapped.get("value"), Some("fresh"));

    let _ = fs::remove_file(&source);
    let _ = fs::remove_file(&cache);
}

#[test]
fn aam_load_uses_default_aot_path() {
    let source = unique_path("aam_load");
    fs::write(&source, "game.title = Example\ngame.fps = 120\n").expect("write source");

    let aam = AAM::load(&source).expect("aam load");
    assert_eq!(aam.get("game.title"), Some("Example"));
    assert_eq!(aam.get("game.fps"), Some("120"));

    let cache = source.with_extension("aam.bin");
    assert!(cache.exists(), "default load should create cooked cache");

    let _ = fs::remove_file(&source);
    let _ = fs::remove_file(&cache);
}

#[test]
fn cooked_blob_uses_hash_table_and_spans() {
    let source = unique_path("layout");
    fs::write(&source, "z = 1\na = 2\nb = 3\n").expect("write source");

    let cache = AamCompiler::cook(&source).expect("cook");
    let mapped = AamLoader::load_fast(&source).expect("load_fast");
    let archived = mapped.archived();

    let hash_table = archived.hash_table.as_slice();
    assert!(!hash_table.is_empty(), "hash table should be generated");
    assert!(
        hash_table
            .iter()
            .any(|entry| entry.node_index.to_native() != u32::MAX),
        "hash table should contain at least one populated slot"
    );

    assert_eq!(mapped.get("a"), Some("2"));
    assert_eq!(mapped.get("b"), Some("3"));
    assert_eq!(mapped.get("z"), Some("1"));

    let _ = fs::remove_file(&source);
    let _ = fs::remove_file(&cache);
}
