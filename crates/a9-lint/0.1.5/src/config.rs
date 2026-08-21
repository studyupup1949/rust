use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub scan: Vec<String>,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub rules: RulesConfig,
}

#[derive(Deserialize, Default)]
pub struct RulesConfig {
    #[serde(default)]
    pub disable: Vec<String>,
}

// --- TOML deserialization shapes ---

#[derive(Deserialize)]
struct CargoTomlShape {
    workspace: Option<WorkspaceShape>,
}

#[derive(Deserialize)]
struct WorkspaceShape {
    metadata: Option<MetadataShape>,
}

#[derive(Deserialize)]
struct MetadataShape {
    #[serde(rename = "a9-lint")]
    a9_lint: Option<Config>,
}

/// Walk upward from `start`, looking for a `Cargo.toml` with a
/// `[workspace.metadata.a9-lint]` section.
///
/// Returns `(config, workspace_root)`. Falls back to scanning `src/` in `start`
/// when no config is found.
pub fn find_config(start: &Path) -> (Config, PathBuf) {
    let mut dir = start.to_path_buf();
    loop {
        let cargo_path = dir.join("Cargo.toml");
        if cargo_path.exists()
            && let Ok(content) = fs::read_to_string(&cargo_path)
            && let Ok(parsed) = toml::from_str::<CargoTomlShape>(&content)
            && let Some(workspace) = parsed.workspace
            && let Some(meta) = workspace.metadata
            && let Some(config) = meta.a9_lint
        {
            return (config, dir);
        }

        match dir.parent() {
            Some(parent) if parent != dir => dir = parent.to_path_buf(),
            _ => break,
        }
    }

    // No config found — scan src/ in cwd as a sensible default.
    let fallback = Config {
        scan: vec!["src".into()],
        features: vec![],
        rules: RulesConfig::default(),
    };
    (fallback, start.to_path_buf())
}
