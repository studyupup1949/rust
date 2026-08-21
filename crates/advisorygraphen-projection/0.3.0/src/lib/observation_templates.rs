use super::*;

pub(super) fn observation_command_template(candidate_type: &str) -> &'static str {
    match candidate_type {
        "owner_assignment" | "ownership_clarification" => {
            "Inspect source_ids_to_inspect for ownership evidence and return the owner claim, source id, and contradiction status."
        }
        "proposed_test" | "proposed_metric" => {
            "Inspect source_ids_to_inspect and define the smallest verification method, metric, or review path that can verify the requirement."
        }
        "proposed_interface" => {
            "Inspect boundary witnesses and source_ids_to_inspect, then identify the minimal interface contract and owner evidence."
        }
        "proposed_auth_guard" => {
            "Inspect route evidence and shared middleware evidence, then decide whether a route-specific auth guard is required."
        }
        _ => {
            "Inspect source_ids_to_inspect and candidate evidence, then return support, falsification, or insufficient-evidence status."
        }
    }
}

pub(super) fn required_observation_inputs(candidate_type: &str) -> Vec<&'static str> {
    let mut inputs = vec![
        "candidate_id",
        "hypothesis_id",
        "source_ids_to_inspect",
        "expected_observation",
        "falsifier",
    ];
    match candidate_type {
        "owner_assignment" | "ownership_clarification" => {
            inputs.push("owner_cell_id");
            inputs.push("blocked_cell_id");
        }
        "proposed_interface" => {
            inputs.push("from_cell_id");
            inputs.push("to_cell_id");
        }
        _ => {}
    }
    inputs
}

pub(super) fn observation_output_schema() -> Value {
    json!({
        "type": "object",
        "required": [
            "observation_status",
            "evidence_ids",
            "summary",
            "supports_hypothesis",
            "falsifies_hypothesis"
        ],
        "properties": {
            "observation_status": {
                "enum": [
                    "supports",
                    "falsifies",
                    "insufficient_evidence",
                    "requires_human_review"
                ]
            },
            "evidence_ids": {
                "type": "array",
                "items": { "type": "string" }
            },
            "summary": { "type": "string" },
            "supports_hypothesis": { "type": "boolean" },
            "falsifies_hypothesis": { "type": "boolean" },
            "review_note": { "type": "string" }
        }
    })
}

pub(super) fn pass_fail_extraction(candidate_type: &str) -> Value {
    match candidate_type {
        "owner_assignment" | "ownership_clarification" => json!({
            "pass_when": "A source-backed owner or ownership incidence is identified for the blocked action.",
            "fail_when": "No owner evidence exists, a different accepted owner is found, or ownership is explicitly collective.",
            "review_required": true
        }),
        "proposed_test" | "proposed_metric" => json!({
            "pass_when": "A concrete verification method, metric, or review path can be named and linked to the requirement.",
            "fail_when": "The requirement is exploratory, already verified, or cannot be verified within the source boundary.",
            "review_required": true
        }),
        "proposed_interface" => json!({
            "pass_when": "The boundary need and minimal interface contract are both supported by source evidence.",
            "fail_when": "The direct dependency is absent, already mediated, or cannot be tied to a source-backed requirement.",
            "review_required": true
        }),
        "proposed_auth_guard" => json!({
            "pass_when": "The route touches protected data and lacks an effective shared or route-specific guard.",
            "fail_when": "Existing middleware protects the route or the data is intentionally public.",
            "review_required": true
        }),
        _ => json!({
            "pass_when": "Source-backed evidence supports the blocking hypothesis and weakens relevant alternatives.",
            "fail_when": "Evidence falsifies the hypothesis or remains insufficient after inspecting bounded sources.",
            "review_required": true
        }),
    }
}

pub(super) fn expected_observation(candidate_type: &str, title: &str) -> String {
    match candidate_type {
        "owner_assignment" | "ownership_clarification" => format!(
            "Find source-backed evidence that the proposed owner is accountable for `{title}`."
        ),
        "proposed_test" | "proposed_metric" => format!(
            "Find or define a verification method that would demonstrate `{title}` without relying on agent inference."
        ),
        "proposed_interface" => format!(
            "Confirm the current cross-boundary need and the minimal interface shape required for `{title}`."
        ),
        "proposed_auth_guard" => format!(
            "Confirm the route lacks an effective guard and identify the guard required for `{title}`."
        ),
        _ => format!("Collect source-backed evidence that justifies `{title}`."),
    }
}

pub(super) fn falsifier_observation(candidate_type: &str, title: &str) -> String {
    match candidate_type {
        "owner_assignment" | "ownership_clarification" => format!(
            "A different accepted owner is documented for `{title}`, or ownership is intentionally collective."
        ),
        "proposed_test" | "proposed_metric" => format!(
            "The requirement behind `{title}` is exploratory or already verified by an existing accepted relation."
        ),
        "proposed_interface" => format!(
            "The direct dependency behind `{title}` is not present, or an accepted interface already exists."
        ),
        "proposed_auth_guard" => format!(
            "The route behind `{title}` is already protected by middleware or does not touch protected data."
        ),
        _ => format!("Accepted evidence contradicts the need for `{title}`."),
    }
}

pub(super) fn competing_hypotheses(candidate: &Value, current_hypothesis_id: &str) -> Vec<Value> {
    candidate
        .get("unsupported_hypothesis_ids")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|id| *id != current_hypothesis_id)
        .map(|id| json!(id))
        .collect()
}
