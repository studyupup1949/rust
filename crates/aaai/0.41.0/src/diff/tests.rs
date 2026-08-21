//! Unit tests for the diff engine.
//!
//! Placed in a dedicated `tests.rs` per the project style guide (別紙).

use std::fs;

use super::engine::DiffEngine;
use super::entry::DiffType;
use super::progress::{DiffProgress, ProgressSink};

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
fn progress_sequence_is_complete_and_deterministic() {
    use std::sync::Mutex;
    use crate::diff::ignore::IgnoreRules;

    #[derive(Default)]
    struct RecordingProgress(Mutex<Vec<DiffProgress>>);
    impl ProgressSink for RecordingProgress {
        fn emit(&self, event: DiffProgress) {
            self.0.lock().unwrap().push(event);
        }
    }

    let before = tmp_dir();
    let after = tmp_dir();
    for name in ["z.txt", "a.txt", "middle.txt", "%literal", "nested/file.txt"] {
        write(after.path(), name, name);
    }

    let mut expected = None;
    for _ in 0..8 {
        let progress = RecordingProgress::default();
        DiffEngine::compare_with_progress(
            before.path(), after.path(), &IgnoreRules::default(), &progress,
        ).unwrap();
        let events = progress.0.into_inner().unwrap();
        assert!(matches!(events.first(), Some(DiffProgress::Started { total: 6 })));
        assert!(matches!(events.get(7), Some(DiffProgress::Sorting)));
        assert!(matches!(events.get(8), Some(DiffProgress::Done { total_files: 6 })));
        let files: Vec<_> = events[1..7].iter().enumerate().map(|(index, event)| {
            let DiffProgress::File { path, processed, total } = event else { panic!() };
            assert_eq!(*processed, index + 1);
            assert_eq!(*total, 6);
            path.clone()
        }).collect();
        assert_eq!(files, ["%25literal", "a.txt", "middle.txt", "nested", "nested/file.txt", "z.txt"]);
        if let Some(previous) = &expected {
            assert_eq!(&files, previous);
        }
        expected = Some(files);
    }
}

