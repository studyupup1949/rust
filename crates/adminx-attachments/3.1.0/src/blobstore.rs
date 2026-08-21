// adminx-storage/src/blobstore.rs
//
// Where the bytes live. `BlobStore` is the swap point: the MVP ships a local
// filesystem store, and an S3 / object-store backend can be added later behind a
// feature without touching the attachment logic or adminx-core.

use adminx_core::storage::StorageError;
use async_trait::async_trait;
use std::path::{Component, Path, PathBuf};

/// A content-addressed byte store. Keys are opaque strings the caller records in
/// the metadata table; the store only has to round-trip bytes under a key.
#[async_trait]
pub trait BlobStore: Send + Sync {
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StorageError>;
    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError>;
    /// Remove a key. Absence is success — deleting twice must not error.
    async fn delete(&self, key: &str) -> Result<(), StorageError>;
}

/// Stores blobs as files under a root directory.
pub struct LocalFsStore {
    root: PathBuf,
}

impl LocalFsStore {
    /// Create a store rooted at `root` (created on first write if missing).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Resolve a key to a path inside `root`, refusing anything that would climb
    /// out. Keys we mint are safe by construction, but this is the boundary
    /// where a key becomes a filesystem path, so it validates rather than trusts.
    fn path_for(&self, key: &str) -> Result<PathBuf, StorageError> {
        let rel = Path::new(key);
        // Reject absolute paths and any `..` / root components: the key must
        // stay within the store root.
        for comp in rel.components() {
            match comp {
                Component::Normal(_) => {}
                _ => {
                    return Err(StorageError::Backend(format!(
                        "unsafe blob key: {key:?}"
                    )))
                }
            }
        }
        Ok(self.root.join(rel))
    }
}

#[async_trait]
impl BlobStore for LocalFsStore {
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StorageError> {
        let path = self.path_for(key)?;
        if let Some(dir) = path.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|e| StorageError::Backend(format!("create blob dir: {e}")))?;
        }
        tokio::fs::write(&path, bytes)
            .await
            .map_err(|e| StorageError::Backend(format!("write blob: {e}")))
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let path = self.path_for(key)?;
        tokio::fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound
            } else {
                StorageError::Backend(format!("read blob: {e}"))
            }
        })
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let path = self.path_for(key)?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            // Already gone is success — delete is idempotent.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(StorageError::Backend(format!("delete blob: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_that_escape_the_root_are_refused() {
        let store = LocalFsStore::new("/var/lib/adminx-blobs");
        assert!(store.path_for("../../etc/passwd").is_err());
        assert!(store.path_for("/etc/passwd").is_err());
        assert!(store.path_for("a/../../b").is_err());
    }

    #[test]
    fn a_normal_nested_key_resolves_under_root() {
        let store = LocalFsStore::new("/var/lib/adminx-blobs");
        let p = store.path_for("posts/1/avatar/abc123").unwrap();
        assert!(p.starts_with("/var/lib/adminx-blobs"));
        assert!(p.ends_with("posts/1/avatar/abc123"));
    }
}
