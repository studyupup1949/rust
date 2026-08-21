//! Shared helpers for the `test_methods.json` sidecar.
//!
//! TIS test methods live in a sidecar file next to project.json (see
//! `autocore_server::project::TEST_METHODS_FILE`), not embedded in
//! project.json itself. Every acctl path that used to read the raw
//! project.json `test_methods` key goes through here so the sidecar-wins
//! semantics match the server exactly: the sidecar is the source of truth
//! when present; a legacy embedded block is the fallback.

use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Sidecar filename, next to project.json. Mirrors the server's
/// `project::TEST_METHODS_FILE`.
pub const TEST_METHODS_FILE: &str = "test_methods.json";

/// Path of the test_methods.json sidecar next to `project_path`
/// (a path to project.json).
pub fn sidecar_path(project_path: &Path) -> PathBuf {
    project_path
        .parent()
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join(TEST_METHODS_FILE)
}

/// Read the sidecar next to `project_path`, if present, and return its
/// methods map. Accepts the canonical `{ "test_methods": { ... } }`
/// wrapper or a bare map. `Ok(None)` when the file doesn't exist; a file
/// that exists but can't be read or parsed is a hard error — silently
/// falling back to a legacy embedded block would codegen/push stale
/// method definitions (matches the server's loader semantics).
pub fn load_sidecar_methods(project_path: &Path) -> Result<Option<Value>> {
    let path = sidecar_path(project_path);
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .with_context(|| format!("parsing {}", path.display()))?;
    Ok(Some(unwrap_methods(value)?))
}

/// Effective methods map for a project: the sidecar next to
/// `project_path` wins over a legacy embedded `test_methods` block in the
/// parsed project.json `project_json`. `Ok(None)` when neither exists.
pub fn effective_test_methods(
    project_path: &Path,
    project_json: &Value,
) -> Result<Option<Value>> {
    if let Some(methods) = load_sidecar_methods(project_path)? {
        return Ok(Some(methods));
    }
    Ok(project_json.get("test_methods").cloned())
}

/// True when the project declares at least one test method (sidecar or
/// legacy embedded block) — the condition under which the server's
/// `Project::normalize()` injects the `tis_*` readiness scalars.
pub fn has_test_methods(project_path: &Path, project_json: &Value) -> Result<bool> {
    Ok(effective_test_methods(project_path, project_json)?
        .and_then(|m| m.as_object().map(|o| !o.is_empty()))
        .unwrap_or(false))
}

/// Canonical wrapped form for writing/uploading a methods map:
/// `{ "test_methods": { ... } }`.
pub fn wrapped(methods: &Value) -> Value {
    serde_json::json!({ "test_methods": methods })
}

/// Unwrap a sidecar document into its methods map: canonical wrapped form
/// or a bare map (tolerated for hand-made files).
fn unwrap_methods(value: Value) -> Result<Value> {
    let methods = match value.get("test_methods") {
        Some(inner) => inner.clone(),
        None => value,
    };
    if !methods.is_object() && !methods.is_null() {
        return Err(anyhow!(
            "{} does not contain a test_methods map",
            TEST_METHODS_FILE
        ));
    }
    Ok(methods)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_wins_over_embedded() {
        let tmp = tempfile::tempdir().unwrap();
        let pj = tmp.path().join("project.json");
        let embedded = serde_json::json!({ "test_methods": { "embedded": {} } });

        // No sidecar → legacy embedded block.
        let m = effective_test_methods(&pj, &embedded).unwrap().unwrap();
        assert!(m.as_object().unwrap().contains_key("embedded"));

        // Wrapped sidecar wins.
        std::fs::write(
            tmp.path().join(TEST_METHODS_FILE),
            r#"{ "test_methods": { "side": {} } }"#,
        )
        .unwrap();
        let m = effective_test_methods(&pj, &embedded).unwrap().unwrap();
        assert!(m.as_object().unwrap().contains_key("side"));
        assert!(has_test_methods(&pj, &embedded).unwrap());

        // Bare-map sidecar is tolerated.
        std::fs::write(tmp.path().join(TEST_METHODS_FILE), r#"{ "bare": {} }"#).unwrap();
        let m = effective_test_methods(&pj, &embedded).unwrap().unwrap();
        assert!(m.as_object().unwrap().contains_key("bare"));

        // Empty sidecar map counts as "no methods" for tag injection.
        std::fs::write(
            tmp.path().join(TEST_METHODS_FILE),
            r#"{ "test_methods": {} }"#,
        )
        .unwrap();
        assert!(!has_test_methods(&pj, &embedded).unwrap());
    }

    #[test]
    fn corrupt_sidecar_is_a_hard_error() {
        let tmp = tempfile::tempdir().unwrap();
        let pj = tmp.path().join("project.json");
        std::fs::write(tmp.path().join(TEST_METHODS_FILE), "{ not json").unwrap();
        assert!(effective_test_methods(&pj, &serde_json::json!({})).is_err());
    }
}
