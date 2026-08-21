use super::*;

pub(super) fn recommendation_trace(candidates: &[Value]) -> Value {
    let mut primary = Vec::new();
    let mut alternatives = Vec::new();
    let mut follow_up = Vec::new();
    let mut unsupported = 0_u64;

    for candidate in candidates {
        let item = recommendation_trace_item(candidate);
        match candidate
            .get("recommendation_role")
            .and_then(Value::as_str)
            .unwrap_or("follow_up_observation")
        {
            "primary" => primary.push(item),
            "alternative" => alternatives.push(item),
            _ => {
                unsupported += 1;
                follow_up.push(item);
            }
        }
    }

    json!({
        "primary_count": primary.len(),
        "alternative_count": alternatives.len(),
        "follow_up_observation_count": follow_up.len(),
        "unsupported_hypothesis_candidate_count": unsupported,
        "primary_recommendations": primary,
        "alternatives": alternatives,
        "follow_up_observations": follow_up,
        "rule": "Only candidates derived from supported or accepted hypotheses can be primary recommendations."
    })
}

pub(super) fn observation_actions(recommendation_trace: &Value) -> Value {
    let actions = recommendation_trace
        .get("follow_up_observations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|item| {
            item.get("ranked_observation_tasks")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        })
        .map(observation_action_from_task)
        .collect::<Vec<_>>();
    json!({
        "count": actions.len(),
        "actions": actions,
        "rule": "Observation actions recommend bounded evidence-gathering steps; they do not execute observations or accept the investigated claim."
    })
}

pub(super) fn observation_action_from_task(task: Value) -> Value {
    let task_id = task
        .get("task_id")
        .and_then(Value::as_str)
        .unwrap_or("observation:unknown");
    let source_ids = task
        .get("source_ids_to_inspect")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let review_required = task
        .pointer("/pass_fail_extraction/review_required")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let mut target_claims = Vec::new();
    if let Some(hypothesis_id) = task.get("hypothesis_id").and_then(Value::as_str) {
        target_claims.push(json!(hypothesis_id));
    } else if let Some(hypothesis_id) = task.get("hypothesis_id").filter(|value| !value.is_null()) {
        target_claims.push(hypothesis_id.clone());
    }
    json!({
        "id": format!("observation-action:{}", id_fragment(task_id)),
        "task_id": task_id,
        "target_claim_ids": target_claims,
        "candidate_id": task.get("candidate_id").cloned().unwrap_or(Value::Null),
        "expected_evidence_kind": expected_evidence_kind(task.get("observation_type").and_then(Value::as_str)),
        "estimated_cost": estimated_observation_cost(source_ids.len()),
        "expected_information_gain": expected_information_gain(task.get("observation_type").and_then(Value::as_str)),
        "policy_blockers": if review_required { json!(["review_required"]) } else { json!([]) },
        "source_ids_to_inspect": source_ids,
        "expected_observation": task.get("expected_observation").cloned().unwrap_or(Value::Null),
        "falsifier": task.get("falsifier").cloned().unwrap_or(Value::Null),
        "output_schema": task.get("output_schema").cloned().unwrap_or(Value::Null),
        "review_status": "unreviewed",
        "provenance": {
            "origin": "inferred",
            "actor": "advisorygraphen-projection",
            "confidence": 0.7,
            "review_status": "unreviewed"
        }
    })
}

pub(super) fn expected_evidence_kind(observation_type: Option<&str>) -> &'static str {
    match observation_type {
        Some("hypothesis_support") => "support_or_falsification_witness",
        Some("proposal_structure_completion") => "structure_witness",
        Some("review_readiness") => "review_readiness_witness",
        _ => "bounded_observation_witness",
    }
}

pub(super) fn expected_information_gain(observation_type: Option<&str>) -> &'static str {
    match observation_type {
        Some("hypothesis_support") => "high",
        Some("proposal_structure_completion") => "medium",
        Some("review_readiness") => "medium",
        _ => "unknown",
    }
}

pub(super) fn estimated_observation_cost(source_count: usize) -> &'static str {
    match source_count {
        0 | 1 => "low",
        2 | 3 => "medium",
        _ => "high",
    }
}

