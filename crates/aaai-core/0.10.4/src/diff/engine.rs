//! Folder diff engine — Phase 4: parallel processing + binary detection + diff stats.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use super::entry::{DiffEntry, DiffStats, DiffType};
use super::ignore::IgnoreRules;
use super::progress::{DiffProgress, NullProgress, ProgressSink};

pub struct DiffEngine;

impl DiffEngine {
    /// Compare two directory trees (sequential — for small trees).
    pub fn compare(before_root: &Path, after_root: &Path) -> anyhow::Result<Vec<DiffEntry>> {
        Self::compare_with_ignore(before_root, after_root, &IgnoreRules::default())
    }

    /// Compare with ignore rules.
    /// Uses parallel processing for the per-file comparison step.
    pub fn compare_with_ignore(
        before_root: &Path,
        after_root: &Path,
        ignore: &IgnoreRules,
    ) -> anyhow::Result<Vec<DiffEntry>> {
        Self::compare_with_progress(before_root, after_root, ignore, &NullProgress)
    }

    /// Compare with ignore rules and a progress sink.
    pub fn compare_with_progress(
        before_root: &Path,
        after_root: &Path,
        ignore: &IgnoreRules,
        progress: &dyn ProgressSink,
    ) -> anyhow::Result<Vec<DiffEntry>> {
        let before_map = collect_paths(before_root)?;
        let after_map  = collect_paths(after_root)?;

        let all_paths: BTreeSet<String> = before_map.keys()
            .chain(after_map.keys())
            .cloned()
            .collect();

        // Filter ignored paths eagerly.
        let paths_to_compare: Vec<String> = all_paths
            .into_iter()
            .filter(|p| !ignore.is_ignored(p))
            .collect();

        let total = paths_to_compare.len();
        progress.emit(DiffProgress::Started { total });

        // ── Parallel per-file comparison ───────────────────────────────────
        use std::sync::atomic::{AtomicUsize, Ordering};
        let processed = AtomicUsize::new(0);

        let mut entries: Vec<DiffEntry> = paths_to_compare
            .into_par_iter()
            .map(|rel_path| {
                let diff_entry = match (before_map.get(&rel_path), after_map.get(&rel_path)) {
                    (None,    Some(a)) => build_added(rel_path, a),
                    (Some(b), None)    => build_removed(rel_path, b),
                    (Some(b), Some(a)) => build_compared(rel_path, b, a),
                    (None,    None)    => unreachable!(),
                };
                let n = processed.fetch_add(1, Ordering::Relaxed) + 1;
                progress.emit(DiffProgress::File {
                    path: diff_entry.path.clone(), processed: n, total,
                });
                diff_entry
            })
            .collect();

        // Restore deterministic sort (parallel iter may reorder).
        progress.emit(DiffProgress::Sorting);
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        progress.emit(DiffProgress::Done { total_files: entries.len() });
        Ok(entries)
    }
}

// ── Path collection ───────────────────────────────────────────────────────

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

// ── Per-file builders ─────────────────────────────────────────────────────

fn build_added(rel: String, after: &Path) -> DiffEntry {
    if after.is_dir() {
        return dir_entry(rel, DiffType::Added);
    }
    let (bytes, sha, size, error) = read_file(after);
    let (text, is_binary) = classify_bytes(&bytes);
    DiffEntry {
        path: rel, diff_type: DiffType::Added, is_dir: false,
        before_text: None, after_text: text.clone(),
        is_binary,
        before_size: None, after_size: size,
        before_sha256: None, after_sha256: sha,
        stats: None, // no before to diff against
        error_detail: error,
    }
}

fn build_removed(rel: String, before: &Path) -> DiffEntry {
    if before.is_dir() {
        return dir_entry(rel, DiffType::Removed);
    }
    let (bytes, sha, size, error) = read_file(before);
    let (text, is_binary) = classify_bytes(&bytes);
    DiffEntry {
        path: rel, diff_type: DiffType::Removed, is_dir: false,
        before_text: text, after_text: None,
        is_binary,
        before_size: size, after_size: None,
        before_sha256: sha, after_sha256: None,
        stats: None,
        error_detail: error,
    }
}

fn build_compared(rel: String, before: &Path, after: &Path) -> DiffEntry {
    if before.is_dir() != after.is_dir() {
        return DiffEntry {
            path: rel, diff_type: DiffType::TypeChanged, is_dir: false,
            before_text: None, after_text: None,
            is_binary: false,
            before_size: None, after_size: None,
            before_sha256: None, after_sha256: None,
            stats: None,
            error_detail: Some("Path kind changed (file ↔ directory).".into()),
        };
    }
    if before.is_dir() {
        return dir_entry(rel, DiffType::Unchanged);
    }

    let (before_bytes, before_sha, before_size, before_err) = read_file(before);
    if let Some(e) = before_err {
        return unreadable(rel, format!("Cannot read before-file: {e}"));
    }
    let (after_bytes, after_sha, after_size, after_err) = read_file(after);
    if let Some(e) = after_err {
        return unreadable(rel, format!("Cannot read after-file: {e}"));
    }

    let diff_type = if before_bytes == after_bytes { DiffType::Unchanged } else { DiffType::Modified };

    let (before_text, before_is_binary) = classify_bytes(&before_bytes);
    let (after_text,  after_is_binary)  = classify_bytes(&after_bytes);
    let is_binary = before_is_binary || after_is_binary;

    // Compute line stats for text-Modified files.
    let stats = if diff_type == DiffType::Modified && !is_binary {
        let bt = before_text.as_deref().unwrap_or("");
        let at = after_text.as_deref().unwrap_or("");
        Some(DiffStats::compute(bt, at))
    } else {
        None
    };

    DiffEntry {
        path: rel, diff_type, is_dir: false,
        before_text, after_text,
        is_binary,
        before_size, after_size,
        before_sha256: before_sha, after_sha256: after_sha,
        stats,
        error_detail: None,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn dir_entry(rel: String, diff_type: DiffType) -> DiffEntry {
    DiffEntry {
        path: rel, diff_type, is_dir: true,
        before_text: None, after_text: None,
        is_binary: false,
        before_size: None, after_size: None,
        before_sha256: None, after_sha256: None,
        stats: None, error_detail: None,
    }
}

fn unreadable(rel: String, detail: String) -> DiffEntry {
    DiffEntry {
        path: rel, diff_type: DiffType::Unreadable, is_dir: false,
        before_text: None, after_text: None,
        is_binary: false,
        before_size: None, after_size: None,
        before_sha256: None, after_sha256: None,
        stats: None, error_detail: Some(detail),
    }
}

/// Read a file returning (bytes, sha256_hex, size_bytes, error).
fn read_file(path: &Path) -> (Vec<u8>, Option<String>, Option<u64>, Option<String>) {
    match std::fs::read(path) {
        Ok(bytes) => {
            let sha  = hex::encode(Sha256::digest(&bytes));
            let size = bytes.len() as u64;
            (bytes, Some(sha), Some(size), None)
        }
        Err(e) => (Vec::new(), None, None, Some(e.to_string())),
    }
}

/// Classify bytes as text or binary.
/// Returns (text_content, is_binary).
fn classify_bytes(bytes: &[u8]) -> (Option<String>, bool) {
    if bytes.is_empty() {
        return (Some(String::new()), false);
    }
    // Heuristic: if any of the first 8 KB contains a null byte, treat as binary.
    let sample = &bytes[..bytes.len().min(8192)];
    if sample.contains(&0u8) {
        return (None, true);
    }
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => (Some(text), false),
        Err(_)   => (None, true),
    }
}
