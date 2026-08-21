//! GitHub Action manifest preflight verifier.
//!
//! `action-proof` validates action metadata before a release tag is published.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, ValueEnum};
use regex::Regex;
use serde::Serialize;
use serde_yml::{Mapping, Value};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "action-proof")]
#[command(about = "Preflight verifier for GitHub Action manifests and release wrappers")]
#[command(version)]
struct Cli {
    /// Path to action.yml/action.yaml. Defaults to auto-discovery in the repository root.
    #[arg(short, long)]
    manifest: Option<PathBuf>,

    /// Repository root used for readiness checks.
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,

    /// Output format.
    #[arg(long, default_value = "text")]
    format: OutputFormat,

    /// Write the receipt to this path instead of stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Treat warnings as failures.
    #[arg(long)]
    strict: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    Markdown,
}

#[derive(Clone, Debug, Serialize)]
struct Receipt {
    schema_version: u8,
    tool: ToolReceipt,
    checked_at: DateTime<Utc>,
    manifest: Option<String>,
    summary: Summary,
    checks: Vec<Check>,
}

#[derive(Clone, Debug, Serialize)]
struct ToolReceipt {
    name: &'static str,
    version: &'static str,
}

#[derive(Clone, Debug, Default, Serialize)]
struct Summary {
    passed: usize,
    warned: usize,
    failed: usize,
    skipped: usize,
}

#[derive(Clone, Debug, Serialize)]
struct Check {
    id: String,
    status: CheckStatus,
    message: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    details: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip,
}

struct Runner {
    repo_root: PathBuf,
    manifest: Option<PathBuf>,
    checks: Vec<Check>,
}

impl Runner {
    fn new(repo_root: PathBuf, manifest: Option<PathBuf>) -> Self {
        Self {
            repo_root,
            manifest,
            checks: Vec::new(),
        }
    }

    fn run(mut self, strict: bool) -> Receipt {
        let manifest_path = self.discover_manifest();
        let manifest_display = manifest_path
            .as_ref()
            .map(|path| path.display().to_string());

        if let Some(path) = manifest_path.as_ref() {
            self.check_manifest(path);
        }
        self.check_repository_readiness();

        let mut summary = Summary::from_checks(&self.checks);
        if strict && summary.warned > 0 {
            summary.failed += summary.warned;
            summary.warned = 0;
        }

        Receipt {
            schema_version: 1,
            tool: ToolReceipt {
                name: "action-proof",
                version: env!("CARGO_PKG_VERSION"),
            },
            checked_at: Utc::now(),
            manifest: manifest_display,
            summary,
            checks: self.checks,
        }
    }

    fn discover_manifest(&mut self) -> Option<PathBuf> {
        if let Some(path) = self.manifest.clone() {
            if path.exists() {
                self.pass(
                    "manifest.discover",
                    format!("using explicit manifest {}", path.display()),
                );
                return Some(path);
            }
            self.fail(
                "manifest.discover",
                format!("explicit manifest {} does not exist", path.display()),
            );
            return None;
        }

        let yml = self.repo_root.join("action.yml");
        let yaml = self.repo_root.join("action.yaml");
        match (yml.exists(), yaml.exists()) {
            (true, false) => {
                self.pass("manifest.discover", format!("found {}", yml.display()));
                Some(yml)
            }
            (false, true) => {
                self.pass("manifest.discover", format!("found {}", yaml.display()));
                Some(yaml)
            }
            (true, true) => {
                self.fail(
                    "manifest.discover",
                    "both action.yml and action.yaml exist; publish one stable metadata filename",
                );
                Some(yml)
            }
            (false, false) => {
                self.fail(
                    "manifest.discover",
                    format!(
                        "no action.yml or action.yaml found under {}",
                        self.repo_root.display()
                    ),
                );
                None
            }
        }
    }

