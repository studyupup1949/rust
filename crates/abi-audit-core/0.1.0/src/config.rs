use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::model::HeaderSyncTool;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AbiAuditConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_snapshot_path")]
    pub snapshot: PathBuf,
    #[serde(default)]
    pub baseline: BaselineConfig,
    #[serde(default)]
    pub rules: BTreeMap<String, RuleConfig>,
    #[serde(default)]
    pub targets: Vec<TargetConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum BaselineConfig {
    Path(PathBuf),
    Source(BaselineSourceConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaselineSourceConfig {
    #[serde(default)]
    pub kind: BaselineSourceKind,
    pub path: PathBuf,
    #[serde(default = "default_baseline_artifact_snapshot")]
    pub snapshot: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BaselineSourceKind {
    #[default]
    Snapshot,
    ArtifactDir,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetConfig {
    pub package: String,
    #[serde(default)]
    pub headers: Vec<PathBuf>,
    #[serde(default)]
    pub header_sync: Option<HeaderSyncConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeaderSyncConfig {
    #[serde(default)]
    pub tool: HeaderSyncTool,
    pub output: PathBuf,
    #[serde(default)]
    pub config: Option<PathBuf>,
    #[serde(default)]
    pub crate_dir: Option<PathBuf>,
    #[serde(default = "default_verify_freshness")]
    pub verify_freshness: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RuleConfig {
    #[serde(default)]
    pub severity: Option<RuleSeverity>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleSeverity {
    Off,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct InitOptions {
    pub path: PathBuf,
    pub force: bool,
}

impl Default for AbiAuditConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            snapshot: default_snapshot_path(),
            baseline: BaselineConfig::default(),
            rules: BTreeMap::new(),
            targets: Vec::new(),
        }
    }
}

impl Default for BaselineConfig {
    fn default() -> Self {
        Self::Path(default_baseline_path())
    }
}

pub fn load_config(workspace_root: &Path, explicit_path: Option<&Path>) -> Result<AbiAuditConfig> {
    let path = explicit_path
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("abi-audit.toml"));
    if !path.exists() {
        return Ok(AbiAuditConfig::default());
    }

    let text = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config at {}", path.display()))?;
    let config: AbiAuditConfig = toml::from_str(&text)
        .with_context(|| format!("failed to parse config at {}", path.display()))?;
    if config.version != 1 {
        bail!(
            "unsupported config version {} in {}",
            config.version,
            path.display()
        );
    }
    Ok(config)
}

pub fn write_starter_config(options: &InitOptions) -> Result<PathBuf> {
    if options.path.exists() && !options.force {
        bail!(
            "refusing to overwrite existing config at {} (use --force to replace it)",
            options.path.display()
        );
    }

    if let Some(parent) = options.path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create parent directory for {}",
                options.path.display()
            )
        })?;
    }

    let template = r#"# Leave `headers` empty to auto-discover `include/**/*.h` in the target package.
# `header_sync` is optional and records an explicit cbindgen workflow for freshness checks.
version = 1
snapshot = "abi-audit/snapshot.json"
baseline = "abi-audit/baseline.json"

[[targets]]
package = "your-ffi-crate"
headers = ["include/your_ffi_crate.h"]

[targets.header_sync]
tool = "cbindgen"
output = "include/your_ffi_crate.h"
config = "cbindgen.toml"
verify_freshness = true

[rules.baseline-drift]
severity = "error"
"#;
    fs::write(&options.path, template)
        .with_context(|| format!("failed to write {}", options.path.display()))?;
    Ok(options.path.clone())
}

const fn default_version() -> u32 {
    1
}

fn default_snapshot_path() -> PathBuf {
    PathBuf::from("abi-audit/snapshot.json")
}

fn default_baseline_path() -> PathBuf {
    PathBuf::from("abi-audit/baseline.json")
}

fn default_baseline_artifact_snapshot() -> PathBuf {
    PathBuf::from("snapshot.json")
}

const fn default_verify_freshness() -> bool {
    true
}
