use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::{Metadata, MetadataCommand, Package, Target};
use quote::ToTokens;
use regex::Regex;
use syn::{
    Attribute, Fields, File, FnArg, Item, ItemEnum, ItemFn, ItemStruct, ItemType, ItemUnion,
    ReturnType, Type,
};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use walkdir::WalkDir;

use crate::artifact::build_and_collect_artifacts;
use crate::config::{
    AbiAuditConfig, BaselineConfig, BaselineSourceConfig, BaselineSourceKind, HeaderSyncConfig,
    RuleSeverity, load_config,
};
use crate::model::{
    BinaryArtifactSnapshot, CheckReport, CheckResult, CheckSummary, ExportRecord, Finding,
    HeaderDeclaration, HeaderSource, HeaderSyncSnapshot, HeaderSyncTool, PackageSnapshot, Severity,
    SnapshotRun, SourceLocation, TargetOrigin, TargetSnapshot, TypeDeclaration, TypeKind,
    TypeMember, WorkspaceSnapshot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Human,
    Json,
}

#[derive(Debug, Clone)]
struct PackageTargetSpec {
    package: String,
    configured_headers: Vec<PathBuf>,
    header_sync: Option<HeaderSyncConfig>,
    origin: TargetOrigin,
}

#[derive(Debug, Clone)]
struct ResolvedTargetSpec {
    package: String,
    headers: Vec<PathBuf>,
    header_sync: Option<ResolvedHeaderSyncSpec>,
    origin: TargetOrigin,
    header_source: HeaderSource,
}

#[derive(Debug, Clone)]
struct ResolvedHeaderSyncSpec {
    tool: HeaderSyncTool,
    output: PathBuf,
    config: Option<PathBuf>,
    crate_dir: PathBuf,
    verify_freshness: bool,
}

#[derive(Clone)]
struct TypeInfo {
    declaration: TypeDeclaration,
    fields: Vec<TypeMemberInfo>,
    alias: Option<Type>,
}

#[derive(Clone)]
struct TypeMemberInfo {
    name: Option<String>,
    ty: Type,
}

#[derive(Clone)]
struct ParsedSource {
    path: PathBuf,
    relative_file: String,
    text: String,
    parsed: File,
}

#[derive(Debug, Clone)]
struct ParsedPackage {
    snapshot: PackageSnapshot,
    findings: Vec<Finding>,
    target: TargetSnapshot,
    include_in_auto_selection: bool,
}

#[derive(Debug, Clone)]
struct HeaderSyncEvaluation {
    snapshot: HeaderSyncSnapshot,
}

#[derive(Debug, Clone)]
struct Analysis {
    snapshot: WorkspaceSnapshot,
    findings: Vec<Finding>,
}

#[derive(Debug, Clone)]
struct ExportScan {
    record: ExportRecord,
    location: SourceLocation,
}

#[derive(Debug, Clone)]
struct SignatureProjection {
    return_type: String,
    param_types: Vec<String>,
    signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeContext {
    Boundary,
    AggregateField,
}

#[derive(Debug, Clone, Default)]
struct TypeCheck {
    ffi_safe: bool,
    unsafe_reasons: Vec<String>,
    missing_repr_types: BTreeSet<String>,
    opaque_handle_types: BTreeSet<String>,
}

impl TypeCheck {
    fn safe() -> Self {
        Self {
            ffi_safe: true,
            ..Self::default()
        }
    }

    fn unsafe_with_reason(reason: impl Into<String>) -> Self {
        Self {
            ffi_safe: false,
            unsafe_reasons: vec![reason.into()],
            ..Self::default()
        }
    }

    fn merge(&mut self, other: Self) {
        self.ffi_safe &= other.ffi_safe;
        self.unsafe_reasons.extend(other.unsafe_reasons);
        self.missing_repr_types.extend(other.missing_repr_types);
        self.opaque_handle_types.extend(other.opaque_handle_types);
    }
}

pub fn snapshot_workspace(
    manifest_path: Option<&Path>,
    config_path: Option<&Path>,
    output_path: Option<&Path>,
) -> Result<SnapshotRun> {
    let metadata = cargo_metadata(manifest_path)?;
    let workspace_root = PathBuf::from(metadata.workspace_root.as_std_path());
    let config = load_config(&workspace_root, config_path)?;
    let analysis = analyze_workspace(&metadata, &config)?;
    let snapshot = analysis.snapshot;
    let resolved_output = output_path
        .map(PathBuf::from)
        .or_else(|| Some(resolve_path(&workspace_root, &config.snapshot)));

    if let Some(path) = &resolved_output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create snapshot output directory {}",
                    parent.display()
                )
            })?;
        }
        let json = serde_json::to_string_pretty(&snapshot)?;
        fs::write(path, json)
            .with_context(|| format!("failed to write snapshot to {}", path.display()))?;
    }

    Ok(SnapshotRun {
        snapshot,
        output_path: resolved_output,
    })
}

pub fn check_workspace(
    manifest_path: Option<&Path>,
    config_path: Option<&Path>,
    baseline_override: Option<&Path>,
    baseline_snapshot_override: Option<&Path>,
) -> Result<CheckResult> {
    let metadata = cargo_metadata(manifest_path)?;
    let workspace_root = PathBuf::from(metadata.workspace_root.as_std_path());
    let config = load_config(&workspace_root, config_path)?;
    let analysis = analyze_workspace(&metadata, &config)?;
    let snapshot = analysis.snapshot;
    let mut findings = analysis.findings;
    findings.extend(lint_snapshot(&snapshot));

    if let Some(baseline_path) = resolve_baseline_snapshot_path(
        &workspace_root,
        &config.baseline,
        baseline_override,
        baseline_snapshot_override,
    )? {
        let baseline_text = fs::read_to_string(&baseline_path)
            .with_context(|| format!("failed to read baseline at {}", baseline_path.display()))?;
        let baseline: WorkspaceSnapshot =
            serde_json::from_str(&baseline_text).with_context(|| {
                format!(
                    "failed to parse baseline snapshot at {}",
                    baseline_path.display()
                )
            })?;
        findings.extend(diff_against_baseline(&snapshot, &baseline));
    }

    apply_rule_overrides(&mut findings, &config);
    normalize_findings(&mut findings);
    let summary = summarize(&snapshot, &findings);
    let exit_code = if summary.errors > 0 { 1 } else { 0 };

    Ok(CheckResult {
        report: CheckReport {
            snapshot,
            findings,
            summary,
        },
        exit_code,
    })
}

pub fn render_snapshot(run: &SnapshotRun) -> Result<String> {
    let packages = run.snapshot.packages.len();
    let exports = run
        .snapshot
        .packages
        .iter()
        .map(|package| package.exports.len())
        .sum::<usize>();
    let headers = run
        .snapshot
        .packages
        .iter()
        .map(|package| package.headers.len())
        .sum::<usize>();
    let types = run
        .snapshot
        .packages
        .iter()
        .map(|package| package.types.len())
        .sum::<usize>();
    let artifacts = run
        .snapshot
        .packages
        .iter()
        .map(|package| package.artifacts.len())
        .sum::<usize>();
    let path = run
        .output_path
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "<stdout>".to_string());
    Ok(format!(
        "snapshot written to {path}\npackages: {packages}\nexports: {exports}\ntype declarations: {types}\nheader declarations: {headers}\nbinary artifacts: {artifacts}"
    ))
}

pub fn render_check(result: &CheckResult, format: Format) -> Result<String> {
    match format {
        Format::Json => Ok(serde_json::to_string_pretty(&result.report)?),
        Format::Human => {
            let mut output = String::new();
            output.push_str("cargo abi-audit check\n");
            output.push_str(&format!(
                "packages: {} | exports: {} | warnings: {} | errors: {}\n",
                result.report.summary.packages_scanned,
                result.report.summary.exports_scanned,
                result.report.summary.warnings,
                result.report.summary.errors
            ));
            if result.report.findings.is_empty() {
                output.push_str("no findings\n");
            } else {
                output.push('\n');
                for finding in &result.report.findings {
                    output.push_str(&format!(
                        "[{:?}] {}: {}\n",
                        finding.severity, finding.code, finding.message
                    ));
                    if let Some(export) = &finding.export {
                        output.push_str(&format!("  export: {export}\n"));
                    }
                    output.push_str(&format!("  package: {}\n", finding.package));
                    if let Some(location) = &finding.location {
                        output.push_str(&format!(
                            "  location: {}:{}\n",
                            location.path, location.line
                        ));
                    }
                    for evidence in &finding.evidence {
                        output.push_str(&format!("  evidence: {evidence}\n"));
                    }
                    output.push('\n');
                }
            }
            Ok(output.trim_end().to_string())
        }
    }
}

fn cargo_metadata(manifest_path: Option<&Path>) -> Result<Metadata> {
    let mut command = MetadataCommand::new();
    command.no_deps();
    command.other_options(vec!["--offline".to_string()]);
    if let Some(manifest_path) = manifest_path {
        command.manifest_path(manifest_path);
    }
    command.exec().context("failed to run `cargo metadata`")
}

fn analyze_workspace(metadata: &Metadata, config: &AbiAuditConfig) -> Result<Analysis> {
    let workspace_root = PathBuf::from(metadata.workspace_root.as_std_path());
    let target_specs = select_targets(metadata, config);
    let explicit_targets = !config.targets.is_empty();
    let mut parsed_packages = Vec::new();
    let mut packages = Vec::new();
    let mut findings = Vec::new();
    let mut targets = Vec::new();

    for spec in target_specs {
        let package = metadata
            .packages
            .iter()
            .find(|package| package.name.to_string() == spec.package)
            .ok_or_else(|| {
                anyhow!(
                    "configured package `{}` not found in workspace",
                    spec.package
                )
            })?;
        let resolved = resolve_target_spec(package, &spec, &workspace_root);
        let parsed = parse_package(package, &resolved, &workspace_root)?;
        if !explicit_targets && !parsed.include_in_auto_selection {
            continue;
        }
        parsed_packages.push((package, parsed));
    }

    let selected_packages = parsed_packages
        .iter()
        .map(|(package, _)| *package)
        .collect::<Vec<_>>();
    let artifact_snapshots = build_and_collect_artifacts(metadata, &selected_packages)?;

    for (package, mut parsed) in parsed_packages {
        parsed.snapshot.artifacts = artifact_snapshots
            .get(package.name.as_str())
            .cloned()
            .unwrap_or_default();
        findings.extend(parsed.findings);
        targets.push(parsed.target);
        packages.push(parsed.snapshot);
    }

    packages.sort_by(|left, right| left.package.cmp(&right.package));
    targets.sort_by(|left, right| left.package.cmp(&right.package));

    Ok(Analysis {
        snapshot: WorkspaceSnapshot {
            schema_version: 3,
            generated_at_utc: OffsetDateTime::now_utc().format(&Rfc3339)?,
            workspace_root: workspace_root.display().to_string(),
            targets,
            packages,
        },
        findings,
    })
}

