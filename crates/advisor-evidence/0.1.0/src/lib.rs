use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use advisor_core::{
    EvidenceBundle, LoadedManifest, Receipt, ReviewDependencyKind, ReviewInputs, TrustNote,
    parse_manifest_dependency_entries,
};
use serde::Deserialize;

pub fn recommendation_evidence(intent: &str, goal: Option<&str>) -> EvidenceBundle {
    let mut receipts = vec![Receipt {
        source: "input".to_string(),
        summary: format!("Recommendation requested for intent '{}'.", intent),
        detail: match goal {
            Some(goal) => format!("Goal override provided: '{}'.", goal),
            None => "No goal override provided.".to_string(),
        },
    }];

    if let Some(goal) = goal {
        receipts.push(Receipt {
            source: "goal".to_string(),
            summary: format!("Goal text '{}' was used for deterministic goal handling.", goal),
            detail:
                "Goal handling in phase 2 normalizes onto a checked-in vocabulary before applying curated fit scores."
                    .to_string(),
        });
    }

    EvidenceBundle {
        receipts,
        trust_notes: catalog_only_trust_notes(),
    }
}

pub fn comparison_evidence(crate_names: &[String], intent: Option<&str>) -> EvidenceBundle {
    EvidenceBundle {
        receipts: vec![Receipt {
            source: "input".to_string(),
            summary: format!("Compared requested crates: {}.", crate_names.join(", ")),
            detail: match intent {
                Some(intent) => format!("Comparison constrained to intent '{}'.", intent),
                None => "Comparison used the unconstrained curated catalog.".to_string(),
            },
        }],
        trust_notes: catalog_only_trust_notes(),
    }
}

pub fn explanation_evidence(crate_name: &str, intent: Option<&str>) -> EvidenceBundle {
    EvidenceBundle {
        receipts: vec![Receipt {
            source: "input".to_string(),
            summary: format!("Explanation requested for '{}'.", crate_name),
            detail: match intent {
                Some(intent) => format!("Requested intent context: '{}'.", intent),
                None => "No explicit intent context provided.".to_string(),
            },
        }],
        trust_notes: catalog_only_trust_notes(),
    }
}

pub fn load_review_inputs(
    manifest_path: Option<&Path>,
    lockfile_path: Option<&Path>,
) -> ReviewInputs {
    let manifest_path = normalize_existing_path(
        manifest_path
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("Cargo.toml")),
    );
    let lockfile_path = normalize_existing_path(
        lockfile_path
            .map(Path::to_path_buf)
            .unwrap_or_else(|| default_lockfile_path(&manifest_path)),
    );

    let (manifest_contents, manifest_receipt) = read_optional_file(&manifest_path, "manifest");
    let (lockfile_contents, lockfile_receipt) = read_optional_file(&lockfile_path, "lockfile");

    let mut receipts = vec![manifest_receipt, lockfile_receipt];
    let mut trust_notes = catalog_only_trust_notes();
    let mut manifests = Vec::new();
    let mut dependencies = Vec::new();

    if let Some(contents) = manifest_contents.as_deref() {
        let root_package_name = parse_package_name(contents);
        manifests.push(LoadedManifest {
            manifest_path: manifest_path.clone(),
            package_name: root_package_name.clone(),
            is_root: true,
        });

        match discover_workspace_metadata(&manifest_path) {
            Ok(metadata) => {
                receipts.push(Receipt {
                    source: "cargo metadata".to_string(),
                    summary: format!(
                        "Discovered {} package manifest(s) for review rooted at '{}'.",
                        metadata.packages.len(),
                        manifest_path.display()
                    ),
                    detail: if metadata.packages.is_empty() {
                        "The manifest resolved without package entries, so review fell back to the root manifest only."
                            .to_string()
                    } else {
                        format!(
                            "Workspace packages: {}",
                            metadata
                                .packages
                                .iter()
                                .map(|package| {
                                    format!("{} at '{}'", package.name, package.manifest_path.display())
                                })
                                .collect::<Vec<_>>()
                                .join("; ")
                        )
                    },
                });

                for package in metadata.packages {
                    let is_root = package.manifest_path == manifest_path;
                    manifests.push(LoadedManifest {
                        manifest_path: package.manifest_path.clone(),
                        package_name: Some(package.name.clone()),
                        is_root,
                    });
                    dependencies.extend(package.dependencies.into_iter().map(|dependency| {
                        advisor_core::ManifestDependency {
                            manifest_path: package.manifest_path.clone(),
                            package_name: Some(package.name.clone()),
                            dependency_name: dependency.name.clone(),
                            declared_name: dependency.rename.unwrap_or(dependency.name),
                            kind: dependency_kind(dependency.kind.as_deref()),
                            target: dependency.target,
                        }
                    }));
                }

                if dependencies.is_empty() {
                    dependencies.extend(parse_manifest_dependency_entries(
                        contents,
                        &manifest_path,
                        root_package_name.as_deref(),
                    ));
                }
            }
            Err(error) => {
                receipts.push(Receipt {
                    source: "cargo metadata".to_string(),
                    summary: format!(
                        "Could not derive workspace package data from '{}'.",
                        manifest_path.display()
                    ),
                    detail: error,
                });
                trust_notes.push(TrustNote {
                    label: "workspace discovery fallback".to_string(),
                    detail:
                        "Review fell back to parsing only the requested manifest because local cargo metadata discovery did not succeed."
                            .to_string(),
                });
                dependencies.extend(parse_manifest_dependency_entries(
                    contents,
                    &manifest_path,
                    root_package_name.as_deref(),
                ));
            }
        }
    }

    ReviewInputs {
        manifest_path,
        manifest_contents,
        lockfile_path,
        lockfile_contents,
        manifests,
        dependencies,
        evidence: EvidenceBundle {
            receipts,
            trust_notes,
        },
    }
}

