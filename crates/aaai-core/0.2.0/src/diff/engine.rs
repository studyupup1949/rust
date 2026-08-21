//! Folder diff engine.
//!
//! Walks both directory trees, compares metadata first, then content for
//! Modified candidates.  Produces a stable, sorted Vec<DiffEntry>.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::entry::{DiffEntry, DiffType};

/// Compares two directory trees and produces a sorted list of DiffEntry.
pub struct DiffEngine;

impl DiffEngine {
    /// Run a full comparison between `before_root` and `after_root`.
    ///
    /// All errors on individual files are surfaced as DiffType::Unreadable
    /// entries; the overall function only fails if a root directory is
    /// inaccessible.
    pub fn compare(before_root: &Path, after_root: &Path) -> anyhow::Result<Vec<DiffEntry>> {
        let before_map = collect_paths(before_root)?;
        let after_map = collect_paths(after_root)?;

        let all_paths: BTreeSet<String> = before_map
            .keys()
            .chain(after_map.keys())
            .cloned()
            .collect();

        let mut entries = Vec::new();

        for rel_path in all_paths {
            let before_abs = before_map.get(&rel_path);
            let after_abs = after_map.get(&rel_path);

            let entry = match (before_abs, after_abs) {
                (None, Some(after)) => build_added(rel_path, after),
                (Some(before), None) => build_removed(rel_path, before),
                (Some(before), Some(after)) => build_compared(rel_path, before, after),
                (None, None) => unreachable!(),
            };
            entries.push(entry);
        }

        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }
}

// ── helpers ──────────────────────────────────────────────────────────────

/// Collect relative paths from a root directory.
/// Key: unix-style relative path ("foo/bar.toml")
/// Value: absolute PathBuf
fn collect_paths(root: &Path) -> anyhow::Result<BTreeMap<String, PathBuf>> {
    if !root.is_dir() {
        anyhow::bail!("Not a directory: {}", root.display());
    }
    let mut map = BTreeMap::new();
    for entry in WalkDir::new(root).into_iter() {
        let entry = entry.map_err(|e| anyhow::anyhow!("Walk error: {e}"))?;
        if entry.path() == root {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        map.insert(rel, entry.path().to_path_buf());
    }
    Ok(map)
}

fn build_added(rel_path: String, after: &Path) -> DiffEntry {
    let is_dir = after.is_dir();
    let (after_text, after_sha256, error_detail) = if is_dir {
        (None, None, None)
    } else {
        read_text_and_hash(after)
    };
    DiffEntry {
        path: rel_path,
        diff_type: DiffType::Added,
        is_dir,
        before_text: None,
        after_text,
        after_sha256,
        error_detail,
    }
}

fn build_removed(rel_path: String, before: &Path) -> DiffEntry {
    let is_dir = before.is_dir();
    let (before_text, _, error_detail) = if is_dir {
        (None, None, None)
    } else {
        read_text_and_hash(before)
    };
    DiffEntry {
        path: rel_path,
        diff_type: DiffType::Removed,
        is_dir,
        before_text,
        after_text: None,
        after_sha256: None,
        error_detail,
    }
}

fn build_compared(rel_path: String, before: &Path, after: &Path) -> DiffEntry {
    // Type mismatch?
    let before_is_dir = before.is_dir();
    let after_is_dir = after.is_dir();
    if before_is_dir != after_is_dir {
        return DiffEntry {
            path: rel_path,
            diff_type: DiffType::TypeChanged,
            is_dir: false,
            before_text: None,
            after_text: None,
            after_sha256: None,
            error_detail: Some("Path kind changed (file ↔ directory).".into()),
        };
    }
    if before_is_dir {
        // Directories: exist on both sides — Unchanged at the dir level.
        return DiffEntry {
            path: rel_path,
            diff_type: DiffType::Unchanged,
            is_dir: true,
            before_text: None,
            after_text: None,
            after_sha256: None,
            error_detail: None,
        };
    }

    // Read both files.
    let before_bytes = match std::fs::read(before) {
        Ok(b) => b,
        Err(e) => {
            return DiffEntry {
                path: rel_path,
                diff_type: DiffType::Unreadable,
                is_dir: false,
                before_text: None,
                after_text: None,
                after_sha256: None,
                error_detail: Some(format!("Cannot read before-file: {e}")),
            };
        }
    };
    let after_bytes = match std::fs::read(after) {
        Ok(b) => b,
        Err(e) => {
            return DiffEntry {
                path: rel_path,
                diff_type: DiffType::Unreadable,
                is_dir: false,
                before_text: None,
                after_text: None,
                after_sha256: None,
                error_detail: Some(format!("Cannot read after-file: {e}")),
            };
        }
    };

    let after_sha256 = hex::encode(Sha256::digest(&after_bytes));

    let diff_type = if before_bytes == after_bytes {
        DiffType::Unchanged
    } else {
        DiffType::Modified
    };

    let before_text = String::from_utf8(before_bytes).ok();
    let after_text = String::from_utf8(after_bytes).ok();

    DiffEntry {
        path: rel_path,
        diff_type,
        is_dir: false,
        before_text,
        after_text,
        after_sha256: Some(after_sha256),
        error_detail: None,
    }
}

/// Read a file and return (text_option, sha256_option, error_option).
fn read_text_and_hash(path: &Path) -> (Option<String>, Option<String>, Option<String>) {
    match std::fs::read(path) {
        Ok(bytes) => {
            let sha = hex::encode(Sha256::digest(&bytes));
            let text = String::from_utf8(bytes).ok();
            (text, Some(sha), None)
        }
        Err(e) => (None, None, Some(format!("Cannot read file: {e}"))),
    }
}