fn lint_snapshot(snapshot: &WorkspaceSnapshot) -> Vec<Finding> {
    let mut findings = Vec::new();
    let targets_by_package = snapshot
        .targets
        .iter()
        .map(|target| (target.package.as_str(), target))
        .collect::<BTreeMap<_, _>>();
    for package in &snapshot.packages {
        let target = targets_by_package.get(package.package.as_str()).copied();
        let header_names = package
            .headers
            .iter()
            .map(|header| header.name.clone())
            .collect::<BTreeSet<_>>();
        let export_names = package
            .exports
            .iter()
            .map(|export| export.export_name.clone())
            .collect::<BTreeSet<_>>();
        let headers_by_name = package.headers.iter().fold(
            BTreeMap::<String, Vec<&HeaderDeclaration>>::new(),
            |mut acc, header| {
                acc.entry(header.name.clone()).or_default().push(header);
                acc
            },
        );
        let cdylibs = package
            .artifacts
            .iter()
            .filter(|artifact| artifact.kind == crate::model::ArtifactKind::Cdylib)
            .collect::<Vec<_>>();
        let inspected_cdylibs = cdylibs
            .iter()
            .copied()
            .filter(|artifact| artifact.inspected)
            .collect::<Vec<_>>();
        let compiled_symbols = inspected_cdylibs
            .iter()
            .flat_map(|artifact| artifact.exported_symbols.iter().cloned())
            .collect::<BTreeSet<_>>();
        let needs_binary_artifact = package.crate_types.iter().any(|kind| kind == "cdylib");
        let needs_any_artifact = package
            .crate_types
            .iter()
            .any(|kind| kind == "cdylib" || kind == "staticlib");

        if !package.exports.is_empty() && !needs_any_artifact {
            findings.push(Finding {
                code: "missing-artifact-target".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "package `{}` exposes C ABI exports but is not configured as cdylib/staticlib",
                    package.package
                ),
                package: package.package.clone(),
                export: None,
                location: None,
                evidence: vec![format!("crate types: {}", package.crate_types.join(", "))],
            });
        }

        if needs_any_artifact && package.artifacts.is_empty() {
            findings.push(Finding {
                code: "artifact-build-missing".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "package `{}` declares native library crate types but no compiled artifacts were captured",
                    package.package
                ),
                package: package.package.clone(),
                export: None,
                location: None,
                evidence: vec![format!("crate types: {}", package.crate_types.join(", "))],
            });
        }

        if needs_binary_artifact && cdylibs.is_empty() {
            findings.push(Finding {
                code: "artifact-build-missing".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "package `{}` is configured as a cdylib but no compiled dynamic library was captured",
                    package.package
                ),
                package: package.package.clone(),
                export: None,
                location: None,
                evidence: package
                    .artifacts
                    .iter()
                    .map(|artifact| format!("captured artifact: {}", artifact.path))
                    .collect(),
            });
        } else if needs_binary_artifact && !cdylibs.is_empty() && inspected_cdylibs.is_empty() {
            findings.push(Finding {
                code: "artifact-inspection-unavailable".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "package `{}` built a cdylib, but export symbol inspection was unavailable on this host",
                    package.package
                ),
                package: package.package.clone(),
                export: None,
                location: None,
                evidence: cdylibs
                    .iter()
                    .flat_map(|artifact| artifact_inspection_evidence(artifact))
                    .collect(),
            });
        }

        if let Some(header_sync) = target.and_then(|target| target.header_sync.as_ref()) {
            if !header_sync.output_exists {
                findings.push(Finding {
                    code: "header-sync-missing-output".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "configured {} header output for package `{}` does not exist",
                        header_sync_tool_label(header_sync.tool),
                        package.package
                    ),
                    package: package.package.clone(),
                    export: None,
                    location: None,
                    evidence: header_sync.evidence.clone(),
                });
            }

            if header_sync.config.is_some() && !header_sync.config_exists {
                findings.push(Finding {
                    code: "header-sync-missing-config".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "configured {} header sync for package `{}` references a missing config file",
                        header_sync_tool_label(header_sync.tool),
                        package.package
                    ),
                    package: package.package.clone(),
                    export: None,
                    location: None,
                    evidence: header_sync.evidence.clone(),
                });
            }

            if header_sync.output_exists
                && !package
                    .headers
                    .iter()
                    .any(|header| header.path == header_sync.output)
            {
                findings.push(Finding {
                    code: "header-sync-untracked-header".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "configured {} output `{}` is not part of the audited public header set",
                        header_sync_tool_label(header_sync.tool),
                        header_sync.output
                    ),
                    package: package.package.clone(),
                    export: None,
                    location: None,
                    evidence: header_sync.evidence.clone(),
                });
            }

            if header_sync.stale {
                findings.push(Finding {
                    code: "header-sync-stale".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "configured {} header output for package `{}` appears older than its Rust or config inputs",
                        header_sync_tool_label(header_sync.tool),
                        package.package
                    ),
                    package: package.package.clone(),
                    export: None,
                    location: None,
                    evidence: header_sync.evidence.clone(),
                });
            }
        }

        for header in &package.headers {
            if !export_names.contains(&header.name) {
                findings.push(Finding {
                    code: "header-missing-export".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "header declares `{}` but no matching Rust export was discovered",
                        header.name
                    ),
                    package: package.package.clone(),
                    export: Some(header.name.clone()),
                    location: Some(SourceLocation {
                        path: header.path.clone(),
                        line: header.line,
                    }),
                    evidence: vec![format!("header signature: {}", header.signature)],
                });
            }
        }

        for export in &package.exports {
            if !export.has_stable_export_attr {
                findings.push(Finding {
                    code: "missing-export-attr".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "export `{}` uses C ABI but does not declare `no_mangle` or `export_name`",
                        export.export_name
                    ),
                    package: package.package.clone(),
                    export: Some(export.export_name.clone()),
                    location: Some(SourceLocation {
                        path: export.file.clone(),
                        line: export.line,
                    }),
                    evidence: vec![format!("signature: {}", export.signature)],
                });
            }

            match headers_by_name.get(&export.export_name) {
                None if !header_names.is_empty() => findings.push(Finding {
                    code: "export-missing-header".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "Rust export `{}` is not declared in any configured public header",
                        export.export_name
                    ),
                    package: package.package.clone(),
                    export: Some(export.export_name.clone()),
                    location: Some(SourceLocation {
                        path: export.file.clone(),
                        line: export.line,
                    }),
                    evidence: vec![format!("signature: {}", export.signature)],
                }),
                Some(headers) => {
                    if let Some(export_signature) = &export.normalized_signature {
                        let comparable_headers = headers
                            .iter()
                            .filter_map(|header| {
                                header
                                    .normalized_signature
                                    .as_ref()
                                    .map(|signature| (*header, signature))
                            })
                            .collect::<Vec<_>>();
                        if !comparable_headers.is_empty()
                            && !comparable_headers
                                .iter()
                                .any(|(_, signature)| *signature == export_signature)
                        {
                            let mut evidence = vec![format!("rust signature: {export_signature}")];
                            evidence.extend(comparable_headers.into_iter().map(
                                |(header, signature)| {
                                    format!(
                                        "header {}:{} => {}",
                                        header.path, header.line, signature
                                    )
                                },
                            ));
                            findings.push(Finding {
                                code: "header-signature-mismatch".to_string(),
                                severity: Severity::Warning,
                                message: format!(
                                    "Rust export `{}` does not match the configured header declaration shape",
                                    export.export_name
                                ),
                                package: package.package.clone(),
                                export: Some(export.export_name.clone()),
                                location: Some(SourceLocation {
                                    path: export.file.clone(),
                                    line: export.line,
                                }),
                                evidence,
                            });
                        }
                    }
                }
                None => {}
            }

            if export.has_stable_export_attr
                && !compiled_symbols.is_empty()
                && !compiled_symbols.contains(&export.export_name)
            {
                let mut evidence = vec![format!("expected symbol: {}", export.export_name)];
                evidence.extend(compiled_symbol_evidence(&inspected_cdylibs));
                findings.push(Finding {
                    code: "artifact-missing-export".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "compiled artifacts for `{}` do not expose the expected symbol `{}`",
                        package.package, export.export_name
                    ),
                    package: package.package.clone(),
                    export: Some(export.export_name.clone()),
                    location: Some(SourceLocation {
                        path: export.file.clone(),
                        line: export.line,
                    }),
                    evidence,
                });
            }
        }
    }
    findings
}

fn diff_against_baseline(
    current: &WorkspaceSnapshot,
    baseline: &WorkspaceSnapshot,
) -> Vec<Finding> {
    let current_exports = export_index(current);
    let baseline_exports = export_index(baseline);
    let mut findings = Vec::new();

    for (key, baseline_export) in &baseline_exports {
        match current_exports.get(key) {
            None => findings.push(Finding {
                code: "baseline-drift".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "baseline export `{}` is missing in the current snapshot",
                    key
                ),
                package: baseline_export.0.clone(),
                export: Some(baseline_export.1.export_name.clone()),
                location: Some(SourceLocation {
                    path: baseline_export.1.file.clone(),
                    line: baseline_export.1.line,
                }),
                evidence: vec![format!(
                    "baseline signature: {}",
                    comparable_signature(&baseline_export.1)
                )],
            }),
            Some(current_export)
                if comparable_signature(&current_export.1)
                    != comparable_signature(&baseline_export.1) =>
            {
                findings.push(Finding {
                    code: "baseline-drift".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "baseline export `{}` changed signature",
                        baseline_export.1.export_name
                    ),
                    package: baseline_export.0.clone(),
                    export: Some(baseline_export.1.export_name.clone()),
                    location: Some(SourceLocation {
                        path: current_export.1.file.clone(),
                        line: current_export.1.line,
                    }),
                    evidence: vec![
                        format!("baseline: {}", comparable_signature(&baseline_export.1)),
                        format!("current: {}", comparable_signature(&current_export.1)),
                    ],
                });
            }
            Some(_) => {}
        }
    }

    for (key, current_export) in &current_exports {
        if !baseline_exports.contains_key(key) {
            findings.push(Finding {
                code: "baseline-drift".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "current snapshot adds export `{}` that is not present in the baseline",
                    current_export.1.export_name
                ),
                package: current_export.0.clone(),
                export: Some(current_export.1.export_name.clone()),
                location: Some(SourceLocation {
                    path: current_export.1.file.clone(),
                    line: current_export.1.line,
                }),
                evidence: vec![format!(
                    "current signature: {}",
                    comparable_signature(&current_export.1)
                )],
            });
        }
    }

    let current_headers = header_index(current);
    let baseline_headers = header_index(baseline);

    for (key, baseline_header) in &baseline_headers {
        match current_headers.get(key) {
            None => findings.push(Finding {
                code: "baseline-drift".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "baseline header declaration `{}` is missing in the current snapshot",
                    baseline_header.1.name
                ),
                package: baseline_header.0.clone(),
                export: Some(baseline_header.1.name.clone()),
                location: Some(SourceLocation {
                    path: baseline_header.1.path.clone(),
                    line: baseline_header.1.line,
                }),
                evidence: vec![format!(
                    "baseline header signature: {}",
                    comparable_header_signature(&baseline_header.1)
                )],
            }),
            Some(current_header)
                if comparable_header_signature(&current_header.1)
                    != comparable_header_signature(&baseline_header.1) =>
            {
                findings.push(Finding {
                    code: "baseline-drift".to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "baseline header declaration `{}` changed signature",
                        baseline_header.1.name
                    ),
                    package: baseline_header.0.clone(),
                    export: Some(baseline_header.1.name.clone()),
                    location: Some(SourceLocation {
                        path: current_header.1.path.clone(),
                        line: current_header.1.line,
                    }),
                    evidence: vec![
                        format!(
                            "baseline header: {}",
                            comparable_header_signature(&baseline_header.1)
                        ),
                        format!(
                            "current header: {}",
                            comparable_header_signature(&current_header.1)
                        ),
                    ],
                });
            }
            Some(_) => {}
        }
    }

    for (key, current_header) in &current_headers {
        if !baseline_headers.contains_key(key) {
            findings.push(Finding {
                code: "baseline-drift".to_string(),
                severity: Severity::Warning,
                message: format!(
                    "current snapshot adds header declaration `{}` that is not present in the baseline",
                    current_header.1.name
                ),
                package: current_header.0.clone(),
                export: Some(current_header.1.name.clone()),
                location: Some(SourceLocation {
                    path: current_header.1.path.clone(),
                    line: current_header.1.line,
                }),
                evidence: vec![format!(
                    "current header signature: {}",
                    comparable_header_signature(&current_header.1)
                )],
            });
        }
    }

    findings
}