#[test]
fn path_read_issue_is_unreadable_and_audit_error() {
    use crate::{AuditDefinition, AuditEngine};

    let issue = super::path_boundary::PathIssue {
        code: "AAAI-PATH-READ",
        detail: "The entry could not be opened for reading.",
        unreadable: true,
    };
    let entry = super::engine::issue_entry("blocked".into(), &issue);
    assert_eq!(entry.diff_type, DiffType::Unreadable);
    let audit = AuditEngine::evaluate(&[entry], &AuditDefinition::new_empty());
    assert_eq!(audit.summary.error, 1);
    assert!(!audit.summary.is_passing());
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
#[cfg(unix)]
#[test]
fn symlinked_file_is_incomparable_and_outside_content_is_absent() {
    use std::os::unix::fs::symlink;

    let before = tmp_dir();
    let after = tmp_dir();
    let outside = tmp_dir();
    write(outside.path(), "canary", "outside-secret-canary");
    symlink(outside.path().join("canary"), after.path().join("linked")).unwrap();

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let entry = find(&diffs, "linked");
    assert_eq!(entry.diff_type, DiffType::Incomparable);
    assert_eq!(
        entry.error_detail.as_deref(),
        Some("[AAAI-PATH-LINK] Link-like entries are not followed.")
    );
    let rendered = diffs
        .iter()
        .map(|entry| {
            format!(
                "{}{:?}{:?}{:?}",
                entry.path, entry.before_text, entry.after_text, entry.error_detail
            )
        })
        .collect::<String>();
    assert!(!rendered.contains("outside-secret-canary"));
    assert!(!rendered.contains(&outside.path().display().to_string()));
}

#[cfg(unix)]
#[test]
fn symlinked_directory_is_not_traversed() {
    use std::os::unix::fs::symlink;

    let before = tmp_dir();
    let after = tmp_dir();
    let outside = tmp_dir();
    write(outside.path(), "nested-secret", "outside-secret-canary");
    symlink(outside.path(), after.path().join("linked-dir")).unwrap();

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    assert_eq!(find(&diffs, "linked-dir").diff_type, DiffType::Incomparable);
    assert!(
        !diffs
            .iter()
            .any(|entry| entry.path.contains("nested-secret"))
    );
}

#[cfg(unix)]
#[test]
fn selected_root_symlink_is_rejected() {
    use std::os::unix::fs::symlink;

    let physical = tmp_dir();
    let holder = tmp_dir();
    let root_link = holder.path().join("selected");
    symlink(physical.path(), &root_link).unwrap();
    let other = tmp_dir();

    let error = DiffEngine::compare(&root_link, other.path()).expect_err("root link must fail");
    assert!(error.to_string().contains("AAAI-ROOT-UNAVAILABLE"));
    assert!(
        !error
            .to_string()
            .contains(&physical.path().display().to_string())
    );
    for bypass in [root_link.join("."), root_link.join("subdir").join("..")] {
        let error = DiffEngine::compare(&bypass, other.path())
            .expect_err("root-link bypass must fail");
        assert!(error.to_string().contains("AAAI-ROOT-UNAVAILABLE"));
    }
}

#[cfg(unix)]
#[test]
fn fifo_is_incomparable_and_never_opened_for_content() {
    use std::process::Command;

    let before = tmp_dir();
    let after = tmp_dir();
    assert!(
        Command::new("mkfifo")
            .arg(after.path().join("pipe"))
            .status()
            .unwrap()
            .success()
    );

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let entry = find(&diffs, "pipe");
    assert_eq!(entry.diff_type, DiffType::Incomparable);
    assert_eq!(
        entry.error_detail.as_deref(),
        Some("[AAAI-PATH-SPECIAL] Special filesystem objects are not read.")
    );
}

#[cfg(unix)]
#[test]
fn unix_socket_is_incomparable_and_never_opened_for_content() {
    use std::os::unix::net::UnixListener;

    let before = tmp_dir();
    let after = tmp_dir();
    let _listener = UnixListener::bind(after.path().join("socket")).unwrap();
    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let entry = find(&diffs, "socket");
    assert_eq!(entry.diff_type, DiffType::Incomparable);
    assert!(entry.error_detail.as_deref().unwrap().starts_with("[AAAI-PATH-SPECIAL]"));
}

#[cfg(unix)]
#[test]
fn ignored_link_is_omitted_without_granting_traversal() {
    use crate::diff::ignore::IgnoreRules;
    use std::os::unix::fs::symlink;

    let before = tmp_dir();
    let after = tmp_dir();
    let outside = tmp_dir();
    write(outside.path(), "nested", "outside-secret-canary");
    symlink(outside.path(), after.path().join("ignored-link")).unwrap();
    let ignore = IgnoreRules::from_str("ignored-link\n").unwrap();

    let diffs = DiffEngine::compare_with_ignore(before.path(), after.path(), &ignore).unwrap();
    assert!(diffs.is_empty());
}

#[cfg(unix)]
#[test]
fn native_names_remain_distinct_and_have_collision_free_display_ids() {
    use std::ffi::OsStr;
    #[cfg(target_os = "linux")]
    use std::os::unix::ffi::OsStrExt;

    let before = tmp_dir();
    let after = tmp_dir();
    #[cfg(target_os = "linux")]
    fs::write(after.path().join(OsStr::from_bytes(b"x\x80")), b"safe").unwrap();
    for name in [
        OsStr::new("%78%80"),
        OsStr::new("back\\slash"),
        OsStr::new("control\nname"),
    ] {
        fs::write(after.path().join(name), b"safe").unwrap();
    }
    write(after.path(), "back/slash", "safe");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    let paths: Vec<&str> = diffs.iter().map(|entry| entry.path.as_str()).collect();
    #[cfg(target_os = "linux")]
    assert!(paths.contains(&"%78%80"));
    #[cfg(target_os = "linux")]
    assert!(paths.contains(&"%2578%2580"));
    #[cfg(target_os = "macos")]
    assert!(paths.contains(&"%2578%2580"));
    assert!(paths.contains(&"back%5Cslash"));
    assert!(paths.contains(&"back/slash"));
    assert!(paths.contains(&"control%0Aname"));
    #[cfg(target_os = "linux")]
    assert_eq!(diffs.len(), 6);
    #[cfg(target_os = "macos")]
    assert_eq!(diffs.len(), 5);
}

#[cfg(unix)]
#[test]
fn broken_and_cyclic_links_are_reported_without_following() {
    use std::os::unix::fs::symlink;

    let before = tmp_dir();
    let after = tmp_dir();
    symlink("absent-target", after.path().join("broken")).unwrap();
    symlink("cycle-b", after.path().join("cycle-a")).unwrap();
    symlink("cycle-a", after.path().join("cycle-b")).unwrap();
    symlink("self", after.path().join("self")).unwrap();

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    for name in ["broken", "cycle-a", "cycle-b", "self"] {
        let entry = find(&diffs, name);
        assert_eq!(entry.diff_type, DiffType::Incomparable);
        assert!(
            entry
                .error_detail
                .as_deref()
                .unwrap()
                .starts_with("[AAAI-PATH-LINK]")
        );
    }
}

#[cfg(unix)]
#[test]
fn links_are_errors_when_removed_or_present_on_both_sides() {
    use std::os::unix::fs::symlink;

    let before = tmp_dir();
    let after = tmp_dir();
    let outside = tmp_dir();
    write(outside.path(), "canary", "outside-secret-canary");
    symlink(
        outside.path().join("canary"),
        before.path().join("removed-link"),
    )
    .unwrap();
    symlink(
        outside.path().join("canary"),
        before.path().join("same-link"),
    )
    .unwrap();
    symlink(
        outside.path().join("canary"),
        after.path().join("same-link"),
    )
    .unwrap();

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    assert_eq!(
        find(&diffs, "removed-link").diff_type,
        DiffType::Incomparable
    );
    assert_eq!(find(&diffs, "same-link").diff_type, DiffType::Incomparable);
}

#[cfg(unix)]
#[test]
fn relative_inside_links_and_link_type_conflicts_are_incomparable() {
    use std::os::unix::fs::symlink;

    let before = tmp_dir();
    let after = tmp_dir();
    write(after.path(), "inside-file", "ordinary-inside");
    write(after.path(), "inside-dir/descendant", "ordinary-descendant");
    symlink("inside-file", after.path().join("relative-file-link")).unwrap();
    symlink("inside-dir", after.path().join("relative-dir-link")).unwrap();
    write(before.path(), "link-vs-file", "regular-before");
    fs::create_dir(before.path().join("link-vs-dir")).unwrap();
    symlink("inside-file", after.path().join("link-vs-file")).unwrap();
    symlink("inside-dir", after.path().join("link-vs-dir")).unwrap();

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    for name in ["relative-file-link", "relative-dir-link", "link-vs-file", "link-vs-dir"] {
        let entry = find(&diffs, name);
        assert_eq!(entry.diff_type, DiffType::Incomparable);
        assert!(entry.error_detail.as_deref().unwrap().starts_with("[AAAI-PATH-LINK]"));
        assert!(entry.before_sha256.is_none() && entry.after_sha256.is_none());
    }
    assert!(!diffs.iter().any(|entry| entry.path == "relative-dir-link/descendant"));
}

#[cfg(unix)]
#[test]
fn ignored_file_link_is_omitted_without_reading_target() {
    use std::os::unix::fs::symlink;
    use crate::diff::ignore::IgnoreRules;

    let before = tmp_dir();
    let after = tmp_dir();
    let outside = tmp_dir();
    write(outside.path(), "canary", "outside-secret-content");
    symlink(outside.path().join("canary"), after.path().join("ignored-file")).unwrap();
    let ignore = IgnoreRules::from_str("ignored-file\n").unwrap();
    let diffs = DiffEngine::compare_with_ignore(before.path(), after.path(), &ignore).unwrap();
    assert!(diffs.is_empty());
}

#[cfg(unix)]
#[test]
fn link_audit_preserves_namespace_content_and_stable_metadata() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};

    fn stable(path: &std::path::Path) -> (u64, u32, u64, u64, std::time::SystemTime, bool, bool) {
        let metadata = fs::symlink_metadata(path).unwrap();
        (
            metadata.len(), metadata.permissions().mode(), metadata.dev(), metadata.ino(),
            metadata.modified().unwrap(), metadata.file_type().is_file(), metadata.file_type().is_symlink(),
        )
    }

    let before = tmp_dir();
    let after = tmp_dir();
    let outside = tmp_dir();
    let canary = outside.path().join("canary");
    let link = after.path().join("linked");
    write(outside.path(), "canary", "outside-secret-canary");
    symlink(&canary, &link).unwrap();
    let before_canary = (fs::read(&canary).unwrap(), stable(&canary));
    let before_link = stable(&link);
    let before_entries: Vec<_> = fs::read_dir(after.path()).unwrap().map(|entry| entry.unwrap().file_name()).collect();

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    assert_eq!(find(&diffs, "linked").diff_type, DiffType::Incomparable);

    assert_eq!((fs::read(&canary).unwrap(), stable(&canary)), before_canary);
    assert_eq!(stable(&link), before_link);
    let after_entries: Vec<_> = fs::read_dir(after.path()).unwrap().map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(after_entries, before_entries);
}

