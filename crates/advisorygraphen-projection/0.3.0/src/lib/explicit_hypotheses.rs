use super::*;

pub(super) fn explicit_hypothesis_matrix(space: &AdvisorySpaceEnvelope) -> Value {
    let hypotheses = space
        .cells
        .iter()
        .filter(|cell| is_explicit_hypothesis(cell))
        .map(|hypothesis| {
            let id = hypothesis.get("id").and_then(Value::as_str).unwrap_or("cell:unknown");
            json!({
                "hypothesis_id": id,
                "title": hypothesis.get("title").cloned().unwrap_or(Value::Null),
                "status": explicit_hypothesis_status(hypothesis),
                "refinement_parent_ids": refinement_parent_ids_for(space, id),
                "refinement_child_ids": refinement_child_ids_for(space, id),
                "refinement_depth": refinement_depth_for(space, id),
                "refinement_status": refinement_status_for(space, id, hypothesis),
                "expected_observations": hypothesis.pointer("/metadata/expected_observations").cloned().unwrap_or_else(|| json!([])),
                "falsifiers": hypothesis.pointer("/metadata/falsifiers").cloned().unwrap_or_else(|| json!([])),
                "supporting_incidence_ids": relation_ids_for(space, id, &["supports", "supported_by"]),
                "falsifying_incidence_ids": relation_ids_for(space, id, &["falsifies", "falsified_by"]),
                "competing_hypothesis_ids": competing_ids_for(space, id),
                "remaining_uncertainty": remaining_hypothesis_uncertainty(space, hypothesis)
            })
        })
        .collect::<Vec<_>>();
    let mut counts = serde_json::Map::new();
    for hypothesis in &hypotheses {
        let status = hypothesis
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let current = counts.get(status).and_then(Value::as_u64).unwrap_or(0);
        counts.insert(status.to_string(), json!(current + 1));
    }
    json!({
        "count": hypotheses.len(),
        "status_counts": counts,
        "hypotheses": hypotheses,
        "rule": "Hypotheses should carry expected observations, falsifiers, support/falsify incidences, competing alternatives, and refinement lineage before driving proposals."
    })
}

