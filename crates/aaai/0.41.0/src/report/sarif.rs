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
use crate::masking::Masking;

pub fn build_sarif(
    result: &AuditResult,
    before_root: &Path,
    after_root: &Path,
    masking: Masking<'_>,
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

            let message_raw = r.detail.as_deref()
                .or_else(|| r.entry.as_ref().and_then(|e|
                    if e.reason.is_empty() { None } else { Some(e.reason.as_str()) }
                ))
                .unwrap_or("Audit issue detected.");
            // F3 — SARIF never had a masker at all; both `message` (which
            // may carry `reason`) and `ticket` are §4 maskable fields.
            let message = masking.mask(message_raw);
            let ticket = r
                .entry
                .as_ref()
                .and_then(|e| e.ticket.as_ref())
                .map(|t| masking.mask(t));

            // Use the after-root path for "current state" location. `path`
            // is encode-only (serde_json escapes it correctly); it must
            // never be masked, or the finding stops matching the real file.
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
                    "ticket":     ticket,
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
            // §4 root paths: masked here, unlike `originalUriBaseIds` and
            // each result's `artifactLocation.uri` above — those are
            // functional navigation targets a SARIF consumer resolves back
            // to a real file, the same reason `path` itself is never
            // masked; these two are purely informational summary fields.
            "properties": {
                "before": masking.mask(&before_root.display().to_string()),
                "after":  masking.mask(&after_root.display().to_string()),
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