    fn check_manifest(&mut self, path: &Path) {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                self.fail("manifest.read", format!("failed to read manifest: {error}"));
                return;
            }
        };
        self.pass("manifest.read", "manifest is readable");

        let value = match serde_yml::from_str::<Value>(&text) {
            Ok(value) => {
                self.pass("manifest.yaml", "manifest YAML parses");
                value
            }
            Err(error) => {
                self.fail(
                    "manifest.yaml",
                    format!("manifest YAML is invalid: {error}"),
                );
                return;
            }
        };

        let Some(root) = value.as_mapping() else {
            self.fail("manifest.root", "manifest root must be a mapping");
            return;
        };
        self.pass("manifest.root", "manifest root is a mapping");

        self.check_known_top_level_fields(root);
        self.check_required_string(root, "name", "metadata.name");
        self.check_required_string(root, "description", "metadata.description");
        self.check_inputs(root);
        self.check_outputs(root);
        self.check_branding(root);
        self.check_runs(root);
    }

    fn check_known_top_level_fields(&mut self, root: &Mapping) {
        let allowed = [
            "name",
            "author",
            "description",
            "inputs",
            "outputs",
            "runs",
            "branding",
            "deprecationMessage",
        ]
        .into_iter()
        .collect::<BTreeSet<_>>();
        let unknown = mapping_keys(root)
            .into_iter()
            .filter(|key| !allowed.contains(key.as_str()))
            .collect::<Vec<_>>();
        if unknown.is_empty() {
            self.pass(
                "metadata.fields",
                "top-level metadata fields are recognized",
            );
        } else {
            self.warn(
                "metadata.fields",
                format!("unknown top-level metadata fields: {}", unknown.join(", ")),
            );
        }
    }

    fn check_required_string(&mut self, root: &Mapping, key: &str, id: &str) {
        match get(root, key).and_then(Value::as_str).map(str::trim) {
            Some(value) if !value.is_empty() => self.pass(id, format!("{key} is present")),
            Some(_) => self.fail(id, format!("{key} must not be empty")),
            None => self.fail(id, format!("{key} is required and must be a string")),
        }
    }

    fn check_inputs(&mut self, root: &Mapping) {
        let Some(value) = get(root, "inputs") else {
            self.skip("inputs", "no inputs defined");
            return;
        };
        let Some(inputs) = value.as_mapping() else {
            self.fail("inputs", "inputs must be a mapping");
            return;
        };
        if inputs.is_empty() {
            self.pass("inputs", "inputs mapping is empty");
            return;
        }

        let name_re = name_regex();
        let mut invalid_names = Vec::new();
        let mut missing_descriptions = Vec::new();
        let mut invalid_required = Vec::new();
        for (name, config) in inputs {
            let Some(input_name) = name.as_str() else {
                invalid_names.push("<non-string>".to_string());
                continue;
            };
            if !name_re.is_match(input_name) {
                invalid_names.push(input_name.to_string());
            }

            let Some(config) = config.as_mapping() else {
                self.fail(
                    format!("inputs.{input_name}"),
                    "input config must be a mapping",
                );
                continue;
            };
            match get(config, "description")
                .and_then(Value::as_str)
                .map(str::trim)
            {
                Some(value) if !value.is_empty() => {}
                _ => missing_descriptions.push(input_name.to_string()),
            }
            if let Some(required) = get(config, "required")
                && required.as_bool().is_none()
            {
                invalid_required.push(input_name.to_string());
            }
        }

        if invalid_names.is_empty() {
            self.pass("inputs.names", "input names are action-compatible");
        } else {
            self.fail(
                "inputs.names",
                format!("invalid input names: {}", invalid_names.join(", ")),
            );
        }

        if missing_descriptions.is_empty() {
            self.pass("inputs.descriptions", "all inputs have descriptions");
        } else {
            self.fail(
                "inputs.descriptions",
                format!(
                    "inputs missing non-empty descriptions: {}",
                    missing_descriptions.join(", ")
                ),
            );
        }

        if invalid_required.is_empty() {
            self.pass(
                "inputs.required",
                "required flags are booleans when present",
            );
        } else {
            self.fail(
                "inputs.required",
                format!(
                    "inputs have non-boolean required flags: {}",
                    invalid_required.join(", ")
                ),
            );
        }
    }

    fn check_outputs(&mut self, root: &Mapping) {
        let Some(value) = get(root, "outputs") else {
            self.skip("outputs", "no outputs defined");
            return;
        };
        let Some(outputs) = value.as_mapping() else {
            self.fail("outputs", "outputs must be a mapping");
            return;
        };
        if outputs.is_empty() {
            self.pass("outputs", "outputs mapping is empty");
            return;
        }

        let name_re = name_regex();
        let mut invalid_names = Vec::new();
        let mut missing_descriptions = Vec::new();
        for (name, config) in outputs {
            let Some(output_name) = name.as_str() else {
                invalid_names.push("<non-string>".to_string());
                continue;
            };
            if !name_re.is_match(output_name) {
                invalid_names.push(output_name.to_string());
            }
            if let Some(config) = config.as_mapping() {
                match get(config, "description")
                    .and_then(Value::as_str)
                    .map(str::trim)
                {
                    Some(value) if !value.is_empty() => {}
                    _ => missing_descriptions.push(output_name.to_string()),
                }
            } else {
                self.fail(
                    format!("outputs.{output_name}"),
                    "output config must be a mapping",
                );
            }
        }

        if invalid_names.is_empty() {
            self.pass("outputs.names", "output names are action-compatible");
        } else {
            self.fail(
                "outputs.names",
                format!("invalid output names: {}", invalid_names.join(", ")),
            );
        }
        if missing_descriptions.is_empty() {
            self.pass("outputs.descriptions", "all outputs have descriptions");
        } else {
            self.warn(
                "outputs.descriptions",
                format!(
                    "outputs missing non-empty descriptions: {}",
                    missing_descriptions.join(", ")
                ),
            );
        }
    }

    fn check_branding(&mut self, root: &Mapping) {
        let Some(value) = get(root, "branding") else {
            self.warn("branding", "no Marketplace branding configured");
            return;
        };
        let Some(branding) = value.as_mapping() else {
            self.fail("branding", "branding must be a mapping");
            return;
        };
        let icon = get(branding, "icon").and_then(Value::as_str).map(str::trim);
        let color = get(branding, "color")
            .and_then(Value::as_str)
            .map(str::trim);
        if icon.is_some_and(|value| !value.is_empty())
            && color.is_some_and(|value| !value.is_empty())
        {
            self.pass("branding", "Marketplace branding has icon and color");
        } else {
            self.warn(
                "branding",
                "branding should include non-empty icon and color",
            );
        }
    }

    fn check_runs(&mut self, root: &Mapping) {
        let Some(value) = get(root, "runs") else {
            self.fail("runs", "runs section is required");
            return;
        };
        let Some(runs) = value.as_mapping() else {
            self.fail("runs", "runs must be a mapping");
            return;
        };
        let using = match get(runs, "using").and_then(Value::as_str).map(str::trim) {
            Some(value) if !value.is_empty() => value,
            _ => {
                self.fail("runs.using", "runs.using is required");
                return;
            }
        };

        match using {
            "composite" => {
                self.pass("runs.using", "runs.using is composite");
                self.check_composite_runs(runs);
            }
            "docker" => {
                self.pass("runs.using", "runs.using is docker");
                self.check_docker_runs(runs);
            }
            "node12" | "node16" => {
                self.fail(
                    "runs.using",
                    format!("{using} is obsolete; use node20 or node24"),
                );
                self.check_node_runs(runs);
            }
            "node20" | "node24" => {
                self.pass("runs.using", format!("runs.using is {using}"));
                self.check_node_runs(runs);
            }
            other => self.fail(
                "runs.using",
                format!("unsupported runs.using value {other:?}"),
            ),
        }
    }

    fn check_composite_runs(&mut self, runs: &Mapping) {
        let Some(steps) = get(runs, "steps").and_then(Value::as_sequence) else {
            self.fail("runs.steps", "composite actions require runs.steps");
            return;
        };
        if steps.is_empty() {
            self.fail("runs.steps", "runs.steps must not be empty");
            return;
        }
        self.pass("runs.steps", format!("{} composite steps", steps.len()));

        let mut missing_shell = Vec::new();
        let mut invalid_shape = Vec::new();
        let mut risky_shell = Vec::new();
        let mut floating_uses = Vec::new();
        let mut both_run_and_uses = Vec::new();

        for (index, step) in steps.iter().enumerate() {
            let step_number = index + 1;
            let Some(step) = step.as_mapping() else {
                invalid_shape.push(step_number.to_string());
                continue;
            };
            let has_run = get(step, "run").is_some();
            let has_uses = get(step, "uses").is_some();
            match (has_run, has_uses) {
                (true, true) => both_run_and_uses.push(step_number.to_string()),
                (false, false) => invalid_shape.push(step_number.to_string()),
                (true, false) => {
                    if get(step, "shell").and_then(Value::as_str).is_none() {
                        missing_shell.push(step_number.to_string());
                    }
                    if let Some(run) = get(step, "run").and_then(Value::as_str)
                        && command_looks_risky(run)
                    {
                        risky_shell.push(step_label(step, step_number));
                    }
                }
                (false, true) => {
                    if let Some(uses) = get(step, "uses").and_then(Value::as_str)
                        && uses_reference_is_floating(uses)
                    {
                        floating_uses.push(format!("{} ({uses})", step_label(step, step_number)));
                    }
                }
            }
        }

        if invalid_shape.is_empty() {
            self.pass(
                "runs.steps.shape",
                "every composite step is a mapping with run or uses",
            );
        } else {
            self.fail(
                "runs.steps.shape",
                format!(
                    "steps must contain exactly one run or uses field; bad steps: {}",
                    invalid_shape.join(", ")
                ),
            );
        }

        if both_run_and_uses.is_empty() {
            self.pass("runs.steps.exclusive", "no step combines run and uses");
        } else {
            self.fail(
                "runs.steps.exclusive",
                format!(
                    "steps combine run and uses: {}",
                    both_run_and_uses.join(", ")
                ),
            );
        }

        if missing_shell.is_empty() {
            self.pass("runs.steps.shell", "all run steps declare shell");
        } else {
            self.fail(
                "runs.steps.shell",
                format!("run steps missing shell: {}", missing_shell.join(", ")),
            );
        }

        if risky_shell.is_empty() {
            self.pass(
                "runs.steps.shell_risk",
                "no obvious download-and-execute shell patterns",
            );
        } else {
            self.warn(
                "runs.steps.shell_risk",
                format!(
                    "review download-and-execute patterns in steps: {}",
                    risky_shell.join(", ")
                ),
            );
        }

        if floating_uses.is_empty() {
            self.pass(
                "runs.steps.uses_pinning",
                "all remote uses references are SHA-pinned or local/docker references",
            );
        } else {
            self.warn(
                "runs.steps.uses_pinning",
                format!(
                    "remote uses references are not full-SHA pinned: {}",
                    floating_uses.join(", ")
                ),
            );
        }
    }

    fn check_node_runs(&mut self, runs: &Mapping) {
        match get(runs, "main").and_then(Value::as_str).map(str::trim) {
            Some(value) if !value.is_empty() => self.pass("runs.main", "JavaScript main is set"),
            _ => self.fail("runs.main", "JavaScript actions require runs.main"),
        }
        for key in ["pre", "post"] {
            if let Some(value) = get(runs, key)
                && value.as_str().is_none()
            {
                self.fail(
                    format!("runs.{key}"),
                    format!("runs.{key} must be a string"),
                );
            }
        }
    }

    fn check_docker_runs(&mut self, runs: &Mapping) {
        match get(runs, "image").and_then(Value::as_str).map(str::trim) {
            Some(value) if !value.is_empty() => self.pass("runs.image", "Docker image is set"),
            _ => self.fail("runs.image", "Docker actions require runs.image"),
        }
    }

    fn check_repository_readiness(&mut self) {
        if self.repo_root.join("README.md").exists() {
            self.pass("repo.readme", "README.md exists");
        } else {
            self.warn(
                "repo.readme",
                "README.md is recommended for Marketplace users",
            );
        }

        let has_license = ["LICENSE", "LICENSE.md", "LICENSE-MIT", "LICENSE-APACHE"]
            .iter()
            .any(|name| self.repo_root.join(name).exists());
        if has_license {
            self.pass("repo.license", "license file exists");
        } else {
            self.warn("repo.license", "license file is recommended");
        }

        let workflows = self.workflow_files();
        if workflows.is_empty() {
            self.warn(
                "repo.consumer_smoke",
                "no GitHub workflow found; add a consumer smoke for released action tags",
            );
            return;
        }

        let action_name = self
            .repo_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let mut mentions_action = false;
        for workflow in workflows {
            if let Ok(text) = fs::read_to_string(&workflow)
                && workflow_consumes_action_tag(&text, &action_name)
            {
                mentions_action = true;
            }
        }
        if mentions_action {
            self.pass(
                "repo.consumer_smoke",
                "workflow directory includes at least one released-action uses reference",
            );
        } else {
            self.warn(
                "repo.consumer_smoke",
                "add a workflow that consumes the released action tag from GitHub",
            );
        }
    }

    fn workflow_files(&self) -> Vec<PathBuf> {
        let workflow_dir = self.repo_root.join(".github").join("workflows");
        if !workflow_dir.exists() {
            return Vec::new();
        }
        WalkDir::new(workflow_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.into_path())
            .filter(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| matches!(ext, "yml" | "yaml"))
            })
            .collect()
    }

    fn pass(&mut self, id: impl Into<String>, message: impl Into<String>) {
        self.push(id, CheckStatus::Pass, message, BTreeMap::new());
    }

    fn warn(&mut self, id: impl Into<String>, message: impl Into<String>) {
        self.push(id, CheckStatus::Warn, message, BTreeMap::new());
    }

    fn fail(&mut self, id: impl Into<String>, message: impl Into<String>) {
        self.push(id, CheckStatus::Fail, message, BTreeMap::new());
    }

    fn skip(&mut self, id: impl Into<String>, message: impl Into<String>) {
        self.push(id, CheckStatus::Skip, message, BTreeMap::new());
    }

    fn push(
        &mut self,
        id: impl Into<String>,
        status: CheckStatus,
        message: impl Into<String>,
        details: BTreeMap<String, String>,
    ) {
        self.checks.push(Check {
            id: id.into(),
            status,
            message: message.into(),
            details,
        });
    }
}

