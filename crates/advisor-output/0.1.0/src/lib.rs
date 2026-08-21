use advisor_core::{
    AdvisorError, BestFitSection, CompareReport, Confidence, ExplainReport, LockfileSummary,
    RecommendReport, Recommendation, ReviewReport, ReviewedManifest, Tradeoff, TrustNote,
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    pub fn parse(value: &str) -> Result<Self, AdvisorError> {
        match value {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => Err(AdvisorError::Usage(format!(
                "unsupported output format '{other}'. Supported formats: text, json"
            ))),
        }
    }
}

pub fn render_recommend(
    report: &RecommendReport,
    format: OutputFormat,
) -> Result<String, AdvisorError> {
    match format {
        OutputFormat::Text => Ok(render_recommend_text(report)),
        OutputFormat::Json => render_json(&RecommendJson::from_report(report)),
    }
}

pub fn render_compare(
    report: &CompareReport,
    format: OutputFormat,
) -> Result<String, AdvisorError> {
    match format {
        OutputFormat::Text => Ok(render_compare_text(report)),
        OutputFormat::Json => render_json(&CompareJson::from_report(report)),
    }
}

pub fn render_explain(
    report: &ExplainReport,
    format: OutputFormat,
) -> Result<String, AdvisorError> {
    match format {
        OutputFormat::Text => Ok(render_explain_text(report)),
        OutputFormat::Json => render_json(&ExplainJson::from_report(report)),
    }
}

pub fn render_review(report: &ReviewReport, format: OutputFormat) -> Result<String, AdvisorError> {
    match format {
        OutputFormat::Text => Ok(render_review_text(report)),
        OutputFormat::Json => render_json(&ReviewJson::from_report(report)),
    }
}

fn render_recommend_text(report: &RecommendReport) -> String {
    let mut output = Vec::new();
    output.push("Recommendation".to_string());
    output.push(report.summary.clone());
    output.push(format!(
        "Intent: {}{}",
        report.intent,
        requested_suffix(report.requested_intent.as_str(), report.intent.as_str())
    ));
    if let Some(goal) = report.goal.as_deref() {
        let requested = report.requested_goal.as_deref().unwrap_or(goal);
        output.push(format!(
            "Goal: {}{}",
            goal,
            requested_suffix(requested, goal)
        ));
    } else if let Some(requested_goal) = report.requested_goal.as_deref() {
        output.push(format!("Goal: {}", requested_goal));
    }
    output.push(format!(
        "Primary choice: {} ({}, {})",
        report.recommendation.crate_name,
        report.recommendation.confidence.as_str(),
        report.recommendation.archetype.label()
    ));
    output.extend(render_recommendation_body(&report.recommendation));
    render_best_fit_sections(&mut output, &report.best_fit_sections);
    render_alternatives(&mut output, "Alternatives", &report.alternatives);
    output.extend(render_shared_sections(
        &report.trust_notes,
        &report.receipts,
        false,
    ));
    output.join("\n")
}

fn render_compare_text(report: &CompareReport) -> String {
    let mut output = Vec::new();
    output.push("Comparison".to_string());
    output.push(report.summary.clone());
    if let Some(intent) = report.intent.as_deref() {
        let requested = report.requested_intent.as_deref().unwrap_or(intent);
        output.push(format!(
            "Intent: {}{}",
            intent,
            requested_suffix(requested, intent)
        ));
    }
    output.push(format!(
        "Best fit: {} ({}, {})",
        report.recommendation.crate_name,
        report.recommendation.confidence.as_str(),
        report.recommendation.archetype.label()
    ));
    output.extend(render_recommendation_body(&report.recommendation));
    output.push("Compared crates".to_string());
    output.push(format!("- {}", report.requested_crates.join(", ")));
    render_alternatives(&mut output, "Other candidates", &report.alternatives);
    output.extend(render_shared_sections(
        &report.trust_notes,
        &report.receipts,
        false,
    ));
    output.join("\n")
}

fn render_explain_text(report: &ExplainReport) -> String {
    let mut output = Vec::new();
    output.push("Explanation".to_string());
    output.push(report.summary.clone());
    let requested = report
        .requested_intent
        .as_deref()
        .unwrap_or(report.intent.as_str());
    output.push(format!(
        "Intent: {}{}",
        report.intent,
        requested_suffix(requested, report.intent.as_str())
    ));
    output.push(format!(
        "Subject: {} ({}, {})",
        report.recommendation.crate_name,
        report.recommendation.confidence.as_str(),
        report.recommendation.archetype.label()
    ));
    output.extend(render_recommendation_body(&report.recommendation));
    output.extend(render_shared_sections(
        &report.trust_notes,
        &report.receipts,
        false,
    ));
    output.join("\n")
}

