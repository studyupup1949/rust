//! Folder diff engine — Phase 4: parallel processing + binary detection + diff stats.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::entry::{DiffEntry, DiffStats, DiffType};
use super::ignore::IgnoreRules;
use super::path_boundary::{self, Node, ObservedPath, PathIssue};
use super::progress::{DiffProgress, NullProgress, ProgressSink};
use rayon::prelude::*;
use sha2::{Digest, Sha256};

/// Parallel folder diff engine.
///
/// Walks two directory trees concurrently (using [`rayon`]) and produces a
/// sorted list of [`DiffEntry`] items — one per file that differs between
/// `before` and `after`.
///
/// # Example
///
/// ```rust,no_run
/// use aaai::DiffEngine;
/// use std::path::Path;
///
/// let entries = DiffEngine::compare(Path::new("./before"), Path::new("./after")).unwrap();
/// for e in &entries {
///     println!("{} — {}", e.path, e.diff_type);
/// }
/// ```
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
        let before_map = path_boundary::collect(before_root)?;
        let after_map = path_boundary::collect(after_root)?;

        let all_paths: BTreeSet<PathBuf> =
            before_map.keys().chain(after_map.keys()).cloned().collect();

        // Filter ignored paths eagerly.
        let mut paths_to_compare: Vec<PathBuf> = all_paths
            .into_iter()
            .filter(|p| {
                let display = before_map
                    .get(p)
                    .or_else(|| after_map.get(p))
                    .map(|observed| observed.display.as_str())
                    .unwrap_or_default();
                !ignore.is_ignored(display)
            })
            .collect();
        paths_to_compare.sort_by(|left, right| {
            let left_display = before_map
                .get(left)
                .or_else(|| after_map.get(left))
                .expect("path came from one map")
                .display
                .as_str();
            let right_display = before_map
                .get(right)
                .or_else(|| after_map.get(right))
                .expect("path came from one map")
                .display
                .as_str();
            left_display.cmp(right_display).then_with(|| left.cmp(right))
        });

        let total = paths_to_compare.len();
        progress.emit(DiffProgress::Started { total });

        // ── Parallel per-file comparison ───────────────────────────────────
        let mut entries: Vec<DiffEntry> = paths_to_compare
            .into_par_iter()
            .map(|relative| {
                let before = before_map.get(&relative);
                let after = after_map.get(&relative);
                let display = before
                    .or(after)
                    .expect("path came from one map")
                    .display
                    .clone();
                match (before, after) {
                    (None, Some(a)) => build_added(display, a),
                    (Some(b), None) => build_removed(display, b),
                    (Some(b), Some(a)) => build_compared(display, b, a),
                    (None, None) => unreachable!(),
                }
            })
            .collect();

        // Rayon preserves indexed input order when collecting into a Vec.
        // Emit progress afterward so worker scheduling cannot reorder events.
        for (index, entry) in entries.iter().enumerate() {
            progress.emit(DiffProgress::File {
                path: entry.path.clone(),
                processed: index + 1,
                total,
            });
        }

        // Restore deterministic sort (parallel iter may reorder).
        progress.emit(DiffProgress::Sorting);
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        progress.emit(DiffProgress::Done {
            total_files: entries.len(),
        });
        Ok(entries)
    }
}

// ── Per-file builders ─────────────────────────────────────────────────────

fn build_added(rel: String, after: &ObservedPath) -> DiffEntry {
    let file = match &after.node {
        Node::Directory => return dir_entry(rel, DiffType::Added),
        Node::Issue(issue) => return issue_entry(rel, issue),
        Node::File(file) => file,
    };
    let bytes = match path_boundary::read_file(file) {
        Ok(bytes) => bytes,
        Err(issue) => return issue_entry(rel, &issue),
    };
    let (sha, size) = digest(&bytes);
    let (text, is_binary) = classify_bytes(&bytes);
    DiffEntry {
        path: rel,
        diff_type: DiffType::Added,
        is_dir: false,
        before_text: None,
        after_text: text.clone(),
        is_binary,
        before_size: None,
        after_size: size,
        before_sha256: None,
        after_sha256: sha,
        stats: None, // no before to diff against
        error_detail: None,
    }
}

