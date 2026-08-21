//! Unit tests for the diff engine.
//!
//! Placed in a dedicated `tests.rs` per the project style guide (別紙).

use std::fs;

use super::engine::DiffEngine;
use super::entry::DiffType;

// ── helpers ──────────────────────────────────────────────────────────────

fn tmp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn write(dir: &std::path::Path, rel: &str, content: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
}

fn find(diffs: &[super::entry::DiffEntry], path: &str) -> super::entry::DiffEntry {
    diffs.iter().find(|d| d.path == path)
        .unwrap_or_else(|| panic!("entry not found: {path}"))
        .clone()
}

// ── tests ─────────────────────────────────────────────────────────────────

#[test]
fn detects_added_file() {
    let before = tmp_dir();
    let after  = tmp_dir();
    write(after.path(), "new.txt", "hello");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let e = find(&diffs, "new.txt");
    assert_eq!(e.diff_type, DiffType::Added);
    assert!(e.after_text.is_some());
}

#[test]
fn detects_removed_file() {
    let before = tmp_dir();
    let after  = tmp_dir();
    write(before.path(), "gone.toml", "[x]");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let e = find(&diffs, "gone.toml");
    assert_eq!(e.diff_type, DiffType::Removed);
}

#[test]
fn detects_modified_file() {
    let before = tmp_dir();
    let after  = tmp_dir();
    write(before.path(), "cfg.toml", "port = 80\n");
    write(after.path(),  "cfg.toml", "port = 8080\n");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let e = find(&diffs, "cfg.toml");
    assert_eq!(e.diff_type, DiffType::Modified);
    assert_ne!(e.before_text, e.after_text);
}

#[test]
fn detects_unchanged_file() {
    let before = tmp_dir();
    let after  = tmp_dir();
    write(before.path(), "same.txt", "content");
    write(after.path(),  "same.txt", "content");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let e = find(&diffs, "same.txt");
    assert_eq!(e.diff_type, DiffType::Unchanged);
}

#[test]
fn output_is_sorted() {
    let before = tmp_dir();
    let after  = tmp_dir();
    write(after.path(), "z.txt", "z");
    write(after.path(), "a.txt", "a");
    write(after.path(), "m.txt", "m");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let paths: Vec<_> = diffs.iter().map(|d| d.path.as_str()).collect();
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted);
}

#[test]
fn nested_paths_use_forward_slash() {
    let before = tmp_dir();
    let after  = tmp_dir();
    write(after.path(), "sub/dir/file.txt", "hi");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let paths: Vec<_> = diffs.iter().map(|d| d.path.as_str()).collect();
    assert!(paths.iter().any(|p| p.contains('/')));
    assert!(!paths.iter().any(|p| p.contains('\\')));
}

#[test]
fn sha256_present_for_after_file() {
    let before = tmp_dir();
    let after  = tmp_dir();
    write(after.path(), "file.bin", "data");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let e = find(&diffs, "file.bin");
    assert!(e.after_sha256.is_some());
    let sha = e.after_sha256.unwrap();
    assert_eq!(sha.len(), 64);
    assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn before_root_must_be_directory() {
    let result = DiffEngine::compare(
        std::path::Path::new("/nonexistent/path"),
        std::path::Path::new("/tmp"),
    );
    assert!(result.is_err());
}

// ── Phase 4 tests ─────────────────────────────────────────────────────────

#[test]
fn binary_file_detected() {
    let before = tmp_dir();
    let after  = tmp_dir();
    // Write a file with null bytes (binary marker).
    let binary_content: Vec<u8> = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE];
    std::fs::write(after.path().join("data.bin"), &binary_content).unwrap();

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let e = find(&diffs, "data.bin");
    assert_eq!(e.diff_type, DiffType::Added);
    assert!(e.is_binary, "null bytes should mark file as binary");
    assert!(e.after_text.is_none(), "binary file should have no text");
    assert!(e.after_sha256.is_some(), "binary file should still have hash");
    assert!(e.after_size.is_some(), "binary file should have size");
}

#[test]
fn diff_stats_computed_for_modified_text() {
    let before = tmp_dir();
    let after  = tmp_dir();
    write(before.path(), "lines.txt", "line1\nline2\nline3\n");
    write(after.path(),  "lines.txt", "line1\nline2_changed\nline3\nline4\n");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let e = find(&diffs, "lines.txt");
    assert_eq!(e.diff_type, DiffType::Modified);
    let stats = e.stats.as_ref().expect("stats should be present for modified text");
    assert!(stats.lines_added   >= 1, "should have added lines");
    assert!(stats.lines_removed >= 1, "should have removed lines");
}

#[test]
fn size_tracking_for_modified_file() {
    let before = tmp_dir();
    let after  = tmp_dir();
    write(before.path(), "f.txt", "short");
    write(after.path(),  "f.txt", "much longer content here");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let e = find(&diffs, "f.txt");
    assert!(e.before_size.is_some());
    assert!(e.after_size.is_some());
    assert!(e.after_size.unwrap() > e.before_size.unwrap());
}

#[test]
fn parallel_compare_produces_sorted_output() {
    use crate::diff::ignore::IgnoreRules;
    let before = tmp_dir();
    let after  = tmp_dir();
    for name in ["z.txt", "a.txt", "m.txt", "b.txt"] {
        write(after.path(), name, name);
    }
    let diffs = DiffEngine::compare_with_ignore(
        before.path(), after.path(), &IgnoreRules::default()
    ).unwrap();
    let paths: Vec<_> = diffs.iter().map(|d| d.path.as_str()).collect();
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted, "parallel output must be sorted");
}

#[test]
fn before_sha256_tracked() {
    let before = tmp_dir();
    let after  = tmp_dir();
    write(before.path(), "f.txt", "content");
    write(after.path(),  "f.txt", "different");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let e = find(&diffs, "f.txt");
    assert!(e.before_sha256.is_some(), "before_sha256 must be present for modified files");
    assert!(e.after_sha256.is_some());
    assert_ne!(e.before_sha256, e.after_sha256, "hashes must differ for modified file");
}
