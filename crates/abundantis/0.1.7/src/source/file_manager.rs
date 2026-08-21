//! File source manager with shared configuration.
//!
//! Manages all file sources with centralized configuration,
//! enabling config-aware refresh operations.

use super::config::{FileSourceConfig, SourceRefreshOptions};
use super::file::FileSource;
use super::traits::EnvSource;
use crate::path_cache::PathCache;
use crate::selection::ActiveFileSelector;
use crate::workspace::WorkspaceManager;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Manages all file sources with shared configuration.
///
/// This manager holds config for all .env files (not per-FileSource),
/// enabling coordinated refresh and config preservation.
pub struct FileSourceManager {
    sources: RwLock<HashMap<PathBuf, Arc<FileSource>>>,
    config: RwLock<FileSourceConfig>,
    selector: Arc<ActiveFileSelector>,
}

impl FileSourceManager {
    /// Create a new FileSourceManager for the given workspace root.
    pub fn new(workspace_root: &Path) -> Self {
        let path_cache = Arc::new(PathCache::new());
        Self {
            sources: RwLock::new(HashMap::new()),
            config: RwLock::new(FileSourceConfig::default()),
            selector: Arc::new(ActiveFileSelector::new(workspace_root, path_cache)),
        }
    }

    /// Create a new FileSourceManager with a shared path cache.
    pub fn with_path_cache(workspace_root: &Path, path_cache: Arc<PathCache>) -> Self {
        Self {
            sources: RwLock::new(HashMap::new()),
            config: RwLock::new(FileSourceConfig::default()),
            selector: Arc::new(ActiveFileSelector::new(workspace_root, path_cache)),
        }
    }

    /// Get or create a FileSource for a path.
    pub fn get_or_create(&self, path: &Path) -> Result<Arc<FileSource>, std::io::Error> {
        let canonical = path.canonicalize()?;

        // Check if already exists
        if let Some(source) = self.sources.read().get(&canonical) {
            return Ok(Arc::clone(source));
        }

        // Create new source
        let source = Arc::new(FileSource::new(path)?);
        self.sources.write().insert(canonical, Arc::clone(&source));
        Ok(source)
    }

    /// Register an existing FileSource.
    pub fn register(&self, source: Arc<FileSource>) {
        if let Ok(canonical) = source.path().canonicalize() {
            self.sources.write().insert(canonical, source);
        }
    }

    /// Unregister a FileSource by path.
    pub fn unregister(&self, path: &Path) {
        if let Ok(canonical) = path.canonicalize() {
            self.sources.write().remove(&canonical);
        }
    }

    /// Set active files (global patterns).
    pub fn set_active_files(&self, patterns: Option<Vec<String>>) {
        self.config.write().active_files = patterns;
    }

    /// Get the current active files pattern.
    pub fn get_active_files(&self) -> Option<Vec<String>> {
        self.config.read().active_files.clone()
    }

    /// Set directory-scoped override.
    pub fn set_directory_override(&self, dir: PathBuf, patterns: Vec<String>) {
        self.config.write().directory_overrides.insert(dir, patterns);
    }

    /// Clear directory override.
    pub fn clear_directory_override(&self, dir: &Path) {
        self.config.write().directory_overrides.remove(dir);
    }

    /// Get directory overrides.
    pub fn get_directory_overrides(&self) -> HashMap<PathBuf, Vec<String>> {
        self.config.read().directory_overrides.clone()
    }

    /// Get active files for a code file path.
    pub fn active_files_for_path(&self, file_path: &Path, workspace: &WorkspaceManager) -> Vec<PathBuf> {
        let config = self.config.read();
        self.selector.compute_active_files(
            file_path,
            config.active_files.as_deref(),
            &config.directory_overrides,
            workspace,
        )
    }

    /// Get selector for active file computation.
    pub fn selector(&self) -> &ActiveFileSelector {
        &self.selector
    }