fn workflow_consumes_action_tag(text: &str, action_name: &str) -> bool {
    if action_name.is_empty() {
        return false;
    }
    let needle = format!("/{action_name}@v");
    text.lines()
        .map(str::trim)
        .filter(|line| line.starts_with("uses:") || line.starts_with("- uses:"))
        .any(|line| line.to_ascii_lowercase().contains(&needle))
}

impl Summary {
    fn from_checks(checks: &[Check]) -> Self {
        let mut summary = Self::default();
        for check in checks {
            match check.status {
                CheckStatus::Pass => summary.passed += 1,
                CheckStatus::Warn => summary.warned += 1,
                CheckStatus::Fail => summary.failed += 1,
                CheckStatus::Skip => summary.skipped += 1,
            }
        }
        summary
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let repo_root = normalize_path(&cli.repo_root)?;
    let manifest = cli
        .manifest
        .as_ref()
        .map(|path| normalize_path(path))
        .transpose()?;
    let receipt = Runner::new(repo_root, manifest).run(cli.strict);
    let failed = receipt.summary.failed > 0;
    let rendered = render_receipt(&receipt, cli.format)?;
    if let Some(path) = cli.output {
        fs::write(&path, rendered)
            .with_context(|| format!("failed to write {}", path.display()))?;
    } else {
        print!("{rendered}");
    }
    if failed {
        std::process::exit(1);
    }
    Ok(())
}

fn normalize_path(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        Ok(dunce::canonicalize(path)
            .with_context(|| format!("failed to resolve {}", path.display()))?)
    } else if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && parent.exists()
    {
        let parent = dunce::canonicalize(parent)
            .with_context(|| format!("failed to resolve {}", parent.display()))?;
        Ok(parent.join(
            path.file_name()
                .with_context(|| format!("invalid path {}", path.display()))?,
        ))
    } else {
        Ok(path.to_path_buf())
    }
}

