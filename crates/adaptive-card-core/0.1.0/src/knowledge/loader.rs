//! Loading knowledge base entries from disk or embedded bytes.

use super::format::{KnowledgeEntry, KnowledgeManifest};
use crate::error::{Error, Result};
use std::path::Path;

/// Load a knowledge base from a directory on disk. Expects the structure:
/// ```text
/// <dir>/_manifest.json
/// <dir>/examples/<path-from-manifest>.json
/// ```
pub fn from_dir(dir: &Path) -> Result<Vec<KnowledgeEntry>> {
    let manifest_path = dir.join("_manifest.json");
    let manifest_str = std::fs::read_to_string(&manifest_path)
        .map_err(|e| Error::KnowledgeLoad(format!("read manifest: {e}")))?;
    let manifest: KnowledgeManifest = serde_json::from_str(&manifest_str)
        .map_err(|e| Error::KnowledgeLoad(format!("parse manifest: {e}")))?;

    let mut entries = Vec::with_capacity(manifest.entries.len());
    for me in &manifest.entries {
        let entry_path = dir.join(&me.path);
        let entry_str = std::fs::read_to_string(&entry_path)
            .map_err(|e| Error::KnowledgeLoad(format!("read {}: {e}", me.path)))?;
        let entry: KnowledgeEntry = serde_json::from_str(&entry_str)
            .map_err(|e| Error::KnowledgeLoad(format!("parse {}: {e}", me.path)))?;
        if entry.id != me.id {
            return Err(Error::KnowledgeLoad(format!(
                "manifest id '{}' does not match entry id '{}'",
                me.id, entry.id
            )));
        }
        entries.push(entry);
    }
    Ok(entries)
}

/// Parse a single entry from JSON string.
pub fn parse_entry(json: &str) -> Result<KnowledgeEntry> {
    serde_json::from_str(json).map_err(|e| Error::KnowledgeLoad(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_empty_manifest() {
        let dir = tempdir();
        fs::write(
            dir.path().join("_manifest.json"),
            r#"{"version":"1","entries":[]}"#,
        )
        .unwrap();
        let entries = from_dir(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn missing_manifest_errors() {
        let dir = tempdir();
        let result = from_dir(dir.path());
        assert!(matches!(result, Err(Error::KnowledgeLoad(_))));
    }

    fn tempdir() -> TempDir {
        let mut path = std::env::temp_dir();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("kb-test-{stamp}"));
        std::fs::create_dir_all(&path).unwrap();
        TempDir { path }
    }

    struct TempDir {
        path: std::path::PathBuf,
    }
    impl TempDir {
        fn path(&self) -> &std::path::Path {
            &self.path
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}
