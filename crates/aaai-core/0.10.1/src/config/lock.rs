//! Definition file write locking — prevents concurrent saves.
//!
//! A lock file `<definition>.lock` is created before writing and removed
//! afterwards.  If the lock file already exists and is recent (< 60 s old),
//! the save is aborted with an error.

use std::path::Path;
use std::time::{Duration, SystemTime};

const LOCK_TTL: Duration = Duration::from_secs(60);
const LOCK_EXT: &str = "lock";

/// Acquire a write lock for `definition_path`.
///
/// Returns `Err` when the lock is already held by another process.
/// On success, returns a [`LockGuard`] that releases the lock on drop.
pub fn acquire(definition_path: &Path) -> anyhow::Result<LockGuard> {
    let lock_path = definition_path.with_extension(LOCK_EXT);

    if lock_path.exists() {
        // Check whether the lock is stale.
        if let Ok(meta) = std::fs::metadata(&lock_path) {
            if let Ok(modified) = meta.modified() {
                if SystemTime::now().duration_since(modified).unwrap_or_default() < LOCK_TTL {
                    anyhow::bail!(
                        "Definition file is locked by another process: {}.\n\
                         Delete {} to force-unlock.",
                        definition_path.display(),
                        lock_path.display()
                    );
                }
                // Stale lock — remove it.
                log::warn!("Removing stale lock: {}", lock_path.display());
                let _ = std::fs::remove_file(&lock_path);
            }
        }
    }

    std::fs::write(&lock_path, format!("pid:{}", std::process::id()))?;
    log::debug!("Lock acquired: {}", lock_path.display());
    Ok(LockGuard { lock_path })
}

/// RAII guard that releases the lock on drop.
#[must_use]
pub struct LockGuard {
    lock_path: std::path::PathBuf,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.lock_path) {
            log::warn!("Could not remove lock file {}: {e}", self.lock_path.display());
        } else {
            log::debug!("Lock released: {}", self.lock_path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn lock_acquire_and_release() {
        let tmp = tempfile::tempdir().unwrap();
        let def = tmp.path().join("audit.yaml");
        std::fs::write(&def, "").unwrap();

        {
            let _guard = acquire(&def).unwrap();
            let lock = def.with_extension("lock");
            assert!(lock.exists(), "lock file should exist while guard is alive");
        }

        let lock = def.with_extension("lock");
        assert!(!lock.exists(), "lock file should be removed on drop");
    }

    #[test]
    fn double_lock_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let def = tmp.path().join("audit.yaml");
        std::fs::write(&def, "").unwrap();

        let _guard1 = acquire(&def).unwrap();
        let result2 = acquire(&def);
        assert!(result2.is_err(), "second acquire should fail while lock is held");
    }
}
