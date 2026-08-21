use advisorygraphen_core::{
    json_id, sorted_values_by_id, AdvisoryResult, AdvisorySpaceEnvelope, ReportEnvelope,
};
use higher_graphen_core::{Confidence, Id};
use higher_graphen_reasoning::completion::{
    CompletionCandidate, CompletionDetectionResult, MissingType, SuggestedStructure,
};
use serde_json::{json, Value};

pub fn propose_completions(
    space: &AdvisorySpaceEnvelope,
    check_report: &ReportEnvelope,
    from_report: &str,
    command: Option<&str>,
) -> AdvisoryResult<ReportEnvelope> {
    let mut candidates = Vec::new();
    let obstructions = check_report
        .result
        .get("obstructions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for obstruction in obstructions {
        let invariant_ids = obstruction_invariant_ids(check_report, json_id(&obstruction));
        match obstruction.get("obstruction_type").and_then(Value::as_str) {
            Some("boundary_violation") => candidates.extend(boundary_completion_candidates(
                space,
                &obstruction,
                &invariant_ids,
            )?),
            Some("missing_owner") => candidates.push(owner_completion_candidate(
                space,
                &obstruction,
                &invariant_ids,
            )?),
            Some("requirement_unverified") => candidates.push(verification_completion_candidate(
                space,
                &obstruction,
                &invariant_ids,
            )?),
            Some("api_route_missing_auth") => candidates.push(auth_guard_completion_candidate(
                space,
                &obstruction,
                &invariant_ids,
            )?),
            _ => {}
        }
    }
    enrich_candidates_with_hypothesis_support(&mut candidates, check_report);
    let higher_candidates = candidates
        .iter()
        .filter_map(|candidate| candidate.get("higher_graphen").cloned())
        .map(serde_json::from_value)
        .collect::<Result<Vec<CompletionCandidate>, _>>()?;
    let higher_detection = CompletionDetectionResult::new(
        hg_id(&space.space_id)?,
        space
            .contexts
            .iter()
            .map(|context| hg_id(json_id(context)))
            .collect::<AdvisoryResult<Vec<_>>>()?,
        higher_candidates,
    )
    .map_err(hg_err)?;
    candidates = sorted_values_by_id(candidates);
    Ok(ReportEnvelope::new(
        "completion_proposal",
        command,
        json!({
            "space_id": space.space_id,
            "from_report": from_report
        }),
        json!({
            "completion_candidates": candidates,
            "higher_graphen": higher_detection
        }),
    ))
}

fn enrich_candidates_with_hypothesis_support(
    candidates: &mut [Value],
    check_report: &ReportEnvelope,
) {
    let hypothesis_status = check_report
        .result
        .get("hypotheses")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|hypothesis| {
            let id = hypothesis.get("id")?.as_str()?.to_string();
            let status = hypothesis
                .get("lifecycle_status")
                .and_then(Value::as_str)
                .unwrap_or("candidate")
                .to_string();
            Some((id, status))
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    for candidate in candidates {
        let derived_hypothesis_id = candidate
            .pointer("/metadata/derived_from_hypothesis_id")
            .and_then(Value::as_str)
            .map(str::to_string);
        let lifecycle_status = derived_hypothesis_id
            .as_deref()
            .and_then(|id| hypothesis_status.get(id).map(String::as_str))
            .unwrap_or("missing");
        let hypothesis_supported = matches!(lifecycle_status, "supported" | "accepted");
        let supported_hypothesis_ids = if hypothesis_supported {
            derived_hypothesis_id.iter().cloned().collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        let unsupported_hypothesis_ids = if hypothesis_supported {
            Vec::new()
        } else {
            derived_hypothesis_id.iter().cloned().collect::<Vec<_>>()
        };
        candidate["supported_hypothesis_ids"] = json!(supported_hypothesis_ids);
        candidate["unsupported_hypothesis_ids"] = json!(unsupported_hypothesis_ids);
        candidate["hypothesis_trace"] = json!({
            "derived_hypothesis_id": derived_hypothesis_id,
            "lifecycle_status": lifecycle_status,
            "support_required_for_primary_recommendation": true,
            "supported": hypothesis_supported
        });

        let has_content_obstructions = candidate
            .pointer("/proposal_content/content_obstructions")
            .and_then(Value::as_array)
            .is_some_and(|items| !items.is_empty());
        let recommendation_role = if hypothesis_supported && !has_content_obstructions {
            "primary"
        } else if hypothesis_supported {
            "alternative"
        } else {
            "follow_up_observation"
        };
        candidate["recommendation_role"] = json!(recommendation_role);
        candidate["metadata"]["hypothesis_lifecycle_status"] = json!(lifecycle_status);
        candidate["metadata"]["recommendation_role"] = json!(recommendation_role);

        if !hypothesis_supported {
            add_proposal_trace_obstruction(candidate, lifecycle_status);
        }
    }
}

fn add_proposal_trace_obstruction(candidate: &mut Value, lifecycle_status: &str) {
    let obstruction = json!({
        "obstruction_type": "proposal_depends_on_unsupported_hypothesis",
        "message": "Proposal is derived from a hypothesis that is not supported or accepted; keep it out of primary recommendations.",
        "required_resolution": "Collect supporting observations or explicitly review the hypothesis before promoting this proposal.",
        "hypothesis_lifecycle_status": lifecycle_status,
        "review_status": "unreviewed"
    });
    if let Some(content) = candidate
        .get_mut("proposal_content")
        .and_then(Value::as_object_mut)
    {
        if let Some(items) = content
            .entry("content_obstructions".to_string())
            .or_insert_with(|| json!([]))
            .as_array_mut()
        {
            items.push(obstruction);
        }
        if let Some(scenario) = content.get_mut("scenario").and_then(Value::as_object_mut) {
            scenario.insert("status".to_string(), json!("blocked"));
        }
        if let Some(derivation) = content.get_mut("derivation").and_then(Value::as_object_mut) {
            derivation.insert(
                "verification_status".to_string(),
                json!("hypothesis_not_supported"),
            );
        }
    }
}

struct CandidateSpec<'a> {
    space: &'a AdvisorySpaceEnvelope,
    id: String,
    candidate_type: &'a str,
    title: String,
    rationale: String,
    resolves_obstruction_ids: Vec<String>,
    proposed_cell_ids: Vec<String>,
    proposed_incidence_ids: Vec<String>,
    source_ids: Vec<String>,
    affected_invariant_ids: Vec<String>,
    witness_ids: Vec<String>,
    blocked_ids: Vec<String>,
    confidence: f64,
    missing_type: MissingType,
    suggested_structure_type: &'a str,
    metadata: Value,
}

mod candidate_assembly;
mod candidate_factories;
mod lookup;
mod proposal_content;

use candidate_assembly::*;
use candidate_factories::*;
use lookup::*;
use proposal_content::*;
