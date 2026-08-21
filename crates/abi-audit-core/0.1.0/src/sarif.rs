use std::collections::BTreeMap;

use anyhow::Result;
use serde::Serialize;

use crate::model::{CheckResult, Severity};
use crate::rules::{all_rule_metadata, rule_metadata};

pub fn render_sarif(result: &CheckResult) -> Result<String> {
    let log = SarifLog {
        schema: "https://json.schemastore.org/sarif-2.1.0.json",
        version: "2.1.0",
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "cargo-abi-audit",
                    information_uri: None,
                    rules: all_rule_metadata()
                        .iter()
                        .map(|rule| SarifRule {
                            id: rule.code.to_string(),
                            name: Some(rule.title.to_string()),
                            short_description: Message {
                                text: rule.title.to_string(),
                            },
                            full_description: Message {
                                text: rule.summary.to_string(),
                            },
                            default_configuration: SarifDefaultConfiguration {
                                level: sarif_level(rule.default_severity),
                            },
                            properties: SarifRuleProperties {
                                tags: vec![
                                    "abi".to_string(),
                                    "ffi".to_string(),
                                    "c-abi".to_string(),
                                ],
                            },
                        })
                        .collect(),
                },
            },
            results: result
                .report
                .findings
                .iter()
                .map(|finding| {
                    let mut partial_fingerprints = BTreeMap::new();
                    partial_fingerprints.insert(
                        "primary".to_string(),
                        format!(
                            "{}|{}|{}|{}",
                            finding.package,
                            finding.export.as_deref().unwrap_or("-"),
                            finding.code,
                            finding.message
                        ),
                    );

                    SarifResult {
                        rule_id: finding.code.clone(),
                        level: sarif_level(finding.severity),
                        message: Message {
                            text: finding.message.clone(),
                        },
                        locations: finding.location.as_ref().map(|location| {
                            vec![SarifLocation {
                                physical_location: SarifPhysicalLocation {
                                    artifact_location: SarifArtifactLocation {
                                        uri: location.path.clone(),
                                    },
                                    region: SarifRegion {
                                        start_line: location.line,
                                    },
                                },
                            }]
                        }),
                        partial_fingerprints,
                        properties: SarifResultProperties {
                            package: finding.package.clone(),
                            export: finding.export.clone(),
                            evidence: finding.evidence.clone(),
                            precision: "high".to_string(),
                            rule_summary: rule_metadata(&finding.code)
                                .map(|rule| rule.summary.to_string()),
                        },
                    }
                })
                .collect(),
            invocations: vec![SarifInvocation {
                execution_successful: result.exit_code == 0,
            }],
        }],
    };
    Ok(serde_json::to_string_pretty(&log)?)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLog {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
    invocations: Vec<SarifInvocation>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifDriver {
    name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    information_uri: Option<String>,
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRule {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    short_description: Message,
    full_description: Message,
    default_configuration: SarifDefaultConfiguration,
    properties: SarifRuleProperties,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifDefaultConfiguration {
    level: &'static str,
}

#[derive(Serialize)]
struct Message {
    text: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRuleProperties {
    tags: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult {
    rule_id: String,
    level: &'static str,
    message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    locations: Option<Vec<SarifLocation>>,
    partial_fingerprints: BTreeMap<String, String>,
    properties: SarifResultProperties,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResultProperties {
    package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    export: Option<String>,
    evidence: Vec<String>,
    precision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule_summary: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLocation {
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRegion {
    start_line: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifInvocation {
    execution_successful: bool,
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}