fn export_index(snapshot: &WorkspaceSnapshot) -> BTreeMap<String, (String, ExportRecord)> {
    let mut index = BTreeMap::new();
    for package in &snapshot.packages {
        for export in &package.exports {
            index.insert(
                format!("{}::{}", package.package, export.export_name),
                (package.package.clone(), export.clone()),
            );
        }
    }
    index
}

fn header_index(snapshot: &WorkspaceSnapshot) -> BTreeMap<String, (String, HeaderDeclaration)> {
    let mut index = BTreeMap::new();
    for package in &snapshot.packages {
        for header in &package.headers {
            index.insert(
                format!("{}::{}", package.package, header.name),
                (package.package.clone(), header.clone()),
            );
        }
    }
    index
}

fn normalize_findings(findings: &mut Vec<Finding>) {
    findings.sort_by(|left, right| {
        (
            severity_rank(left.severity),
            &left.package,
            &left.export,
            &left.code,
            &left.message,
        )
            .cmp(&(
                severity_rank(right.severity),
                &right.package,
                &right.export,
                &right.code,
                &right.message,
            ))
    });
}

fn summarize(snapshot: &WorkspaceSnapshot, findings: &[Finding]) -> CheckSummary {
    let exports_scanned = snapshot
        .packages
        .iter()
        .map(|package| package.exports.len())
        .sum::<usize>();
    let warnings = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Warning)
        .count();
    let errors = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();

    CheckSummary {
        packages_scanned: snapshot.packages.len(),
        exports_scanned,
        warnings,
        errors,
    }
}

fn select_targets(metadata: &Metadata, config: &AbiAuditConfig) -> Vec<PackageTargetSpec> {
    if !config.targets.is_empty() {
        return config
            .targets
            .iter()
            .map(|target| PackageTargetSpec {
                package: target.package.clone(),
                configured_headers: target.headers.clone(),
                header_sync: target.header_sync.clone(),
                origin: TargetOrigin::Configured,
            })
            .collect();
    }

    let mut targets = metadata
        .workspace_packages()
        .iter()
        .filter(|package| package.targets.iter().any(is_library_target))
        .map(|package| PackageTargetSpec {
            package: package.name.to_string(),
            configured_headers: Vec::new(),
            header_sync: None,
            origin: TargetOrigin::Auto,
        })
        .collect::<Vec<_>>();
    targets.sort_by(|left, right| left.package.cmp(&right.package));
    targets
}

fn resolve_target_spec(
    package: &Package,
    spec: &PackageTargetSpec,
    workspace_root: &Path,
) -> ResolvedTargetSpec {
    let package_root = manifest_dir(package);
    let header_sync = spec
        .header_sync
        .as_ref()
        .map(|header_sync| resolve_header_sync(workspace_root, &package_root, header_sync));
    if spec.configured_headers.is_empty() {
        let headers = discover_headers(&package_root);
        let header_source = if headers.is_empty() {
            HeaderSource::None
        } else {
            HeaderSource::Auto
        };
        return ResolvedTargetSpec {
            package: spec.package.clone(),
            headers,
            header_sync,
            origin: spec.origin,
            header_source,
        };
    }

    let headers = spec
        .configured_headers
        .iter()
        .map(|path| resolve_header_path(workspace_root, &package_root, path))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    ResolvedTargetSpec {
        package: spec.package.clone(),
        headers,
        header_sync,
        origin: spec.origin,
        header_source: HeaderSource::Configured,
    }
}

fn resolve_header_sync(
    workspace_root: &Path,
    package_root: &Path,
    config: &HeaderSyncConfig,
) -> ResolvedHeaderSyncSpec {
    ResolvedHeaderSyncSpec {
        tool: config.tool,
        output: resolve_header_path(workspace_root, package_root, &config.output),
        config: config
            .config
            .as_ref()
            .map(|path| resolve_header_path(workspace_root, package_root, path)),
        crate_dir: config
            .crate_dir
            .as_ref()
            .map(|path| resolve_header_path(workspace_root, package_root, path))
            .unwrap_or_else(|| package_root.to_path_buf()),
        verify_freshness: config.verify_freshness,
    }
}

fn evaluate_header_sync(
    header_sync: &ResolvedHeaderSyncSpec,
    sources: &[ParsedSource],
    workspace_root: &Path,
) -> Result<HeaderSyncEvaluation> {
    let output_exists = header_sync.output.exists();
    let config_exists = header_sync
        .config
        .as_ref()
        .map(|path| path.exists())
        .unwrap_or(true);
    let output_modified = output_exists
        .then(|| fs::metadata(&header_sync.output))
        .transpose()?
        .and_then(|metadata| metadata.modified().ok());
    let mut stale_inputs = Vec::new();

    if header_sync.verify_freshness {
        if let Some(output_modified) = output_modified {
            for source in sources {
                if file_is_newer(&source.path, output_modified)? {
                    stale_inputs.push(source.relative_file.clone());
                }
            }
            if let Some(config) = &header_sync.config {
                if file_is_newer(config, output_modified)? {
                    stale_inputs.push(relative_display(workspace_root, config));
                }
            }
        }
    }

    let output_display = relative_display(workspace_root, &header_sync.output);
    let config_display = header_sync
        .config
        .as_ref()
        .map(|path| relative_display(workspace_root, path));
    let crate_dir_display = relative_display(workspace_root, &header_sync.crate_dir);
    let mut evidence = vec![
        format!("tool: {}", header_sync_tool_label(header_sync.tool)),
        format!("expected output: {output_display}"),
        format!(
            "suggested command: {}",
            render_header_sync_command(
                &crate_dir_display,
                &output_display,
                config_display.as_deref()
            )
        ),
    ];
    if let Some(config) = &config_display {
        evidence.push(format!("config: {config}"));
    } else {
        evidence.push("config: <cbindgen default search>".to_string());
    }
    evidence.push(format!("crate dir: {crate_dir_display}"));
    evidence.push(format!("output exists: {output_exists}"));
    evidence.push(format!("config exists: {config_exists}"));
    evidence.push(format!(
        "freshness check enabled: {}",
        header_sync.verify_freshness
    ));
    if let Some(output_modified) = output_modified {
        evidence.push(format!(
            "output modified: {}",
            system_time_display(output_modified)
        ));
    }
    if !stale_inputs.is_empty() {
        evidence.extend(
            stale_inputs
                .iter()
                .map(|path| format!("newer input: {path}")),
        );
    }

    Ok(HeaderSyncEvaluation {
        snapshot: HeaderSyncSnapshot {
            tool: header_sync.tool,
            output: output_display,
            crate_dir: crate_dir_display.clone(),
            command: render_header_sync_command(
                &crate_dir_display,
                &relative_display(workspace_root, &header_sync.output),
                config_display.as_deref(),
            ),
            config: config_display,
            output_exists,
            config_exists,
            freshness_checked: header_sync.verify_freshness,
            stale: header_sync.verify_freshness && output_exists && !stale_inputs.is_empty(),
            evidence,
        },
    })
}

fn parse_package(
    package: &Package,
    spec: &ResolvedTargetSpec,
    workspace_root: &Path,
) -> Result<ParsedPackage> {
    let package_root = manifest_dir(package);
    let sources = load_package_sources(&package_root, workspace_root)?;
    let header_sync = spec
        .header_sync
        .as_ref()
        .map(|header_sync| evaluate_header_sync(header_sync, &sources, workspace_root))
        .transpose()?;
    let type_db = collect_type_db(&sources)?;
    let types = snapshot_types(&type_db);
    let mut exports = Vec::new();
    let mut findings = Vec::new();

    for source in &sources {
        visit_items(&source.parsed.items, &mut |item| {
            if let Item::Fn(item_fn) = item {
                let scan = parse_export_fn(item_fn, &source.text, &source.relative_file, &type_db);
                match scan {
                    Ok(Some(scan)) => {
                        let export_name = scan.record.export_name.clone();
                        let package_name = package.name.to_string();
                        let location = scan.location.clone();
                        let signature_check = check_signature(&item_fn.sig, &type_db);

                        if !signature_check.unsafe_reasons.is_empty() {
                            findings.push(Finding {
                                code: "non-ffi-safe-signature".to_string(),
                                severity: Severity::Error,
                                message: format!(
                                    "export `{}` uses types that are not C-ABI-safe in this MVP",
                                    export_name
                                ),
                                package: package_name.clone(),
                                export: Some(export_name.clone()),
                                location: Some(location.clone()),
                                evidence: type_check_notes(signature_check.clone()),
                            });
                        }

                        if !signature_check.missing_repr_types.is_empty() {
                            findings.push(Finding {
                                code: "missing-repr".to_string(),
                                severity: Severity::Error,
                                message: format!(
                                    "export `{}` passes local Rust types by value without `repr(C)`, `repr(transparent)`, or an explicit integer enum repr",
                                    export_name
                                ),
                                package: package_name,
                                export: Some(export_name),
                                location: Some(location),
                                evidence: signature_check
                                    .missing_repr_types
                                    .into_iter()
                                    .map(|name| {
                                        format!("by-value type `{name}` is missing a stable repr")
                                    })
                                    .collect(),
                            });
                        }

                        exports.push(scan.record);
                    }
                    Ok(None) => {}
                    Err(error) => findings.push(Finding {
                        code: "export-parse-failure".to_string(),
                        severity: Severity::Error,
                        message: format!("failed to inspect Rust export candidate: {error:#}"),
                        package: package.name.to_string(),
                        export: None,
                        location: Some(SourceLocation {
                            path: source.relative_file.clone(),
                            line: 1,
                        }),
                        evidence: Vec::new(),
                    }),
                }
            }
        });
    }

    let mut headers = Vec::new();
    for header_path in &spec.headers {
        headers.extend(parse_header_file(header_path, workspace_root)?);
    }

    headers.sort_by(|left, right| {
        (&left.path, left.line, &left.name, &left.signature).cmp(&(
            &right.path,
            right.line,
            &right.name,
            &right.signature,
        ))
    });
    exports.sort_by(|left, right| {
        (&left.export_name, &left.signature).cmp(&(&right.export_name, &right.signature))
    });

    let crate_types = package
        .targets
        .iter()
        .filter(|target| is_library_target(target))
        .flat_map(|target| target.crate_types.iter().map(ToString::to_string))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let include_in_auto_selection = !exports.is_empty()
        || !headers.is_empty()
        || crate_types
            .iter()
            .any(|kind| kind == "cdylib" || kind == "staticlib");

    Ok(ParsedPackage {
        target: TargetSnapshot {
            package: spec.package.clone(),
            headers: spec
                .headers
                .iter()
                .map(|path| relative_display(workspace_root, path))
                .collect(),
            origin: spec.origin,
            header_source: spec.header_source,
            header_sync: header_sync.map(|header_sync| header_sync.snapshot),
        },
        snapshot: PackageSnapshot {
            package: package.name.to_string(),
            manifest_path: relative_display(workspace_root, package.manifest_path.as_std_path()),
            crate_types,
            types,
            headers,
            exports,
            artifacts: Vec::new(),
        },
        findings,
        include_in_auto_selection,
    })
}