pub(super) fn recommendation_trace_item(candidate: &Value) -> Value {
    json!({
        "candidate_id": candidate.get("id").cloned().unwrap_or(Value::Null),
        "title": candidate.get("title").cloned().unwrap_or(Value::Null),
        "candidate_type": candidate.get("candidate_type").cloned().unwrap_or(Value::Null),
        "recommendation_role": candidate.get("recommendation_role").cloned().unwrap_or_else(|| json!("follow_up_observation")),
        "derived_hypothesis_id": candidate.pointer("/hypothesis_trace/derived_hypothesis_id").cloned().unwrap_or(Value::Null),
        "hypothesis_lifecycle_status": candidate.pointer("/hypothesis_trace/lifecycle_status").cloned().unwrap_or(Value::Null),
        "supported_hypothesis_ids": candidate.get("supported_hypothesis_ids").cloned().unwrap_or_else(|| json!([])),
        "unsupported_hypothesis_ids": candidate.get("unsupported_hypothesis_ids").cloned().unwrap_or_else(|| json!([])),
        "required_verification": candidate.pointer("/proposal_content/content_obstructions")
            .and_then(Value::as_array)
            .map(|items| {
                items.iter()
                    .filter_map(|item| item.get("required_resolution").and_then(Value::as_str))
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        "ranked_observation_tasks": ranked_observation_tasks(candidate)
    })
}

pub(super) fn ranked_observation_tasks(candidate: &Value) -> Vec<Value> {
    let mut tasks = Vec::new();
    let candidate_id = candidate
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("candidate:unknown");
    let title = candidate
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("candidate");
    let candidate_type = candidate
        .get("candidate_type")
        .and_then(Value::as_str)
        .unwrap_or("completion_candidate");
    let source_ids = candidate
        .get("source_ids")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let unsupported_hypothesis_ids = candidate
        .get("unsupported_hypothesis_ids")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut rank = 1_u64;

    for hypothesis_id in unsupported_hypothesis_ids.iter().filter_map(Value::as_str) {
        tasks.push(json!({
            "rank": rank,
            "task_id": format!("observation:{}:support-{}", id_tail(candidate_id), rank),
            "observation_type": "hypothesis_support",
            "candidate_id": candidate_id,
            "hypothesis_id": hypothesis_id,
            "source_ids_to_inspect": source_ids,
            "command_template": observation_command_template(candidate_type),
            "required_inputs": required_observation_inputs(candidate_type),
            "output_schema": observation_output_schema(),
            "pass_fail_extraction": pass_fail_extraction(candidate_type),
            "expected_observation": expected_observation(candidate_type, title),
            "falsifier": falsifier_observation(candidate_type, title),
            "weakens_hypothesis_ids": competing_hypotheses(candidate, hypothesis_id),
            "promotion_effect": "If this observation supports the hypothesis, review-gated hypothesis support or acceptance can allow the candidate to be reconsidered as primary or alternative."
        }));
        rank += 1;
    }

    for obstruction in candidate
        .pointer("/proposal_content/content_obstructions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if obstruction.get("obstruction_type").and_then(Value::as_str)
            != Some("proposal_content_underspecified")
        {
            continue;
        }
        tasks.push(json!({
            "rank": rank,
            "task_id": format!("observation:{}:complete-structure", id_tail(candidate_id)),
            "observation_type": "proposal_structure_completion",
            "candidate_id": candidate_id,
            "hypothesis_id": candidate.pointer("/hypothesis_trace/derived_hypothesis_id").cloned().unwrap_or(Value::Null),
            "source_ids_to_inspect": source_ids,
            "command_template": "Inspect the candidate proposal_content and source snapshot, then draft the exact cell or incidence required to repair the obstruction.",
            "required_inputs": [
                "candidate_id",
                "proposal_content",
                "resolves_obstruction_ids",
                "bounded_source_snapshot"
            ],
            "output_schema": observation_output_schema(),
            "pass_fail_extraction": {
                "pass_when": "The output names concrete cells or incidences to add and maps them to the repaired obstruction.",
                "fail_when": "The output cannot name concrete structure without inventing facts outside the source boundary.",
                "review_required": true
            },
            "expected_observation": "Identify the concrete cell or incidence that would be added, plus the exact obstruction it repairs.",
            "falsifier": "No concrete structure can be named without inventing facts beyond the bounded source snapshot.",
            "weakens_hypothesis_ids": [],
            "promotion_effect": "A concrete proposed structure removes underspecification but still requires review before acceptance."
        }));
        rank += 1;
    }

    if tasks.is_empty()
        && candidate.get("recommendation_role").and_then(Value::as_str) != Some("primary")
    {
        tasks.push(json!({
            "rank": rank,
            "task_id": format!("observation:{}:review-readiness", id_tail(candidate_id)),
            "observation_type": "review_readiness",
            "candidate_id": candidate_id,
            "hypothesis_id": candidate.pointer("/hypothesis_trace/derived_hypothesis_id").cloned().unwrap_or(Value::Null),
            "source_ids_to_inspect": source_ids,
            "command_template": "Review candidate evidence, owners, verification fields, and proposal_content obstructions for promotion readiness.",
            "required_inputs": [
                "candidate_id",
                "supported_hypothesis_ids",
                "unsupported_hypothesis_ids",
                "proposal_content.content_obstructions"
            ],
            "output_schema": observation_output_schema(),
            "pass_fail_extraction": {
                "pass_when": "The candidate has supported or accepted hypotheses and no unresolved content obstructions.",
                "fail_when": "The candidate still depends on unreviewed hypotheses or underspecified proposal content.",
                "review_required": true
            },
            "expected_observation": "Confirm whether the candidate has enough accepted evidence, owner, and verification to enter review.",
            "falsifier": "The candidate still depends on inferred or unreviewed structure.",
            "weakens_hypothesis_ids": [],
            "promotion_effect": "A positive readiness observation identifies the next explicit review event; a negative one keeps the candidate as follow-up."
        }));
    }

    tasks
}