fn render_receipt(receipt: &Receipt, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(format!("{}\n", serde_json::to_string_pretty(receipt)?)),
        OutputFormat::Markdown => Ok(render_markdown(receipt)),
        OutputFormat::Text => Ok(render_text(receipt)),
    }
}

fn render_text(receipt: &Receipt) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "action-proof {} for {}\n",
        receipt.tool.version,
        receipt.manifest.as_deref().unwrap_or("<no manifest>")
    ));
    out.push_str(&format!(
        "summary: {} passed, {} warned, {} failed, {} skipped\n",
        receipt.summary.passed,
        receipt.summary.warned,
        receipt.summary.failed,
        receipt.summary.skipped
    ));
    for check in &receipt.checks {
        out.push_str(&format!(
            "{} {:<7} {}\n",
            status_symbol(check.status),
            status_word(check.status),
            check.id
        ));
        out.push_str(&format!("  {}\n", check.message));
    }
    out
}

fn render_markdown(receipt: &Receipt) -> String {
    let mut out = String::new();
    out.push_str("# Action Proof\n\n");
    out.push_str(&format!("- Checked at: `{}`\n", receipt.checked_at));
    if let Some(manifest) = &receipt.manifest {
        out.push_str(&format!("- Manifest: `{}`\n", markdown_escape(manifest)));
    }
    out.push_str(&format!(
        "- Summary: **{} passed**, **{} warned**, **{} failed**, **{} skipped**\n\n",
        receipt.summary.passed,
        receipt.summary.warned,
        receipt.summary.failed,
        receipt.summary.skipped
    ));
    out.push_str("| Status | Check | Message |\n| --- | --- | --- |\n");
    for check in &receipt.checks {
        out.push_str(&format!(
            "| {} {} | `{}` | {} |\n",
            status_symbol(check.status),
            status_word(check.status),
            check.id,
            markdown_escape(&check.message)
        ));
    }
    out
}

