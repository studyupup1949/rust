use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceContract {
    #[serde(default)]
    pub hard_requirements: Vec<Requirement>,
    #[serde(default)]
    pub soft_requirements: Vec<Requirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub weight: Option<f64>,
    #[serde(default)]
    pub skip: Option<bool>,
    pub probe: Probe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Probe {
    CommandExitCode {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        cwd: Option<PathBuf>,
        #[serde(default = "default_exit_code")]
        expected_exit_code: i32,
    },
    FileHashChanged {
        path: PathBuf,
        #[serde(default)]
        expected_hash: Option<String>,
        #[serde(default = "default_file_hash_changed")]
        expect_changed: bool,
    },
    FileSizeBytes {
        path: PathBuf,
        #[serde(default)]
        expected_min_bytes: Option<u64>,
        #[serde(default)]
        expected_max_bytes: Option<u64>,
    },
    LocCount {
        path: PathBuf,
        #[serde(default)]
        expected_min_lines: Option<u64>,
        #[serde(default)]
        expected_max_lines: Option<u64>,
    },
    CommandExtract {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        cwd: Option<PathBuf>,
        #[serde(default)]
        regex: Option<String>,
        #[serde(default)]
        jsonpath: Option<String>,
        #[serde(default)]
        expected: Option<Value>,
        #[serde(default)]
        delegate_bench_guard: bool,
    },
    LogAbsent {
        path: PathBuf,
        pattern: String,
        #[serde(default)]
        regex: bool,
        #[serde(default)]
        expected_absent: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BaselineFile {
    #[serde(default)]
    pub schema_version: String,
    #[serde(default)]
    pub requirements: HashMap<String, BaselineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineEntry {
    pub value: Value,
    #[serde(default)]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequirementResult {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
    pub probe: String,
    pub passed: bool,
    pub score: Option<f64>,
    pub value: Value,
    #[serde(default)]
    pub baseline_comparison: Option<BaselineDelta>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub skipped: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BaselineDelta {
    pub baseline: Value,
    pub changed: bool,
    pub score_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreCard {
    pub contract_path: PathBuf,
    pub hard_requirements: Vec<RequirementResult>,
    pub soft_requirements: Vec<RequirementResult>,
    pub hard_passed: bool,
    pub total_soft_score: f64,
    pub max_soft_score: f64,
    pub soft_percentage: f64,
    #[serde(default)]
    pub delegated_checks: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub skip_delegation: bool,
    pub update_baseline: bool,
}

fn default_exit_code() -> i32 {
    0
}

fn default_file_hash_changed() -> bool {
    false
}

fn command_output(command: &str, args: &[String], cwd: Option<&Path>) -> Result<String> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).output()?;
    if !output.status.success() {
        let code = output
            .status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        return Err(anyhow!("command failed with exit code {}", code));
    }

    let text = String::from_utf8(output.stdout)
        .or_else(|_| String::from_utf8(output.stderr))
        .context("command output was not valid utf-8")?;
    Ok(text)
}

fn count_loc(path: &Path) -> Result<u64> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(content.lines().count() as u64)
}

fn file_sha(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    // sha2 v0.11 returns a GenericArray, which doesn't implement LowerHex directly.
    // Format the digest as a hex string manually from the byte slice.
    let digest = hasher.finalize();
    let mut hex_str = String::with_capacity(digest.len() * 2);
    for byte in digest.iter() {
        use std::fmt::Write;
        let _ = write!(&mut hex_str, "{byte:02x}");
    }
    Ok(hex_str)
}

fn file_size(path: &Path) -> Result<u64> {
    Ok(fs::metadata(path)
        .with_context(|| format!("failed to stat {}", path.display()))?
        .len())
}

pub fn load_contract(path: &Path) -> Result<AcceptanceContract> {
    let bytes = fs::read_to_string(path)
        .with_context(|| format!("failed to read contract {}", path.display()))?;
    serde_yaml::from_str(&bytes)
        .or_else(|_| serde_json::from_str(&bytes).map_err(anyhow::Error::from))
        .with_context(|| format!("failed to parse contract {} as yaml/json", path.display()))
}

pub fn load_baseline(path: &Path) -> Result<BaselineFile> {
    if !path.exists() {
        return Ok(BaselineFile {
            schema_version: "1.0".to_string(),
            requirements: HashMap::new(),
        });
    }

    let bytes = fs::read_to_string(path)
        .with_context(|| format!("failed to read baseline {}", path.display()))?;
    if bytes.trim().is_empty() {
        return Ok(BaselineFile {
            schema_version: "1.0".to_string(),
            requirements: HashMap::new(),
        });
    }

    serde_json::from_str(&bytes)
        .with_context(|| format!("failed to parse baseline {}", path.display()))
}

pub fn save_baseline(path: &Path, baseline: &BaselineFile) -> Result<()> {
    let json = serde_json::to_string_pretty(baseline)?;
    fs::write(path, json)
        .with_context(|| format!("failed to write baseline {}", path.display()))?;
    Ok(())
}

pub fn run_acceptance(
    contract_path: &Path,
    baseline_path: &Path,
    workspace_root: &Path,
    options: ExecutionContext,
) -> Result<ScoreCard> {
    let contract = load_contract(contract_path)?;
    let mut baseline = load_baseline(baseline_path)?;
    let mut card = ScoreCard {
        contract_path: contract_path.to_path_buf(),
        hard_requirements: Vec::new(),
        soft_requirements: Vec::new(),
        hard_passed: true,
        total_soft_score: 0.0,
        max_soft_score: 0.0,
        soft_percentage: 0.0,
        delegated_checks: Vec::new(),
    };

    let delegated = delegate_context(workspace_root, options.skip_delegation)?;
    card.delegated_checks = delegated;

    for req in &contract.hard_requirements {
        let result = evaluate_requirement(req, &mut baseline, true)?;
        if !result.passed {
            card.hard_passed = false;
        }
        card.hard_requirements.push(result);
    }

    for req in &contract.soft_requirements {
        let result = evaluate_requirement(req, &mut baseline, false)?;
        let max = req.weight.unwrap_or(1.0);
        card.max_soft_score += max;
        card.total_soft_score += result.score.unwrap_or(0.0);
        card.soft_requirements.push(result);
    }

    if card.max_soft_score > 0.0 {
        card.soft_percentage = (card.total_soft_score / card.max_soft_score) * 100.0;
    }

    if options.update_baseline {
        let mut next = BaselineFile {
            schema_version: "1.0".to_string(),
            requirements: HashMap::new(),
        };

        for result in card
            .hard_requirements
            .iter()
            .chain(card.soft_requirements.iter())
        {
            next.requirements.insert(
                result.id.clone(),
                BaselineEntry {
                    value: result.value.clone(),
                    score: result.score,
                },
            );
        }

        save_baseline(baseline_path, &next)?;
    }

    Ok(card)
}

fn evaluate_requirement(
    req: &Requirement,
    baseline: &mut BaselineFile,
    is_hard: bool,
) -> Result<RequirementResult> {
    if req.skip.unwrap_or(false) {
        return Ok(RequirementResult {
            id: req.id.clone(),
            description: req.description.clone(),
            probe: probe_name(req),
            passed: true,
            score: Some(0.0),
            value: Value::Null,
            baseline_comparison: None,
            message: Some("skipped".to_string()),
            skipped: true,
        });
    }

    let result = match &req.probe {
        Probe::CommandExitCode {
            command,
            args,
            cwd,
            expected_exit_code,
        } => {
            let output = Command::new(command)
                .args(args)
                .current_dir(cwd.as_deref().unwrap_or(Path::new(".")))
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let exit_code = output.ok().and_then(|status| status.code()).unwrap_or(-1);
            let passed = exit_code == *expected_exit_code;
            let score = if passed {
                req.weight.unwrap_or(1.0)
            } else {
                0.0
            };

            ProbeRunResult {
                passed,
                value: Value::from(exit_code),
                score,
                message: Some(format!("expected_exit_code={}", expected_exit_code)),
            }
        }
        Probe::FileHashChanged {
            path,
            expected_hash,
            expect_changed,
        } => {
            let hash = file_sha(path)?;
            let changed = if let Some(expected) = expected_hash {
                hash != *expected
            } else {
                false
            };
            let passed = if *expect_changed { changed } else { !changed };
            let weight = req.weight.unwrap_or(1.0);
            let score = if passed { weight } else { 0.0 };
            ProbeRunResult {
                passed,
                value: Value::String(hash),
                score,
                message: Some(format!("expect_changed={}", expect_changed)),
            }
        }
        Probe::FileSizeBytes {
            path,
            expected_min_bytes,
            expected_max_bytes,
        } => {
            let size = file_size(path)?;
            let min_ok = expected_min_bytes.is_none_or(|min| size >= min);
            let max_ok = expected_max_bytes.is_none_or(|max| size <= max);
            let passed = min_ok && max_ok;
            let weight = req.weight.unwrap_or(1.0);
            let score = if passed { weight } else { 0.0 };
            ProbeRunResult {
                passed,
                value: Value::from(size),
                score,
                message: Some(format!("size={}", size)),
            }
        }
        Probe::LocCount {
            path,
            expected_min_lines,
            expected_max_lines,
        } => {
            let lines = count_loc(path)?;
            let min_ok = expected_min_lines.is_none_or(|min| lines >= min);
            let max_ok = expected_max_lines.is_none_or(|max| lines <= max);
            let passed = min_ok && max_ok;
            let weight = req.weight.unwrap_or(1.0);
            let score = if passed { weight } else { 0.0 };
            ProbeRunResult {
                passed,
                value: Value::from(lines),
                score,
                message: Some(format!("lines={}", lines)),
            }
        }
        Probe::CommandExtract {
            command,
            args,
            cwd,
            regex,
            jsonpath,
            expected,
            delegate_bench_guard,
        } => {
            let output = command_output(command, args, cwd.as_deref())?;
            let extracted = match (regex, jsonpath) {
                (Some(pattern), _) => {
                    let re = Regex::new(pattern).context("invalid regex")?;
                    if let Some(captures) = re.captures(&output) {
                        if let Some(first) = captures.get(0) {
                            Value::String(first.as_str().to_string())
                        } else {
                            Value::Null
                        }
                    } else {
                        Value::Null
                    }
                }
                (_, Some(path_expr)) => extract_jsonpath(&output, path_expr)?,
                (None, None) => Value::String(output.trim().to_string()),
            };

            let passed = match expected {
                Some(expected_value) => extracted == *expected_value,
                None => !extracted.is_null(),
            };

            let score = if passed {
                req.weight.unwrap_or(1.0)
            } else {
                0.0
            };

            let mut detail = format!("command_extract expected={:?}", expected);
            if *delegate_bench_guard {
                match run_bench_guard_probe(workspace_root_of(cwd)) {
                    Ok(bench_msg) => {
                        detail.push_str(&format!(" | delegated_bench_guard={}", bench_msg));
                    }
                    Err(err) => {
                        detail.push_str(&format!(" | delegated_bench_guard=FAILED: {}", err));
                    }
                }
            }

            ProbeRunResult {
                passed,
                value: extracted,
                score,
                message: Some(detail),
            }
        }
        Probe::LogAbsent {
            path,
            pattern,
            regex,
            expected_absent,
        } => {
            let text = fs::read_to_string(path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let is_present = if *regex {
                let re = Regex::new(pattern).context("invalid regex")?;
                re.is_match(&text)
            } else {
                text.contains(pattern)
            };
            let passed = (*expected_absent && !is_present) || (!*expected_absent && is_present);
            let weight = req.weight.unwrap_or(1.0);
            let score = if passed { weight } else { 0.0 };
            ProbeRunResult {
                passed,
                value: Value::Bool(!is_present),
                score,
                message: Some(format!("pattern_present={}", is_present)),
            }
        }
    };

    let baseline_delta = compare_with_baseline(req, &result.value, &result.score, baseline);
    Ok(RequirementResult {
        id: req.id.clone(),
        description: req.description.clone(),
        probe: probe_name(req),
        passed: if is_hard { result.passed } else { true },
        score: Some(result.score),
        value: result.value,
        baseline_comparison: baseline_delta,
        message: result.message,
        skipped: false,
    })
}

fn compare_with_baseline(
    req: &Requirement,
    value: &Value,
    score: &f64,
    baseline: &mut BaselineFile,
) -> Option<BaselineDelta> {
    let previous = baseline.requirements.get(&req.id).cloned();
    let changed = previous.as_ref().is_none_or(|entry| entry.value != *value);
    let ratio = match (previous.as_ref().map(|entry| &entry.value), value) {
        (Some(Value::Number(base)), Value::Number(cur)) => {
            let base = base.as_f64().unwrap_or(0.0);
            let current = cur.as_f64().unwrap_or(0.0);
            if base == 0.0 {
                None
            } else {
                Some((current / base).clamp(0.0, 1.5))
            }
        }
        _ => None,
    };

    baseline.requirements.insert(
        req.id.clone(),
        BaselineEntry {
            value: value.clone(),
            score: Some(*score),
        },
    );

    previous.map(|entry| BaselineDelta {
        baseline: entry.value,
        changed,
        score_ratio: ratio,
    })
}

fn probe_name(req: &Requirement) -> String {
    match &req.probe {
        Probe::CommandExitCode { .. } => "command_exit_code".to_string(),
        Probe::FileHashChanged { .. } => "file_hash_changed".to_string(),
        Probe::FileSizeBytes { .. } => "file_size_bytes".to_string(),
        Probe::LocCount { .. } => "loc_count".to_string(),
        Probe::CommandExtract { .. } => "command_extract".to_string(),
        Probe::LogAbsent { .. } => "log_absent".to_string(),
    }
}

fn extract_jsonpath(raw: &str, expr: &str) -> Result<Value> {
    if expr.trim().is_empty() {
        return Ok(Value::Null);
    }

    let value: Value = serde_json::from_str(raw)
        .with_context(|| "command_extract requested jsonpath but output is not JSON".to_string())?;
    let expr = expr.trim().trim_start_matches('$');
    if expr.is_empty() {
        return Ok(value);
    }

    let mut current = &value;
    let mut offset = 0;
    while offset < expr.len() {
        if expr.as_bytes()[offset] == b'.' {
            offset += 1;
            continue;
        }

        if expr.as_bytes()[offset] == b'[' {
            let end = expr[offset..]
                .find(']')
                .ok_or_else(|| anyhow!("invalid jsonpath: missing ] in {}", expr))?;
            let inner = expr[(offset + 1)..(offset + end)].trim();
            let idx: usize = inner.parse().context("array index must be numeric")?;
            current = current
                .get(idx)
                .ok_or_else(|| anyhow!("jsonpath index {} not found", idx))?;
            offset += end + 1;
            continue;
        }

        let next_dot = expr[offset..].find('.').unwrap_or(expr.len() - offset) + offset;
        let segment = expr[offset..next_dot].trim();
        if segment.is_empty() {
            return Err(anyhow!("invalid jsonpath segment"));
        }

        current = current
            .get(segment)
            .ok_or_else(|| anyhow!("jsonpath segment {} not found", segment))?;
        offset = next_dot;
    }

    Ok(current.clone())
}

fn workspace_root_of(cwd: &Option<PathBuf>) -> &Path {
    cwd.as_deref().unwrap_or_else(|| Path::new("."))
}

fn delegate_context(workspace_root: &Path, skip: bool) -> Result<Vec<String>> {
    if skip {
        return Ok(Vec::new());
    }

    let mut delegated = Vec::new();

    match run_quality_gate_delegation(workspace_root) {
        Ok(output) => delegated.push(format!("quality-gate: ok ({})", output)),
        Err(err) => delegated.push(format!("quality-gate: failed ({})", err)),
    }

    match run_legacy_scan_delegation(workspace_root) {
        Ok(output) => delegated.push(format!("legacy-scan: ok ({})", output)),
        Err(err) => delegated.push(format!("legacy-scan: failed ({})", err)),
    }

    match run_bench_guard_delegation(workspace_root) {
        Ok(output) => delegated.push(format!("bench-guard: ok ({})", output)),
        Err(err) => delegated.push(format!("bench-guard: failed ({})", err)),
    }

    Ok(delegated)
}

fn run_quality_gate_delegation(workspace_root: &Path) -> Result<String> {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "quality-gate",
            "--",
            "--path",
            workspace_root.to_string_lossy().as_ref(),
            "--json",
            "--skip-clippy",
            "--skip-test",
            "--skip-fmt",
        ])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("quality-gate delegation failed"));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn run_legacy_scan_delegation(workspace_root: &Path) -> Result<String> {
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "legacy-scan",
            "--",
            "--path",
            workspace_root.to_string_lossy().as_ref(),
            "--json",
        ])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("legacy-scan delegation failed"));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn run_bench_guard_probe(workspace_root: &Path) -> Result<String> {
    let baseline = workspace_root.join("docs/reference/perf_baseline.json");
    let baseline = baseline.to_string_lossy();
    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "bench-guard",
            "--",
            "--baseline",
            baseline.as_ref(),
        ])
        .current_dir(workspace_root)
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("bench-guard delegation failed"));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn run_bench_guard_delegation(workspace_root: &Path) -> Result<String> {
    run_bench_guard_probe(workspace_root)
}