fn load_package_sources(package_root: &Path, workspace_root: &Path) -> Result<Vec<ParsedSource>> {
    let mut sources = Vec::new();
    for source_path in collect_package_sources(package_root)? {
        let text = fs::read_to_string(&source_path)
            .with_context(|| format!("failed to read {}", source_path.display()))?;
        let parsed: File = syn::parse_file(&text)
            .with_context(|| format!("failed to parse {}", source_path.display()))?;
        let relative_file = relative_display(workspace_root, &source_path);
        sources.push(ParsedSource {
            path: source_path,
            relative_file,
            text,
            parsed,
        });
    }
    Ok(sources)
}

fn collect_type_db(sources: &[ParsedSource]) -> Result<BTreeMap<String, TypeInfo>> {
    let mut db = BTreeMap::new();
    for source in sources {
        visit_items(&source.parsed.items, &mut |item| match item {
            Item::Struct(item_struct) => {
                collect_item_type(&mut db, ItemTypeRef::Struct(item_struct), source);
            }
            Item::Enum(item_enum) => {
                collect_item_type(&mut db, ItemTypeRef::Enum(item_enum), source);
            }
            Item::Union(item_union) => {
                collect_item_type(&mut db, ItemTypeRef::Union(item_union), source);
            }
            Item::Type(item_type) => {
                collect_item_type(&mut db, ItemTypeRef::Alias(item_type), source);
            }
            _ => {}
        });
    }

    let summaries = db
        .keys()
        .cloned()
        .map(|name| {
            let mut visiting = BTreeSet::new();
            let check = inspect_local_type_by_value(&name, &db, &mut visiting);
            (name, (check.ffi_safe, type_check_notes(check)))
        })
        .collect::<Vec<_>>();

    for (name, (ffi_safe, notes)) in summaries {
        if let Some(info) = db.get_mut(&name) {
            info.declaration.by_value_ffi_safe = ffi_safe;
            info.declaration.by_value_notes = notes;
        }
    }

    Ok(db)
}

fn snapshot_types(type_db: &BTreeMap<String, TypeInfo>) -> Vec<TypeDeclaration> {
    type_db
        .values()
        .map(|info| info.declaration.clone())
        .collect::<Vec<_>>()
}

fn collect_item_type(
    db: &mut BTreeMap<String, TypeInfo>,
    item: ItemTypeRef<'_>,
    source: &ParsedSource,
) {
    let (ident, attrs, vis_public, kind, fields, alias, fieldless, needle) = match item {
        ItemTypeRef::Struct(item) => (
            &item.ident,
            &item.attrs,
            is_pub(&item.vis),
            TypeKind::Struct,
            collect_fields(&item.fields),
            None,
            matches!(item.fields, Fields::Unit),
            format!("struct {}", item.ident),
        ),
        ItemTypeRef::Enum(item) => (
            &item.ident,
            &item.attrs,
            is_pub(&item.vis),
            TypeKind::Enum,
            collect_enum_fields(item),
            None,
            item.variants
                .iter()
                .all(|variant| matches!(variant.fields, Fields::Unit)),
            format!("enum {}", item.ident),
        ),
        ItemTypeRef::Union(item) => (
            &item.ident,
            &item.attrs,
            is_pub(&item.vis),
            TypeKind::Union,
            item.fields
                .named
                .iter()
                .map(|field| TypeMemberInfo {
                    name: field.ident.as_ref().map(ToString::to_string),
                    ty: field.ty.clone(),
                })
                .collect(),
            None,
            false,
            format!("union {}", item.ident),
        ),
        ItemTypeRef::Alias(item) => (
            &item.ident,
            &item.attrs,
            is_pub(&item.vis),
            TypeKind::Alias,
            vec![TypeMemberInfo {
                name: None,
                ty: (*item.ty).clone(),
            }],
            Some((*item.ty).clone()),
            false,
            format!("type {}", item.ident),
        ),
    };

    if !vis_public {
        return;
    }

    let line = find_line_number(&source.text, &needle).unwrap_or(1);
    let reprs = parse_repr_attrs(attrs);
    let declaration = TypeDeclaration {
        name: ident.to_string(),
        kind,
        file: source.relative_file.clone(),
        line,
        canonical_name: to_canonical_identifier(&ident.to_string()),
        reprs,
        fields: fields
            .iter()
            .map(|field| TypeMember {
                name: field.name.clone(),
                ty: normalize_ws(&field.ty.to_token_stream().to_string()),
            })
            .collect(),
        fieldless,
        by_value_ffi_safe: false,
        by_value_notes: Vec::new(),
    };

    db.insert(
        ident.to_string(),
        TypeInfo {
            declaration,
            fields,
            alias,
        },
    );
}

fn collect_fields(fields: &Fields) -> Vec<TypeMemberInfo> {
    match fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|field| TypeMemberInfo {
                name: field.ident.as_ref().map(ToString::to_string),
                ty: field.ty.clone(),
            })
            .collect(),
        Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(index, field)| TypeMemberInfo {
                name: Some(index.to_string()),
                ty: field.ty.clone(),
            })
            .collect(),
        Fields::Unit => Vec::new(),
    }
}

fn collect_enum_fields(item: &ItemEnum) -> Vec<TypeMemberInfo> {
    let mut fields = Vec::new();
    for variant in &item.variants {
        match &variant.fields {
            Fields::Named(named) => {
                for field in &named.named {
                    let field_name = field
                        .ident
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "field".to_string());
                    fields.push(TypeMemberInfo {
                        name: Some(format!("{}::{}", variant.ident, field_name)),
                        ty: field.ty.clone(),
                    });
                }
            }
            Fields::Unnamed(unnamed) => {
                for (index, field) in unnamed.unnamed.iter().enumerate() {
                    fields.push(TypeMemberInfo {
                        name: Some(format!("{}::{}", variant.ident, index)),
                        ty: field.ty.clone(),
                    });
                }
            }
            Fields::Unit => {}
        }
    }
    fields
}

fn collect_package_sources(package_root: &Path) -> Result<Vec<PathBuf>> {
    let src_root = package_root.join("src");
    if !src_root.exists() {
        return Ok(Vec::new());
    }

    let mut files = WalkDir::new(&src_root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn parse_export_fn(
    item_fn: &ItemFn,
    source_text: &str,
    relative_file: &str,
    type_db: &BTreeMap<String, TypeInfo>,
) -> Result<Option<ExportScan>> {
    let abi = item_fn
        .sig
        .abi
        .as_ref()
        .and_then(|abi| abi.name.as_ref())
        .map(|name| name.value())
        .unwrap_or_default();
    if abi != "C" && abi != "system" {
        return Ok(None);
    }

    let has_export_attr = item_fn.attrs.iter().any(is_export_attr);
    if !is_pub(&item_fn.vis) && !has_export_attr {
        return Ok(None);
    }

    let export_attr = item_fn.attrs.iter().find_map(export_attr_label);
    let export_name = export_attr
        .as_ref()
        .and_then(|attr| parse_export_name_value(attr))
        .unwrap_or_else(|| item_fn.sig.ident.to_string());
    let line = find_line_number(source_text, &format!("fn {}", item_fn.sig.ident)).unwrap_or(1);
    let signature = signature_string(&item_fn.sig);
    let projection = project_rust_signature(&item_fn.sig, &export_name, type_db);
    let signature_check = check_signature(&item_fn.sig, type_db);

    Ok(Some(ExportScan {
        record: ExportRecord {
            rust_name: item_fn.sig.ident.to_string(),
            export_name,
            abi,
            signature,
            normalized_signature: projection
                .as_ref()
                .map(|projection| projection.signature.clone()),
            return_type: projection
                .as_ref()
                .map(|projection| projection.return_type.clone()),
            param_types: projection
                .as_ref()
                .map(|projection| projection.param_types.clone())
                .unwrap_or_default(),
            file: relative_file.to_string(),
            line,
            has_stable_export_attr: has_export_attr,
            export_attr,
            opaque_handle_types: signature_check.opaque_handle_types.into_iter().collect(),
        },
        location: SourceLocation {
            path: relative_file.to_string(),
            line,
        },
    }))
}

fn parse_header_file(path: &Path, workspace_root: &Path) -> Result<Vec<HeaderDeclaration>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read header {}", path.display()))?;
    let regex = Regex::new(
        r"^(?P<ret>.+?)\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*\((?P<params>.*)\)\s*;\s*$",
    )?;
    let typedef_regex =
        Regex::new(r"^typedef\s+(?P<base>.+?)\s+(?P<alias>[A-Za-z_][A-Za-z0-9_]*)\s*;\s*$")?;
    let path_display = relative_display(workspace_root, path);
    let mut declarations = Vec::new();
    let mut aliases = BTreeMap::new();
    let mut pending = String::new();
    let mut start_line = 1usize;
    for (index, raw_line) in text.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }

        if pending.is_empty() {
            start_line = line_number;
        } else {
            pending.push(' ');
        }
        pending.push_str(line);
        if !line.ends_with(';') {
            continue;
        }

        let candidate = normalize_ws(&pending);
        pending.clear();
        if candidate.starts_with("typedef") {
            if !candidate.contains('{') && !candidate.contains('}') {
                if let Some(capture) = typedef_regex.captures(&candidate) {
                    let base = capture
                        .name("base")
                        .map(|value| value.as_str().trim())
                        .unwrap_or_default();
                    let alias = capture
                        .name("alias")
                        .map(|value| value.as_str().trim().to_string())
                        .unwrap_or_default();
                    if let Some(canonical) = canonicalize_c_type(base, &aliases) {
                        aliases.insert(alias, canonical);
                    }
                }
            }
            continue;
        }
        if candidate.contains('{') || candidate.contains('}') {
            continue;
        }

        if let Some(capture) = regex.captures(&candidate) {
            let return_text = capture
                .name("ret")
                .map(|value| value.as_str().trim().to_string())
                .ok_or_else(|| anyhow!("header parse error in {}", path.display()))?;
            let name = capture
                .name("name")
                .map(|value| value.as_str().trim().to_string())
                .ok_or_else(|| anyhow!("header parse error in {}", path.display()))?;
            let params = capture
                .name("params")
                .map(|value| value.as_str().trim().to_string())
                .unwrap_or_default();
            let projection = project_header_signature(&name, &return_text, &params, &aliases);
            declarations.push(HeaderDeclaration {
                path: path_display.clone(),
                line: start_line,
                name,
                signature: candidate,
                normalized_signature: projection
                    .as_ref()
                    .map(|projection| projection.signature.clone()),
                return_type: projection
                    .as_ref()
                    .map(|projection| projection.return_type.clone()),
                param_types: projection
                    .as_ref()
                    .map(|projection| projection.param_types.clone())
                    .unwrap_or_default(),
            });
        }
    }
    Ok(declarations)
}

