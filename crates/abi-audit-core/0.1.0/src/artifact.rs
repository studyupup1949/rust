use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::{CrateType, Message, Metadata, Package};

use crate::model::{ArtifactKind, BinaryArtifactSnapshot};

pub(crate) fn build_and_collect_artifacts(
    metadata: &Metadata,
    packages: &[&Package],
) -> Result<BTreeMap<String, Vec<BinaryArtifactSnapshot>>> {
    let mut artifacts = BTreeMap::new();
    if packages.is_empty() {
        return Ok(artifacts);
    }

    let manifest_path = PathBuf::from(metadata.workspace_root.as_std_path()).join("Cargo.toml");
    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--offline")
        .arg("--lib")
        .arg("--message-format=json-render-diagnostics")
        .arg("--manifest-path")
        .arg(&manifest_path);
    for package in packages {
        command.arg("--package").arg(package.name.as_str());
    }

    let output = command
        .output()
        .context("failed to run `cargo build` for artifact inspection")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(
            "artifact build failed for `{}`: {}",
            manifest_path.display(),
            if stderr.is_empty() {
                "cargo build exited unsuccessfully without diagnostics".to_string()
            } else {
                stderr
            }
        );
    }

    let selected_packages = packages
        .iter()
        .map(|package| (package.id.clone(), package.name.to_string()))
        .collect::<BTreeMap<_, _>>();

    for message in Message::parse_stream(Cursor::new(output.stdout)) {
        let message = message.context("failed to parse cargo build output")?;
        let Message::CompilerArtifact(artifact) = message else {
            continue;
        };

        let Some(package_name) = selected_packages.get(&artifact.package_id) else {
            continue;
        };
        for filename in artifact.filenames {
            let path = filename.into_std_path_buf();
            let Some(kind) = artifact_kind_for_path(&path, &artifact.target.crate_types) else {
                continue;
            };
            artifacts
                .entry(package_name.clone())
                .or_insert_with(Vec::new)
                .push(snapshot_artifact(
                    &path,
                    kind,
                    Path::new(metadata.workspace_root.as_std_path()),
                ));
        }
    }

    for package_artifacts in artifacts.values_mut() {
        let mut deduped = BTreeMap::new();
        for artifact in package_artifacts.drain(..) {
            deduped.insert(artifact.path.clone(), artifact);
        }
        *package_artifacts = deduped.into_values().collect();
    }

    Ok(artifacts)
}

fn artifact_kind_for_path(path: &Path, crate_types: &[CrateType]) -> Option<ArtifactKind> {
    let extension = path.extension()?.to_string_lossy();
    match extension.as_ref() {
        "dylib" | "so" | "dll" => Some(ArtifactKind::Cdylib),
        "a" => Some(ArtifactKind::Staticlib),
        "lib"
            if crate_types
                .iter()
                .map(ToString::to_string)
                .any(|crate_type| crate_type == "staticlib")
                && !crate_types
                    .iter()
                    .map(ToString::to_string)
                    .any(|crate_type| crate_type == "cdylib") =>
        {
            Some(ArtifactKind::Staticlib)
        }
        _ => None,
    }
}

fn snapshot_artifact(
    path: &Path,
    kind: ArtifactKind,
    workspace_root: &Path,
) -> BinaryArtifactSnapshot {
    let path_display = relative_display(workspace_root, path);
    match kind {
        ArtifactKind::Staticlib => BinaryArtifactSnapshot {
            path: path_display,
            kind,
            format: "archive".to_string(),
            inspected: false,
            inspector: None,
            exported_symbols: Vec::new(),
            notes: vec![
                "static libraries are recorded as build evidence only; phase 3 does not treat archive members as authoritative public export truth".to_string(),
            ],
        },
        ArtifactKind::Cdylib => {
            let format = dynamic_library_format(path);
            match collect_dynamic_symbols(path, &format) {
                Ok((inspector, exported_symbols, mut notes)) => {
                    if format == "mach_o" {
                        notes.push(
                            "Mach-O export names are normalized by stripping the leading underscore used in the symbol table".to_string(),
                        );
                    }
                    BinaryArtifactSnapshot {
                        path: path_display,
                        kind,
                        format,
                        inspected: true,
                        inspector: Some(inspector),
                        exported_symbols,
                        notes,
                    }
                }
                Err(error) => BinaryArtifactSnapshot {
                    path: path_display,
                    kind,
                    format,
                    inspected: false,
                    inspector: None,
                    exported_symbols: Vec::new(),
                    notes: vec![format!("symbol inspection unavailable: {error:#}")],
                },
            }
        }
    }
}