fn read_optional_file(path: &Path, label: &str) -> (Option<String>, Receipt) {
    match fs::read_to_string(path) {
        Ok(contents) => (
            Some(contents),
            Receipt {
                source: "local file".to_string(),
                summary: format!("Loaded {} from '{}'.", label, path.display()),
                detail: "Only local file contents were used; no external metadata was fetched."
                    .to_string(),
            },
        ),
        Err(error) => (
            None,
            Receipt {
                source: "local file".to_string(),
                summary: format!("Could not load {} from '{}'.", label, path.display()),
                detail: format!("Read failed with: {error}"),
            },
        ),
    }
}

fn default_lockfile_path(manifest_path: &Path) -> PathBuf {
    manifest_path
        .parent()
        .map(|parent| parent.join("Cargo.lock"))
        .unwrap_or_else(|| PathBuf::from("Cargo.lock"))
}

fn normalize_existing_path(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).unwrap_or(path)
}

fn discover_workspace_metadata(manifest_path: &Path) -> Result<CargoMetadata, String> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--no-deps")
        .arg("--manifest-path")
        .arg(manifest_path)
        .output()
        .map_err(|error| format!("failed to run cargo metadata: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("failed to parse cargo metadata json: {error}"))
}

fn parse_package_name(contents: &str) -> Option<String> {
    let mut in_package_section = false;
    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_package_section = line == "[package]";
            continue;
        }
        if !in_package_section || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "name" {
            continue;
        }
        let value = value.trim().trim_matches('"');
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

fn dependency_kind(kind: Option<&str>) -> ReviewDependencyKind {
    match kind {
        Some("dev") => ReviewDependencyKind::Dev,
        Some("build") => ReviewDependencyKind::Build,
        _ => ReviewDependencyKind::Normal,
    }
}

fn catalog_only_trust_notes() -> Vec<TrustNote> {
    vec![TrustNote {
        label: "local evidence boundary".to_string(),
        detail: "No live registry, docs, benchmark, or security sources were consulted."
            .to_string(),
    }]
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
}

#[derive(Debug, Deserialize)]
struct MetadataPackage {
    name: String,
    manifest_path: PathBuf,
    dependencies: Vec<MetadataDependency>,
}

#[derive(Debug, Deserialize)]
struct MetadataDependency {
    name: String,
    kind: Option<String>,
    rename: Option<String>,
    target: Option<String>,
}