fn check_signature(sig: &syn::Signature, type_db: &BTreeMap<String, TypeInfo>) -> TypeCheck {
    let mut check = TypeCheck::safe();
    for input in &sig.inputs {
        match input {
            FnArg::Receiver(_) => check.merge(TypeCheck::unsafe_with_reason(
                "methods with receivers are not part of a C ABI export surface",
            )),
            FnArg::Typed(arg) => {
                let mut visiting = BTreeSet::new();
                check.merge(inspect_type(
                    &arg.ty,
                    type_db,
                    false,
                    TypeContext::Boundary,
                    &mut visiting,
                ));
            }
        }
    }
    if let ReturnType::Type(_, ty) = &sig.output {
        let mut visiting = BTreeSet::new();
        check.merge(inspect_type(
            ty,
            type_db,
            false,
            TypeContext::Boundary,
            &mut visiting,
        ));
    }
    check
}

fn inspect_type(
    ty: &Type,
    type_db: &BTreeMap<String, TypeInfo>,
    behind_pointer: bool,
    context: TypeContext,
    visiting: &mut BTreeSet<String>,
) -> TypeCheck {
    match ty {
        Type::Path(type_path) => inspect_type_path(type_path, type_db, behind_pointer, visiting),
        Type::Ptr(type_ptr) => inspect_type(&type_ptr.elem, type_db, true, context, visiting),
        Type::Reference(_) => TypeCheck::unsafe_with_reason(format!(
            "`{}` uses Rust references",
            normalize_ws(&ty.to_token_stream().to_string())
        )),
        Type::Slice(_) => TypeCheck::unsafe_with_reason(format!(
            "`{}` uses Rust slices",
            normalize_ws(&ty.to_token_stream().to_string())
        )),
        Type::Tuple(tuple) if tuple.elems.is_empty() => TypeCheck::safe(),
        Type::Tuple(_) => TypeCheck::unsafe_with_reason(format!(
            "`{}` uses tuples",
            normalize_ws(&ty.to_token_stream().to_string())
        )),
        Type::Array(array) => {
            if behind_pointer || context == TypeContext::AggregateField {
                inspect_type(
                    &array.elem,
                    type_db,
                    false,
                    TypeContext::AggregateField,
                    visiting,
                )
            } else {
                TypeCheck::unsafe_with_reason(format!(
                    "`{}` uses arrays by value directly at the ABI boundary, which this MVP does not treat as stable",
                    normalize_ws(&ty.to_token_stream().to_string())
                ))
            }
        }
        Type::BareFn(bare_fn) => inspect_bare_fn_type(bare_fn, type_db, visiting),
        Type::TraitObject(_) => TypeCheck::unsafe_with_reason(format!(
            "`{}` uses a trait object",
            normalize_ws(&ty.to_token_stream().to_string())
        )),
        Type::ImplTrait(_) => TypeCheck::unsafe_with_reason(format!(
            "`{}` uses impl Trait",
            normalize_ws(&ty.to_token_stream().to_string())
        )),
        Type::Infer(_) => TypeCheck::unsafe_with_reason("signature uses inferred type `_`"),
        Type::Macro(_) => TypeCheck::unsafe_with_reason(format!(
            "`{}` uses a macro type, which this MVP cannot validate",
            normalize_ws(&ty.to_token_stream().to_string())
        )),
        Type::Never(_) => TypeCheck::unsafe_with_reason("signature uses the never type `!`"),
        Type::Paren(paren) => inspect_type(&paren.elem, type_db, behind_pointer, context, visiting),
        Type::Group(group) => inspect_type(&group.elem, type_db, behind_pointer, context, visiting),
        _ => TypeCheck::unsafe_with_reason(format!(
            "`{}` uses a type shape not supported by this MVP",
            normalize_ws(&ty.to_token_stream().to_string())
        )),
    }
}

fn inspect_bare_fn_type(
    bare_fn: &syn::TypeBareFn,
    type_db: &BTreeMap<String, TypeInfo>,
    visiting: &mut BTreeSet<String>,
) -> TypeCheck {
    let abi = bare_fn
        .abi
        .as_ref()
        .and_then(|abi| abi.name.as_ref())
        .map(|name| name.value())
        .unwrap_or_default();
    if abi != "C" && abi != "system" {
        return TypeCheck::unsafe_with_reason(format!(
            "`{}` uses a function pointer without `extern \"C\"`/`extern \"system\"`",
            normalize_ws(&bare_fn.to_token_stream().to_string())
        ));
    }

    let mut check = TypeCheck::safe();
    for input in &bare_fn.inputs {
        check.merge(inspect_type(
            &input.ty,
            type_db,
            false,
            TypeContext::Boundary,
            visiting,
        ));
    }
    if let ReturnType::Type(_, ty) = &bare_fn.output {
        check.merge(inspect_type(
            ty,
            type_db,
            false,
            TypeContext::Boundary,
            visiting,
        ));
    }
    check
}

fn inspect_type_path(
    type_path: &syn::TypePath,
    type_db: &BTreeMap<String, TypeInfo>,
    behind_pointer: bool,
    visiting: &mut BTreeSet<String>,
) -> TypeCheck {
    let rendered = normalize_ws(&type_path.to_token_stream().to_string());
    let ident = type_path
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
        .unwrap_or_default();
    if ident.is_empty() {
        return TypeCheck::unsafe_with_reason(format!(
            "`{rendered}` could not be resolved into a C-ABI-safe type"
        ));
    }

    if is_primitive_ffi_safe(&ident) || is_c_ffi_primitive(&ident) || ident == "c_void" {
        return TypeCheck::safe();
    }

    if matches!(ident.as_str(), "String" | "Vec" | "str") {
        return TypeCheck::unsafe_with_reason(format!(
            "`{rendered}` uses Rust-owned string/vector types"
        ));
    }

    if ident == "char" {
        return TypeCheck::unsafe_with_reason("Rust `char` is not a C `char`");
    }

    if let Some(bare_fn) = option_bare_fn(type_path) {
        return inspect_bare_fn_type(bare_fn, type_db, visiting);
    }

    if type_path
        .path
        .segments
        .iter()
        .any(|segment| !matches!(segment.arguments, syn::PathArguments::None))
    {
        return TypeCheck::unsafe_with_reason(format!(
            "`{rendered}` uses generic arguments, which this MVP does not model as C ABI safe"
        ));
    }

    if let Some(type_info) = type_db.get(&ident) {
        if behind_pointer {
            let mut check = TypeCheck::safe();
            if !type_info.declaration.by_value_ffi_safe {
                check.opaque_handle_types.insert(ident);
            }
            return check;
        }
        return inspect_local_type_by_value(&ident, type_db, visiting);
    }

    TypeCheck::safe()
}

fn inspect_local_type_by_value(
    name: &str,
    type_db: &BTreeMap<String, TypeInfo>,
    visiting: &mut BTreeSet<String>,
) -> TypeCheck {
    let Some(type_info) = type_db.get(name) else {
        return TypeCheck::safe();
    };

    if !visiting.insert(name.to_string()) {
        return TypeCheck::unsafe_with_reason(format!(
            "local type `{name}` recurses in a way this MVP cannot prove ABI-stable"
        ));
    }

    let check = match type_info.declaration.kind {
        TypeKind::Alias => type_info
            .alias
            .as_ref()
            .map(|alias| inspect_type(alias, type_db, false, TypeContext::Boundary, visiting))
            .unwrap_or_else(|| {
                TypeCheck::unsafe_with_reason(format!("type alias `{name}` could not be resolved"))
            }),
        TypeKind::Struct | TypeKind::Union => {
            inspect_aggregate_type(name, type_info, type_db, visiting)
        }
        TypeKind::Enum => inspect_enum_type(name, type_info),
    };

    visiting.remove(name);
    check
}

fn inspect_aggregate_type(
    name: &str,
    type_info: &TypeInfo,
    type_db: &BTreeMap<String, TypeInfo>,
    visiting: &mut BTreeSet<String>,
) -> TypeCheck {
    if has_repr(&type_info.declaration.reprs, "transparent") {
        if type_info.declaration.kind == TypeKind::Union {
            return TypeCheck::unsafe_with_reason(format!(
                "union `{name}` cannot be validated as `repr(transparent)` in this MVP"
            ));
        }
        if type_info.fields.len() != 1 {
            return TypeCheck::unsafe_with_reason(format!(
                "`{name}` uses `repr(transparent)` but does not have exactly one field"
            ));
        }
        return inspect_type(
            &type_info.fields[0].ty,
            type_db,
            false,
            TypeContext::AggregateField,
            visiting,
        );
    }

    if !has_repr(&type_info.declaration.reprs, "C") {
        let mut check = TypeCheck::safe();
        check.ffi_safe = false;
        check.missing_repr_types.insert(name.to_string());
        return check;
    }

    if type_info.fields.is_empty() && type_info.declaration.kind == TypeKind::Struct {
        return TypeCheck::unsafe_with_reason(format!(
            "`{name}` is a zero-sized struct, which this MVP does not treat as a stable C ABI value"
        ));
    }

    let mut check = TypeCheck::safe();
    for field in &type_info.fields {
        check.merge(inspect_type(
            &field.ty,
            type_db,
            false,
            TypeContext::AggregateField,
            visiting,
        ));
    }
    check
}

fn inspect_enum_type(name: &str, type_info: &TypeInfo) -> TypeCheck {
    if !type_info.declaration.fieldless {
        return TypeCheck::unsafe_with_reason(format!(
            "enum `{name}` carries data and is not modeled as a stable C ABI enum in this MVP"
        ));
    }

    if has_repr(&type_info.declaration.reprs, "C")
        || integer_repr(&type_info.declaration.reprs).is_some()
    {
        return TypeCheck::safe();
    }

    let mut check = TypeCheck::safe();
    check.ffi_safe = false;
    check.missing_repr_types.insert(name.to_string());
    check
}

fn signature_string(sig: &syn::Signature) -> String {
    normalize_ws(&sig.to_token_stream().to_string())
}

fn project_rust_signature(
    sig: &syn::Signature,
    export_name: &str,
    type_db: &BTreeMap<String, TypeInfo>,
) -> Option<SignatureProjection> {
    let mut param_types = Vec::new();
    for input in &sig.inputs {
        match input {
            FnArg::Receiver(_) => return None,
            FnArg::Typed(arg) => {
                let mut visiting = BTreeSet::new();
                param_types.push(project_rust_type(&arg.ty, type_db, &mut visiting)?);
            }
        }
    }

    let return_type = match &sig.output {
        ReturnType::Default => "void".to_string(),
        ReturnType::Type(_, ty) => {
            let mut visiting = BTreeSet::new();
            project_rust_type(ty, type_db, &mut visiting)?
        }
    };

    Some(SignatureProjection {
        signature: format_canonical_signature(export_name, &return_type, &param_types),
        return_type,
        param_types,
    })
}