#[cfg(windows)]
fn assert_windows_reparse(after: &std::path::Path, name: &str) {
    use std::os::windows::fs::{MetadataExt as _, OpenOptionsExt as _};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    };

    let file = fs::OpenOptions::new()
        .access_mode(0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(after.join(name))
        .unwrap();
    let metadata = file.metadata().unwrap();
    assert_ne!(
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT,
        0,
        "fixture must be a reparse point"
    );
    let before = tmp_dir();
    let diffs = DiffEngine::compare(before.path(), after).unwrap();
    let entry = find(&diffs, name);
    assert_eq!(entry.diff_type, DiffType::Incomparable);
    assert_eq!(
        entry.error_detail.as_deref(),
        Some("[AAAI-PATH-REPARSE] Windows reparse points are not read.")
    );
    assert!(entry.before_sha256.is_none() && entry.after_sha256.is_none());
}

#[cfg(windows)]
#[test]
fn windows_file_and_directory_symlinks_are_reparse_errors() {
    use std::os::windows::fs::{symlink_dir, symlink_file};

    let after = tmp_dir();
    let outside = tmp_dir();
    write(outside.path(), "canary", "outside-secret-canary");
    fs::create_dir(outside.path().join("folder")).unwrap();
    symlink_file(
        outside.path().join("canary"),
        after.path().join("file-link"),
    )
    .expect("hosted Windows file-symlink fixture");
    symlink_dir(outside.path().join("folder"), after.path().join("dir-link"))
        .expect("hosted Windows directory-symlink fixture");
    assert_windows_reparse(after.path(), "file-link");
    assert_windows_reparse(after.path(), "dir-link");
    let before = tmp_dir();
    let root_error = DiffEngine::compare(&after.path().join("dir-link"), before.path())
        .expect_err("selected reparse root must be rejected");
    assert!(root_error.to_string().contains("AAAI-ROOT-UNAVAILABLE"));
    for bypass in [
        after.path().join("dir-link").join("."),
        after.path().join("dir-link").join("subdir").join(".."),
    ] {
        let error = DiffEngine::compare(&bypass, before.path())
            .expect_err("selected reparse-root bypass must fail");
        assert!(error.to_string().contains("AAAI-ROOT-UNAVAILABLE"));
    }
}