fn render_review_text(report: &ReviewReport) -> String {
    let mut output = Vec::new();
    output.push("Review".to_string());
    output.push(report.summary.clone());
    output.push(format!(
        "Manifest: {}",
        report.manifest_path.as_path().display()
    ));
    output.push(format!(
        "Lockfile: {}",
        report.lockfile_path.as_path().display()
    ));
    if !report.manifests.is_empty() {
        output.push("Manifests reviewed".to_string());
        for manifest in &report.manifests {
            output.push(format!("- {}", render_manifest_summary(manifest)));
        }
    }
    output.push(format!(
        "Dependencies seen: {}",
        if report.dependencies.is_empty() {
            "none".to_string()
        } else {
            report.dependencies.join(", ")
        }
    ));
    if let Some(lockfile_summary) = report.lockfile_summary.as_ref() {
        output.push("Lockfile summary".to_string());
        output.push(format!(
            "- package entries: {}",
            lockfile_summary.package_count
        ));
        if lockfile_summary.duplicate_versions.is_empty() {
            output.push("- duplicate versions: none".to_string());
        } else {
            output.push(format!(
                "- duplicate versions: {}",
                render_duplicate_versions(lockfile_summary)
            ));
        }
    }
    output.push("Findings".to_string());
    for finding in &report.findings {
        output.push(format!(
            "- {}: {}",
            finding.severity.as_str(),
            finding.title
        ));
        output.push(format!("  {}", finding.detail));
    }
    render_alternatives(
        &mut output,
        "Follow-up recommendations",
        &report.follow_up_recommendations,
    );
    output.extend(render_shared_sections(
        &report.trust_notes,
        &report.receipts,
        true,
    ));
    output.join("\n")
}

fn render_manifest_summary(manifest: &ReviewedManifest) -> String {
    let role = if manifest.is_root {
        "review root".to_string()
    } else {
        manifest
            .package_name
            .as_deref()
            .map(|name| format!("workspace member {name}"))
            .unwrap_or_else(|| "workspace member".to_string())
    };
    format!(
        "{} at {} ({} direct dependencies)",
        role,
        manifest.manifest_path.display(),
        manifest.dependency_count
    )
}

