//! Core diff vocabulary: DiffType and DiffEntry.

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
            DiffType::Added => "Added",
            DiffType::Removed => "Removed",
            DiffType::Modified => "Modified",
            DiffType::Unchanged => "Unchanged",
            DiffType::TypeChanged => "TypeChanged",
            DiffType::Unreadable => "Unreadable",
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

/// One entry in the diff result.
#[derive(Debug, Clone)]
pub struct DiffEntry {
    /// Root-relative path with forward slashes.
    pub path: String,
    pub diff_type: DiffType,
    pub is_dir: bool,
    pub before_text: Option<String>,
    pub after_text: Option<String>,
    /// SHA-256 hex of the after-file if readable.
    pub after_sha256: Option<String>,
    pub error_detail: Option<String>,
}

impl DiffEntry {
    pub fn has_text_diff(&self) -> bool {
        self.before_text.is_some() || self.after_text.is_some()
    }
}