#[cfg(windows)]
#[test]
fn windows_link_matrix_remains_metadata_only() {
    use std::os::windows::fs::{symlink_dir, symlink_file};
    use crate::diff::ignore::IgnoreRules;

    let before = tmp_dir();
    let after = tmp_dir();
    let outside = tmp_dir();
    write(after.path(), "inside-target", "ordinary-inside");
    write(outside.path(), "canary", "outside-secret-content");
    fs::create_dir(outside.path().join("folder")).unwrap();
    write(outside.path(), "folder/outside-descendant-name", "outside-descendant-content");
    write(after.path(), "inside-dir/inside-descendant", "ordinary-descendant");

    symlink_file("inside-target", after.path().join("relative-inside")).expect("Windows relative symlink fixture");
    symlink_file(outside.path().join("canary"), after.path().join("absolute-outside")).expect("Windows absolute symlink fixture");
    symlink_file("missing-target", after.path().join("broken")).expect("Windows broken symlink fixture");
    symlink_file("cycle-b", after.path().join("cycle-a")).expect("Windows cycle fixture");
    symlink_file("cycle-a", after.path().join("cycle-b")).expect("Windows cycle fixture");
    symlink_file(outside.path().join("canary"), before.path().join("removed")).expect("Windows removed symlink fixture");
    symlink_file(outside.path().join("canary"), before.path().join("same")).expect("Windows same-path symlink fixture");
    symlink_file(outside.path().join("canary"), after.path().join("same")).expect("Windows same-path symlink fixture");
    symlink_dir(outside.path().join("folder"), after.path().join("ignored-dir")).expect("Windows ignored directory-link fixture");
    symlink_file(outside.path().join("canary"), after.path().join("ignored-file")).expect("Windows ignored file-link fixture");
    symlink_dir(outside.path().join("folder"), after.path().join("outside-dir")).expect("Windows outside directory-link fixture");
    symlink_dir("inside-dir", after.path().join("inside-dir-link")).expect("Windows inside directory-link fixture");
    symlink_dir("self-dir", after.path().join("self-dir")).expect("Windows self-cycle fixture");
    write(before.path(), "link-vs-file", "regular-before");
    fs::create_dir(before.path().join("link-vs-dir")).unwrap();
    symlink_file("inside-target", after.path().join("link-vs-file")).expect("Windows file conflict fixture");
    symlink_dir("inside-dir", after.path().join("link-vs-dir")).expect("Windows directory conflict fixture");

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    for name in [
        "relative-inside", "absolute-outside", "broken", "cycle-a", "cycle-b",
        "removed", "same", "ignored-dir", "ignored-file", "outside-dir", "inside-dir-link", "self-dir",
        "link-vs-file", "link-vs-dir",
    ] {
        let entry = find(&diffs, name);
        assert_eq!(entry.diff_type, DiffType::Incomparable);
        assert!(entry.error_detail.as_deref().unwrap().starts_with("[AAAI-PATH-REPARSE]"));
        assert!(entry.before_sha256.is_none() && entry.after_sha256.is_none());
    }
    let rendered = diffs.iter().map(|entry| format!("{}{:?}", entry.path, entry.error_detail)).collect::<String>();
    assert!(!rendered.contains("outside-secret-content"));
    assert!(!rendered.contains("outside-descendant-name"));
    assert!(!rendered.contains("outside-descendant-content"));
    assert!(!diffs.iter().any(|entry| entry.path == "outside-dir/outside-descendant-name"));
    assert!(!rendered.contains(&outside.path().display().to_string()));

    let ignore = IgnoreRules::from_str("ignored-dir\nignored-file\n").unwrap();
    let ignored = DiffEngine::compare_with_ignore(before.path(), after.path(), &ignore).unwrap();
    assert!(!ignored.iter().any(|entry| entry.path == "ignored-dir"));
    assert!(!ignored.iter().any(|entry| entry.path == "ignored-file"));
    assert!(!diffs.iter().any(|entry| entry.path == "inside-dir-link/inside-descendant"));
}