fn dynamic_library_format(path: &Path) -> String {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("dylib") => "mach_o".to_string(),
        Some("so") => "elf".to_string(),
        Some("dll") => "pe".to_string(),
        _ => "unknown".to_string(),
    }
}

fn collect_dynamic_symbols(
    path: &Path,
    format: &str,
) -> Result<(String, Vec<String>, Vec<String>)> {
    let attempts = match format {
        "mach_o" => vec![
            ("nm", vec!["-gjU".to_string(), path.display().to_string()]),
            (
                "llvm-nm",
                vec![
                    "--defined-only".to_string(),
                    "--extern-only".to_string(),
                    "--just-symbol-name".to_string(),
                    path.display().to_string(),
                ],
            ),
        ],
        "elf" => vec![
            (
                "nm",
                vec![
                    "-D".to_string(),
                    "--defined-only".to_string(),
                    "--just-symbol-name".to_string(),
                    path.display().to_string(),
                ],
            ),
            (
                "llvm-nm",
                vec![
                    "--defined-only".to_string(),
                    "--extern-only".to_string(),
                    "--just-symbol-name".to_string(),
                    path.display().to_string(),
                ],
            ),
        ],
        "pe" => vec![
            (
                "llvm-nm",
                vec![
                    "--defined-only".to_string(),
                    "--extern-only".to_string(),
                    "--just-symbol-name".to_string(),
                    path.display().to_string(),
                ],
            ),
            (
                "dumpbin",
                vec!["/EXPORTS".to_string(), path.display().to_string()],
            ),
        ],
        _ => vec![(
            "llvm-nm",
            vec![
                "--defined-only".to_string(),
                "--extern-only".to_string(),
                "--just-symbol-name".to_string(),
                path.display().to_string(),
            ],
        )],
    };

    let mut errors = Vec::new();
    for (tool, args) in attempts {
        let output = match Command::new(tool).args(&args).output() {
            Ok(output) => output,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                errors.push(format!("{tool}: not installed"));
                continue;
            }
            Err(error) => {
                errors.push(format!("{tool}: {error}"));
                continue;
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            errors.push(format!(
                "{tool}: {}",
                if stderr.is_empty() {
                    format!("exited with status {}", output.status)
                } else {
                    stderr
                }
            ));
            continue;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let exported_symbols = if tool == "dumpbin" {
            parse_dumpbin_exports(&stdout, format)
        } else {
            parse_nm_exports(&stdout, format)
        };
        return Ok((tool.to_string(), exported_symbols, Vec::new()));
    }

    Err(anyhow!(errors.join(" | ")))
}

fn parse_nm_exports(stdout: &str, format: &str) -> Vec<String> {
    let mut exports = BTreeSet::new();
    for line in stdout.lines() {
        let symbol = normalize_symbol(line, format);
        if !symbol.is_empty() {
            exports.insert(symbol);
        }
    }
    exports.into_iter().collect()
}

fn parse_dumpbin_exports(stdout: &str, format: &str) -> Vec<String> {
    let mut exports = BTreeSet::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let columns = trimmed.split_whitespace().collect::<Vec<_>>();
        if columns.len() < 4 || !columns[0].chars().all(|ch| ch.is_ascii_digit()) {
            continue;
        }
        let symbol = normalize_symbol(columns.last().copied().unwrap_or_default(), format);
        if !symbol.is_empty() {
            exports.insert(symbol);
        }
    }
    exports.into_iter().collect()
}

fn normalize_symbol(symbol: &str, format: &str) -> String {
    let trimmed = symbol.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if format == "mach_o"
        && trimmed.starts_with('_')
        && trimmed.chars().nth(1).is_some_and(is_symbol_char)
    {
        return trimmed[1..].to_string();
    }
    trimmed.to_string()
}

fn is_symbol_char(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()
}

fn relative_display(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}