pub(super) fn merged_hypotheses(
    mut report_hypotheses: Vec<Value>,
    explicit_matrix: &Value,
) -> Vec<Value> {
    let mut known_ids = report_hypotheses
        .iter()
        .filter_map(|hypothesis| hypothesis.get("id").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    known_ids.extend(
        report_hypotheses
            .iter()
            .filter_map(|hypothesis| hypothesis.get("hypothesis_id").and_then(Value::as_str))
            .map(str::to_string),
    );

    let explicit = explicit_matrix
        .get("hypotheses")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for hypothesis in explicit {
        let Some(id) = hypothesis.get("hypothesis_id").and_then(Value::as_str) else {
            continue;
        };
        if known_ids.iter().any(|known| known == id) {
            continue;
        }
        known_ids.push(id.to_string());
        report_hypotheses.push(json!({
            "id": id,
            "hypothesis_id": id,
            "title": hypothesis.get("title").cloned().unwrap_or(Value::Null),
            "lifecycle_status": hypothesis.get("status").cloned().unwrap_or_else(|| json!("candidate")),
            "status": hypothesis.get("status").cloned().unwrap_or_else(|| json!("candidate")),
            "source": "explicit_advisory_space",
            "expected_observations": hypothesis.get("expected_observations").cloned().unwrap_or_else(|| json!([])),
            "falsifiers": hypothesis.get("falsifiers").cloned().unwrap_or_else(|| json!([])),
            "supporting_incidence_ids": hypothesis.get("supporting_incidence_ids").cloned().unwrap_or_else(|| json!([])),
            "falsifying_incidence_ids": hypothesis.get("falsifying_incidence_ids").cloned().unwrap_or_else(|| json!([])),
            "competing_hypothesis_ids": hypothesis.get("competing_hypothesis_ids").cloned().unwrap_or_else(|| json!([])),
            "refinement_parent_ids": hypothesis.get("refinement_parent_ids").cloned().unwrap_or_else(|| json!([])),
            "refinement_child_ids": hypothesis.get("refinement_child_ids").cloned().unwrap_or_else(|| json!([])),
            "refinement_depth": hypothesis.get("refinement_depth").cloned().unwrap_or_else(|| json!(0)),
            "refinement_status": hypothesis.get("refinement_status").cloned().unwrap_or_else(|| json!("seed")),
            "remaining_uncertainty": hypothesis.get("remaining_uncertainty").cloned().unwrap_or_else(|| json!([]))
        }));
    }
    report_hypotheses
}

pub(super) fn explicit_proposal_trace(space: &AdvisorySpaceEnvelope) -> Value {
    let proposals = space
        .cells
        .iter()
        .filter(|cell| cell["cell_type"] == "action")
        .map(|action| {
            let action_id = action.get("id").and_then(Value::as_str).unwrap_or("cell:unknown");
            let derived = explicit_derived_hypothesis_ids(space, action_id, action);
            json!({
                "action_id": action_id,
                "title": action.get("title").cloned().unwrap_or(Value::Null),
                "priority": action.pointer("/metadata/priority").cloned().unwrap_or(Value::Null),
                "derived_hypothesis_ids": derived,
                "derived_hypothesis_statuses": derived.iter().map(|id| {
                    json!({
                        "hypothesis_id": id,
                        "status": space.cells.iter()
                            .find(|cell| cell.get("id").and_then(Value::as_str) == Some(id.as_str()))
                            .map(explicit_hypothesis_status)
                            .unwrap_or("missing")
                    })
                }).collect::<Vec<_>>(),
                "required_verification": action.pointer("/metadata/required_verification").cloned().unwrap_or(Value::Null),
                "owner_state": if relation_ids_for(space, action_id, &["owns"]).is_empty() { "missing" } else { "present" },
                "proposal_quality_notes": proposal_quality_notes(space, action)
            })
        })
        .collect::<Vec<_>>();
    json!({
        "count": proposals.len(),
        "proposals": proposals,
        "rule": "Proposal trace is problem -> hypothesis -> evidence -> classification -> proposal -> required verification/owner."
    })
}

pub(super) fn is_explicit_hypothesis(cell: &Value) -> bool {
    cell["cell_type"] == "hypothesis"
        || cell
            .pointer("/metadata/hypothesis")
            .and_then(Value::as_bool)
            == Some(true)
        || cell.pointer("/metadata/hypothesis_status").is_some()
        || cell.get("lifecycle_status").is_some()
}

pub(super) fn explicit_hypothesis_status(cell: &Value) -> &str {
    cell.pointer("/metadata/hypothesis_status")
        .and_then(Value::as_str)
        .or_else(|| cell.get("lifecycle_status").and_then(Value::as_str))
        .unwrap_or("candidate")
}

pub(super) fn relation_ids_for(
    space: &AdvisorySpaceEnvelope,
    target_id: &str,
    relation_types: &[&str],
) -> Vec<Value> {
    space
        .incidences
        .iter()
        .filter(|incidence| {
            let relation_type = incidence
                .get("relation_type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            relation_types.contains(&relation_type)
                && (incidence.get("from_id").and_then(Value::as_str) == Some(target_id)
                    || incidence.get("to_id").and_then(Value::as_str) == Some(target_id))
        })
        .filter_map(|incidence| incidence.get("id").cloned())
        .collect()
}

pub(super) fn competing_ids_for(space: &AdvisorySpaceEnvelope, hypothesis_id: &str) -> Vec<Value> {
    space
        .incidences
        .iter()
        .filter(|incidence| {
            incidence.get("relation_type").and_then(Value::as_str) == Some("competes_with")
        })
        .filter_map(|incidence| {
            let from = incidence.get("from_id").and_then(Value::as_str)?;
            let to = incidence.get("to_id").and_then(Value::as_str)?;
            if from == hypothesis_id {
                Some(json!(to))
            } else if to == hypothesis_id {
                Some(json!(from))
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn refinement_parent_ids_for(
    space: &AdvisorySpaceEnvelope,
    hypothesis_id: &str,
) -> Vec<Value> {
    refinement_related_ids(space, hypothesis_id, RefinementDirection::Parent)
}

pub(super) fn refinement_child_ids_for(
    space: &AdvisorySpaceEnvelope,
    hypothesis_id: &str,
) -> Vec<Value> {
    refinement_related_ids(space, hypothesis_id, RefinementDirection::Child)
}

pub(super) enum RefinementDirection {
    Parent,
    Child,
}

pub(super) fn refinement_related_ids(
    space: &AdvisorySpaceEnvelope,
    hypothesis_id: &str,
    direction: RefinementDirection,
) -> Vec<Value> {
    let mut ids = space
        .incidences
        .iter()
        .filter(|incidence| is_refinement_relation(incidence))
        .filter_map(|incidence| {
            let from = incidence.get("from_id").and_then(Value::as_str)?;
            let to = incidence.get("to_id").and_then(Value::as_str)?;
            match direction {
                RefinementDirection::Parent if from == hypothesis_id => Some(json!(to)),
                RefinementDirection::Child if to == hypothesis_id => Some(json!(from)),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    ids.sort_by_key(|id| id.as_str().unwrap_or_default().to_string());
    ids.dedup();
    ids
}

pub(super) fn refinement_depth_for(space: &AdvisorySpaceEnvelope, hypothesis_id: &str) -> u64 {
    let mut depth = 0_u64;
    let mut current = hypothesis_id.to_string();
    let mut seen = vec![current.clone()];
    while let Some(parent) = refinement_parent_ids_for(space, &current)
        .into_iter()
        .find_map(|id| id.as_str().map(str::to_string))
    {
        if seen.contains(&parent) {
            break;
        }
        depth += 1;
        current = parent.clone();
        seen.push(parent);
    }
    depth
}

pub(super) fn refinement_status_for(
    space: &AdvisorySpaceEnvelope,
    hypothesis_id: &str,
    hypothesis: &Value,
) -> &'static str {
    if hypothesis
        .pointer("/metadata/hypothesis_refinement")
        .and_then(Value::as_bool)
        == Some(true)
        || refinement_depth_for(space, hypothesis_id) > 0
    {
        "refined"
    } else if !refinement_child_ids_for(space, hypothesis_id).is_empty() {
        "has_refinements"
    } else {
        "seed"
    }
}

pub(super) fn is_refinement_relation(incidence: &Value) -> bool {
    matches!(
        incidence.get("relation_type").and_then(Value::as_str),
        Some("refines" | "refined_from" | "revises" | "revised_from")
    )
}

pub(super) fn remaining_hypothesis_uncertainty(
    space: &AdvisorySpaceEnvelope,
    hypothesis: &Value,
) -> Vec<Value> {
    let mut items = Vec::new();
    let id = hypothesis
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("cell:unknown");
    if hypothesis
        .pointer("/metadata/expected_observations")
        .and_then(Value::as_array)
        .is_none_or(Vec::is_empty)
    {
        items.push(json!("missing_expected_observations"));
    }
    if hypothesis
        .pointer("/metadata/falsifiers")
        .and_then(Value::as_array)
        .is_none_or(Vec::is_empty)
    {
        items.push(json!("missing_falsifiers"));
    }
    let status = explicit_hypothesis_status(hypothesis);
    if matches!(
        status,
        "supported" | "strongly_supported" | "supported_needs_followup"
    ) && relation_ids_for(space, id, &["supports", "supported_by"]).is_empty()
    {
        items.push(json!("missing_support_incidence"));
    }
    if status == "falsified"
        && relation_ids_for(space, id, &["falsifies", "falsified_by"]).is_empty()
    {
        items.push(json!("missing_falsifying_incidence"));
    }
    if hypothesis
        .pointer("/metadata/refinement_required")
        .and_then(Value::as_bool)
        == Some(true)
        && refinement_parent_ids_for(space, id).is_empty()
        && refinement_child_ids_for(space, id).is_empty()
    {
        items.push(json!("missing_refinement_lineage"));
    }
    items
}

pub(super) fn explicit_derived_hypothesis_ids(
    space: &AdvisorySpaceEnvelope,
    action_id: &str,
    action: &Value,
) -> Vec<String> {
    let mut ids = Vec::new();
    if let Some(id) = action
        .pointer("/metadata/derived_from_hypothesis")
        .and_then(Value::as_str)
    {
        ids.push(normalize_projection_cell_id(id));
    }
    if let Some(id) = action
        .pointer("/metadata/derived_from_hypothesis_id")
        .and_then(Value::as_str)
    {
        ids.push(normalize_projection_cell_id(id));
    }
    if let Some(values) = action
        .pointer("/metadata/derived_from_hypotheses")
        .and_then(Value::as_array)
    {
        ids.extend(
            values
                .iter()
                .filter_map(Value::as_str)
                .map(normalize_projection_cell_id),
        );
    }
    ids.extend(
        space
            .incidences
            .iter()
            .filter(|incidence| {
                incidence.get("relation_type").and_then(Value::as_str) == Some("derives_from")
                    && incidence.get("from_id").and_then(Value::as_str) == Some(action_id)
            })
            .filter_map(|incidence| incidence.get("to_id").and_then(Value::as_str))
            .map(normalize_projection_cell_id),
    );
    ids.sort();
    ids.dedup();
    ids
}

pub(super) fn normalize_projection_cell_id(id: &str) -> String {
    if id.starts_with("record:") {
        format!("cell:{}", id.trim_start_matches("record:"))
    } else {
        id.to_string()
    }
}

pub(super) fn proposal_quality_notes(space: &AdvisorySpaceEnvelope, action: &Value) -> Vec<Value> {
    let action_id = action
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("cell:unknown");
    let mut notes = Vec::new();
    let derived = explicit_derived_hypothesis_ids(space, action_id, action);
    if derived.is_empty() {
        notes.push(json!("missing_hypothesis_trace"));
    }
    if action
        .pointer("/metadata/required_verification")
        .and_then(Value::as_str)
        .is_none_or(|value| value.trim().is_empty())
    {
        notes.push(json!("missing_required_verification"));
    }
    if relation_ids_for(space, action_id, &["owns"]).is_empty() {
        notes.push(json!("missing_owner"));
    }
    notes
}
