//! Core diff vocabulary: [`DiffType`] and [`DiffEntry`] (Phase 4: binary support + stats).

use serde::{Deserialize, Serialize};

/// The kind of change detected for a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffType {
    Added,
    Removed,
    Modified,
    Unchanged,
    TypeChanged,
    Unreadable,
    Incomparable,
}

impl std::fmt::Display for DiffType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DiffType::Added        => "Added",
            DiffType::Removed      => "Removed",
            DiffType::Modified     => "Modified",
            DiffType::Unchanged    => "Unchanged",
            DiffType::TypeChanged  => "TypeChanged",
            DiffType::Unreadable   => "Unreadable",
            DiffType::Incomparable => "Incomparable",
        };
        write!(f, "{s}")
    }
}

impl DiffType {
    pub fn is_changed(self) -> bool {
        !matches!(self, DiffType::Unchanged)
    }
    pub fn is_error(self) -> bool {
        matches!(self, DiffType::Unreadable | DiffType::Incomparable)
    }
}

/// Line-level diff statistics for text files.
#[derive(Debug, Clone, Default)]
pub struct DiffStats {
    pub lines_added:     usize,
    pub lines_removed:   usize,
    pub lines_unchanged: usize,
}

impl DiffStats {
    pub fn lines_changed(&self) -> usize {
        self.lines_added + self.lines_removed
    }

    /// Compute from before/after text using the similar crate.
    pub fn compute(before: &str, after: &str) -> Self {
        use similar::{ChangeTag, TextDiff};
        let td = TextDiff::from_lines(before, after);
        let mut stats = DiffStats::default();
        for change in td.iter_all_changes() {
            match change.tag() {
                ChangeTag::Insert => stats.lines_added     += 1,
                ChangeTag::Delete => stats.lines_removed   += 1,
                ChangeTag::Equal  => stats.lines_unchanged += 1,
            }
        }
        stats
    }
}

/// One entry in the diff result.
#[derive(Debug, Clone)]
pub struct DiffEntry {
    /// Root-relative path with forward slashes.
    pub path: String,
    pub diff_type: DiffType,
    pub is_dir: bool,

    // ── Text content (None for binary or missing files) ───────────────
    pub before_text: Option<String>,
    pub after_text:  Option<String>,

    // ── Binary / size tracking (Phase 4) ─────────────────────────────
    /// True when the file cannot be decoded as UTF-8.
    pub is_binary: bool,
    /// File size in bytes of the before-file, if available.
    pub before_size: Option<u64>,
    /// File size in bytes of the after-file, if available.
    pub after_size: Option<u64>,

    // ── Hashes ────────────────────────────────────────────────────────
    /// SHA-256 hex digest of the before-file.
    pub before_sha256: Option<String>,
    /// SHA-256 hex digest of the after-file.
    pub after_sha256: Option<String>,

    // ── Line statistics (Phase 4) ─────────────────────────────────────
    /// Set for Modified text files; None for binary / non-Modified entries.
    pub stats: Option<DiffStats>,

    pub error_detail: Option<String>,
}

impl DiffEntry {
    pub fn has_text_diff(&self) -> bool {
        !self.is_binary && (self.before_text.is_some() || self.after_text.is_some())
    }

    /// Human-readable size change description, e.g. "12 KB → 15 KB".
    pub fn size_change_label(&self) -> Option<String> {
        match (self.before_size, self.after_size) {
            (Some(b), Some(a)) => Some(format!("{} → {}", fmt_size(b), fmt_size(a))),
            (None,    Some(a)) => Some(format!("(new) {}", fmt_size(a))),
            (Some(b), None)    => Some(format!("{} (removed)", fmt_size(b))),
            (None, None)       => None,
        }
    }
}

/// Format bytes as human-readable string.
pub fn fmt_size(bytes: u64) -> String {
    if bytes < 1_024 {
        format!("{bytes} B")
    } else if bytes < 1_024 * 1_024 {
        format!("{:.1} KB", bytes as f64 / 1_024.0)
    } else if bytes < 1_024 * 1_024 * 1_024 {
        format!("{:.1} MB", bytes as f64 / (1_024.0 * 1_024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1_024.0 * 1_024.0 * 1_024.0))
    }
}

/// Threshold above which Exact / LineMatch strategies warn about file size.
pub const LARGE_FILE_THRESHOLD: u64 = 1_024 * 1_024; // 1 MB