fn project_rust_type(
    ty: &Type,
    type_db: &BTreeMap<String, TypeInfo>,
    visiting: &mut BTreeSet<String>,
) -> Option<String> {
    match ty {
        Type::Path(type_path) => project_rust_type_path(type_path, type_db, visiting),
        Type::Ptr(type_ptr) => {
            let inner = project_rust_type(&type_ptr.elem, type_db, visiting)?;
            Some(if type_ptr.mutability.is_some() {
                format!("*mut {inner}")
            } else {
                format!("*const {inner}")
            })
        }
        Type::Tuple(tuple) if tuple.elems.is_empty() => Some("void".to_string()),
        Type::BareFn(bare_fn) => project_rust_bare_fn(bare_fn, type_db),
        Type::Paren(paren) => project_rust_type(&paren.elem, type_db, visiting),
        Type::Group(group) => project_rust_type(&group.elem, type_db, visiting),
        Type::Array(array) => {
            let inner = project_rust_type(&array.elem, type_db, visiting)?;
            Some(format!(
                "[{}; {}]",
                inner,
                normalize_ws(&array.len.to_token_stream().to_string())
            ))
        }
        _ => None,
    }
}

fn project_rust_type_path(
    type_path: &syn::TypePath,
    type_db: &BTreeMap<String, TypeInfo>,
    visiting: &mut BTreeSet<String>,
) -> Option<String> {
    let last = type_path.path.segments.last()?;
    let ident = last.ident.to_string();
    if is_primitive_ffi_safe(&ident) {
        return Some(ident);
    }
    if ident == "c_void" {
        return Some("void".to_string());
    }
    if let Some(mapped) = canonical_rust_c_alias(&ident) {
        return Some(mapped.to_string());
    }
    if let Some(bare_fn) = option_bare_fn(type_path) {
        return project_rust_bare_fn(bare_fn, type_db);
    }
    if let Some(type_info) = type_db.get(&ident) {
        if let Some(alias) = &type_info.alias {
            if !visiting.insert(ident.clone()) {
                return None;
            }
            let projected = project_rust_type(alias, type_db, visiting);
            visiting.remove(&ident);
            return projected;
        }
        if has_repr(&type_info.declaration.reprs, "transparent") && type_info.fields.len() == 1 {
            if !visiting.insert(ident.clone()) {
                return None;
            }
            let projected = project_rust_type(&type_info.fields[0].ty, type_db, visiting);
            visiting.remove(&ident);
            return projected;
        }
        if let Some(integer_repr) = integer_repr(&type_info.declaration.reprs) {
            return Some(integer_repr.to_string());
        }
        return Some(type_info.declaration.canonical_name.clone());
    }
    if type_path
        .path
        .segments
        .iter()
        .any(|segment| !matches!(segment.arguments, syn::PathArguments::None))
    {
        return None;
    }
    Some(to_canonical_identifier(&ident))
}

fn project_rust_bare_fn(
    bare_fn: &syn::TypeBareFn,
    type_db: &BTreeMap<String, TypeInfo>,
) -> Option<String> {
    let abi = bare_fn
        .abi
        .as_ref()
        .and_then(|abi| abi.name.as_ref())
        .map(|name| name.value())
        .unwrap_or_default();
    if abi != "C" && abi != "system" {
        return None;
    }

    let mut args = Vec::new();
    for input in &bare_fn.inputs {
        let mut visiting = BTreeSet::new();
        args.push(project_rust_type(&input.ty, type_db, &mut visiting)?);
    }
    let ret = match &bare_fn.output {
        ReturnType::Default => "void".to_string(),
        ReturnType::Type(_, ty) => {
            let mut visiting = BTreeSet::new();
            project_rust_type(ty, type_db, &mut visiting)?
        }
    };
    Some(format!("extern_c_fn({}) -> {ret}", args.join(", ")))
}

fn project_header_signature(
    name: &str,
    return_text: &str,
    params_text: &str,
    aliases: &BTreeMap<String, String>,
) -> Option<SignatureProjection> {
    let return_type = canonicalize_c_type(return_text, aliases)?;
    let raw_params = split_header_parameters(params_text);
    let param_types = if raw_params.len() == 1 && raw_params[0].trim() == "void" {
        Vec::new()
    } else {
        raw_params
            .into_iter()
            .map(|param| canonicalize_header_param(&param, aliases))
            .collect::<Option<Vec<_>>>()?
    };

    Some(SignatureProjection {
        signature: format_canonical_signature(name, &return_type, &param_types),
        return_type,
        param_types,
    })
}

fn canonicalize_header_param(raw: &str, aliases: &BTreeMap<String, String>) -> Option<String> {
    let normalized = normalize_c_tokens(raw);
    canonicalize_c_type(&normalized, aliases).or_else(|| {
        let stripped = strip_trailing_identifier(&normalized)?;
        canonicalize_c_type(&stripped, aliases)
    })
}

fn canonicalize_c_type(raw: &str, aliases: &BTreeMap<String, String>) -> Option<String> {
    let normalized = normalize_c_tokens(raw);
    if normalized.is_empty() {
        return None;
    }
    if normalized.contains("(*") || normalized.contains(")") || normalized.contains("[") {
        return None;
    }

    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    let first_star = tokens.iter().position(|token| *token == "*");
    let base_tokens = match first_star {
        Some(index) => &tokens[..index],
        None => &tokens[..],
    };
    let pointer_count = tokens.iter().filter(|token| **token == "*").count();
    let pointee_const = base_tokens.iter().any(|token| *token == "const");
    let base = canonicalize_c_base(base_tokens, aliases)?;

    let mut rendered = base;
    if pointer_count > 0 {
        rendered = if pointee_const {
            format!("*const {rendered}")
        } else {
            format!("*mut {rendered}")
        };
        for _ in 1..pointer_count {
            rendered = format!("*mut {rendered}");
        }
    }
    Some(rendered)
}

fn canonicalize_c_base(tokens: &[&str], aliases: &BTreeMap<String, String>) -> Option<String> {
    let filtered = tokens
        .iter()
        .copied()
        .filter(|token| !matches!(*token, "const" | "volatile" | "restrict"))
        .collect::<Vec<_>>();
    let base = filtered.join(" ");
    if base.is_empty() {
        return None;
    }

    Some(match base.as_str() {
        "void" => "void".to_string(),
        "bool" => "bool".to_string(),
        "char" => "c_char".to_string(),
        "signed char" => "i8".to_string(),
        "unsigned char" => "u8".to_string(),
        "short" | "short int" | "signed short" | "signed short int" => "c_short".to_string(),
        "unsigned short" | "unsigned short int" => "c_ushort".to_string(),
        "int" | "signed" | "signed int" => "c_int".to_string(),
        "unsigned" | "unsigned int" => "c_uint".to_string(),
        "long" | "long int" | "signed long" | "signed long int" => "c_long".to_string(),
        "unsigned long" | "unsigned long int" => "c_ulong".to_string(),
        "long long" | "long long int" | "signed long long" | "signed long long int" => {
            "c_longlong".to_string()
        }
        "unsigned long long" | "unsigned long long int" => "c_ulonglong".to_string(),
        "float" => "f32".to_string(),
        "double" => "f64".to_string(),
        "size_t" | "uintptr_t" => "usize".to_string(),
        "intptr_t" => "isize".to_string(),
        "int8_t" => "i8".to_string(),
        "uint8_t" => "u8".to_string(),
        "int16_t" => "i16".to_string(),
        "uint16_t" => "u16".to_string(),
        "int32_t" => "i32".to_string(),
        "uint32_t" => "u32".to_string(),
        "int64_t" => "i64".to_string(),
        "uint64_t" => "u64".to_string(),
        "int128_t" => "i128".to_string(),
        "uint128_t" => "u128".to_string(),
        _ => {
            if let Some(stripped) = base
                .strip_prefix("struct ")
                .or_else(|| base.strip_prefix("enum "))
                .or_else(|| base.strip_prefix("union "))
            {
                stripped.to_string()
            } else if let Some(alias) = aliases.get(&base) {
                alias.clone()
            } else if filtered.len() == 1 {
                base
            } else {
                return None;
            }
        }
    })
}

fn split_header_parameters(params: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for ch in params.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => {
                values.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        values.push(current.trim().to_string());
    }
    values
}

fn strip_trailing_identifier(raw: &str) -> Option<String> {
    let tokens = raw.split_whitespace().collect::<Vec<_>>();
    if tokens.len() < 2 {
        return None;
    }
    let last = tokens.last()?;
    if !is_identifier_like(last) {
        return None;
    }
    Some(tokens[..tokens.len() - 1].join(" "))
}

fn comparable_signature(export: &ExportRecord) -> &str {
    export
        .normalized_signature
        .as_deref()
        .unwrap_or(&export.signature)
}

fn comparable_header_signature(header: &HeaderDeclaration) -> &str {
    header
        .normalized_signature
        .as_deref()
        .unwrap_or(&header.signature)
}

fn artifact_inspection_evidence(artifact: &BinaryArtifactSnapshot) -> Vec<String> {
    let mut evidence = vec![format!("artifact: {} ({})", artifact.path, artifact.format)];
    if let Some(inspector) = &artifact.inspector {
        evidence.push(format!("inspector: {inspector}"));
    }
    evidence.extend(
        artifact
            .notes
            .iter()
            .map(|note| format!("artifact note: {note}")),
    );
    evidence
}

fn compiled_symbol_evidence(artifacts: &[&BinaryArtifactSnapshot]) -> Vec<String> {
    let mut evidence = Vec::new();
    for artifact in artifacts {
        evidence.push(format!(
            "inspected artifact: {} via {}",
            artifact.path,
            artifact.inspector.as_deref().unwrap_or("<unknown>")
        ));
        let preview = artifact
            .exported_symbols
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>();
        if !preview.is_empty() {
            evidence.push(format!("observed exports: {}", preview.join(", ")));
        }
    }
    evidence
}

fn header_sync_tool_label(tool: HeaderSyncTool) -> &'static str {
    match tool {
        HeaderSyncTool::Cbindgen => "cbindgen",
    }
}

fn render_header_sync_command(
    crate_dir_display: &str,
    output_display: &str,
    config_display: Option<&str>,
) -> String {
    let mut command = vec!["cbindgen".to_string(), crate_dir_display.to_string()];
    if let Some(config) = config_display {
        command.push("--config".to_string());
        command.push(config.to_string());
    }
    command.push("--output".to_string());
    command.push(output_display.to_string());
    command.join(" ")
}

fn file_is_newer(path: &Path, than: SystemTime) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    Ok(fs::metadata(path)?
        .modified()
        .is_ok_and(|modified| modified > than))
}

fn system_time_display(time: SystemTime) -> String {
    OffsetDateTime::from(time)
        .format(&Rfc3339)
        .unwrap_or_else(|_| "<invalid timestamp>".to_string())
}