#[cfg(windows)]
#[test]
fn windows_link_audit_preserves_namespace_content_and_stable_metadata() {
    use std::os::windows::fs::{MetadataExt, symlink_file};

    fn stable(path: &std::path::Path) -> (u64, u32, u64, std::time::SystemTime, bool, bool) {
        let metadata = fs::symlink_metadata(path).unwrap();
        (
            metadata.file_size(), metadata.file_attributes(), metadata.last_write_time(),
            metadata.modified().unwrap(), metadata.permissions().readonly(),
            metadata.file_type().is_symlink(),
        )
    }

    let before = tmp_dir();
    let after = tmp_dir();
    let outside = tmp_dir();
    let canary = outside.path().join("canary");
    let link = after.path().join("linked");
    write(outside.path(), "canary", "outside-secret-content");
    symlink_file(&canary, &link).expect("hosted Windows non-mutation symlink fixture");
    let before_canary = (fs::read(&canary).unwrap(), stable(&canary));
    let before_link = stable(&link);
    let before_entries: Vec<_> = fs::read_dir(after.path()).unwrap().map(|entry| entry.unwrap().file_name()).collect();

    let diffs = DiffEngine::compare(before.path(), after.path()).unwrap();
    assert_eq!(find(&diffs, "linked").diff_type, DiffType::Incomparable);
    assert_eq!((fs::read(&canary).unwrap(), stable(&canary)), before_canary);
    assert_eq!(stable(&link), before_link);
    let after_entries: Vec<_> = fs::read_dir(after.path()).unwrap().map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(after_entries, before_entries);
}

#[cfg(windows)]
#[test]
fn windows_junction_is_a_reparse_error() {
    use std::process::Command;

    let after = tmp_dir();
    let outside = tmp_dir();
    let status = Command::new("cmd.exe")
        .args(["/C", "mklink", "/J"])
        .arg(after.path().join("junction"))
        .arg(outside.path())
        .status()
        .expect("hosted Windows junction fixture");
    assert!(
        status.success(),
        "hosted Windows junction fixture is required"
    );
    assert_windows_reparse(after.path(), "junction");
}

