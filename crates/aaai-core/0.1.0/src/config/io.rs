//! File I/O for [`AuditDefinition`].
//!
//! Reads and writes the audit-definition YAML file with the following
//! guarantees:
//! * Atomic save: write to a `.tmp` sibling first, then rename.
//! * Optional pre-save backup (`.bak`).
//! * Stable field order (determined by `#[derive(Serialize)]` field order).

use std::path::Path;

use anyhow::{Context, Result};

use super::definition::AuditDefinition;

/// Load an [`AuditDefinition`] from a YAML file.
///
/// Returns a detailed, actionable error on parse failure.
pub fn load(path: &Path) -> Result<AuditDefinition> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read audit definition file: {}", path.display()))?;

    serde_yaml::from_str::<AuditDefinition>(&raw).with_context(|| {
        format!(
            "Audit definition file is not valid YAML or has an unexpected structure: {}",
            path.display()
        )
    })
}

/// Save an [`AuditDefinition`] to `path`.
///
/// * If `backup` is `true` and the target file already exists, the current
///   content is copied to `<path>.bak` before overwriting.
/// * The actual write goes to `<path>.tmp` first, then `rename` is used for
///   an atomic swap so that a failed write never leaves a partial file.
pub fn save(def: &AuditDefinition, path: &Path, backup: bool) -> Result<()> {
    // Optional backup.
    if backup && path.exists() {
        let bak = path.with_extension("bak");
        std::fs::copy(path, &bak).with_context(|| {
            format!("Cannot create backup {}", bak.display())
        })?;
        log::info!("Backup written to {}", bak.display());
    }

    // Serialize to YAML.
    let yaml = serde_yaml::to_string(def)
        .context("Failed to serialize audit definition to YAML")?;

    // Write to a temp file in the same directory for atomic rename.
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, yaml.as_bytes())
        .with_context(|| format!("Cannot write to temp file {}", tmp.display()))?;

    // Atomic rename.
    std::fs::rename(&tmp, path).with_context(|| {
        format!(
            "Cannot rename {} → {}",
            tmp.display(),
            path.display()
        )
    })?;

    log::info!("Audit definition saved to {}", path.display());
    Ok(())
}
