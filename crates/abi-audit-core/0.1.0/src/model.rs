use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub package: String,
    pub export: Option<String>,
    pub location: Option<SourceLocation>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceLocation {
    pub path: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceSnapshot {
    pub schema_version: u32,
    pub generated_at_utc: String,
    pub workspace_root: String,
    #[serde(default, alias = "configured_targets")]
    pub targets: Vec<TargetSnapshot>,
    pub packages: Vec<PackageSnapshot>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TargetOrigin {
    #[default]
    Configured,
    Auto,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum HeaderSource {
    Configured,
    Auto,
    #[default]
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum HeaderSyncTool {
    #[default]
    Cbindgen,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Cdylib,
    Staticlib,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TargetSnapshot {
    pub package: String,
    pub headers: Vec<String>,
    #[serde(default)]
    pub origin: TargetOrigin,
    #[serde(default)]
    pub header_source: HeaderSource,
    #[serde(default)]
    pub header_sync: Option<HeaderSyncSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageSnapshot {
    pub package: String,
    pub manifest_path: String,
    pub crate_types: Vec<String>,
    #[serde(default)]
    pub types: Vec<TypeDeclaration>,
    pub headers: Vec<HeaderDeclaration>,
    pub exports: Vec<ExportRecord>,
    #[serde(default)]
    pub artifacts: Vec<BinaryArtifactSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeaderDeclaration {
    pub path: String,
    #[serde(default = "default_header_line")]
    pub line: usize,
    pub name: String,
    pub signature: String,
    #[serde(default)]
    pub normalized_signature: Option<String>,
    #[serde(default)]
    pub return_type: Option<String>,
    #[serde(default)]
    pub param_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportRecord {
    pub rust_name: String,
    pub export_name: String,
    pub abi: String,
    pub signature: String,
    #[serde(default)]
    pub normalized_signature: Option<String>,
    #[serde(default)]
    pub return_type: Option<String>,
    #[serde(default)]
    pub param_types: Vec<String>,
    pub file: String,
    pub line: usize,
    pub has_stable_export_attr: bool,
    pub export_attr: Option<String>,
    pub opaque_handle_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeaderSyncSnapshot {
    pub tool: HeaderSyncTool,
    pub output: String,
    pub crate_dir: String,
    pub command: String,
    #[serde(default)]
    pub config: Option<String>,
    #[serde(default)]
    pub output_exists: bool,
    #[serde(default)]
    pub config_exists: bool,
    #[serde(default)]
    pub freshness_checked: bool,
    #[serde(default)]
    pub stale: bool,
    #[serde(default)]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BinaryArtifactSnapshot {
    pub path: String,
    pub kind: ArtifactKind,
    pub format: String,
    #[serde(default)]
    pub inspected: bool,
    #[serde(default)]
    pub inspector: Option<String>,
    #[serde(default)]
    pub exported_symbols: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TypeKind {
    Struct,
    Enum,
    Union,
    Alias,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeDeclaration {
    pub name: String,
    pub kind: TypeKind,
    pub file: String,
    pub line: usize,
    #[serde(default)]
    pub canonical_name: String,
    #[serde(default)]
    pub reprs: Vec<String>,
    #[serde(default)]
    pub fields: Vec<TypeMember>,
    #[serde(default)]
    pub fieldless: bool,
    #[serde(default)]
    pub by_value_ffi_safe: bool,
    #[serde(default)]
    pub by_value_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeMember {
    pub name: Option<String>,
    pub ty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckReport {
    pub snapshot: WorkspaceSnapshot,
    pub findings: Vec<Finding>,
    pub summary: CheckSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckSummary {
    pub packages_scanned: usize,
    pub exports_scanned: usize,
    pub warnings: usize,
    pub errors: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotRun {
    pub snapshot: WorkspaceSnapshot,
    pub output_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckResult {
    pub report: CheckReport,
    pub exit_code: i32,
}

const fn default_header_line() -> usize {
    1
}
