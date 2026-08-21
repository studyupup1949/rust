use super::*;

pub(super) fn completion_candidate(spec: CandidateSpec<'_>) -> AdvisoryResult<Value> {
    let proposal_content = proposal_content(&spec.id, &spec.rationale, &spec);
    let application_plan = application_plan(&spec);
    let id = spec.id;
    let rationale = spec.rationale;
    let related_ids = spec
        .resolves_obstruction_ids
        .iter()
        .chain(spec.proposed_cell_ids.iter())
        .chain(spec.proposed_incidence_ids.iter())
        .map(|id| hg_id(id))
        .collect::<AdvisoryResult<Vec<_>>>()?;
    let suggested_structure = SuggestedStructure::new(spec.suggested_structure_type, &spec.title)
        .map_err(hg_err)?
        .with_related_ids(related_ids);
    let suggested_structure = match spec.proposed_cell_ids.first() {
        Some(cell_id) => suggested_structure.with_structure_id(hg_id(cell_id)?),
        None => suggested_structure,
    };
    let higher_candidate = CompletionCandidate::new(
        hg_id(&id)?,
        hg_id(&spec.space.space_id)?,
        spec.missing_type,
        suggested_structure,
        spec.resolves_obstruction_ids
            .iter()
            .map(|id| hg_id(id))
            .collect::<AdvisoryResult<Vec<_>>>()?,
        rationale.clone(),
        Confidence::new(spec.confidence).map_err(hg_err)?,
    )
    .map_err(hg_err)?;

    Ok(json!({
        "id": id,
        "candidate_type": spec.candidate_type,
        "title": spec.title,
        "rationale": rationale,
        "resolves_obstruction_ids": spec.resolves_obstruction_ids,
        "proposed_cell_ids": spec.proposed_cell_ids,
        "proposed_incidence_ids": spec.proposed_incidence_ids,
        "source_ids": spec.source_ids,
        "confidence": spec.confidence,
        "review_status": "unreviewed",
        "application_plan": application_plan,
        "proposal_content": proposal_content,
        "metadata": spec.metadata,
        "higher_graphen": higher_candidate
    }))
}

pub(super) fn application_plan(spec: &CandidateSpec<'_>) -> Value {
    let candidate_id = &spec.id;
    let mut operations = Vec::new();
    for cell_id in &spec.proposed_cell_ids {
        operations.push(json!({
            "operation": "upsert_cell",
            "cell_id": cell_id,
            "review_status": "unreviewed"
        }));
    }
    for incidence_id in &spec.proposed_incidence_ids {
        let relation_type = match spec.candidate_type {
            "owner_assignment" => "owns",
            "lift_verification_link" => "verifies",
            _ => "related",
        };
        operations.push(json!({
            "operation": "upsert_incidence",
            "incidence_id": incidence_id,
            "relation_type": relation_type,
            "review_status": "unreviewed"
        }));
    }
    if matches!(
        spec.candidate_type,
        "proposed_interface" | "proposed_refactor_action"
    ) {
        if let Some(incidence_id) = spec
            .metadata
            .pointer("/incidence_id")
            .and_then(Value::as_str)
        {
            operations.push(json!({
                "operation": "remove_incidence",
                "incidence_id": incidence_id,
                "reason": "replace direct access with proposed boundary-safe structure",
                "review_status": "unreviewed"
            }));
        }
    }

    json!({
        "schema": "advisorygraphen.completion.application_plan.v1",
        "candidate_id": candidate_id,
        "candidate_type": spec.candidate_type,
        "review_status": "unreviewed",
        "dry_run_supported": !operations.is_empty(),
        "operations": operations,
        "expected_effects": {
            "repairs_obstruction_ids": spec.resolves_obstruction_ids,
            "affected_invariant_ids": spec.affected_invariant_ids
        },
        "safety": {
            "requires_explicit_review_for_persistent_application": true,
            "dry_run_does_not_accept_candidate": true
        }
    })
}