    /// Get current config.
    pub fn config(&self) -> FileSourceConfig {
        self.config.read().clone()
    }

    /// Apply config (for restore after refresh).
    pub fn apply_config(&self, config: FileSourceConfig) {
        *self.config.write() = config;
    }

    /// Refresh all file sources.
    pub fn refresh(&self, options: &SourceRefreshOptions) {
        // Backup config if preserving
        let config_backup = if options.preserve_config {
            Some(self.config.read().clone())
        } else {
            None
        };

        // Invalidate all sources
        for source in self.sources.read().values() {
            source.invalidate();
        }

        // Restore config if preserved
        if let Some(config) = config_backup {
            *self.config.write() = config;
        }
    }

    /// Get all registered sources.
    pub fn sources(&self) -> Vec<Arc<FileSource>> {
        self.sources.read().values().cloned().collect()
    }

    /// Check if a source is registered for the given path.
    pub fn is_registered(&self, path: &Path) -> bool {
        if let Ok(canonical) = path.canonicalize() {
            self.sources.read().contains_key(&canonical)
        } else {
            false
        }
    }

    /// Get a source by path.
    pub fn get(&self, path: &Path) -> Option<Arc<FileSource>> {
        path.canonicalize()
            .ok()
            .and_then(|canonical| self.sources.read().get(&canonical).cloned())
    }

    /// Get the number of registered sources.
    pub fn len(&self) -> usize {
        self.sources.read().len()
    }

    /// Check if the manager has no registered sources.
    pub fn is_empty(&self) -> bool {
        self.sources.read().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_env() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let env_path = temp_dir.path().join(".env");
        fs::write(&env_path, "TEST_VAR=value").unwrap();
        (temp_dir, env_path)
    }

    #[test]
    fn test_get_or_create() {
        let (temp_dir, env_path) = setup_test_env();
        let manager = FileSourceManager::new(temp_dir.path());

        let source1 = manager.get_or_create(&env_path).unwrap();
        let source2 = manager.get_or_create(&env_path).unwrap();

        // Same source should be returned
        assert!(Arc::ptr_eq(&source1, &source2));
    }

    #[test]
    fn test_set_active_files() {
        let (temp_dir, _env_path) = setup_test_env();
        let manager = FileSourceManager::new(temp_dir.path());

        assert!(manager.get_active_files().is_none());

        manager.set_active_files(Some(vec![".env".to_string()]));
        assert_eq!(manager.get_active_files(), Some(vec![".env".to_string()]));

        manager.set_active_files(None);
        assert!(manager.get_active_files().is_none());
    }

    #[test]
    fn test_directory_overrides() {
        let (temp_dir, _env_path) = setup_test_env();
        let manager = FileSourceManager::new(temp_dir.path());

        let dir = temp_dir.path().join("subdir");
        manager.set_directory_override(dir.clone(), vec![".env.local".to_string()]);

        let overrides = manager.get_directory_overrides();
        assert!(overrides.contains_key(&dir));

        manager.clear_directory_override(&dir);
        let overrides = manager.get_directory_overrides();
        assert!(!overrides.contains_key(&dir));
    }

    #[test]
    fn test_config_preservation() {
        let (temp_dir, env_path) = setup_test_env();
        let manager = FileSourceManager::new(temp_dir.path());

        manager.get_or_create(&env_path).unwrap();
        manager.set_active_files(Some(vec![".env.local".to_string()]));

        // Refresh with preservation
        manager.refresh(&SourceRefreshOptions { preserve_config: true });
        assert_eq!(manager.get_active_files(), Some(vec![".env.local".to_string()]));

        // Refresh without preservation
        manager.refresh(&SourceRefreshOptions { preserve_config: false });
        // Config should still be there because we don't reset it, just preserve/not preserve
        assert_eq!(manager.get_active_files(), Some(vec![".env.local".to_string()]));
    }
}