struct ProbeRunResult {
    passed: bool,
    value: Value,
    score: f64,
    message: Option<String>,
}

pub fn render_markdown(score_card: &ScoreCard) -> String {
    let mut out = String::new();
    out.push_str("# Acceptance Scorecard\n\n");
    out.push_str(&format!(
        "- Contract: `{}`\n",
        score_card.contract_path.display()
    ));
    out.push_str(&format!(
        "- Hard requirements passed: `{}`\n",
        score_card.hard_passed
    ));
    out.push_str(&format!(
        "- Soft score: {:.2} / {:.2} ({:.1}%)\n\n",
        score_card.total_soft_score, score_card.max_soft_score, score_card.soft_percentage
    ));

    out.push_str("## Hard requirements\n");
    for req in &score_card.hard_requirements {
        out.push_str(&format!(
            "- `{}` {}\n",
            req.id,
            if req.passed { "PASS" } else { "FAIL" }
        ));
        out.push_str(&format!("  - probe: {}\n", req.probe));
        out.push_str(&format!("  - value: `{}`\n", req.value));
        if let Some(msg) = &req.message {
            out.push_str(&format!("  - detail: {}\n", msg));
        }
    }

    out.push_str("\n## Soft requirements\n");
    for req in &score_card.soft_requirements {
        out.push_str(&format!(
            "- `{}` {:.2}/{:.2}\n",
            req.id,
            req.score.unwrap_or(0.0),
            req.score.unwrap_or(0.0)
        ));
        out.push_str(&format!("  - probe: {}\n", req.probe));
        out.push_str(&format!("  - value: `{}`\n", req.value));
        if let Some(delta) = &req.baseline_comparison {
            out.push_str(&format!("  - baseline: `{}`\n", delta.baseline));
            out.push_str(&format!("  - changed: {}\n", delta.changed));
            if let Some(ratio) = delta.score_ratio {
                out.push_str(&format!("  - ratio: {:.3}\n", ratio));
            }
        }
        if let Some(msg) = &req.message {
            out.push_str(&format!("  - detail: {}\n", msg));
        }
    }

    if !score_card.delegated_checks.is_empty() {
        out.push_str("\n## Delegated checks\n");
        for item in &score_card.delegated_checks {
            out.push_str(&format!("- {}\n", item));
        }
    }

    out
}