fn parse_repr_attrs(attrs: &[Attribute]) -> Vec<String> {
    let mut reprs = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("repr") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if let Some(ident) = meta.path.get_ident() {
                let repr = ident.to_string();
                if !reprs.contains(&repr) {
                    reprs.push(repr);
                }
            }
            Ok(())
        });
    }
    reprs
}

fn has_repr(reprs: &[String], expected: &str) -> bool {
    reprs.iter().any(|repr| repr == expected)
}

fn integer_repr(reprs: &[String]) -> Option<&str> {
    reprs.iter().find_map(|repr| {
        matches!(
            repr.as_str(),
            "u8" | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
        )
        .then_some(repr.as_str())
    })
}

fn option_bare_fn(type_path: &syn::TypePath) -> Option<&syn::TypeBareFn> {
    let last = type_path.path.segments.last()?;
    if last.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    match args.args.first()? {
        syn::GenericArgument::Type(Type::BareFn(bare_fn)) => Some(bare_fn),
        _ => None,
    }
}

fn export_attr_label(attr: &Attribute) -> Option<String> {
    if has_attr_name(attr, "no_mangle") {
        return Some("no_mangle".to_string());
    }
    if has_attr_name(attr, "export_name") {
        return Some(attr.to_token_stream().to_string());
    }
    None
}

fn parse_export_name_value(attr: &str) -> Option<String> {
    let regex = Regex::new(r#""([^"]+)""#).ok()?;
    regex
        .captures(attr)
        .and_then(|capture| capture.get(1))
        .map(|value| value.as_str().to_string())
}

fn has_attr_name(attr: &Attribute, expected: &str) -> bool {
    if attr.path().is_ident(expected) {
        return true;
    }
    if !attr.path().is_ident("unsafe") {
        return false;
    }

    let mut found = false;
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident(expected) {
            found = true;
        }
        Ok(())
    });
    found
}

fn is_export_attr(attr: &Attribute) -> bool {
    has_attr_name(attr, "no_mangle") || has_attr_name(attr, "export_name")
}

fn is_library_target(target: &Target) -> bool {
    target.kind.iter().any(|kind| {
        let kind = kind.to_string();
        matches!(kind.as_str(), "lib" | "rlib" | "cdylib" | "staticlib")
    })
}

