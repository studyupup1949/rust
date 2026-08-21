//! SARIF v2.1.0 report output.
//!
//! Static Analysis Results Interchange Format — the industry standard for
//! CI/CD tool annotations on GitHub, GitLab, and Azure DevOps.
//!
//! aaai maps audit results as follows:
//! * Failed  → level "error"
//! * Pending → level "warning"
//! * Error   → level "error"
//! * OK      → (omitted or "note")
//! * Ignored → (omitted)

use std::path::Path;

use serde_json::{json, Value};

use crate::audit::result::{AuditResult, AuditStatus};

pub fn build_sarif(
    result: &AuditResult,
    before_root: &Path,
    after_root: &Path,
) -> Value {
    let rules: Vec<Value> = vec![
        sarif_rule("AAAI001", "AuditFailed",
            "A diff entry did not match its expected audit rule.",
            "error"),
        sarif_rule("AAAI002", "AuditPending",
            "A diff entry has no audit rule — human review required.",
            "warning"),
        sarif_rule("AAAI003", "AuditError",
            "A file could not be read or compared.",
            "error"),
    ];

    let results: Vec<Value> = result.results.iter()
        .filter_map(|r| {
            let (rule_id, level) = match r.status {
                AuditStatus::Failed  => ("AAAI001", "error"),
                AuditStatus::Pending => ("AAAI002", "warning"),
                AuditStatus::Error   => ("AAAI003", "error"),
                _                    => return None,
            };

            let message = r.detail.as_deref()
                .or_else(|| r.entry.as_ref().and_then(|e|
                    if e.reason.is_empty() { None } else { Some(e.reason.as_str()) }
                ))
                .unwrap_or("Audit issue detected.")
                .to_string();

            // Use the after-root path for "current state" location.
            let uri = format!("{}/{}", after_root.display(), r.diff.path);

            Some(json!({
                "ruleId": rule_id,
                "level": level,
                "message": { "text": message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": uri,
                            "uriBaseId": "%SRCROOT%"
                        }
                    }
                }],
                "properties": {
                    "diffType":   r.diff.diff_type.to_string(),
                    "status":     r.status.to_string(),
                    "isBinary":   r.diff.is_binary,
                    "ticket":     r.entry.as_ref().and_then(|e| e.ticket.as_ref()),
                    "approvedBy": r.entry.as_ref().and_then(|e| e.approved_by.as_ref()),
                }
            }))
        })
        .collect();

    json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "aaai",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/nabbisen/aaai",
                    "rules": rules,
                }
            },
            "originalUriBaseIds": {
                "%SRCROOT%": { "uri": format!("{}/", after_root.display()) }
            },
            "results": results,
            "properties": {
                "before": before_root.display().to_string(),
                "after":  after_root.display().to_string(),
                "passed": result.summary.is_passing(),
            }
        }]
    })
}

fn sarif_rule(id: &str, name: &str, description: &str, level: &str) -> Value {
    json!({
        "id": id,
        "name": name,
        "shortDescription": { "text": description },
        "defaultConfiguration": { "level": level },
    })
}

#[cfg(test)]
mod tests;