fn render_duplicate_versions(summary: &LockfileSummary) -> String {
    summary
        .duplicate_versions
        .iter()
        .map(|duplicate| {
            format!(
                "{} ({})",
                duplicate.crate_name,
                duplicate.versions.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn render_recommendation_body(recommendation: &Recommendation) -> Vec<String> {
    let mut output = Vec::new();
    output.push("Why it fits".to_string());
    for reason in &recommendation.rationale {
        output.push(format!("- {reason}"));
    }
    if !recommendation.fit_notes.is_empty() {
        output.push("Fit notes".to_string());
        for note in &recommendation.fit_notes {
            output.push(format!("- {note}"));
        }
    }
    render_tradeoffs(&mut output, &recommendation.tradeoffs);
    output
}

fn render_best_fit_sections(output: &mut Vec<String>, sections: &[BestFitSection]) {
    if sections.is_empty() {
        return;
    }
    output.push("Best-fit views".to_string());
    for section in sections {
        output.push(format!(
            "- {}: {} ({})",
            section.label,
            section.recommendation.crate_name,
            section.recommendation.confidence.as_str()
        ));
        output.push(format!("  {}", section.summary));
    }
}

fn render_alternatives(output: &mut Vec<String>, title: &str, alternatives: &[Recommendation]) {
    if alternatives.is_empty() {
        return;
    }
    output.push(title.to_string());
    for alternative in alternatives {
        output.push(format!(
            "- {} ({}, {})",
            alternative.crate_name,
            alternative.confidence.as_str(),
            alternative.archetype.label()
        ));
        output.push(format!("  {}", alternative.summary));
    }
}

fn render_tradeoffs(output: &mut Vec<String>, tradeoffs: &[Tradeoff]) {
    if tradeoffs.is_empty() {
        return;
    }
    output.push("Tradeoffs".to_string());
    for tradeoff in tradeoffs {
        output.push(format!("- {}: {}", tradeoff.area, tradeoff.detail));
    }
}

fn render_shared_sections(
    trust_notes: &[TrustNote],
    receipts: &[advisor_core::Receipt],
    receipts_only: bool,
) -> Vec<String> {
    let mut output = Vec::new();
    if !receipts_only && !trust_notes.is_empty() {
        output.push("Trust notes".to_string());
        for note in trust_notes {
            output.push(format!("- {}: {}", note.label, note.detail));
        }
    }
    if !receipts.is_empty() {
        output.push("Receipts".to_string());
        for receipt in receipts {
            output.push(format!("- {}: {}", receipt.source, receipt.summary));
            output.push(format!("  {}", receipt.detail));
        }
    }
    output
}

fn requested_suffix(requested: &str, resolved: &str) -> String {
    if requested == resolved {
        String::new()
    } else {
        format!(" (requested as '{requested}')")
    }
}

fn render_json<T: Serialize>(value: &T) -> Result<String, AdvisorError> {
    serde_json::to_string_pretty(value)
        .map_err(|error| AdvisorError::Usage(format!("failed to serialize json output: {error}")))
}

#[derive(Serialize)]
struct RecommendJson<'a> {
    format_version: &'static str,
    command: &'static str,
    summary: &'a str,
    requested_intent: &'a str,
    intent: &'a str,
    requested_goal: Option<&'a str>,
    goal: Option<&'a str>,
    recommendation: &'a Recommendation,
    confidence: &'a Confidence,
    tradeoffs: &'a [Tradeoff],
    best_fit_sections: &'a [BestFitSection],
    alternatives: &'a [Recommendation],
    trust_notes: &'a [TrustNote],
    receipts: &'a [advisor_core::Receipt],
}

impl<'a> RecommendJson<'a> {
    fn from_report(report: &'a RecommendReport) -> Self {
        Self {
            format_version: "0.1",
            command: "recommend",
            summary: report.summary.as_str(),
            requested_intent: report.requested_intent.as_str(),
            intent: report.intent.as_str(),
            requested_goal: report.requested_goal.as_deref(),
            goal: report.goal.as_deref(),
            recommendation: &report.recommendation,
            confidence: &report.confidence,
            tradeoffs: &report.tradeoffs,
            best_fit_sections: &report.best_fit_sections,
            alternatives: &report.alternatives,
            trust_notes: &report.trust_notes,
            receipts: &report.receipts,
        }
    }
}

#[derive(Serialize)]
struct CompareJson<'a> {
    format_version: &'static str,
    command: &'static str,
    summary: &'a str,
    requested_intent: Option<&'a str>,
    intent: Option<&'a str>,
    requested_crates: &'a [String],
    recommendation: &'a Recommendation,
    confidence: &'a Confidence,
    tradeoffs: &'a [Tradeoff],
    alternatives: &'a [Recommendation],
    trust_notes: &'a [TrustNote],
    receipts: &'a [advisor_core::Receipt],
}

impl<'a> CompareJson<'a> {
    fn from_report(report: &'a CompareReport) -> Self {
        Self {
            format_version: "0.1",
            command: "compare",
            summary: report.summary.as_str(),
            requested_intent: report.requested_intent.as_deref(),
            intent: report.intent.as_deref(),
            requested_crates: &report.requested_crates,
            recommendation: &report.recommendation,
            confidence: &report.confidence,
            tradeoffs: &report.tradeoffs,
            alternatives: &report.alternatives,
            trust_notes: &report.trust_notes,
            receipts: &report.receipts,
        }
    }
}

#[derive(Serialize)]
struct ExplainJson<'a> {
    format_version: &'static str,
    command: &'static str,
    summary: &'a str,
    requested_intent: Option<&'a str>,
    intent: &'a str,
    recommendation: &'a Recommendation,
    confidence: &'a Confidence,
    tradeoffs: &'a [Tradeoff],
    trust_notes: &'a [TrustNote],
    receipts: &'a [advisor_core::Receipt],
}

impl<'a> ExplainJson<'a> {
    fn from_report(report: &'a ExplainReport) -> Self {
        Self {
            format_version: "0.1",
            command: "explain",
            summary: report.summary.as_str(),
            requested_intent: report.requested_intent.as_deref(),
            intent: report.intent.as_str(),
            recommendation: &report.recommendation,
            confidence: &report.confidence,
            tradeoffs: &report.tradeoffs,
            trust_notes: &report.trust_notes,
            receipts: &report.receipts,
        }
    }
}

#[derive(Serialize)]
struct ReviewJson<'a> {
    format_version: &'static str,
    command: &'static str,
    summary: &'a str,
    manifest_path: String,
    lockfile_path: String,
    manifests: &'a [ReviewedManifest],
    dependencies: &'a [String],
    lockfile_summary: Option<&'a LockfileSummary>,
    findings: &'a [advisor_core::ReviewFinding],
    recommendation: Option<&'a Recommendation>,
    confidence: Option<&'a Confidence>,
    tradeoffs: &'a [Tradeoff],
    follow_up_recommendations: &'a [Recommendation],
    trust_notes: &'a [TrustNote],
    receipts: &'a [advisor_core::Receipt],
}

