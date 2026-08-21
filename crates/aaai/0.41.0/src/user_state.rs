//! Shared resolution for aaai's OS-level user-state files.

use std::ffi::OsString;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

const TEST_STATE_DIR_ENV: &str = "AAAI_TEST_STATE_DIR";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UserStatePaths {
    root: PathBuf,
}

impl UserStatePaths {
    pub(crate) fn resolve() -> anyhow::Result<Self> {
        Self::resolve_from(std::env::var_os(TEST_STATE_DIR_ENV), dirs::config_dir)
    }

    #[cfg(test)]
    pub(crate) fn from_root(root: impl Into<PathBuf>) -> anyhow::Result<Self> {
        Self::from_override(root.into().into_os_string())
    }

    fn resolve_from<F>(
        override_root: Option<OsString>,
        default_config_dir: F,
    ) -> anyhow::Result<Self>
    where
        F: FnOnce() -> Option<PathBuf>,
    {
        if let Some(root) = override_root {
            return Self::from_override(root);
        }

        let root = default_config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine OS config directory"))?
            .join("aaai");
        Ok(Self { root })
    }

    fn from_override(root: OsString) -> anyhow::Result<Self> {
        let root = PathBuf::from(root);
        if root.as_os_str().is_empty() {
            anyhow::bail!("{TEST_STATE_DIR_ENV} must not be empty");
        }
        if !root.is_absolute() {
            anyhow::bail!("{TEST_STATE_DIR_ENV} must be an absolute path");
        }
        Ok(Self { root })
    }

    #[cfg(test)]
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn history(&self) -> PathBuf {
        self.root.join("history.jsonl")
    }

    pub(crate) fn profiles(&self) -> PathBuf {
        self.root.join("profiles.yaml")
    }

    pub(crate) fn prefs(&self) -> PathBuf {
        self.root.join("prefs.yaml")
    }

    pub(crate) fn ensure_for_write(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
