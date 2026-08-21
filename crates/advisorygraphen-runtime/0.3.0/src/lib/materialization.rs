use super::*;

pub(super) fn materialize_candidate_structure(
    space: &AdvisorySpaceEnvelope,
    blocker: &Value,
    candidate: &Value,
    reviewer: &str,
) -> Materialization {
    if candidate.get("review_status").and_then(Value::as_str) != Some("accepted") {
        return Materialization::Skipped {
            reason: "candidate is not accepted".to_string(),
        };
    }
    let Some(blocked_id) = blocker
        .get("blocked_ids")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .find(|id| space.cells.iter().any(|cell| json_id(cell) == *id))
    else {
        return Materialization::Skipped {
            reason: "blocker has no materializable blocked cell".to_string(),
        };
    };
    let Some(blocked_cell) = space.cells.iter().find(|cell| json_id(cell) == blocked_id) else {
        return Materialization::Skipped {
            reason: "blocked cell not found in materialized space".to_string(),
        };
    };
    let candidate_id = json_id(candidate);
    let blocked_slug = id_suffix(blocked_id);
    let provenance = reviewed_materialization_provenance(reviewer);
    let source_ids = materialization_source_ids(candidate, blocker);
    let context_ids = blocked_cell
        .get("context_ids")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    match candidate.get("candidate_type").and_then(Value::as_str) {
        Some("ownership_clarification") => {
            let owner_id = format!("cell:auto-owner-{blocked_slug}");
            let incidence_id = format!("incidence:auto-owner-{blocked_slug}-owns-{blocked_slug}");
            Materialization::Applied {
                cells: vec![json!({
                    "id": owner_id,
                    "cell_type": "owner",
                    "title": format!("Owner for {}", title(blocked_cell)),
                    "summary": format!(
                        "Placeholder owner materialized from accepted completion candidate {candidate_id}."
                    ),
                    "context_ids": context_ids,
                    "source_ids": source_ids,
                    "structure_refs": [],
                    "provenance": provenance.clone(),
                    "metadata": {
                        "materialized_from_candidate_id": candidate_id,
                        "materialization_kind": "accepted_completion",
                        "placeholder": true,
                        "requires_human_named_owner": true
                    }
                })],
                incidences: vec![json!({
                    "id": incidence_id,
                    "relation_type": "owns",
                    "from_id": owner_id,
                    "to_id": blocked_id,
                    "context_ids": [],
                    "evidence_ids": [],
                    "strength": "soft",
                    "provenance": provenance,
                    "metadata": {
                        "materialized_from_candidate_id": candidate_id,
                        "materialization_kind": "accepted_completion"
                    }
                })],
            }
        }
        Some("proposed_test") => {
            let verification_id = format!("cell:auto-verification-{blocked_slug}");
            let incidence_id =
                format!("incidence:auto-verification-{blocked_slug}-verifies-{blocked_slug}");
            Materialization::Applied {
                cells: vec![json!({
                    "id": verification_id,
                    "cell_type": "test_or_verification",
                    "title": format!("Verification for {}", title(blocked_cell)),
                    "summary": candidate
                        .get("rationale")
                        .and_then(Value::as_str)
                        .unwrap_or("Verification method materialized from an accepted completion candidate."),
                    "context_ids": context_ids,
                    "source_ids": source_ids,
                    "structure_refs": [],
                    "provenance": provenance.clone(),
                    "metadata": {
                        "materialized_from_candidate_id": candidate_id,
                        "materialization_kind": "accepted_completion",
                        "placeholder": true,
                        "requires_concrete_test_details": true
                    }
                })],
                incidences: vec![json!({
                    "id": incidence_id,
                    "relation_type": "verifies",
                    "from_id": verification_id,
                    "to_id": blocked_id,
                    "context_ids": [],
                    "evidence_ids": [],
                    "strength": "soft",
                    "provenance": provenance,
                    "metadata": {
                        "materialized_from_candidate_id": candidate_id,
                        "materialization_kind": "accepted_completion"
                    }
                })],
            }
        }
        Some(other) => Materialization::Skipped {
            reason: format!("candidate_type {other} is not supported for automatic application"),
        },
        None => Materialization::Skipped {
            reason: "candidate_type is missing".to_string(),
        },
    }
}

pub(super) fn reviewed_materialization_provenance(reviewer: &str) -> Value {
    json!({
        "origin": "reviewed",
        "actor": reviewer,
        "confidence": 0.7,
        "review_status": "accepted"
    })
}

pub(super) fn materialization_source_ids(candidate: &Value, blocker: &Value) -> Vec<Value> {
    let mut ids = candidate
        .get("source_ids")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if ids.is_empty() {
        ids = blocker
            .get("evidence_ids")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
    }
    ids.sort_by(|left, right| {
        left.as_str()
            .unwrap_or("")
            .cmp(right.as_str().unwrap_or(""))
    });
    ids.dedup();
    ids
}

pub(super) fn upsert_by_id(items: &mut Vec<Value>, value: Value) {
    let id = json_id(&value);
    if let Some(existing) = items.iter_mut().find(|item| json_id(item) == id) {
        *existing = value;
    } else {
        items.push(value);
    }
}

pub(super) fn ids_of(items: &[Value]) -> Vec<String> {
    items.iter().map(json_id).map(str::to_string).collect()
}

pub(super) fn title(value: &Value) -> &str {
    value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_else(|| json_id(value))
}

pub(super) fn id_suffix(id: &str) -> String {
    let raw = id.split_once(':').map(|(_, suffix)| suffix).unwrap_or(id);
    advisorygraphen_core::slugify_id(raw)
}