fn build_removed(rel: String, before: &ObservedPath) -> DiffEntry {
    let file = match &before.node {
        Node::Directory => return dir_entry(rel, DiffType::Removed),
        Node::Issue(issue) => return issue_entry(rel, issue),
        Node::File(file) => file,
    };
    let bytes = match path_boundary::read_file(file) {
        Ok(bytes) => bytes,
        Err(issue) => return issue_entry(rel, &issue),
    };
    let (sha, size) = digest(&bytes);
    let (text, is_binary) = classify_bytes(&bytes);
    DiffEntry {
        path: rel,
        diff_type: DiffType::Removed,
        is_dir: false,
        before_text: text,
        after_text: None,
        is_binary,
        before_size: size,
        after_size: None,
        before_sha256: sha,
        after_sha256: None,
        stats: None,
        error_detail: None,
    }
}

fn build_compared(rel: String, before: &ObservedPath, after: &ObservedPath) -> DiffEntry {
    if let Node::Issue(issue) = &before.node {
        return issue_entry(rel, issue);
    }
    if let Node::Issue(issue) = &after.node {
        return issue_entry(rel, issue);
    }

    let before_is_dir = matches!(before.node, Node::Directory);
    let after_is_dir = matches!(after.node, Node::Directory);
    if before_is_dir != after_is_dir {
        return DiffEntry {
            path: rel,
            diff_type: DiffType::TypeChanged,
            is_dir: false,
            before_text: None,
            after_text: None,
            is_binary: false,
            before_size: None,
            after_size: None,
            before_sha256: None,
            after_sha256: None,
            stats: None,
            error_detail: Some("Path kind changed (file ↔ directory).".into()),
        };
    }
    if before_is_dir {
        return dir_entry(rel, DiffType::Unchanged);
    }

    let Node::File(before_file) = &before.node else {
        unreachable!()
    };
    let Node::File(after_file) = &after.node else {
        unreachable!()
    };
    let before_bytes = match path_boundary::read_file(before_file) {
        Ok(bytes) => bytes,
        Err(issue) => return issue_entry(rel, &issue),
    };
    let after_bytes = match path_boundary::read_file(after_file) {
        Ok(bytes) => bytes,
        Err(issue) => return issue_entry(rel, &issue),
    };
    let (before_sha, before_size) = digest(&before_bytes);
    let (after_sha, after_size) = digest(&after_bytes);

    let diff_type = if before_bytes == after_bytes {
        DiffType::Unchanged
    } else {
        DiffType::Modified
    };

    let (before_text, before_is_binary) = classify_bytes(&before_bytes);
    let (after_text, after_is_binary) = classify_bytes(&after_bytes);
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
        path: rel,
        diff_type,
        is_dir: false,
        before_text,
        after_text,
        is_binary,
        before_size,
        after_size,
        before_sha256: before_sha,
        after_sha256: after_sha,
        stats,
        error_detail: None,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn dir_entry(rel: String, diff_type: DiffType) -> DiffEntry {
    DiffEntry {
        path: rel,
        diff_type,
        is_dir: true,
        before_text: None,
        after_text: None,
        is_binary: false,
        before_size: None,
        after_size: None,
        before_sha256: None,
        after_sha256: None,
        stats: None,
        error_detail: None,
    }
}

pub(super) fn issue_entry(rel: String, issue: &PathIssue) -> DiffEntry {
    DiffEntry {
        path: rel,
        diff_type: if issue.unreadable {
            DiffType::Unreadable
        } else {
            DiffType::Incomparable
        },
        is_dir: false,
        before_text: None,
        after_text: None,
        is_binary: false,
        before_size: None,
        after_size: None,
        before_sha256: None,
        after_sha256: None,
        stats: None,
        error_detail: Some(path_boundary::issue_text(issue)),
    }
}

fn digest(bytes: &[u8]) -> (Option<String>, Option<u64>) {
    (
        Some(hex::encode(Sha256::digest(bytes))),
        Some(bytes.len() as u64),
    )
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
        Err(_) => (None, true),
    }
}