// RFC 098 §9.1 — the only behavioural control for the regression where
// production's `windows_reparse` decision is based on the reparse attribute
// (correct) rather than `is_symlink()` (which is blind to non-name-surrogate
// tags). The tag (0x00000042) clears both the Microsoft-owned bit
// (0x80000000) and the name-surrogate bit (0x20000000), so NTFS accepts it
// from user mode via `REPARSE_GUID_DATA_BUFFER`, and Rust's `is_symlink()`
// — which checks the tag, not just the attribute — must be false for it.
//
// The raw `DeviceIoControl`/`FSCTL_SET_REPARSE_POINT` call happens entirely
// in an external PowerShell/C# helper process, never inside this crate: see
// `rfcs/handoffs/098-selected-folder-and-symlink-policy/part2-behavioural-reparse-fixture.md`
// §5 B1 — this is not project `unsafe` and not FFI in `crates/` (SEC-1,
// DEC-012 unaffected).
#[cfg(windows)]
const AAAI_REPARSE_HELPER_SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class Aaai098ReparseHelper {
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    public static extern IntPtr CreateFileW(
        string lpFileName, uint dwDesiredAccess, uint dwShareMode,
        IntPtr lpSecurityAttributes, uint dwCreationDisposition,
        uint dwFlagsAndAttributes, IntPtr hTemplateFile);

    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern bool DeviceIoControl(
        IntPtr hDevice, uint dwIoControlCode,
        byte[] lpInBuffer, uint nInBufferSize,
        IntPtr lpOutBuffer, uint nOutBufferSize,
        out uint lpBytesReturned, IntPtr lpOverlapped);

    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern bool CloseHandle(IntPtr hObject);

    const uint GENERIC_WRITE = 0x40000000;
    const uint FILE_SHARE_READ = 0x1;
    const uint FILE_SHARE_WRITE = 0x2;
    const uint OPEN_EXISTING = 3;
    const uint FILE_FLAG_BACKUP_SEMANTICS = 0x02000000;
    const uint FILE_FLAG_OPEN_REPARSE_POINT = 0x00200000;
    const uint FSCTL_SET_REPARSE_POINT = 0x000900A4;
    const uint FSCTL_DELETE_REPARSE_POINT = 0x000900AC;
    // Clears bit 31 (Microsoft-owned) and bit 29 (name surrogate).
    const uint AAAI_REPARSE_TAG = 0x00000042;

    // Fixed (not random) so Set and Delete always agree on the GUID NTFS
    // stores alongside a non-Microsoft reparse tag.
    static readonly byte[] ReparseGuid = new byte[16] {
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10
    };

    static IntPtr OpenForControl(string path) {
        return CreateFileW(path, GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE,
            IntPtr.Zero, OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT, IntPtr.Zero);
    }

    public static int SetReparse(string path) {
        IntPtr h = OpenForControl(path);
        if (h == new IntPtr(-1)) return Marshal.GetLastWin32Error();
        try {
            byte[] payload = new byte[] { 0xAA, 0xBB, 0xCC, 0xDD };
            byte[] buffer = new byte[24 + payload.Length];
            BitConverter.GetBytes(AAAI_REPARSE_TAG).CopyTo(buffer, 0);
            BitConverter.GetBytes((ushort)payload.Length).CopyTo(buffer, 4);
            BitConverter.GetBytes((ushort)0).CopyTo(buffer, 6);
            ReparseGuid.CopyTo(buffer, 8);
            payload.CopyTo(buffer, 24);

            uint bytesReturned;
            bool ok = DeviceIoControl(h, FSCTL_SET_REPARSE_POINT, buffer, (uint)buffer.Length,
                IntPtr.Zero, 0, out bytesReturned, IntPtr.Zero);
            return ok ? 0 : Marshal.GetLastWin32Error();
        } finally {
            CloseHandle(h);
        }
    }

    public static int DeleteReparse(string path) {
        IntPtr h = OpenForControl(path);
        if (h == new IntPtr(-1)) return Marshal.GetLastWin32Error();
        try {
            byte[] buffer = new byte[24];
            BitConverter.GetBytes(AAAI_REPARSE_TAG).CopyTo(buffer, 0);
            BitConverter.GetBytes((ushort)0).CopyTo(buffer, 4);
            BitConverter.GetBytes((ushort)0).CopyTo(buffer, 6);
            ReparseGuid.CopyTo(buffer, 8);

            uint bytesReturned;
            bool ok = DeviceIoControl(h, FSCTL_DELETE_REPARSE_POINT, buffer, (uint)buffer.Length,
                IntPtr.Zero, 0, out bytesReturned, IntPtr.Zero);
            return ok ? 0 : Marshal.GetLastWin32Error();
        } finally {
            CloseHandle(h);
        }
    }
}
'@

