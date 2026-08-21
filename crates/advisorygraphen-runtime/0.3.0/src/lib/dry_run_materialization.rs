use super::*;

pub(super) fn materialize_candidate_dry_run(
    space: &AdvisorySpaceEnvelope,
    blockers: &[Value],
    candidate: &Value,
) -> DryRunMaterialization {
    let candidate_type = candidate
        .get("candidate_type")
        .and_then(Value::as_str)
        .unwrap_or("");
    match candidate_type {
        "owner_assignment" => relation_candidate_dry_run(space, candidate, "owner_cell_id", "owns"),
        "lift_verification_link" => {
            relation_candidate_dry_run(space, candidate, "verification_cell_id", "verifies")
        }
        "ownership_clarification" | "proposed_test" => {
            let Some(blocker) = candidate_blocker(blockers, candidate) else {
                return DryRunMaterialization::Skipped {
                    reason: "candidate does not resolve a known obstruction".to_string(),
                };
            };
            let mut reviewed = candidate.clone();
            reviewed["review_status"] = json!("accepted");
            match materialize_candidate_structure(space, blocker, &reviewed, "dry-run") {
                Materialization::Applied { cells, incidences } => DryRunMaterialization::Applied {
                    cells,
                    incidences,
                    removed_incidence_ids: Vec::new(),
                },
                Materialization::Skipped { reason } => DryRunMaterialization::Skipped { reason },
            }
        }
        "proposed_interface" => interface_candidate_dry_run(space, candidate),
        "proposed_refactor_action" => refactor_candidate_dry_run(space, candidate),
        other => DryRunMaterialization::Skipped {
            reason: format!("candidate_type {other} is not supported for dry-run application"),
        },
    }
}

pub(super) fn relation_candidate_dry_run(
    space: &AdvisorySpaceEnvelope,
    candidate: &Value,
    from_metadata_key: &str,
    relation_type: &str,
) -> DryRunMaterialization {
    let Some(from_id) = candidate
        .pointer(&format!("/metadata/{from_metadata_key}"))
        .and_then(Value::as_str)
    else {
        return DryRunMaterialization::Skipped {
            reason: format!("metadata.{from_metadata_key} is missing"),
        };
    };
    let Some(to_id) = candidate
        .pointer("/metadata/blocked_cell_id")
        .and_then(Value::as_str)
    else {
        return DryRunMaterialization::Skipped {
            reason: "metadata.blocked_cell_id is missing".to_string(),
        };
    };
    if !space.cells.iter().any(|cell| json_id(cell) == from_id)
        || !space.cells.iter().any(|cell| json_id(cell) == to_id)
    {
        return DryRunMaterialization::Skipped {
            reason: "proposed relation endpoint is not present in the space".to_string(),
        };
    }
    let incidence_id = candidate
        .get("proposed_incidence_ids")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .next()
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "incidence:{}-{relation_type}-{}",
                id_suffix(from_id),
                id_suffix(to_id)
            )
        });
    DryRunMaterialization::Applied {
        cells: Vec::new(),
        incidences: vec![json!({
            "id": incidence_id,
            "relation_type": relation_type,
            "from_id": from_id,
            "to_id": to_id,
            "context_ids": [],
            "evidence_ids": evidence_cell_ids_for_candidate(space, candidate),
            "strength": "soft",
            "provenance": dry_run_provenance(),
            "metadata": {
                "materialized_from_candidate_id": json_id(candidate),
                "materialization_kind": "completion_dry_run"
            }
        })],
        removed_incidence_ids: Vec::new(),
    }
}

