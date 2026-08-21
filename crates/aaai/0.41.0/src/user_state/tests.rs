use std::cell::Cell;
use std::ffi::OsString;
use std::path::PathBuf;

use super::UserStatePaths;

fn counted_default<'a>(
    calls: &'a Cell<usize>,
    path: PathBuf,
) -> impl FnOnce() -> Option<PathBuf> + 'a {
    move || {
        calls.set(calls.get() + 1);
        Some(path)
    }
}

#[test]
fn absent_override_uses_default_once() {
    let calls = Cell::new(0);
    let base = std::env::temp_dir().join("aaai-default-config");
    let paths = UserStatePaths::resolve_from(None, counted_default(&calls, base.clone())).unwrap();

    assert_eq!(calls.get(), 1);
    assert_eq!(paths.root(), base.join("aaai"));
}

#[test]
fn absolute_override_skips_default() {
    let calls = Cell::new(0);
    let root = std::env::temp_dir().join("aaai-override");
    let paths = UserStatePaths::resolve_from(
        Some(root.clone().into_os_string()),
        counted_default(&calls, PathBuf::from("unused")),
    )
    .unwrap();

    assert_eq!(calls.get(), 0);
    assert_eq!(paths.root(), root);
}

#[test]
fn empty_override_fails_without_default() {
    let calls = Cell::new(0);
    let result = UserStatePaths::resolve_from(
        Some(OsString::new()),
        counted_default(&calls, PathBuf::from("unused")),
    );

    assert!(result.is_err());
    assert_eq!(calls.get(), 0);
}

#[test]
fn relative_override_fails_without_default() {
    let calls = Cell::new(0);
    let result = UserStatePaths::resolve_from(
        Some(OsString::from("relative/state")),
        counted_default(&calls, PathBuf::from("unused")),
    );

    assert!(result.is_err());
    assert_eq!(calls.get(), 0);
}

#[cfg(unix)]
#[test]
fn non_unicode_absolute_override_skips_default() {
    use std::os::unix::ffi::OsStringExt;

    let calls = Cell::new(0);
    let root = OsString::from_vec(b"/tmp/aaai-\xFF-state".to_vec());
    let paths = UserStatePaths::resolve_from(
        Some(root.clone()),
        counted_default(&calls, PathBuf::from("unused")),
    )
    .unwrap();

    assert_eq!(calls.get(), 0);
    assert_eq!(paths.root(), PathBuf::from(root));
}

#[cfg(unix)]
#[test]
fn non_unicode_relative_override_fails_without_default() {
    use std::os::unix::ffi::OsStringExt;

    let calls = Cell::new(0);
    let root = OsString::from_vec(b"relative-\xFF-state".to_vec());
    let result =
        UserStatePaths::resolve_from(Some(root), counted_default(&calls, PathBuf::from("unused")));

    assert!(result.is_err());
    assert_eq!(calls.get(), 0);
}

#[test]
fn resolving_paths_does_not_create_root() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("missing-state");
    let paths = UserStatePaths::from_root(&root).unwrap();

    assert_eq!(paths.history(), root.join("history.jsonl"));
    assert_eq!(paths.profiles(), root.join("profiles.yaml"));
    assert_eq!(paths.prefs(), root.join("prefs.yaml"));
    assert!(!root.exists());
}

#[test]
fn ensure_for_write_creates_root() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("state");
    let paths = UserStatePaths::from_root(&root).unwrap();

    paths.ensure_for_write().unwrap();

    assert!(root.is_dir());
}