$result = [Aaai098ReparseHelper]::__AAAI_OP__('__AAAI_PATH__')
Write-Output "AAAI-RESULT:$result"
"#;

/// Runs the external PowerShell/C# helper that issues the raw
/// `DeviceIoControl` call — the only place in this project's Windows test
/// suite that touches `FSCTL_SET_REPARSE_POINT`/`FSCTL_DELETE_REPARSE_POINT`
/// directly, and it runs outside the crate entirely (see the module comment
/// above `AAAI_REPARSE_HELPER_SCRIPT`).
///
/// Returns `Ok(())` on success, `Err(win32_error_code)` if the underlying
/// `DeviceIoControl` call failed. Panics (with the process's stdout/stderr)
/// if the helper process itself could not run to completion, since a silent
/// or generic failure here would cost a hosted CI cycle to diagnose.
#[cfg(windows)]
fn run_reparse_helper(path: &std::path::Path, op: &str) -> Result<(), u32> {
    use std::process::Command;

    let path_str = path.to_str().expect("temp path must be valid UTF-8");
    let escaped_path = path_str.replace('\'', "''");
    let script = AAAI_REPARSE_HELPER_SCRIPT
        .replace("__AAAI_OP__", op)
        .replace("__AAAI_PATH__", &escaped_path);

    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .output()
        .expect("hosted Windows PowerShell reparse helper must be invocable");

    if !output.status.success() {
        panic!(
            "reparse helper PowerShell process failed (exit {:?}): stdout={} stderr={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result_line = stdout
        .lines()
        .find(|line| line.starts_with("AAAI-RESULT:"))
        .unwrap_or_else(|| {
            panic!(
                "reparse helper produced no result line: stdout={stdout} stderr={}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
    let code: u32 = result_line["AAAI-RESULT:".len()..]
        .trim()
        .parse()
        .unwrap_or_else(|e| {
            panic!("reparse helper result line not parseable ({e}): {result_line}")
        });

    if code == 0 { Ok(()) } else { Err(code) }
}

#[cfg(windows)]
#[test]
fn windows_non_name_surrogate_reparse_is_rejected() {
    use std::os::windows::fs::{MetadataExt as _, OpenOptionsExt as _};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    };

    let after = tmp_dir();
    let target = after.path().join("non-name-surrogate");
    // B1 — create the target as an empty file first.
    fs::write(&target, b"").unwrap();

    if let Err(code) = run_reparse_helper(&target, "SetReparse") {
        panic!(
            "FSCTL_SET_REPARSE_POINT failed with Win32 error {code} (0x{code:08X}) \
             — capture this and escalate per RFC 098 Part 2 handoff §8; \
             do not substitute a structural guard on your own initiative"
        );
    }

    // B2 — the assertion this fixture uniquely enables, checked directly
    // before reusing the accepted `assert_windows_reparse` helper below.
    let file = fs::OpenOptions::new()
        .access_mode(0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(&target)
        .unwrap();
    let metadata = file.metadata().unwrap();
    assert!(metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0);
    assert!(
        !metadata.file_type().is_symlink(),
        "fixture must be a NON-name-surrogate reparse point"
    );
    drop(file);

    assert_windows_reparse(after.path(), "non-name-surrogate");

    // B3 — explicit cleanup rather than relying on `TempDir` drop; removal
    // of a file carrying an unrecognized tag is an untested path, so a
    // cleanup failure must surface as a test failure, not a leaked temp dir.
    run_reparse_helper(&target, "DeleteReparse").unwrap_or_else(|code| {
        panic!("cleanup: FSCTL_DELETE_REPARSE_POINT failed with Win32 error {code}")
    });
    fs::remove_file(&target)
        .expect("cleanup: reparse point file must be removable after tag delete");
}