fn manifest_dir(package: &Package) -> PathBuf {
    package
        .manifest_path
        .as_std_path()
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn relative_display(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn resolve_path(workspace_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

fn resolve_baseline_snapshot_path(
    workspace_root: &Path,
    configured: &BaselineConfig,
    baseline_override: Option<&Path>,
    baseline_snapshot_override: Option<&Path>,
) -> Result<Option<PathBuf>> {
    if let Some(path) = baseline_override {
        let resolved = resolve_path(workspace_root, path);
        if !resolved.exists() {
            bail!("baseline path `{}` does not exist", resolved.display());
        }
        return resolve_baseline_override_snapshot(
            workspace_root,
            &resolved,
            baseline_snapshot_override,
        )
        .map(Some);
    }

    match configured {
        BaselineConfig::Path(path) => {
            let resolved = resolve_path(workspace_root, path);
            if resolved.exists() {
                Ok(Some(resolved))
            } else {
                Ok(None)
            }
        }
        BaselineConfig::Source(source) => {
            let root = resolve_path(workspace_root, &source.path);
            if !root.exists() {
                bail!(
                    "configured baseline path `{}` does not exist",
                    root.display()
                );
            }
            resolve_configured_baseline_snapshot(workspace_root, source).map(Some)
        }
    }
}

fn resolve_baseline_override_snapshot(
    workspace_root: &Path,
    baseline_root: &Path,
    baseline_snapshot_override: Option<&Path>,
) -> Result<PathBuf> {
    if baseline_root.is_file() {
        return Ok(baseline_root.to_path_buf());
    }

    let snapshot = baseline_snapshot_override
        .map(|path| resolve_path(workspace_root, path))
        .unwrap_or_else(|| baseline_root.join("snapshot.json"));
    if !snapshot.exists() {
        bail!(
            "baseline directory `{}` does not contain `{}`",
            baseline_root.display(),
            snapshot.display()
        );
    }
    Ok(snapshot)
}

fn resolve_configured_baseline_snapshot(
    workspace_root: &Path,
    source: &BaselineSourceConfig,
) -> Result<PathBuf> {
    let root = resolve_path(workspace_root, &source.path);
    match source.kind {
        BaselineSourceKind::Snapshot => {
            if root.is_dir() {
                bail!(
                    "configured snapshot baseline `{}` points to a directory; use `kind = \"artifact_dir\"` for extracted release artifacts",
                    root.display()
                );
            }
            Ok(root)
        }
        BaselineSourceKind::ArtifactDir => {
            if !root.is_dir() {
                bail!(
                    "configured artifact baseline `{}` is not a directory",
                    root.display()
                );
            }
            let snapshot = root.join(&source.snapshot);
            if !snapshot.exists() {
                bail!(
                    "configured artifact baseline `{}` does not contain `{}`",
                    root.display(),
                    snapshot.display()
                );
            }
            Ok(snapshot)
        }
    }
}

fn apply_rule_overrides(findings: &mut Vec<Finding>, config: &AbiAuditConfig) {
    findings.retain_mut(|finding| {
        let Some(rule) = config.rules.get(&finding.code) else {
            return true;
        };
        match rule.severity {
            Some(RuleSeverity::Off) => false,
            Some(RuleSeverity::Warning) => {
                finding.severity = Severity::Warning;
                true
            }
            Some(RuleSeverity::Error) => {
                finding.severity = Severity::Error;
                true
            }
            None => true,
        }
    });
}

fn resolve_header_path(workspace_root: &Path, package_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let workspace_relative = workspace_root.join(path);
    if workspace_relative.exists() {
        return workspace_relative;
    }
    let package_relative = package_root.join(path);
    if package_relative.exists() {
        return package_relative;
    }
    workspace_relative
}

fn discover_headers(package_root: &Path) -> Vec<PathBuf> {
    let mut headers = BTreeSet::new();
    let include_root = package_root.join("include");
    if include_root.exists() {
        for entry in WalkDir::new(&include_root)
            .into_iter()
            .filter_map(Result::ok)
            .map(|entry| entry.into_path())
        {
            if entry.extension().is_some_and(|ext| ext == "h") {
                headers.insert(entry);
            }
        }
    }

    if let Ok(entries) = fs::read_dir(package_root) {
        for entry in entries.filter_map(Result::ok).map(|entry| entry.path()) {
            if entry.extension().is_some_and(|ext| ext == "h") {
                headers.insert(entry);
            }
        }
    }

    headers.into_iter().collect()
}

fn visit_items(items: &[Item], f: &mut impl FnMut(&Item)) {
    for item in items {
        f(item);
        if let Item::Mod(item_mod) = item {
            if let Some((_, nested)) = &item_mod.content {
                visit_items(nested, f);
            }
        }
    }
}

fn find_line_number(source: &str, needle: &str) -> Option<usize> {
    source
        .lines()
        .enumerate()
        .find_map(|(index, line)| line.contains(needle).then_some(index + 1))
}

fn normalize_ws(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_c_tokens(value: &str) -> String {
    normalize_ws(
        &value
            .replace('*', " * ")
            .replace('(', " ( ")
            .replace(')', " ) "),
    )
}

fn type_check_notes(check: TypeCheck) -> Vec<String> {
    let mut notes = check.unsafe_reasons;
    notes.extend(
        check
            .missing_repr_types
            .into_iter()
            .map(|name| format!("type `{name}` is missing a stable repr")),
    );
    notes.sort();
    notes.dedup();
    notes
}

fn format_canonical_signature(name: &str, return_type: &str, param_types: &[String]) -> String {
    format!("{return_type} {name}({})", param_types.join(", "))
}

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
    }
}

fn is_primitive_ffi_safe(name: &str) -> bool {
    matches!(
        name,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "bool"
    )
}

fn is_c_ffi_primitive(name: &str) -> bool {
    matches!(
        name,
        "c_char"
            | "c_schar"
            | "c_uchar"
            | "c_short"
            | "c_ushort"
            | "c_int"
            | "c_uint"
            | "c_long"
            | "c_ulong"
            | "c_longlong"
            | "c_ulonglong"
            | "c_float"
            | "c_double"
    )
}

fn canonical_rust_c_alias(name: &str) -> Option<&'static str> {
    Some(match name {
        "c_char" => "c_char",
        "c_schar" => "i8",
        "c_uchar" => "u8",
        "c_short" => "c_short",
        "c_ushort" => "c_ushort",
        "c_int" => "c_int",
        "c_uint" => "c_uint",
        "c_long" => "c_long",
        "c_ulong" => "c_ulong",
        "c_longlong" => "c_longlong",
        "c_ulonglong" => "c_ulonglong",
        "c_float" => "f32",
        "c_double" => "f64",
        _ => return None,
    })
}

fn is_pub(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

fn is_identifier_like(value: &str) -> bool {
    let trimmed = value.trim_matches(|ch: char| ch == '[' || ch == ']');
    let mut chars = trimmed.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn to_canonical_identifier(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    let mut rendered = String::new();
    for (index, ch) in chars.iter().enumerate() {
        if ch.is_ascii_uppercase() {
            let prev = index.checked_sub(1).and_then(|idx| chars.get(idx));
            let next = chars.get(index + 1);
            if index > 0
                && !rendered.ends_with('_')
                && (prev.is_some_and(|prev| prev.is_ascii_lowercase() || prev.is_ascii_digit())
                    || next.is_some_and(|next| next.is_ascii_lowercase()))
            {
                rendered.push('_');
            }
            rendered.push(ch.to_ascii_lowercase());
        } else if *ch == '-' || *ch == ' ' {
            if !rendered.ends_with('_') {
                rendered.push('_');
            }
        } else {
            rendered.push(ch.to_ascii_lowercase());
        }
    }
    rendered
}

enum ItemTypeRef<'a> {
    Struct(&'a ItemStruct),
    Enum(&'a ItemEnum),
    Union(&'a ItemUnion),
    Alias(&'a ItemType),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::thread;
    use std::time::Duration;

    use anyhow::Result;
    use serde_json::Value;
    use tempfile::tempdir;

    use super::{Format, check_workspace, render_check, snapshot_workspace};
    use crate::config::{InitOptions, write_starter_config};
    use crate::model::{
        ArtifactKind, BinaryArtifactSnapshot, ExportRecord, HeaderSource, PackageSnapshot,
        Severity, TargetOrigin, TargetSnapshot, TypeKind, WorkspaceSnapshot,
    };
    use crate::render_sarif;

    fn repo_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
    }

    #[test]
    fn snapshot_fixture_workspace_tracks_phase_three_details() -> Result<()> {
        let output = tempdir()?.path().join("snapshot.json");
        let run = snapshot_workspace(
            Some(&repo_root().join("Cargo.toml")),
            Some(&repo_root().join("abi-audit.toml")),
            Some(&output),
        )?;
        assert!(output.exists());
        assert_eq!(run.snapshot.schema_version, 3);
        assert_eq!(run.snapshot.targets.len(), 1);
        assert!(run.snapshot.targets[0].header_sync.is_some());
        assert_eq!(
            run.snapshot.targets[0]
                .header_sync
                .as_ref()
                .map(|header_sync| header_sync.freshness_checked),
            Some(false)
        );
        let package = &run.snapshot.packages[0];
        assert_eq!(package.package, "outbound-c-api");
        assert!(
            package
                .types
                .iter()
                .any(|ty| ty.name == "AuditVersion" && ty.by_value_ffi_safe)
        );
        assert!(
            package.types.iter().any(|ty| ty.name == "AuditMode"
                && ty.kind == TypeKind::Enum
                && ty.by_value_ffi_safe)
        );
        assert!(package.types.iter().any(|ty| ty.name == "AuditFlags"
            && ty.kind == TypeKind::Alias
            && ty.by_value_ffi_safe));
        assert!(
            package
                .exports
                .iter()
                .any(|export| export.export_name == "abi_audit_mode_default")
        );
        assert_eq!(
            package
                .exports
                .iter()
                .find(|export| export.export_name == "abi_audit_add")
                .and_then(|export| export.normalized_signature.as_deref()),
            Some("u32 abi_audit_add(u32, u32)")
        );
        assert!(
            package
                .artifacts
                .iter()
                .any(|artifact| artifact.kind == ArtifactKind::Cdylib)
        );
        if cfg!(any(target_os = "macos", target_os = "linux")) {
            assert!(package.artifacts.iter().any(|artifact| {
                artifact.inspected
                    && artifact
                        .exported_symbols
                        .iter()
                        .any(|symbol| symbol == "abi_audit_add")
            }));
        }
        Ok(())
    }

    #[test]
    fn check_reports_expected_fixture_findings() -> Result<()> {
        let result = check_workspace(
            Some(&repo_root().join("Cargo.toml")),
            Some(&repo_root().join("abi-audit.toml")),
            None,
            None,
        )?;
        let rendered = render_check(&result, Format::Human)?;
        assert!(rendered.contains("non-ffi-safe-signature"));
        assert!(rendered.contains("missing-export-attr"));
        assert!(rendered.contains("export-missing-header"));
        Ok(())
    }

    #[test]
    fn snapshot_auto_discovers_target_and_headers() -> Result<()> {
        let output = tempdir()?.path().join("auto-snapshot.json");
        let manifest = repo_root().join("fixtures/auto-discovery-ffi/Cargo.toml");
        let run = snapshot_workspace(Some(&manifest), None, Some(&output))?;
        assert_eq!(run.snapshot.targets.len(), 1);
        assert_eq!(run.snapshot.targets[0].package, "auto-discovery-ffi");
        assert_eq!(run.snapshot.targets[0].origin, TargetOrigin::Auto);
        assert_eq!(run.snapshot.targets[0].header_source, HeaderSource::Auto);
        assert!(run.snapshot.targets[0].header_sync.is_none());
        assert_eq!(run.snapshot.packages[0].headers.len(), 3);
        assert!(
            run.snapshot.packages[0]
                .exports
                .iter()
                .any(|export| export.export_name == "phase_two_sum")
        );
        assert!(
            run.snapshot.packages[0]
                .artifacts
                .iter()
                .any(|artifact| artifact.kind == ArtifactKind::Cdylib)
        );
        Ok(())
    }

    #[test]
    fn check_reports_header_signature_mismatch_for_auto_fixture() -> Result<()> {
        let manifest = repo_root().join("fixtures/auto-discovery-ffi/Cargo.toml");
        let result = check_workspace(Some(&manifest), None, None, None)?;
        let rendered = render_check(&result, Format::Human)?;
        assert_eq!(result.exit_code, 0);
        assert!(rendered.contains("header-signature-mismatch"));
        Ok(())
    }

    #[test]
    fn init_writes_config_template() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("abi-audit.toml");
        let written = write_starter_config(&InitOptions {
            path: path.clone(),
            force: false,
        })?;
        assert_eq!(written, path);
        let text = fs::read_to_string(path)?;
        assert!(text.contains("[[targets]]"));
        assert!(text.contains("auto-discover"));
        assert!(text.contains("[targets.header_sync]"));
        assert!(text.contains("[rules.baseline-drift]"));
        Ok(())
    }

    #[test]
    fn lint_reports_artifact_missing_export_when_binary_truth_disagrees() {
        let findings = super::lint_snapshot(&WorkspaceSnapshot {
            schema_version: 3,
            generated_at_utc: "2026-03-08T00:00:00Z".to_string(),
            workspace_root: ".".to_string(),
            targets: vec![TargetSnapshot {
                package: "demo".to_string(),
                headers: Vec::new(),
                origin: TargetOrigin::Configured,
                header_source: HeaderSource::None,
                header_sync: None,
            }],
            packages: vec![PackageSnapshot {
                package: "demo".to_string(),
                manifest_path: "Cargo.toml".to_string(),
                crate_types: vec!["cdylib".to_string()],
                types: Vec::new(),
                headers: Vec::new(),
                exports: vec![ExportRecord {
                    rust_name: "demo".to_string(),
                    export_name: "demo".to_string(),
                    abi: "C".to_string(),
                    signature: "pub extern \"C\" fn demo()".to_string(),
                    normalized_signature: Some("void demo()".to_string()),
                    return_type: Some("void".to_string()),
                    param_types: Vec::new(),
                    file: "src/lib.rs".to_string(),
                    line: 1,
                    has_stable_export_attr: true,
                    export_attr: Some("no_mangle".to_string()),
                    opaque_handle_types: Vec::new(),
                }],
                artifacts: vec![BinaryArtifactSnapshot {
                    path: "target/debug/libdemo.dylib".to_string(),
                    kind: ArtifactKind::Cdylib,
                    format: "mach_o".to_string(),
                    inspected: true,
                    inspector: Some("nm".to_string()),
                    exported_symbols: vec!["different_symbol".to_string()],
                    notes: Vec::new(),
                }],
            }],
        });

        assert!(findings.iter().any(|finding| {
            finding.code == "artifact-missing-export"
                && finding.severity == Severity::Warning
                && finding.export.as_deref() == Some("demo")
        }));
    }

    #[test]
    fn check_reports_stale_cbindgen_header_sync() -> Result<()> {
        let dir = tempdir()?;
        fs::create_dir_all(dir.path().join("src"))?;
        fs::create_dir_all(dir.path().join("include"))?;
        fs::write(
            dir.path().join("Cargo.toml"),
            r#"[package]
name = "header-sync-fixture"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "staticlib"]

[workspace]
"#,
        )?;
        fs::write(
            dir.path().join("include/header_sync_fixture.h"),
            "uint32_t header_sync_fixture_sum(uint32_t left, uint32_t right);\n",
        )?;
        fs::write(
            dir.path().join("cbindgen.toml"),
            "language = \"C\"\ninclude_guard = \"HEADER_SYNC_FIXTURE_H\"\n",
        )?;
        fs::write(
            dir.path().join("abi-audit.toml"),
            r#"version = 1

[[targets]]
package = "header-sync-fixture"
headers = ["include/header_sync_fixture.h"]

[targets.header_sync]
tool = "cbindgen"
output = "include/header_sync_fixture.h"
config = "cbindgen.toml"
"#,
        )?;

        thread::sleep(Duration::from_millis(1100));
        fs::write(
            dir.path().join("src/lib.rs"),
            r#"#[unsafe(no_mangle)]
pub extern "C" fn header_sync_fixture_sum(left: u32, right: u32) -> u32 {
    left + right
}
"#,
        )?;

        let result = check_workspace(Some(&dir.path().join("Cargo.toml")), None, None, None)?;
        let rendered = render_check(&result, Format::Human)?;
        assert_eq!(result.exit_code, 0);
        assert!(rendered.contains("header-sync-stale"));
        Ok(())
    }

    #[test]
    fn render_sarif_reports_rule_metadata_and_locations() -> Result<()> {
        let result = check_workspace(
            Some(&repo_root().join("Cargo.toml")),
            Some(&repo_root().join("abi-audit.toml")),
            None,
            None,
        )?;
        let sarif = render_sarif(&result)?;
        let parsed: Value = serde_json::from_str(&sarif)?;
        assert_eq!(parsed["version"], "2.1.0");
        assert_eq!(
            parsed["runs"][0]["tool"]["driver"]["name"],
            "cargo-abi-audit"
        );
        assert!(
            parsed["runs"][0]["tool"]["driver"]["rules"]
                .as_array()
                .is_some_and(|rules| rules
                    .iter()
                    .any(|rule| rule["id"] == "non-ffi-safe-signature"
                        && rule["defaultConfiguration"]["level"] == "error"))
        );
        assert!(
            parsed["runs"][0]["results"]
                .as_array()
                .is_some_and(|results| results
                    .iter()
                    .any(|result| result["ruleId"] == "non-ffi-safe-signature"
                        && result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                            == "fixtures/outbound-c-api/src/lib.rs"))
        );
        Ok(())
    }

    #[test]
    fn check_uses_baseline_artifact_directory_and_rule_override() -> Result<()> {
        let dir = tempdir()?;
        fs::create_dir_all(dir.path().join("src"))?;
        fs::create_dir_all(dir.path().join("release-baseline"))?;
        fs::write(
            dir.path().join("Cargo.toml"),
            r#"[package]
name = "baseline-artifact-fixture"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[workspace]
"#,
        )?;
        fs::write(
            dir.path().join("src/lib.rs"),
            r#"#[unsafe(no_mangle)]
pub extern "C" fn baseline_artifact_fixture_sum(left: u32, right: u32) -> u32 {
    left + right
}
"#,
        )?;
        fs::write(
            dir.path().join("abi-audit.toml"),
            r#"version = 1

[baseline]
kind = "artifact_dir"
path = "release-baseline"
snapshot = "snapshot.json"

[rules.baseline-drift]
severity = "error"
"#,
        )?;
        fs::write(
            dir.path().join("release-baseline/snapshot.json"),
            serde_json::to_string_pretty(&WorkspaceSnapshot {
                schema_version: 3,
                generated_at_utc: "2026-03-08T00:00:00Z".to_string(),
                workspace_root: ".".to_string(),
                targets: vec![TargetSnapshot {
                    package: "baseline-artifact-fixture".to_string(),
                    headers: Vec::new(),
                    origin: TargetOrigin::Auto,
                    header_source: HeaderSource::None,
                    header_sync: None,
                }],
                packages: vec![PackageSnapshot {
                    package: "baseline-artifact-fixture".to_string(),
                    manifest_path: "Cargo.toml".to_string(),
                    crate_types: vec!["cdylib".to_string()],
                    types: Vec::new(),
                    headers: Vec::new(),
                    exports: Vec::new(),
                    artifacts: Vec::new(),
                }],
            })?,
        )?;

        let result = check_workspace(Some(&dir.path().join("Cargo.toml")), None, None, None)?;

        assert_eq!(result.exit_code, 1);
        assert!(result.report.findings.iter().any(|finding| {
            finding.code == "baseline-drift"
                && finding.severity == Severity::Error
                && finding.export.as_deref() == Some("baseline_artifact_fixture_sum")
        }));
        Ok(())
    }
}
