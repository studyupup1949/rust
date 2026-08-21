use crate::write_json_if_requested;
use advisorygraphen_core::{validate_document, AdvisoryResult};
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CodeRepoSnapshotOptions {
    pub repo: PathBuf,
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct CodeFile {
    relative_path: String,
    source_id: String,
    contents: String,
    kind: CodeFileKind,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum CodeFileKind {
    Manifest,
    Source,
    Test,
    ApiRoute,
}

#[derive(Debug, Default)]
struct Coverage {
    parsed_files: usize,
    skipped_files: usize,
    unsupported_extensions: BTreeMap<String, usize>,
    api_route_files: usize,
    test_files: usize,
    db_access_files: usize,
    env_usage_files: usize,
}

pub fn code_repo_snapshot_workflow(options: &CodeRepoSnapshotOptions) -> AdvisoryResult<Value> {
    let captured_at = Utc::now().to_rfc3339();
    let repo_name = options
        .repo
        .file_name()
        .and_then(|name| name.to_str())
        .map(slug)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "repository".to_string());
    let mut coverage = Coverage::default();
    let files = collect_code_files(&options.repo, &mut coverage)?;
    let sources = files
        .iter()
        .map(|file| code_source(file, &captured_at))
        .collect::<Vec<_>>();
    let records = code_records(&files, &mut coverage);
    let source_ids = sources
        .iter()
        .filter_map(|source| source.get("id").and_then(Value::as_str))
        .collect::<Vec<_>>();
    let snapshot = json!({
        "schema": "advisorygraphen.engagement.snapshot.v1",
        "snapshot_id": format!("snapshot:code-{repo_name}"),
        "engagement_id": format!("engagement:code-review-{repo_name}"),
        "captured_at": captured_at,
        "source_boundary": {
            "included_source_ids": source_ids,
            "excluded_summary": [
                "Generated, dependency, build output, hidden, and unsupported files were not parsed.",
                "This adapter currently extracts deterministic TypeScript/JavaScript/Next.js signals only."
            ],
            "extraction_loss": [
                "Code is represented as route, dependency, database, environment, and test records, not full source text.",
                "Detection is lexical and path-based; it does not resolve TypeScript types or runtime control flow."
            ],
            "trust_notes": [
                "AST-free deterministic scanner intended to seed AdvisoryGraphen review, not prove whole-program behavior.",
                "Use coverage_summary before treating findings as complete."
            ],
            "adapter_version": "code_repo_snapshot:0.1.0"
        },
        "sources": sources,
        "records": records,
        "metadata": {
            "adapter": "code_repo_snapshot",
            "repo": options.repo.display().to_string(),
            "coverage_summary": coverage_json(&coverage)
        }
    });
    validate_document(&snapshot, Some(advisorygraphen_core::SNAPSHOT_SCHEMA))?;
    write_json_if_requested(&options.output, &snapshot)?;
    Ok(snapshot)
}

mod detectors;
mod discovery;
mod records;

use detectors::*;
use discovery::*;
use records::*;
