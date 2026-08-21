//! Project-level configuration — `.aaai.yaml`.
//!
//! Placed in a project's root (e.g. beside the repository's `.git` directory),
//! `.aaai.yaml` provides defaults so that team members don't need to specify
//! common paths on every invocation.
//!
//! # Example `.aaai.yaml`
//!
//! ```yaml
//! version: "1"
//! default_definition: "audit/audit.yaml"
//! default_ignore: "audit/.aaaiignore"
//! approver_name: "alice"
//! mask_secrets: true
//! custom_mask_patterns:
//!   - "MY_INTERNAL_TOKEN_[A-Z0-9]{16}"
//! ```

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const CONFIG_FILENAME: &str = ".aaai.yaml";

/// The project-level configuration document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Schema version. Currently `"1"`.
    #[serde(default = "default_version")]
    pub version: String,

    /// Default audit definition path, relative to project root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_definition: Option<String>,

    /// Default `.aaaiignore` path, relative to project root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_ignore: Option<String>,

    /// Default approver name stamped on approvals (overridden by CLI flag).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approver_name: Option<String>,

    /// Enable secret masking by default.
    #[serde(default)]
    pub mask_secrets: bool,

    /// Custom regex patterns added to the masking engine.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_mask_patterns: Vec<String>,
}

fn default_version() -> String { "1".into() }

impl ProjectConfig {
    /// Load from `path`.  Returns `None` when the file does not exist.
    pub fn load(path: &Path) -> anyhow::Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {}: {e}", path.display()))?;
        let cfg: Self = serde_yaml::from_str(&text)
            .map_err(|e| anyhow::anyhow!("Invalid {}: {e}", path.display()))?;
        Ok(Some(cfg))
    }

    /// Discover `.aaai.yaml` by walking up from `start_dir`.
    /// Returns the config and the directory in which it was found.
    pub fn discover(start_dir: &Path) -> anyhow::Result<Option<(Self, PathBuf)>> {
        let mut dir = start_dir.to_path_buf();
        loop {
            let candidate = dir.join(CONFIG_FILENAME);
            if let Some(cfg) = Self::load(&candidate)? {
                log::info!("Discovered {} at {}", CONFIG_FILENAME, dir.display());
                return Ok(Some((cfg, dir)));
            }
            match dir.parent() {
                Some(p) => dir = p.to_path_buf(),
                None    => return Ok(None),
            }
        }
    }

    /// Write to `path`, creating parent directories as needed.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    /// Generate a starter config with helpful comments embedded as YAML string.
    pub fn starter_yaml() -> &'static str {
        r#"# aaai project configuration
# Place this file at the root of your project.
version: "1"

# Path to the default audit definition, relative to this file.
# default_definition: "audit/audit.yaml"

# Path to the default .aaaiignore file, relative to this file.
# default_ignore: "audit/.aaaiignore"

# Default approver name stamped when approving entries via CLI.
# approver_name: "your-name"

# Automatically mask secrets in CLI output and reports.
mask_secrets: false

# Additional regex patterns to mask (beyond built-in patterns).
# custom_mask_patterns:
#   - "MY_INTERNAL_[A-Z0-9]{16}"
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_yaml() {
        let cfg = ProjectConfig {
            version: "1".into(),
            default_definition: Some("audit/audit.yaml".into()),
            approver_name: Some("alice".into()),
            mask_secrets: true,
            custom_mask_patterns: vec!["PATTERN_[A-Z]+".into()],
            ..Default::default()
        };
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let restored: ProjectConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(restored.approver_name.as_deref(), Some("alice"));
        assert!(restored.mask_secrets);
        assert_eq!(restored.custom_mask_patterns.len(), 1);
    }

    #[test]
    fn load_nonexistent_returns_none() {
        let result = ProjectConfig::load(Path::new("/nonexistent/.aaai.yaml")).unwrap();
        assert!(result.is_none());
    }
}
