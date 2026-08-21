//! Shared helpers for the `asset_management.json` sidecar.
//!
//! AMS *configuration* — the custom `asset_types` catalog, the optional
//! `enabled_builtin_asset_types` allowlist, and the project-level
//! `asset_refs` — lives in a sidecar file next to project.json (see
//! `autocore_server::project::ASSET_MANAGEMENT_FILE`), not embedded in
//! project.json itself. Every acctl path that used to read those raw
//! project.json keys goes through here so the sidecar-wins semantics match
//! the server exactly: the sidecar is the source of truth when present; a
//! legacy embedded block is the fallback.
//!
//! This is the AMS *config* only. The asset *instances* (datastore/assets/,
//! `acctl push assets`) are a separate concern handled elsewhere.

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Sidecar filename, next to project.json. Mirrors the server's
/// `project::ASSET_MANAGEMENT_FILE`.
pub const ASSET_MANAGEMENT_FILE: &str = "asset_management.json";

/// Path of the asset_management.json sidecar next to `project_path`
/// (a path to project.json).
pub fn sidecar_path(project_path: &Path) -> PathBuf {
    project_path
        .parent()
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join(ASSET_MANAGEMENT_FILE)
}

/// Read the sidecar next to `project_path`, if present, and return its
/// inner AMS object. Accepts the canonical `{ "ams": { ... } }` wrapper or
/// a bare `{ asset_types, enabled_builtin_asset_types, asset_refs }`
/// object. `Ok(None)` when the file doesn't exist; a file that exists but
/// can't be read or parsed is a hard error — silently falling back to a
/// legacy embedded block would push/seed stale AMS definitions (matches the
/// server's loader semantics).
pub fn load_sidecar_ams(project_path: &Path) -> Result<Option<Value>> {
    let path = sidecar_path(project_path);
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(Some(unwrap_ams(value)?))
}

/// Effective AMS config for a project: the sidecar next to `project_path`
/// wins over a legacy embedded block in the parsed project.json
/// `project_json`. The embedded block is assembled from the three raw keys
/// (`asset_types` / `enabled_builtin_asset_types` / `asset_refs`).
/// `Ok(None)` when neither exists.
pub fn effective_asset_management(
    project_path: &Path,
    project_json: &Value,
) -> Result<Option<Value>> {
    if let Some(ams) = load_sidecar_ams(project_path)? {
        return Ok(Some(ams));
    }
    let asset_types = project_json.get("asset_types");
    let enabled = project_json.get("enabled_builtin_asset_types");
    let refs = project_json.get("asset_refs");
    if asset_types.is_none() && enabled.is_none() && refs.is_none() {
        return Ok(None);
    }
    Ok(Some(serde_json::json!({
        "asset_types": asset_types.cloned().unwrap_or(Value::Null),
        "enabled_builtin_asset_types": enabled.cloned().unwrap_or(Value::Null),
        "asset_refs": refs.cloned().unwrap_or_else(|| serde_json::json!([])),
    })))
}

/// True when AMS is enabled for this project — i.e. `asset_types` is present
/// (sidecar or legacy embedded block), even if it's an empty map. Mirrors
/// the server's `Project::normalize()` gate (`asset_types.is_some()`).
pub fn has_asset_management(project_path: &Path, project_json: &Value) -> Result<bool> {
    Ok(effective_asset_management(project_path, project_json)?
        .and_then(|a| a.get("asset_types").cloned())
        .map(|t| !t.is_null())
        .unwrap_or(false))
}

/// Canonical wrapped form for writing/uploading an AMS config object:
/// `{ "ams": { ... } }`.
pub fn wrapped(ams: &Value) -> Value {
    serde_json::json!({ "ams": ams })
}

/// Unwrap a sidecar document into its AMS object: canonical wrapped form
/// or a bare object (tolerated for hand-made files).
fn unwrap_ams(value: Value) -> Result<Value> {
    let ams = match value.get("ams") {
        Some(inner) => inner.clone(),
        None => value,
    };
    if !ams.is_object() && !ams.is_null() {
        return Err(anyhow!(
            "{} does not contain an ams object",
            ASSET_MANAGEMENT_FILE
        ));
    }
    Ok(ams)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_wins_over_embedded() {
        let tmp = tempfile::tempdir().unwrap();
        let pj = tmp.path().join("project.json");
        let embedded = serde_json::json!({
            "asset_types": { "embedded": { "fields": [] } },
            "enabled_builtin_asset_types": ["load_cell"]
        });

        // No sidecar → legacy embedded block, assembled from the raw keys.
        let a = effective_asset_management(&pj, &embedded).unwrap().unwrap();
        assert!(a["asset_types"].as_object().unwrap().contains_key("embedded"));
        assert_eq!(a["enabled_builtin_asset_types"][0].as_str(), Some("load_cell"));
        assert!(has_asset_management(&pj, &embedded).unwrap());

        // Wrapped sidecar wins.
        std::fs::write(
            tmp.path().join(ASSET_MANAGEMENT_FILE),
            r#"{ "ams": { "asset_types": { "side": { "fields": [] } } } }"#,
        )
        .unwrap();
        let a = effective_asset_management(&pj, &embedded).unwrap().unwrap();
        assert!(a["asset_types"].as_object().unwrap().contains_key("side"));
        assert!(!a["asset_types"].as_object().unwrap().contains_key("embedded"));

        // Bare-object sidecar is tolerated.
        std::fs::write(
            tmp.path().join(ASSET_MANAGEMENT_FILE),
            r#"{ "asset_types": { "bare": { "fields": [] } } }"#,
        )
        .unwrap();
        let a = effective_asset_management(&pj, &embedded).unwrap().unwrap();
        assert!(a["asset_types"].as_object().unwrap().contains_key("bare"));

        // A sidecar without an asset_types key → AMS disabled.
        std::fs::write(
            tmp.path().join(ASSET_MANAGEMENT_FILE),
            r#"{ "ams": { "asset_refs": [] } }"#,
        )
        .unwrap();
        assert!(!has_asset_management(&pj, &embedded).unwrap());
    }

    #[test]
    fn no_config_anywhere_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        let pj = tmp.path().join("project.json");
        let bare = serde_json::json!({ "name": "t", "version": "0.1.0" });
        assert!(effective_asset_management(&pj, &bare).unwrap().is_none());
        assert!(!has_asset_management(&pj, &bare).unwrap());
    }

    #[test]
    fn corrupt_sidecar_is_a_hard_error() {
        let tmp = tempfile::tempdir().unwrap();
        let pj = tmp.path().join("project.json");
        std::fs::write(tmp.path().join(ASSET_MANAGEMENT_FILE), "{ not json").unwrap();
        assert!(effective_asset_management(&pj, &serde_json::json!({})).is_err());
    }

    #[test]
    fn wrapped_is_canonical() {
        let ams = serde_json::json!({ "asset_types": {} });
        assert_eq!(wrapped(&ams), serde_json::json!({ "ams": { "asset_types": {} } }));
    }
}
