//! Folder diff engine with optional ignore-pattern support.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::entry::{DiffEntry, DiffType};
use super::ignore::IgnoreRules;

pub struct DiffEngine;

impl DiffEngine {
    /// Compare two directory trees.
    /// Paths matching `ignore` rules are excluded from the result.
    pub fn compare(before_root: &Path, after_root: &Path) -> anyhow::Result<Vec<DiffEntry>> {
        Self::compare_with_ignore(before_root, after_root, &IgnoreRules::default())
    }

    /// Compare with explicit ignore rules.
    pub fn compare_with_ignore(
        before_root: &Path,
        after_root: &Path,
        ignore: &IgnoreRules,
    ) -> anyhow::Result<Vec<DiffEntry>> {
        let before_map = collect_paths(before_root)?;
        let after_map  = collect_paths(after_root)?;

        let all_paths: BTreeSet<String> = before_map.keys()
            .chain(after_map.keys())
            .cloned()
            .collect();

        let mut entries = Vec::new();
        for rel_path in all_paths {
            if ignore.is_ignored(&rel_path) {
                log::debug!("ignored: {rel_path}");
                continue;
            }
            let entry = match (before_map.get(&rel_path), after_map.get(&rel_path)) {
                (None,    Some(a)) => build_added(rel_path, a),
                (Some(b), None)    => build_removed(rel_path, b),
                (Some(b), Some(a)) => build_compared(rel_path, b, a),
                (None,    None)    => unreachable!(),
            };
            entries.push(entry);
        }

        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }
}

fn collect_paths(root: &Path) -> anyhow::Result<BTreeMap<String, PathBuf>> {
    if !root.is_dir() {
        anyhow::bail!("Not a directory: {}", root.display());
    }
    let mut map = BTreeMap::new();
    for entry in WalkDir::new(root).into_iter() {
        let entry = entry.map_err(|e| anyhow::anyhow!("Walk error: {e}"))?;
        if entry.path() == root { continue; }
        let rel = entry.path()
            .strip_prefix(root).unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        map.insert(rel, entry.path().to_path_buf());
    }
    Ok(map)
}

fn build_added(rel: String, after: &Path) -> DiffEntry {
    let is_dir = after.is_dir();
    let (after_text, after_sha256, error_detail) =
        if is_dir { (None, None, None) } else { read_text_and_hash(after) };
    DiffEntry { path: rel, diff_type: DiffType::Added, is_dir,
                before_text: None, after_text, after_sha256, error_detail }
}

fn build_removed(rel: String, before: &Path) -> DiffEntry {
    let is_dir = before.is_dir();
    let (before_text, _, error_detail) =
        if is_dir { (None, None, None) } else { read_text_and_hash(before) };
    DiffEntry { path: rel, diff_type: DiffType::Removed, is_dir,
                before_text, after_text: None, after_sha256: None, error_detail }
}

fn build_compared(rel: String, before: &Path, after: &Path) -> DiffEntry {
    let before_is_dir = before.is_dir();
    let after_is_dir  = after.is_dir();
    if before_is_dir != after_is_dir {
        return DiffEntry {
            path: rel, diff_type: DiffType::TypeChanged, is_dir: false,
            before_text: None, after_text: None, after_sha256: None,
            error_detail: Some("Path kind changed (file ↔ directory).".into()),
        };
    }
    if before_is_dir {
        return DiffEntry { path: rel, diff_type: DiffType::Unchanged, is_dir: true,
                           before_text: None, after_text: None, after_sha256: None,
                           error_detail: None };
    }

    let before_bytes = match std::fs::read(before) {
        Ok(b) => b,
        Err(e) => return DiffEntry {
            path: rel, diff_type: DiffType::Unreadable, is_dir: false,
            before_text: None, after_text: None, after_sha256: None,
            error_detail: Some(format!("Cannot read before-file: {e}")),
        },
    };
    let after_bytes = match std::fs::read(after) {
        Ok(b) => b,
        Err(e) => return DiffEntry {
            path: rel, diff_type: DiffType::Unreadable, is_dir: false,
            before_text: None, after_text: None, after_sha256: None,
            error_detail: Some(format!("Cannot read after-file: {e}")),
        },
    };

    let after_sha256 = hex::encode(Sha256::digest(&after_bytes));
    let diff_type = if before_bytes == after_bytes { DiffType::Unchanged } else { DiffType::Modified };
    DiffEntry {
        path: rel, diff_type, is_dir: false,
        before_text: String::from_utf8(before_bytes).ok(),
        after_text: String::from_utf8(after_bytes).ok(),
        after_sha256: Some(after_sha256),
        error_detail: None,
    }
}

fn read_text_and_hash(path: &Path) -> (Option<String>, Option<String>, Option<String>) {
    match std::fs::read(path) {
        Ok(bytes) => {
            let sha = hex::encode(Sha256::digest(&bytes));
            (String::from_utf8(bytes).ok(), Some(sha), None)
        }
        Err(e) => (None, None, Some(format!("Cannot read file: {e}"))),
    }
}