impl<'a> ReviewJson<'a> {
    fn from_report(report: &'a ReviewReport) -> Self {
        Self {
            format_version: "0.1",
            command: "review",
            summary: report.summary.as_str(),
            manifest_path: report.manifest_path.display().to_string(),
            lockfile_path: report.lockfile_path.display().to_string(),
            manifests: &report.manifests,
            dependencies: &report.dependencies,
            lockfile_summary: report.lockfile_summary.as_ref(),
            findings: &report.findings,
            recommendation: report.recommendation.as_ref(),
            confidence: report.confidence.as_ref(),
            tradeoffs: &report.tradeoffs,
            follow_up_recommendations: &report.follow_up_recommendations,
            trust_notes: &report.trust_notes,
            receipts: &report.receipts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{OutputFormat, render_recommend};
    use advisor_core::{
        BestFitSection, Confidence, Receipt, RecommendReport, Recommendation,
        RecommendationArchetype, Tradeoff, TrustNote,
    };

    fn sample_recommend_report() -> RecommendReport {
        let recommendation = Recommendation {
            crate_name: "clap".to_string(),
            intent: "cli-parsing".to_string(),
            summary: "Best default for rich CLIs.".to_string(),
            confidence: Confidence::High,
            archetype: RecommendationArchetype::BestDefault,
            rationale: vec!["Strong derive support.".to_string()],
            fit_notes: vec!["Goal 'derive' normalized to 'fastest-to-ship'.".to_string()],
            tradeoffs: vec![Tradeoff {
                area: "compile time".to_string(),
                detail: "The feature surface is larger than leaner parsers.".to_string(),
            }],
            trust_notes: vec![TrustNote {
                label: "catalog".to_string(),
                detail: "Checked in locally.".to_string(),
            }],
            receipts: vec![Receipt {
                source: "catalog".to_string(),
                summary: "Used the curated cli-parsing entry for clap.".to_string(),
                detail: "Best default for rich CLIs.".to_string(),
            }],
            score: 114,
        };

        RecommendReport {
            requested_intent: "cli".to_string(),
            intent: "cli-parsing".to_string(),
            requested_goal: Some("derive".to_string()),
            goal: Some("fastest-to-ship".to_string()),
            summary: "clap is the current best best default for cli-parsing (requested as 'cli') when 'derive' resolves to 'fastest-to-ship'.".to_string(),
            recommendation: recommendation.clone(),
            confidence: Confidence::High,
            tradeoffs: recommendation.tradeoffs.clone(),
            alternatives: Vec::new(),
            best_fit_sections: vec![BestFitSection {
                label: "best default".to_string(),
                summary: "clap currently leads the best default view.".to_string(),
                recommendation,
            }],
            trust_notes: vec![TrustNote {
                label: "phase boundary".to_string(),
                detail: "No live telemetry was consulted.".to_string(),
            }],
            receipts: vec![Receipt {
                source: "intent normalization".to_string(),
                summary: "Normalized requested intent 'cli' to 'cli-parsing'.".to_string(),
                detail: "Aliases are deterministic.".to_string(),
            }],
        }
    }

    #[test]
    fn text_output_includes_best_fit_views() {
        let output = render_recommend(&sample_recommend_report(), OutputFormat::Text)
            .expect("text render should succeed");

        assert!(output.contains("Best-fit views"));
        assert!(output.contains("Fit notes"));
        assert!(output.contains("Goal: fastest-to-ship (requested as 'derive')"));
    }

    #[test]
    fn json_output_exposes_contract_fields() {
        let output = render_recommend(&sample_recommend_report(), OutputFormat::Json)
            .expect("json render should succeed");
        let value: serde_json::Value =
            serde_json::from_str(&output).expect("json render should parse");

        assert_eq!(value["command"], "recommend");
        assert_eq!(value["intent"], "cli-parsing");
        assert_eq!(value["goal"], "fastest-to-ship");
        assert_eq!(value["confidence"], "high");
        assert!(value["recommendation"]["fit_notes"].is_array());
        assert!(value["best_fit_sections"].is_array());
        assert!(value["receipts"].is_array());
    }
}