fn status_symbol(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Pass => "[PASS]",
        CheckStatus::Warn => "[WARN]",
        CheckStatus::Fail => "[FAIL]",
        CheckStatus::Skip => "[SKIP]",
    }
}

fn status_word(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Pass => "pass",
        CheckStatus::Warn => "warn",
        CheckStatus::Fail => "fail",
        CheckStatus::Skip => "skip",
    }
}

fn markdown_escape(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', "<br>")
}

fn mapping_keys(mapping: &Mapping) -> Vec<String> {
    mapping
        .keys()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

fn get<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Value> {
    mapping.get(Value::String(key.to_string()))
}

fn name_regex() -> Regex {
    Regex::new(r"^[A-Za-z0-9_-]+$").expect("input/output name regex is valid")
}

fn step_label(step: &Mapping, step_number: usize) -> String {
    get(step, "name")
        .and_then(Value::as_str)
        .map(|name| format!("#{step_number} {name}"))
        .unwrap_or_else(|| format!("#{step_number}"))
}

fn command_looks_risky(run: &str) -> bool {
    let normalized = run.to_ascii_lowercase();
    [
        "curl",
        "wget",
        "invoke-webrequest",
        "iwr ",
        "irm ",
        "invoke-restmethod",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
        && [
            "| sh", "|sh", "| bash", "|bash", "| pwsh", "|pwsh", "| iex", "|iex",
        ]
        .iter()
        .any(|needle| normalized.contains(needle))
}

fn uses_reference_is_floating(reference: &str) -> bool {
    if reference.starts_with("./")
        || reference.starts_with("../")
        || reference.starts_with("docker://")
        || reference.starts_with(".github/")
    {
        return false;
    }
    let Some((_, version)) = reference.rsplit_once('@') else {
        return true;
    };
    !(version.len() == 40 && version.chars().all(|ch| ch.is_ascii_hexdigit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_unquoted_colon_yaml_failure() {
        let bad = r#"
name: Bad Action
description: Example
runs:
  using: composite
  steps: []
inputs:
  format:
    description: Receipt format: text, json, or markdown.
"#;
        assert!(serde_yml::from_str::<Value>(bad).is_err());
    }

    #[test]
    fn accepts_quoted_colon_yaml() {
        let good = r#"
name: Good Action
description: Example
runs:
  using: composite
  steps: []
inputs:
  format:
    description: "Receipt format: text, json, or markdown."
"#;
        assert!(serde_yml::from_str::<Value>(good).is_ok());
    }

    #[test]
    fn detects_risky_download_execute_shell() {
        assert!(command_looks_risky(
            "curl -fsSL https://example.test/install.sh | bash"
        ));
        assert!(command_looks_risky(
            "irm https://example.test/install.ps1 | iex"
        ));
        assert!(!command_looks_risky(
            "curl -fsSL https://example.test/checksum.txt"
        ));
    }

    #[test]
    fn classifies_uses_pinning() {
        assert!(!uses_reference_is_floating("./local-action"));
        assert!(!uses_reference_is_floating(
            "actions/checkout@93cb6efe18208431cddfb8368fd83d5badbf9bfd"
        ));
        assert!(uses_reference_is_floating("actions/checkout@v5"));
        assert!(uses_reference_is_floating("owner/action"));
    }

    #[test]
    fn detects_workflow_consuming_this_action_tag() {
        let workflow =
            "steps:\n  - uses: actions/checkout@v5\n  - uses: wildmason/action-proof@v1\n";
        assert!(workflow_consumes_action_tag(workflow, "action-proof"));
        assert!(!workflow_consumes_action_tag(workflow, "release-proof"));
    }

    #[test]
    fn summary_counts_warnings() {
        let checks = vec![
            Check {
                id: "a".to_string(),
                status: CheckStatus::Pass,
                message: String::new(),
                details: BTreeMap::new(),
            },
            Check {
                id: "b".to_string(),
                status: CheckStatus::Warn,
                message: String::new(),
                details: BTreeMap::new(),
            },
            Check {
                id: "c".to_string(),
                status: CheckStatus::Fail,
                message: String::new(),
                details: BTreeMap::new(),
            },
        ];
        let summary = Summary::from_checks(&checks);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.warned, 1);
        assert_eq!(summary.failed, 1);
    }

    #[test]
    fn render_text_includes_summary() {
        let receipt = Receipt {
            schema_version: 1,
            tool: ToolReceipt {
                name: "action-proof",
                version: "test",
            },
            checked_at: Utc::now(),
            manifest: Some("action.yml".to_string()),
            summary: Summary {
                passed: 1,
                warned: 2,
                failed: 0,
                skipped: 0,
            },
            checks: Vec::new(),
        };
        assert!(render_text(&receipt).contains("1 passed, 2 warned, 0 failed"));
    }
}
