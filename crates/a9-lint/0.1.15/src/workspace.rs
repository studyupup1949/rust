use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

/// Returns all workspace member paths (relative to root), respecting `exclude`.
/// Expands single-`*` glob patterns.
pub fn workspace_members(root: &Path) -> Vec<String> {
    let cargo_path = root.join("Cargo.toml");

    let Ok(content) = fs::read_to_string(&cargo_path) else {
        return vec![];
    };

    let Ok(parsed) = toml::from_str::<CargoTomlShape>(&content) else {
        return vec![];
    };

    let Some(workspace) = parsed.workspace else {
        return vec![];
    };

    let Some(members) = workspace.members else {
        return vec![];
    };

    let excluded: HashSet<String> = workspace.exclude.unwrap_or_default().into_iter().collect();

    let mut result = vec![];

    for pattern in members {
        if pattern.contains('*') {
            result.extend(expand_member_glob(root, &pattern, &excluded));
        } else if !excluded.contains(&pattern) {
            result.push(pattern);
        }
    }

    result
}

/// Returns scan directories: member `src/` dirs plus workspace root `src/` if present.
/// Falls back to `["src"]` for single-crate projects.
pub fn discover_scan_dirs(root: &Path) -> Vec<String> {
    let members = workspace_members(root);

    if members.is_empty() {
        return vec!["src".into()];
    }

    let mut dirs: Vec<String> = members.iter().map(|m| format!("{m}/src")).collect();

    if root.join("src").exists() {
        dirs.push("src".into());
    }

    dirs
}

/// Returns the effective src dirs to scan given an optional explicit `scan` config.
pub fn effective_scan_dirs(explicit_scan: &[String], root: &Path) -> Vec<String> {
    if !explicit_scan.is_empty() {
        explicit_scan.to_vec()
    } else {
        discover_scan_dirs(root)
    }
}

pub fn member_src_dirs(root: &Path) -> Vec<PathBuf> {
    workspace_members(root)
        .iter()
        .map(|m| root.join(m).join("src"))
        .filter(|p| p.exists())
        .collect()
}

#[derive(Deserialize)]
struct MetadataShape {
    #[allow(dead_code)]
    #[serde(rename = "a9-lint")]
    a9_lint: Option<toml::Value>,
}

#[derive(Deserialize)]
struct WorkspaceShape {
    members: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    #[allow(dead_code)]
    metadata: Option<MetadataShape>,
}

#[derive(Deserialize)]
struct CargoTomlShape {
    workspace: Option<WorkspaceShape>,
}

fn expand_member_glob(root: &Path, pattern: &str, excluded: &HashSet<String>) -> Vec<String> {
    let (prefix, _suffix) = pattern.split_once('*').unwrap_or((pattern, ""));
    let base = root.join(prefix.trim_end_matches('/'));

    let Ok(entries) = fs::read_dir(&base) else {
        return vec![];
    };

    let mut expanded = vec![];

    for entry in entries.flatten() {
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let member_path = format!("{}{}", prefix, name);

        if excluded.contains(&member_path) || !root.join(&member_path).join("Cargo.toml").exists() {
            continue;
        }

        expanded.push(member_path);
    }

    expanded
}