pub(super) fn interface_candidate_dry_run(
    space: &AdvisorySpaceEnvelope,
    candidate: &Value,
) -> DryRunMaterialization {
    let Some(interface_id) = candidate
        .get("proposed_cell_ids")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .next()
    else {
        return DryRunMaterialization::Skipped {
            reason: "proposed interface candidate has no proposed_cell_ids".to_string(),
        };
    };
    let Some(from_id) = candidate
        .pointer("/metadata/from_cell_id")
        .and_then(Value::as_str)
    else {
        return DryRunMaterialization::Skipped {
            reason: "metadata.from_cell_id is missing".to_string(),
        };
    };
    let removed_incidence_ids = candidate
        .pointer("/metadata/incidence_id")
        .and_then(Value::as_str)
        .map(|id| vec![id.to_string()])
        .unwrap_or_default();
    let context_ids = space
        .cells
        .iter()
        .find(|cell| json_id(cell) == from_id)
        .and_then(|cell| cell.get("context_ids").and_then(Value::as_array).cloned())
        .unwrap_or_default();
    let title = candidate
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Proposed interface");
    DryRunMaterialization::Applied {
        cells: vec![json!({
            "id": interface_id,
            "cell_type": "interface",
            "title": title,
            "summary": candidate.get("rationale").and_then(Value::as_str),
            "context_ids": context_ids,
            "source_ids": candidate.get("source_ids").cloned().unwrap_or_else(|| json!([])),
            "structure_refs": [],
            "provenance": dry_run_provenance(),
            "metadata": {
                "materialized_from_candidate_id": json_id(candidate),
                "materialization_kind": "completion_dry_run"
            }
        })],
        incidences: vec![json!({
            "id": format!("incidence:{}-uses-{}", id_suffix(from_id), id_suffix(interface_id)),
            "relation_type": "uses",
            "from_id": from_id,
            "to_id": interface_id,
            "context_ids": [],
            "evidence_ids": evidence_cell_ids_for_candidate(space, candidate),
            "strength": "soft",
            "provenance": dry_run_provenance(),
            "metadata": {
                "materialized_from_candidate_id": json_id(candidate),
                "materialization_kind": "completion_dry_run"
            }
        })],
        removed_incidence_ids,
    }
}

pub(super) fn refactor_candidate_dry_run(
    space: &AdvisorySpaceEnvelope,
    candidate: &Value,
) -> DryRunMaterialization {
    let Some(action_id) = candidate
        .get("proposed_cell_ids")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .next()
    else {
        return DryRunMaterialization::Skipped {
            reason: "refactor candidate has no proposed_cell_ids".to_string(),
        };
    };
    let removed_incidence_ids = candidate
        .pointer("/metadata/incidence_id")
        .and_then(Value::as_str)
        .map(|id| vec![id.to_string()])
        .unwrap_or_default();
    let context_ids = candidate
        .pointer("/metadata/from_cell_id")
        .and_then(Value::as_str)
        .and_then(|from_id| {
            space
                .cells
                .iter()
                .find(|cell| json_id(cell) == from_id)
                .and_then(|cell| cell.get("context_ids").and_then(Value::as_array).cloned())
        })
        .unwrap_or_default();
    DryRunMaterialization::Applied {
        cells: vec![json!({
            "id": action_id,
            "cell_type": "action",
            "title": candidate.get("title").and_then(Value::as_str).unwrap_or("Proposed refactor action"),
            "summary": candidate.get("rationale").and_then(Value::as_str),
            "context_ids": context_ids,
            "source_ids": candidate.get("source_ids").cloned().unwrap_or_else(|| json!([])),
            "structure_refs": [],
            "provenance": dry_run_provenance(),
            "metadata": {
                "materialized_from_candidate_id": json_id(candidate),
                "materialization_kind": "completion_dry_run"
            }
        })],
        incidences: Vec::new(),
        removed_incidence_ids,
    }
}

pub(super) fn candidate_blocker<'a>(blockers: &'a [Value], candidate: &Value) -> Option<&'a Value> {
    let resolved_ids = candidate
        .get("resolves_obstruction_ids")
        .and_then(Value::as_array)?;
    blockers.iter().find(|blocker| {
        let blocker_id = json_id(blocker);
        resolved_ids
            .iter()
            .any(|id| id.as_str() == Some(blocker_id))
    })
}

pub(super) fn dry_run_provenance() -> Value {
    json!({
        "origin": "inferred",
        "actor": "advisorygraphen:completion-dry-run",
        "confidence": 0.6,
        "review_status": "unreviewed"
    })
}

pub(super) fn evidence_cell_ids_for_candidate(
    space: &AdvisorySpaceEnvelope,
    candidate: &Value,
) -> Vec<Value> {
    candidate
        .get("source_ids")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter_map(|source_id| {
            let evidence_id = format!("cell:evidence-{}", source_id.trim_start_matches("source:"));
            space
                .cells
                .iter()
                .any(|cell| json_id(cell) == evidence_id)
                .then_some(json!(evidence_id))
        })
        .collect()
}

pub(super) fn obstruction_ids(report: &ReportEnvelope) -> Vec<String> {
    let mut ids = report
        .result
        .get("obstructions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|obstruction| obstruction.get("id").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    ids.sort();
    ids
}
