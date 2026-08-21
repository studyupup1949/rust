use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    fn rank_score(&self) -> i32 {
        match self {
            Self::High => 90,
            Self::Medium => 75,
            Self::Low => 60,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Tradeoff {
    pub area: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TrustNote {
    pub label: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Receipt {
    pub source: String,
    pub summary: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
pub struct EvidenceBundle {
    pub receipts: Vec<Receipt>,
    pub trust_notes: Vec<TrustNote>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecommendationArchetype {
    BestDefault,
    LeanOption,
    PowerOption,
    Specialist,
}

impl RecommendationArchetype {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BestDefault => "best default",
            Self::LeanOption => "lean option",
            Self::PowerOption => "power option",
            Self::Specialist => "specialist option",
        }
    }

    fn rank_bonus(&self) -> i32 {
        match self {
            Self::BestDefault => 6,
            Self::LeanOption => 3,
            Self::PowerOption => 2,
            Self::Specialist => 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GoalFitStrength {
    Strong,
    Good,
    Weak,
}

impl GoalFitStrength {
    fn score_delta(&self) -> i32 {
        match self {
            Self::Strong => 24,
            Self::Good => 12,
            Self::Weak => -8,
        }
    }

    fn summary(&self) -> &'static str {
        match self {
            Self::Strong => "strong fit",
            Self::Good => "good fit",
            Self::Weak => "weaker fit",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GoalFit {
    pub goal: String,
    pub strength: GoalFitStrength,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CatalogEntry {
    pub crate_name: String,
    pub intent: String,
    pub summary: String,
    pub rationale: Vec<String>,
    pub goal_fits: Vec<GoalFit>,
    pub tradeoffs: Vec<Tradeoff>,
    pub trust_notes: Vec<TrustNote>,
    pub confidence: Confidence,
    pub archetype: RecommendationArchetype,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Recommendation {
    pub crate_name: String,
    pub intent: String,
    pub summary: String,
    pub confidence: Confidence,
    pub archetype: RecommendationArchetype,
    pub rationale: Vec<String>,
    pub fit_notes: Vec<String>,
    pub tradeoffs: Vec<Tradeoff>,
    pub trust_notes: Vec<TrustNote>,
    pub receipts: Vec<Receipt>,
    pub score: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BestFitSection {
    pub label: String,
    pub summary: String,
    pub recommendation: Recommendation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RecommendReport {
    pub requested_intent: String,
    pub intent: String,
    pub requested_goal: Option<String>,
    pub goal: Option<String>,
    pub summary: String,
    pub recommendation: Recommendation,
    pub confidence: Confidence,
    pub tradeoffs: Vec<Tradeoff>,
    pub alternatives: Vec<Recommendation>,
    pub best_fit_sections: Vec<BestFitSection>,
    pub trust_notes: Vec<TrustNote>,
    pub receipts: Vec<Receipt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CompareReport {
    pub requested_intent: Option<String>,
    pub intent: Option<String>,
    pub requested_crates: Vec<String>,
    pub summary: String,
    pub recommendation: Recommendation,
    pub confidence: Confidence,
    pub tradeoffs: Vec<Tradeoff>,
    pub alternatives: Vec<Recommendation>,
    pub trust_notes: Vec<TrustNote>,
    pub receipts: Vec<Receipt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ExplainReport {
    pub requested_intent: Option<String>,
    pub intent: String,
    pub summary: String,
    pub recommendation: Recommendation,
    pub confidence: Confidence,
    pub tradeoffs: Vec<Tradeoff>,
    pub trust_notes: Vec<TrustNote>,
    pub receipts: Vec<Receipt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingSeverity {
    Info,
    Warning,
}

impl FindingSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReviewFinding {
    pub severity: FindingSeverity,
    pub title: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedManifest {
    pub manifest_path: PathBuf,
    pub package_name: Option<String>,
    pub is_root: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewDependencyKind {
    Normal,
    Dev,
    Build,
}

impl ReviewDependencyKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Dev => "dev",
            Self::Build => "build",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestDependency {
    pub manifest_path: PathBuf,
    pub package_name: Option<String>,
    pub dependency_name: String,
    pub declared_name: String,
    pub kind: ReviewDependencyKind,
    pub target: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewInputs {
    pub manifest_path: PathBuf,
    pub manifest_contents: Option<String>,
    pub lockfile_path: PathBuf,
    pub lockfile_contents: Option<String>,
    pub manifests: Vec<LoadedManifest>,
    pub dependencies: Vec<ManifestDependency>,
    pub evidence: EvidenceBundle,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReviewedManifest {
    pub manifest_path: PathBuf,
    pub package_name: Option<String>,
    pub is_root: bool,
    pub dependency_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LockfileDuplicateVersion {
    pub crate_name: String,
    pub versions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LockfileSummary {
    pub package_count: usize,
    pub duplicate_versions: Vec<LockfileDuplicateVersion>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReviewReport {
    pub summary: String,
    pub manifest_path: PathBuf,
    pub lockfile_path: PathBuf,
    pub manifests: Vec<ReviewedManifest>,
    pub dependencies: Vec<String>,
    pub lockfile_summary: Option<LockfileSummary>,
    pub findings: Vec<ReviewFinding>,
    pub recommendation: Option<Recommendation>,
    pub confidence: Option<Confidence>,
    pub tradeoffs: Vec<Tradeoff>,
    pub follow_up_recommendations: Vec<Recommendation>,
    pub trust_notes: Vec<TrustNote>,
    pub receipts: Vec<Receipt>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdvisorError {
    Usage(String),
    UnsupportedIntent {
        requested: String,
        supported: Vec<String>,
    },
}

impl fmt::Display for AdvisorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}"),
            Self::UnsupportedIntent {
                requested,
                supported,
            } => write!(
                f,
                "unsupported intent '{requested}'. Supported intents: {}",
                supported.join(", ")
            ),
        }
    }
}

impl std::error::Error for AdvisorError {}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GoalContext {
    requested: String,
    canonical: Option<String>,
}

pub fn supported_intents(catalog: &[CatalogEntry]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    catalog
        .iter()
        .filter_map(|entry| {
            if seen.insert(entry.intent.clone()) {
                Some(entry.intent.clone())
            } else {
                None
            }
        })
        .collect()
}

pub fn recommend(
    catalog: &[CatalogEntry],
    intent: &str,
    goal: Option<&str>,
    evidence: &EvidenceBundle,
) -> Result<RecommendReport, AdvisorError> {
    let supported = supported_intents(catalog);
    let resolved_intent =
        resolve_intent(catalog, intent).ok_or_else(|| AdvisorError::UnsupportedIntent {
            requested: intent.to_string(),
            supported,
        })?;
    let goal_context = goal.map(goal_context);
    let mut matches: Vec<&CatalogEntry> = catalog
        .iter()
        .filter(|entry| normalize_key(&entry.intent) == normalize_key(&resolved_intent))
        .collect();

    matches.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));
    let mut ranked = build_ranked_recommendations(matches, goal_context.as_ref(), evidence);
    ranked.sort_by(recommendation_sort);

    let recommendation = ranked.remove(0);
    let candidate_count = ranked.len() + 1;
    let summary = build_recommend_summary(
        &recommendation,
        intent,
        &resolved_intent,
        goal_context.as_ref(),
    );
    let best_fit_sections = build_best_fit_sections(&recommendation, &ranked);

    Ok(RecommendReport {
        requested_intent: intent.to_string(),
        intent: resolved_intent.clone(),
        requested_goal: goal.map(ToOwned::to_owned),
        goal: goal_context.as_ref().and_then(|goal| goal.canonical.clone()),
        summary,
        confidence: recommendation.confidence.clone(),
        tradeoffs: recommendation.tradeoffs.clone(),
        recommendation: recommendation.clone(),
        alternatives: ranked,
        best_fit_sections,
        trust_notes: merge_trust_notes([
            recommendation.trust_notes.clone(),
            build_resolution_trust_notes(goal_context.as_ref()),
            evidence.trust_notes.clone(),
        ]),
        receipts: merge_receipts([
            recommendation.receipts.clone(),
            build_resolution_receipts(intent, &resolved_intent, goal_context.as_ref()),
            evidence.receipts.clone(),
            vec![Receipt {
                source: "catalog".to_string(),
                summary: format!(
                    "Matched {} curated candidates for intent '{}'.",
                    candidate_count,
                    resolved_intent
                ),
                detail:
                    "Phase 2 still ranks only checked-in catalog entries and explicit command inputs."
                        .to_string(),
            }],
        ]),
    })
}

pub fn compare(
    catalog: &[CatalogEntry],
    requested_crates: &[String],
    intent: Option<&str>,
    evidence: &EvidenceBundle,
) -> Result<CompareReport, AdvisorError> {
    let resolved_intent = intent
        .map(|requested| {
            ensure_supported_intent(catalog, requested).map(|value| (requested.to_string(), value))
        })
        .transpose()?;

    let mut ranked: Vec<Recommendation> = requested_crates
        .iter()
        .map(|crate_name| {
            explain_candidate(
                catalog,
                crate_name,
                resolved_intent
                    .as_ref()
                    .map(|(_, resolved)| resolved.as_str()),
                evidence,
            )
        })
        .collect();
    ranked.sort_by(recommendation_sort);

    let recommendation = ranked.remove(0);
    let summary = match resolved_intent.as_ref() {
        Some((requested, resolved)) if requested != resolved => format!(
            "{} is the best current fit for {} (requested as '{}') among {}.",
            recommendation.crate_name,
            resolved,
            requested,
            requested_crates.join(", ")
        ),
        Some((_, resolved)) => format!(
            "{} is the best current fit for {} among {}.",
            recommendation.crate_name,
            resolved,
            requested_crates.join(", ")
        ),
        None => format!(
            "{} has the strongest fit in the phase-2 catalog among {}.",
            recommendation.crate_name,
            requested_crates.join(", ")
        ),
    };

    Ok(CompareReport {
        requested_intent: resolved_intent.as_ref().map(|(requested, _)| requested.clone()),
        intent: resolved_intent.as_ref().map(|(_, resolved)| resolved.clone()),
        requested_crates: requested_crates.to_vec(),
        summary,
        confidence: recommendation.confidence.clone(),
        tradeoffs: recommendation.tradeoffs.clone(),
        recommendation: recommendation.clone(),
        alternatives: ranked,
        trust_notes: merge_trust_notes([
            recommendation.trust_notes.clone(),
            evidence.trust_notes.clone(),
        ]),
        receipts: merge_receipts([
            recommendation.receipts.clone(),
            resolved_intent
                .as_ref()
                .filter(|(requested, resolved)| requested != resolved)
                .map(|(requested, resolved)| {
                    vec![Receipt {
                        source: "intent normalization".to_string(),
                        summary: format!(
                            "Normalized requested intent '{}' to '{}'.",
                            requested, resolved
                        ),
                        detail:
                            "Intent aliases are deterministic and map to the checked-in canonical taxonomy."
                                .to_string(),
                    }]
                })
                .unwrap_or_default(),
            evidence.receipts.clone(),
            vec![Receipt {
                source: "catalog".to_string(),
                summary: format!(
                    "Compared {} requested crates against the local catalog.",
                    requested_crates.len()
                ),
                detail: "No live registry or documentation sources were queried during compare."
                    .to_string(),
            }],
        ]),
    })
}

pub fn explain(
    catalog: &[CatalogEntry],
    crate_name: &str,
    intent: Option<&str>,
    evidence: &EvidenceBundle,
) -> Result<ExplainReport, AdvisorError> {
    let resolved_intent = intent
        .map(|requested| {
            ensure_supported_intent(catalog, requested).map(|value| (requested.to_string(), value))
        })
        .transpose()?;

    let recommendation = explain_candidate(
        catalog,
        crate_name,
        resolved_intent
            .as_ref()
            .map(|(_, resolved)| resolved.as_str()),
        evidence,
    );
    let summary = match resolved_intent.as_ref() {
        Some((requested, resolved)) if requested != resolved => format!(
            "{} is evaluated against {} (requested as '{}') with {} confidence.",
            recommendation.crate_name,
            resolved,
            requested,
            recommendation.confidence.as_str()
        ),
        Some((_, resolved)) => format!(
            "{} is evaluated against '{}' with {} confidence.",
            recommendation.crate_name,
            resolved,
            recommendation.confidence.as_str()
        ),
        None => format!(
            "{} is evaluated from the phase-2 curated catalog with {} confidence.",
            recommendation.crate_name,
            recommendation.confidence.as_str()
        ),
    };

    Ok(ExplainReport {
        requested_intent: resolved_intent.as_ref().map(|(requested, _)| requested.clone()),
        intent: recommendation.intent.clone(),
        summary,
        confidence: recommendation.confidence.clone(),
        tradeoffs: recommendation.tradeoffs.clone(),
        recommendation: recommendation.clone(),
        trust_notes: merge_trust_notes([
            recommendation.trust_notes.clone(),
            evidence.trust_notes.clone(),
        ]),
        receipts: merge_receipts([
            recommendation.receipts.clone(),
            resolved_intent
                .as_ref()
                .filter(|(requested, resolved)| requested != resolved)
                .map(|(requested, resolved)| {
                    vec![Receipt {
                        source: "intent normalization".to_string(),
                        summary: format!(
                            "Normalized requested intent '{}' to '{}'.",
                            requested, resolved
                        ),
                        detail:
                            "Intent aliases are deterministic and map to the checked-in canonical taxonomy."
                                .to_string(),
                    }]
                })
                .unwrap_or_default(),
            evidence.receipts.clone(),
            vec![Receipt {
                source: "catalog".to_string(),
                summary: format!("Explained '{}' from the local curated catalog.", crate_name),
                detail: "Phase 2 explain still does not pull fresh registry or security metadata."
                    .to_string(),
            }],
        ]),
    })
}

pub fn review(catalog: &[CatalogEntry], inputs: &ReviewInputs) -> ReviewReport {
    let reviewed_manifests = build_reviewed_manifests(inputs);
    let dependencies = dedup_strings(
        inputs
            .dependencies
            .iter()
            .map(|dependency| dependency.dependency_name.clone())
            .collect(),
    );
    let lockfile_packages = inputs
        .lockfile_contents
        .as_deref()
        .map(parse_lockfile_package_entries)
        .unwrap_or_default();
    let lockfile_summary = inputs.lockfile_contents.as_deref().map(summarize_lockfile);

    let mut findings = Vec::new();
    if inputs.manifest_contents.is_none() {
        findings.push(ReviewFinding {
            severity: FindingSeverity::Warning,
            title: "Manifest missing".to_string(),
            detail: format!(
                "No manifest was loaded from '{}', so review could not inspect dependencies.",
                inputs.manifest_path.display()
            ),
        });
    }

    if inputs.dependencies.is_empty() && inputs.manifest_contents.is_some() {
        findings.push(ReviewFinding {
            severity: FindingSeverity::Info,
            title: "No dependencies found".to_string(),
            detail:
                "The local review did not detect direct dependencies in normal, dev, build, or target-specific dependency sections."
                    .to_string(),
        });
    }

    if inputs.lockfile_contents.is_none() {
        findings.push(ReviewFinding {
            severity: FindingSeverity::Info,
            title: "Lockfile missing".to_string(),
            detail: format!(
                "No lockfile was loaded from '{}'; version-level review receipts are limited.",
                inputs.lockfile_path.display()
            ),
        });
    } else if lockfile_packages.is_empty() {
        findings.push(ReviewFinding {
            severity: FindingSeverity::Info,
            title: "Lockfile parsed with no packages".to_string(),
            detail: "The local lockfile did not yield package entries for version summarization."
                .to_string(),
        });
    }

    let overlap_findings = detect_overlap_findings(catalog, &inputs.dependencies);
    let mut overlap_receipts = Vec::new();
    let mut follow_up_recommendations = Vec::new();
    let mut seen_recommendations = BTreeSet::new();
    for overlap in overlap_findings {
        findings.push(overlap.finding);
        overlap_receipts.push(overlap.receipt);
        if let Ok(report) = recommend(catalog, &overlap.intent, None, &EvidenceBundle::default()) {
            let recommendation = report.recommendation;
            if seen_recommendations.insert(recommendation.crate_name.clone()) {
                follow_up_recommendations.push(recommendation);
            }
        }
    }

    if let Some(summary) = lockfile_summary.as_ref() {
        let direct_dependency_names: BTreeSet<_> = dependencies.iter().cloned().collect();
        let direct_duplicates: Vec<_> = summary
            .duplicate_versions
            .iter()
            .filter(|duplicate| direct_dependency_names.contains(&duplicate.crate_name))
            .cloned()
            .collect();

        if !direct_duplicates.is_empty() {
            findings.push(ReviewFinding {
                severity: FindingSeverity::Warning,
                title: "Direct dependency version spread".to_string(),
                detail: format!(
                    "The lockfile carries multiple resolved versions for direct dependencies: {}.",
                    format_duplicate_versions(&direct_duplicates)
                ),
            });
        } else if !summary.duplicate_versions.is_empty() {
            findings.push(ReviewFinding {
                severity: FindingSeverity::Info,
                title: "Lockfile version spread".to_string(),
                detail: format!(
                    "The lockfile shows multiple resolved versions for transitive crates: {}.",
                    format_duplicate_versions(&summary.duplicate_versions)
                ),
            });
        }
    }

    let mut receipts = inputs.evidence.receipts.clone();
    if !reviewed_manifests.is_empty() {
        receipts.push(Receipt {
            source: "review manifests".to_string(),
            summary: format!(
                "Reviewed {} manifest(s) and {} direct dependency declarations.",
                reviewed_manifests.len(),
                inputs.dependencies.len()
            ),
            detail: format!(
                "Manifests: {}",
                reviewed_manifests
                    .iter()
                    .map(format_manifest_receipt)
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        });
    }

    if let Some(summary) = lockfile_summary.as_ref() {
        receipts.push(Receipt {
            source: "review lockfile".to_string(),
            summary: format!(
                "Parsed {} package entries from '{}'.",
                summary.package_count,
                inputs.lockfile_path.display()
            ),
            detail: if summary.duplicate_versions.is_empty() {
                "No duplicate crate versions were observed in the local lockfile.".to_string()
            } else {
                format!(
                    "Duplicate versions observed for: {}.",
                    format_duplicate_versions(&summary.duplicate_versions)
                )
            },
        });
    }
    receipts.extend(overlap_receipts);

    if findings.is_empty() {
        findings.push(ReviewFinding {
            severity: FindingSeverity::Info,
            title: "No consolidation hotspots".to_string(),
            detail:
                "The local review did not detect overlapping dependency families or lockfile version spread that needs follow-up."
                    .to_string(),
        });
    }

    let summary = if let Some(finding) = findings
        .iter()
        .find(|finding| finding.severity == FindingSeverity::Warning)
    {
        format!("Local review flagged a warning: {}.", finding.title)
    } else {
        format!(
            "Local review scanned {} direct dependencies across {} manifest(s) without warning-level consolidation issues.",
            dependencies.len(),
            reviewed_manifests.len()
        )
    };

    let recommendation = follow_up_recommendations.first().cloned();
    let confidence = recommendation
        .as_ref()
        .map(|value| value.confidence.clone());
    let tradeoffs = recommendation
        .as_ref()
        .map(|value| value.tradeoffs.clone())
        .unwrap_or_default();

    ReviewReport {
        summary,
        manifest_path: inputs.manifest_path.clone(),
        lockfile_path: inputs.lockfile_path.clone(),
        manifests: reviewed_manifests,
        dependencies,
        lockfile_summary,
        findings,
        recommendation,
        confidence,
        tradeoffs,
        follow_up_recommendations,
        trust_notes: merge_trust_notes([
            inputs.evidence.trust_notes.clone(),
            vec![TrustNote {
                label: "phase-3 local review".to_string(),
                detail:
                    "Review uses local manifests, local cargo metadata, and local lockfile contents only; no live registry, docs, benchmark, or security sources were consulted."
                        .to_string(),
            }],
        ]),
        receipts,
    }
}

pub fn parse_manifest_dependency_entries(
    contents: &str,
    manifest_path: &Path,
    package_name: Option<&str>,
) -> Vec<ManifestDependency> {
    let mut dependencies = Vec::new();
    let mut current_section = None;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = parse_dependency_section(line);
            if let Some(section) = current_section.as_ref() {
                if let Some(dependency_name) = section.dependency_name.as_deref() {
                    dependencies.push(ManifestDependency {
                        manifest_path: manifest_path.to_path_buf(),
                        package_name: package_name.map(ToOwned::to_owned),
                        dependency_name: dependency_name.to_string(),
                        declared_name: dependency_name.to_string(),
                        kind: section.kind.clone(),
                        target: section.target.clone(),
                    });
                }
            }
            continue;
        }

        let Some(section) = current_section.as_ref() else {
            continue;
        };
        if section.dependency_name.is_some() {
            continue;
        }

        let Some((raw_key, raw_value)) = line.split_once('=') else {
            continue;
        };
        let key = raw_key.trim();
        let declared_name = key.strip_suffix(".workspace").unwrap_or(key).trim();
        if !is_valid_dependency_key(declared_name) {
            continue;
        }

        dependencies.push(ManifestDependency {
            manifest_path: manifest_path.to_path_buf(),
            package_name: package_name.map(ToOwned::to_owned),
            dependency_name: inline_package_name(raw_value.trim())
                .unwrap_or_else(|| declared_name.to_string()),
            declared_name: declared_name.to_string(),
            kind: section.kind.clone(),
            target: section.target.clone(),
        });
    }

    dedup_manifest_dependencies(dependencies)
}

pub fn parse_manifest_dependencies(contents: &str) -> Vec<String> {
    dedup_strings(
        parse_manifest_dependency_entries(contents, Path::new("Cargo.toml"), None)
            .into_iter()
            .map(|dependency| dependency.dependency_name)
            .collect(),
    )
}

pub fn parse_lockfile_packages(contents: &str) -> Vec<String> {
    dedup_strings(
        parse_lockfile_package_entries(contents)
            .into_iter()
            .map(|package| package.name)
            .collect(),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DependencySection {
    kind: ReviewDependencyKind,
    target: Option<String>,
    dependency_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LockfilePackage {
    name: String,
    version: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OverlapFinding {
    intent: String,
    finding: ReviewFinding,
    receipt: Receipt,
}

fn build_reviewed_manifests(inputs: &ReviewInputs) -> Vec<ReviewedManifest> {
    let mut dependency_counts = BTreeMap::new();
    for dependency in &inputs.dependencies {
        *dependency_counts
            .entry(dependency.manifest_path.clone())
            .or_insert(0usize) += 1;
    }

    let mut seen = BTreeSet::new();
    let mut manifests = Vec::new();
    for manifest in &inputs.manifests {
        if seen.insert(manifest.manifest_path.clone()) {
            manifests.push(ReviewedManifest {
                manifest_path: manifest.manifest_path.clone(),
                package_name: manifest.package_name.clone(),
                is_root: manifest.is_root,
                dependency_count: dependency_counts
                    .get(&manifest.manifest_path)
                    .copied()
                    .unwrap_or_default(),
            });
        }
    }

    if manifests.is_empty() && inputs.manifest_contents.is_some() {
        manifests.push(ReviewedManifest {
            manifest_path: inputs.manifest_path.clone(),
            package_name: None,
            is_root: true,
            dependency_count: inputs.dependencies.len(),
        });
    }

    manifests.sort_by(|left, right| {
        right
            .is_root
            .cmp(&left.is_root)
            .then_with(|| left.manifest_path.cmp(&right.manifest_path))
    });
    manifests
}

fn format_manifest_receipt(manifest: &ReviewedManifest) -> String {
    let role = if manifest.is_root {
        "review root"
    } else {
        manifest
            .package_name
            .as_deref()
            .unwrap_or("workspace member")
    };
    format!(
        "{} at '{}' ({} direct dependencies)",
        role,
        manifest.manifest_path.display(),
        manifest.dependency_count
    )
}

fn parse_dependency_section(header: &str) -> Option<DependencySection> {
    let section = header.trim_matches(&['[', ']'][..]);
    let (kind, target, dependency_name) = if section == "dependencies" {
        (ReviewDependencyKind::Normal, None, None)
    } else if section == "dev-dependencies" {
        (ReviewDependencyKind::Dev, None, None)
    } else if section == "build-dependencies" {
        (ReviewDependencyKind::Build, None, None)
    } else if let Some(dependency_name) = section.strip_prefix("dependencies.") {
        (
            ReviewDependencyKind::Normal,
            None,
            Some(dependency_name.to_string()),
        )
    } else if let Some(dependency_name) = section.strip_prefix("dev-dependencies.") {
        (
            ReviewDependencyKind::Dev,
            None,
            Some(dependency_name.to_string()),
        )
    } else if let Some(dependency_name) = section.strip_prefix("build-dependencies.") {
        (
            ReviewDependencyKind::Build,
            None,
            Some(dependency_name.to_string()),
        )
    } else if let Some(rest) = section.strip_prefix("target.") {
        if let Some((target, suffix)) = rest.rsplit_once(".dependencies.") {
            (
                ReviewDependencyKind::Normal,
                Some(target.to_string()),
                Some(suffix.to_string()),
            )
        } else if let Some((target, suffix)) = rest.rsplit_once(".dev-dependencies.") {
            (
                ReviewDependencyKind::Dev,
                Some(target.to_string()),
                Some(suffix.to_string()),
            )
        } else if let Some((target, suffix)) = rest.rsplit_once(".build-dependencies.") {
            (
                ReviewDependencyKind::Build,
                Some(target.to_string()),
                Some(suffix.to_string()),
            )
        } else if let Some(target) = rest.strip_suffix(".dependencies") {
            (ReviewDependencyKind::Normal, Some(target.to_string()), None)
        } else if let Some(target) = rest.strip_suffix(".dev-dependencies") {
            (ReviewDependencyKind::Dev, Some(target.to_string()), None)
        } else if let Some(target) = rest.strip_suffix(".build-dependencies") {
            (ReviewDependencyKind::Build, Some(target.to_string()), None)
        } else {
            return None;
        }
    } else {
        return None;
    };

    Some(DependencySection {
        kind,
        target,
        dependency_name,
    })
}

fn is_valid_dependency_key(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
}

fn inline_package_name(value: &str) -> Option<String> {
    let (_, tail) = value.split_once("package")?;
    let (_, tail) = tail.split_once('"')?;
    let (package_name, _) = tail.split_once('"')?;
    if is_valid_dependency_key(package_name) {
        Some(package_name.to_string())
    } else {
        None
    }
}

fn dedup_manifest_dependencies(dependencies: Vec<ManifestDependency>) -> Vec<ManifestDependency> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for dependency in dependencies {
        let key = (
            dependency.manifest_path.clone(),
            dependency.package_name.clone(),
            dependency.dependency_name.clone(),
            dependency.declared_name.clone(),
            dependency.kind.label().to_string(),
            dependency.target.clone(),
        );
        if seen.insert(key) {
            deduped.push(dependency);
        }
    }
    deduped
}

fn parse_lockfile_package_entries(contents: &str) -> Vec<LockfilePackage> {
    let mut packages = Vec::new();
    let mut in_package = false;
    let mut current_name = None;
    let mut current_version = None;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line == "[[package]]" {
            if in_package {
                if let (Some(name), Some(version)) = (current_name.take(), current_version.take()) {
                    packages.push(LockfilePackage { name, version });
                }
            }
            in_package = true;
            current_name = None;
            current_version = None;
            continue;
        }

        if !in_package {
            continue;
        }

        if let Some(value) = line.strip_prefix("name = \"") {
            if let Some(value) = value.strip_suffix('"') {
                current_name = Some(value.to_string());
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("version = \"") {
            if let Some(value) = value.strip_suffix('"') {
                current_version = Some(value.to_string());
            }
        }
    }

    if let (Some(name), Some(version)) = (current_name, current_version) {
        packages.push(LockfilePackage { name, version });
    }

    packages
}

fn summarize_lockfile(contents: &str) -> LockfileSummary {
    let packages = parse_lockfile_package_entries(contents);
    let mut versions_by_crate: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for package in &packages {
        versions_by_crate
            .entry(package.name.clone())
            .or_default()
            .insert(package.version.clone());
    }

    let duplicate_versions = versions_by_crate
        .into_iter()
        .filter_map(|(crate_name, versions)| {
            if versions.len() > 1 {
                Some(LockfileDuplicateVersion {
                    crate_name,
                    versions: versions.into_iter().collect(),
                })
            } else {
                None
            }
        })
        .collect();

    LockfileSummary {
        package_count: packages.len(),
        duplicate_versions,
    }
}

fn format_duplicate_versions(duplicates: &[LockfileDuplicateVersion]) -> String {
    duplicates
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

fn detect_overlap_findings(
    catalog: &[CatalogEntry],
    dependencies: &[ManifestDependency],
) -> Vec<OverlapFinding> {
    let mut by_intent: BTreeMap<String, BTreeMap<String, Vec<&ManifestDependency>>> =
        BTreeMap::new();

    for dependency in dependencies {
        if let Some(entry) = catalog.iter().find(|entry| {
            normalize_key(&entry.crate_name) == normalize_key(&dependency.dependency_name)
        }) {
            by_intent
                .entry(entry.intent.clone())
                .or_default()
                .entry(entry.crate_name.clone())
                .or_default()
                .push(dependency);
        }
    }

    let mut overlaps = Vec::new();
    for (intent, crates) in by_intent {
        if !should_flag_overlap(&intent, &crates) {
            continue;
        }

        let crate_names: Vec<_> = crates.keys().cloned().collect();
        let severity = overlap_severity(&crates);
        let migration_target = default_migration_target(catalog, &intent);
        let detail = format!(
            "{} Consolidate toward {}.",
            format_overlap_detail(&intent, &crates),
            migration_target
                .as_deref()
                .map(|target| migration_guidance(&intent, target))
                .unwrap_or_else(|| "one primary stack for this decision area".to_string())
        );

        overlaps.push(OverlapFinding {
            intent: intent.clone(),
            finding: ReviewFinding {
                severity,
                title: format!("Overlapping stack: {intent}"),
                detail: detail.clone(),
            },
            receipt: Receipt {
                source: "review overlap".to_string(),
                summary: format!(
                    "Detected overlapping crates for '{}': {}.",
                    intent,
                    crate_names.join(", ")
                ),
                detail,
            },
        });
    }

    overlaps
}

fn should_flag_overlap(intent: &str, crates: &BTreeMap<String, Vec<&ManifestDependency>>) -> bool {
    if crates.len() <= 1 {
        return false;
    }

    match intent {
        "error-handling" | "testing" => false,
        "logging-tracing" => {
            let crate_names: BTreeSet<_> = crates.keys().map(String::as_str).collect();
            crate_names != BTreeSet::from(["env_logger", "log"])
        }
        _ => true,
    }
}

fn overlap_severity(crates: &BTreeMap<String, Vec<&ManifestDependency>>) -> FindingSeverity {
    if crates
        .values()
        .flatten()
        .any(|dependency| dependency.kind != ReviewDependencyKind::Dev)
    {
        FindingSeverity::Warning
    } else {
        FindingSeverity::Info
    }
}

fn default_migration_target(catalog: &[CatalogEntry], intent: &str) -> Option<String> {
    catalog
        .iter()
        .filter(|entry| normalize_key(&entry.intent) == normalize_key(intent))
        .max_by(|left, right| {
            left.confidence
                .rank_score()
                .cmp(&right.confidence.rank_score())
                .then_with(|| {
                    left.archetype
                        .rank_bonus()
                        .cmp(&right.archetype.rank_bonus())
                })
        })
        .map(|entry| entry.crate_name.clone())
}

fn migration_guidance(intent: &str, target: &str) -> String {
    match intent {
        "cli-parsing" => format!(
            "{target} so help text, completions, and parser behavior stay on one CLI surface"
        ),
        "config" => format!(
            "{target} so configuration layering and overrides follow one model across the project"
        ),
        "logging-tracing" => format!(
            "{target} as the primary telemetry stack unless a library intentionally keeps a facade-only `log` surface"
        ),
        "http-client" => {
            format!("{target} to avoid duplicating TLS, retry, middleware, and client ergonomics")
        }
        "http-server" => format!(
            "{target} so server/runtime integration, middleware, and handler style stay consistent"
        ),
        "serialization" => format!(
            "{target} as the main serialization boundary to reduce conversion and trait-surface churn"
        ),
        "async-runtime" => format!(
            "{target} so executors, timing primitives, and ecosystem integrations do not split"
        ),
        "database-access" => {
            format!("{target} so the query model and runtime/database integration stay on one path")
        }
        _ => "one primary stack for this decision area".to_string(),
    }
}

fn format_overlap_detail(
    intent: &str,
    crates: &BTreeMap<String, Vec<&ManifestDependency>>,
) -> String {
    format!(
        "The review found multiple crates for '{}': {}.",
        intent,
        crates
            .iter()
            .map(|(crate_name, entries)| format!(
                "{crate_name} [{}]",
                format_dependency_locations(entries)
            ))
            .collect::<Vec<_>>()
            .join("; ")
    )
}

fn format_dependency_locations(entries: &[&ManifestDependency]) -> String {
    let mut locations = Vec::new();
    let mut seen = BTreeSet::new();
    for entry in entries {
        let label = format!(
            "{}{}{}",
            entry.manifest_path.display(),
            match entry.package_name.as_deref() {
                Some(package_name) => format!(" ({package_name})"),
                None => String::new(),
            },
            match entry.target.as_deref() {
                Some(target) => format!(" [{} target {}]", entry.kind.label(), target),
                None => format!(" [{}]", entry.kind.label()),
            }
        );
        if seen.insert(label.clone()) {
            locations.push(label);
        }
    }
    locations.join(", ")
}

fn build_recommend_summary(
    recommendation: &Recommendation,
    requested_intent: &str,
    resolved_intent: &str,
    goal: Option<&GoalContext>,
) -> String {
    let intent_clause = if requested_intent == resolved_intent {
        resolved_intent.to_string()
    } else {
        format!("{resolved_intent} (requested as '{requested_intent}')")
    };

    match goal {
        Some(goal) => match goal.canonical.as_deref() {
            Some(canonical) if canonical != goal.requested => format!(
                "{} is the current best {} for {} when '{}' resolves to '{}'.",
                recommendation.crate_name,
                recommendation.archetype.label(),
                intent_clause,
                goal.requested,
                canonical
            ),
            Some(canonical) => format!(
                "{} is the current best {} for {} when the goal is '{}'.",
                recommendation.crate_name,
                recommendation.archetype.label(),
                intent_clause,
                canonical
            ),
            None => format!(
                "{} is the current best {} for {} using the raw goal '{}'.",
                recommendation.crate_name,
                recommendation.archetype.label(),
                intent_clause,
                goal.requested
            ),
        },
        None => format!(
            "{} is the current best {} for {} in the curated phase-2 catalog.",
            recommendation.crate_name,
            recommendation.archetype.label(),
            intent_clause
        ),
    }
}

fn build_best_fit_sections(
    recommendation: &Recommendation,
    alternatives: &[Recommendation],
) -> Vec<BestFitSection> {
    let mut candidates = vec![recommendation.clone()];
    candidates.extend(alternatives.iter().cloned());

    let views = [
        RecommendationArchetype::BestDefault,
        RecommendationArchetype::LeanOption,
        RecommendationArchetype::PowerOption,
    ];

    let mut sections = Vec::new();
    for archetype in views {
        if let Some(best) = candidates
            .iter()
            .filter(|candidate| candidate.archetype == archetype)
            .max_by(|left, right| {
                left.score
                    .cmp(&right.score)
                    .then_with(|| right.confidence.cmp(&left.confidence))
                    .then_with(|| right.crate_name.cmp(&left.crate_name))
            })
            .cloned()
        {
            sections.push(BestFitSection {
                label: archetype.label().to_string(),
                summary: format!(
                    "{} currently leads the {} view.",
                    best.crate_name,
                    archetype.label()
                ),
                recommendation: best,
            });
        }
    }
    sections
}

fn ensure_supported_intent(catalog: &[CatalogEntry], intent: &str) -> Result<String, AdvisorError> {
    let supported = supported_intents(catalog);
    resolve_intent(catalog, intent).ok_or_else(|| AdvisorError::UnsupportedIntent {
        requested: intent.to_string(),
        supported,
    })
}

fn resolve_intent(catalog: &[CatalogEntry], intent: &str) -> Option<String> {
    let normalized = normalize_key(intent);
    let supported = supported_intents(catalog);
    supported
        .iter()
        .find(|supported_intent| normalize_key(supported_intent) == normalized)
        .cloned()
        .or_else(|| {
            intent_aliases().iter().find_map(|(alias, canonical)| {
                if normalized == *alias {
                    supported
                        .iter()
                        .find(|supported_intent| normalize_key(supported_intent) == *canonical)
                        .cloned()
                } else {
                    None
                }
            })
        })
}

fn explain_candidate(
    catalog: &[CatalogEntry],
    crate_name: &str,
    requested_intent: Option<&str>,
    evidence: &EvidenceBundle,
) -> Recommendation {
    if let Some(entry) = catalog.iter().find(|entry| {
        normalize_key(&entry.crate_name) == normalize_key(crate_name)
            && requested_intent
                .map(|intent| normalize_key(&entry.intent) == normalize_key(intent))
                .unwrap_or(true)
    }) {
        score_entry(entry, None, evidence)
    } else if let Some(entry) = catalog
        .iter()
        .find(|entry| normalize_key(&entry.crate_name) == normalize_key(crate_name))
    {
        let mut recommendation = score_entry(entry, None, evidence);
        if let Some(intent) = requested_intent {
            recommendation.confidence = Confidence::Low;
            recommendation.summary = format!(
                "{} is cataloged for {}, not {}.",
                entry.crate_name, entry.intent, intent
            );
            recommendation.fit_notes.push(format!(
                "Requested intent '{}' does not match the curated '{}' entry.",
                intent, entry.intent
            ));
            recommendation.tradeoffs.push(Tradeoff {
                area: "intent mismatch".to_string(),
                detail: format!(
                    "The crate is curated under '{}' instead of the requested '{}'.",
                    entry.intent, intent
                ),
            });
            recommendation.trust_notes.push(TrustNote {
                label: "curated mismatch".to_string(),
                detail: "The catalog found the crate, but not under the requested intent."
                    .to_string(),
            });
            recommendation.score -= 25;
        }
        recommendation
    } else {
        Recommendation {
            crate_name: crate_name.to_string(),
            intent: requested_intent.unwrap_or("uncurated").to_string(),
            summary: format!(
                "{} is not in the phase-2 curated catalog, so explain is limited.",
                crate_name
            ),
            confidence: Confidence::Low,
            archetype: RecommendationArchetype::Specialist,
            rationale: vec![
                "The crate name was provided explicitly.".to_string(),
                "No checked-in catalog entry matched it.".to_string(),
            ],
            fit_notes: vec!["No curated fit profile was available for this crate.".to_string()],
            tradeoffs: vec![Tradeoff {
                area: "coverage".to_string(),
                detail: "Phase 2 still does not perform live registry, docs, or security lookups."
                    .to_string(),
            }],
            trust_notes: merge_trust_notes([
                vec![TrustNote {
                    label: "catalog gap".to_string(),
                    detail:
                        "This result is a bounded fallback because the crate is outside the local curated catalog."
                            .to_string(),
                }],
                evidence.trust_notes.clone(),
            ]),
            receipts: merge_receipts([
                vec![Receipt {
                    source: "catalog".to_string(),
                    summary: format!("No curated entry matched '{}'.", crate_name),
                    detail: "The explain fallback was generated without live external evidence."
                        .to_string(),
                }],
                evidence.receipts.clone(),
            ]),
            score: 30,
        }
    }
}

fn build_ranked_recommendations(
    entries: Vec<&CatalogEntry>,
    goal: Option<&GoalContext>,
    evidence: &EvidenceBundle,
) -> Vec<Recommendation> {
    entries
        .into_iter()
        .map(|entry| score_entry(entry, goal, evidence))
        .collect()
}

fn score_entry(
    entry: &CatalogEntry,
    goal: Option<&GoalContext>,
    evidence: &EvidenceBundle,
) -> Recommendation {
    let mut score = entry.confidence.rank_score() + entry.archetype.rank_bonus();
    let mut rationale = entry.rationale.clone();
    let mut fit_notes = vec![format!(
        "Curated as the {} for {}.",
        entry.archetype.label(),
        entry.intent
    )];

    if let Some(goal) = goal {
        match goal.canonical.as_deref() {
            Some(canonical) => {
                if let Some(goal_fit) = entry
                    .goal_fits
                    .iter()
                    .find(|goal_fit| normalize_key(&goal_fit.goal) == normalize_key(canonical))
                {
                    score += goal_fit.strength.score_delta();
                    fit_notes.push(format!(
                        "Goal '{}' normalized to '{}' and {} for {}.",
                        goal.requested,
                        canonical,
                        goal_fit.strength.summary(),
                        entry.crate_name
                    ));
                    fit_notes.push(goal_fit.detail.clone());
                    rationale.push(format!(
                        "The '{}' goal semantics materially influence {} toward this choice.",
                        canonical, entry.crate_name
                    ));
                } else {
                    fit_notes.push(format!(
                        "Goal '{}' normalized to '{}', but this entry has no explicit curated boost for it.",
                        goal.requested, canonical
                    ));
                }
            }
            None => {
                fit_notes.push(format!(
                    "Goal '{}' was kept as free text because it did not map to the curated phase-2 goal vocabulary.",
                    goal.requested
                ));
            }
        }
    }

    Recommendation {
        crate_name: entry.crate_name.clone(),
        intent: entry.intent.clone(),
        summary: entry.summary.clone(),
        confidence: entry.confidence.clone(),
        archetype: entry.archetype.clone(),
        rationale,
        fit_notes,
        tradeoffs: entry.tradeoffs.clone(),
        trust_notes: merge_trust_notes([entry.trust_notes.clone(), evidence.trust_notes.clone()]),
        receipts: merge_receipts([
            vec![Receipt {
                source: "catalog".to_string(),
                summary: format!(
                    "Used the curated '{}' entry for '{}'.",
                    entry.intent, entry.crate_name
                ),
                detail: entry.summary.clone(),
            }],
            evidence.receipts.clone(),
        ]),
        score,
    }
}

fn goal_context(value: &str) -> GoalContext {
    GoalContext {
        requested: value.to_string(),
        canonical: resolve_goal(value),
    }
}

fn resolve_goal(value: &str) -> Option<String> {
    let normalized = normalize_key(value);
    goal_aliases().iter().find_map(|(alias, canonical)| {
        if normalized == *alias {
            Some((*canonical).to_string())
        } else {
            None
        }
    })
}

fn build_resolution_receipts(
    requested_intent: &str,
    resolved_intent: &str,
    goal: Option<&GoalContext>,
) -> Vec<Receipt> {
    let mut receipts = Vec::new();
    if requested_intent != resolved_intent {
        receipts.push(Receipt {
            source: "intent normalization".to_string(),
            summary: format!(
                "Normalized requested intent '{}' to '{}'.",
                requested_intent, resolved_intent
            ),
            detail:
                "Intent aliases are deterministic and map to the checked-in canonical taxonomy."
                    .to_string(),
        });
    }

    if let Some(goal) = goal {
        match goal.canonical.as_deref() {
            Some(canonical) if canonical != goal.requested => receipts.push(Receipt {
                source: "goal normalization".to_string(),
                summary: format!(
                    "Normalized requested goal '{}' to '{}'.",
                    goal.requested, canonical
                ),
                detail:
                    "Goal normalization uses a small checked-in phase-2 vocabulary instead of live ecosystem signals."
                        .to_string(),
            }),
            Some(canonical) => receipts.push(Receipt {
                source: "goal normalization".to_string(),
                summary: format!("Goal '{}' matched the curated goal vocabulary.", canonical),
                detail:
                    "Scoring used the canonical goal semantics defined in the checked-in catalog."
                        .to_string(),
            }),
            None => receipts.push(Receipt {
                source: "goal normalization".to_string(),
                summary: format!(
                    "Goal '{}' did not map to the curated goal vocabulary.",
                    goal.requested
                ),
                detail:
                    "No semantic score boost was invented beyond the deterministic checked-in catalog."
                        .to_string(),
            }),
        }
    }

    receipts
}

fn build_resolution_trust_notes(goal: Option<&GoalContext>) -> Vec<TrustNote> {
    match goal {
        Some(goal) if goal.canonical.is_none() => vec![TrustNote {
            label: "goal boundary".to_string(),
            detail:
                "The goal text stayed as free text because it did not map to the curated phase-2 goal vocabulary."
                    .to_string(),
        }],
        _ => Vec::new(),
    }
}

fn recommendation_sort(left: &Recommendation, right: &Recommendation) -> std::cmp::Ordering {
    right
        .score
        .cmp(&left.score)
        .then_with(|| left.confidence.cmp(&right.confidence))
        .then_with(|| left.crate_name.cmp(&right.crate_name))
}

fn normalize_key(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_dash = false;
    for character in value.chars().flat_map(|character| character.to_lowercase()) {
        if character.is_ascii_alphanumeric() {
            normalized.push(character);
            previous_dash = false;
        } else if !previous_dash {
            normalized.push('-');
            previous_dash = true;
        }
    }
    normalized.trim_matches('-').to_string()
}

fn intent_aliases() -> &'static [(&'static str, &'static str)] {
    &[
        ("cli", "cli-parsing"),
        ("cli-parser", "cli-parsing"),
        ("args", "cli-parsing"),
        ("argument-parsing", "cli-parsing"),
        ("configuration", "config"),
        ("settings", "config"),
        ("telemetry", "logging-tracing"),
        ("logging", "logging-tracing"),
        ("tracing", "logging-tracing"),
        ("http", "http-client"),
        ("http-client", "http-client"),
        ("api-client", "http-client"),
        ("rest-client", "http-client"),
        ("http-api", "http-server"),
        ("rest-api", "http-server"),
        ("web-api", "http-server"),
        ("web-server", "http-server"),
        ("serde", "serialization"),
        ("json", "serialization"),
        ("encoding", "serialization"),
        ("async", "async-runtime"),
        ("runtime", "async-runtime"),
        ("executor", "async-runtime"),
        ("errors", "error-handling"),
        ("error", "error-handling"),
        ("diagnostics", "error-handling"),
        ("tests", "testing"),
        ("snapshots", "testing"),
        ("db", "database-access"),
        ("database", "database-access"),
        ("orm", "database-access"),
        ("sql", "database-access"),
    ]
}

fn goal_aliases() -> &'static [(&'static str, &'static str)] {
    &[
        ("default", "boring-default"),
        ("boring", "boring-default"),
        ("safe-default", "boring-default"),
        ("conventional", "boring-default"),
        ("minimal", "minimal-footprint"),
        ("small", "minimal-footprint"),
        ("small-binary", "minimal-footprint"),
        ("lightweight", "minimal-footprint"),
        ("lean", "minimal-footprint"),
        ("footprint", "minimal-footprint"),
        ("control", "maximum-control"),
        ("power", "maximum-control"),
        ("custom", "maximum-control"),
        ("low-level", "maximum-control"),
        ("maximum-control", "maximum-control"),
        ("ship", "fastest-to-ship"),
        ("quick", "fastest-to-ship"),
        ("quickly", "fastest-to-ship"),
        ("ergonomic", "fastest-to-ship"),
        ("productive", "fastest-to-ship"),
        ("derive", "fastest-to-ship"),
        ("blocking", "blocking"),
        ("sync", "blocking"),
        ("synchronous", "blocking"),
        ("async", "async"),
        ("non-blocking", "async"),
        ("tokio", "async"),
        ("typed", "typed-surfaces"),
        ("typed-api", "typed-surfaces"),
        ("explicit", "typed-surfaces"),
        ("library", "typed-surfaces"),
        ("diagnostics", "rich-diagnostics"),
        ("reports", "rich-diagnostics"),
        ("observability", "rich-diagnostics"),
        ("property", "property-coverage"),
        ("properties", "property-coverage"),
        ("randomized", "property-coverage"),
        ("layered", "layered-config"),
        ("providers", "layered-config"),
        ("merge", "layered-config"),
        ("env-and-files", "layered-config"),
    ]
}

fn merge_trust_notes<const N: usize>(groups: [Vec<TrustNote>; N]) -> Vec<TrustNote> {
    let mut seen = BTreeSet::new();
    let mut merged = Vec::new();
    for group in groups {
        for note in group {
            let key = format!("{}::{}", note.label, note.detail);
            if seen.insert(key) {
                merged.push(note);
            }
        }
    }
    merged
}

fn merge_receipts<const N: usize>(groups: [Vec<Receipt>; N]) -> Vec<Receipt> {
    let mut seen = BTreeSet::new();
    let mut merged = Vec::new();
    for group in groups {
        for receipt in group {
            let key = format!(
                "{}::{}::{}",
                receipt.source, receipt.summary, receipt.detail
            );
            if seen.insert(key) {
                merged.push(receipt);
            }
        }
    }
    merged
}

fn dedup_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_catalog() -> Vec<CatalogEntry> {
        vec![
            CatalogEntry {
                crate_name: "clap".to_string(),
                intent: "cli-parsing".to_string(),
                summary: "Default pick for rich CLIs.".to_string(),
                rationale: vec!["Strong derive support.".to_string()],
                goal_fits: vec![
                    GoalFit {
                        goal: "fastest-to-ship".to_string(),
                        strength: GoalFitStrength::Strong,
                        detail: "Derive-first ergonomics keep full-featured CLI work fast."
                            .to_string(),
                    },
                    GoalFit {
                        goal: "boring-default".to_string(),
                        strength: GoalFitStrength::Strong,
                        detail: "It is the lowest-risk default for most application CLIs."
                            .to_string(),
                    },
                ],
                tradeoffs: vec![Tradeoff {
                    area: "compile time".to_string(),
                    detail: "Feature-rich setup increases compile cost.".to_string(),
                }],
                trust_notes: vec![TrustNote {
                    label: "catalog".to_string(),
                    detail: "Curated locally.".to_string(),
                }],
                confidence: Confidence::High,
                archetype: RecommendationArchetype::BestDefault,
            },
            CatalogEntry {
                crate_name: "argh".to_string(),
                intent: "cli-parsing".to_string(),
                summary: "Small and direct for simple CLIs.".to_string(),
                rationale: vec!["Good for minimal argument surfaces.".to_string()],
                goal_fits: vec![GoalFit {
                    goal: "minimal-footprint".to_string(),
                    strength: GoalFitStrength::Strong,
                    detail: "It keeps parser surface and dependency weight low.".to_string(),
                }],
                tradeoffs: vec![Tradeoff {
                    area: "breadth".to_string(),
                    detail: "Smaller feature surface than clap.".to_string(),
                }],
                trust_notes: vec![],
                confidence: Confidence::Medium,
                archetype: RecommendationArchetype::LeanOption,
            },
            CatalogEntry {
                crate_name: "bpaf".to_string(),
                intent: "cli-parsing".to_string(),
                summary: "Combinator-first for precise parser control.".to_string(),
                rationale: vec!["Good for bespoke argument flows.".to_string()],
                goal_fits: vec![GoalFit {
                    goal: "maximum-control".to_string(),
                    strength: GoalFitStrength::Strong,
                    detail: "The composable parser model keeps more behavior explicit.".to_string(),
                }],
                tradeoffs: vec![Tradeoff {
                    area: "learning curve".to_string(),
                    detail: "Less familiar than derive-heavy CLIs.".to_string(),
                }],
                trust_notes: vec![],
                confidence: Confidence::Medium,
                archetype: RecommendationArchetype::PowerOption,
            },
        ]
    }

    #[test]
    fn recommend_normalizes_intent_aliases() {
        let report = recommend(&sample_catalog(), "cli", None, &EvidenceBundle::default())
            .expect("recommend should succeed");

        assert_eq!(report.intent, "cli-parsing");
        assert_eq!(report.recommendation.crate_name, "clap");
        assert!(
            report
                .receipts
                .iter()
                .any(|receipt| receipt.source == "intent normalization")
        );
    }

    #[test]
    fn recommend_uses_canonical_goal_semantics() {
        let report = recommend(
            &sample_catalog(),
            "cli parsing",
            Some("small binary"),
            &EvidenceBundle::default(),
        )
        .expect("recommend should succeed");

        assert_eq!(report.goal.as_deref(), Some("minimal-footprint"));
        assert_eq!(report.recommendation.crate_name, "argh");
        assert!(
            report
                .recommendation
                .fit_notes
                .iter()
                .any(|line| line.contains("minimal-footprint"))
        );
    }

    #[test]
    fn recommend_exposes_best_fit_sections() {
        let report = recommend(
            &sample_catalog(),
            "cli-parsing",
            None,
            &EvidenceBundle::default(),
        )
        .expect("recommend should succeed");

        assert_eq!(report.best_fit_sections.len(), 3);
        assert_eq!(report.best_fit_sections[1].label, "lean option");
        assert_eq!(
            report.best_fit_sections[2].recommendation.crate_name,
            "bpaf"
        );
    }

    #[test]
    fn parse_manifest_dependencies_reads_dependency_sections() {
        let manifest = r#"
            [package]
            name = "demo"

            [dependencies]
            clap = "4"
            serde = { version = "1", features = ["derive"] }

            [dev-dependencies]
            insta = "1"
        "#;

        assert_eq!(
            parse_manifest_dependencies(manifest),
            vec!["clap".to_string(), "serde".to_string(), "insta".to_string()]
        );
    }

    #[test]
    fn parse_manifest_dependency_entries_captures_targets_and_renames() {
        let manifest = r#"
            [package]
            name = "demo"

            [dependencies]
            http = { package = "reqwest", version = "0.12" }

            [target.'cfg(unix)'.build-dependencies]
            cc = "1"

            [dev-dependencies.insta]
            version = "1"
        "#;

        let dependencies =
            parse_manifest_dependency_entries(manifest, Path::new("Cargo.toml"), Some("demo"));

        assert!(dependencies.iter().any(|dependency| {
            dependency.dependency_name == "reqwest"
                && dependency.declared_name == "http"
                && dependency.kind == ReviewDependencyKind::Normal
        }));
        assert!(dependencies.iter().any(|dependency| {
            dependency.dependency_name == "cc"
                && dependency.kind == ReviewDependencyKind::Build
                && dependency.target.as_deref() == Some("'cfg(unix)'")
        }));
        assert!(dependencies.iter().any(|dependency| {
            dependency.dependency_name == "insta" && dependency.kind == ReviewDependencyKind::Dev
        }));
    }

    #[test]
    fn parse_lockfile_packages_reads_package_names() {
        let lockfile = r#"
            [[package]]
            name = "clap"
            version = "4.0.0"

            [[package]]
            name = "serde"
            version = "1.0.0"
        "#;

        assert_eq!(
            parse_lockfile_packages(lockfile),
            vec!["clap".to_string(), "serde".to_string()]
        );
    }

    #[test]
    fn review_flags_direct_dependency_version_spread() {
        let report = review(
            &sample_catalog(),
            &ReviewInputs {
                manifest_path: PathBuf::from("Cargo.toml"),
                manifest_contents: Some("[package]\nname = \"demo\"\n".to_string()),
                lockfile_path: PathBuf::from("Cargo.lock"),
                lockfile_contents: Some(
                    r#"
                        version = 3

                        [[package]]
                        name = "reqwest"
                        version = "0.11.27"

                        [[package]]
                        name = "reqwest"
                        version = "0.12.15"
                    "#
                    .to_string(),
                ),
                manifests: vec![LoadedManifest {
                    manifest_path: PathBuf::from("Cargo.toml"),
                    package_name: Some("demo".to_string()),
                    is_root: true,
                }],
                dependencies: vec![ManifestDependency {
                    manifest_path: PathBuf::from("Cargo.toml"),
                    package_name: Some("demo".to_string()),
                    dependency_name: "reqwest".to_string(),
                    declared_name: "reqwest".to_string(),
                    kind: ReviewDependencyKind::Normal,
                    target: None,
                }],
                evidence: EvidenceBundle::default(),
            },
        );

        assert_eq!(report.findings[0].title, "Direct dependency version spread");
        assert_eq!(
            report.lockfile_summary.as_ref().unwrap().duplicate_versions[0].crate_name,
            "reqwest"
        );
    }
}
