use super::*;

pub(super) fn boundary_completion_candidates(
    space: &AdvisorySpaceEnvelope,
    obstruction: &Value,
    invariant_ids: &[String],
) -> AdvisoryResult<Vec<Value>> {
    let from_id = obstruction
        .pointer("/metadata/from_cell_id")
        .and_then(Value::as_str)
        .or_else(|| witness_cell_id(space, obstruction, |cell| cell["cell_type"] != "data_store"));
    let to_id = obstruction
        .pointer("/metadata/to_cell_id")
        .and_then(Value::as_str)
        .or_else(|| witness_cell_id(space, obstruction, |cell| cell["cell_type"] == "data_store"));
    let Some(from_cell) = find_cell(space, from_id) else {
        return Ok(Vec::new());
    };
    let Some(to_cell) = find_cell(space, to_id) else {
        return Ok(Vec::new());
    };
    let obstruction_id = json_id(obstruction).to_string();
    let from_title = title(from_cell);
    let to_title = title(to_cell);
    let domain_title = data_store_domain_title(to_title);
    let domain_id = id_suffix(json_id(to_cell))
        .trim_end_matches("-db")
        .to_string();
    let source_ids = completion_source_ids(space, obstruction);
    let evidence_strength = if source_ids.is_empty() {
        "rule_derived_without_source_ids"
    } else {
        "source_backed_obstruction"
    };
    let stem = obstruction_id.trim_start_matches("obstruction:");
    let h_implicit_interface = format!("hypothesis:{stem}-implicit-interface");
    Ok(vec![
        completion_candidate(CandidateSpec {
            space,
            id: format!("candidate:{domain_id}-status-api"),
            candidate_type: "proposed_interface",
            title: format!("Add {domain_title} status query API"),
            rationale: format!(
                "Remove cross-context direct database access while preserving {} status check.",
                domain_title.to_ascii_lowercase()
            ),
            resolves_obstruction_ids: vec![obstruction_id.clone()],
            proposed_cell_ids: vec![format!("cell:{domain_id}-status-api")],
            source_ids: source_ids.clone(),
            affected_invariant_ids: invariant_ids.to_vec(),
            witness_ids: obstruction_string_array(obstruction, "witness_ids"),
            blocked_ids: obstruction_string_array(obstruction, "blocked_ids"),
            proposed_incidence_ids: Vec::new(),
            confidence: 0.82,
            missing_type: MissingType::Cell,
            suggested_structure_type: "interface_cell",
            metadata: json!({
                "specificity": "source_derived",
                "evidence_strength": evidence_strength,
                "precision_note": "Derived from boundary violation witness cells and obstruction evidence_ids.",
                "derived_from_hypothesis_id": h_implicit_interface,
                "from_cell_id": json_id(from_cell),
                "to_cell_id": json_id(to_cell),
                "incidence_id": obstruction.pointer("/metadata/incidence_id")
            }),
        })?,
        completion_candidate(CandidateSpec {
            space,
            id: format!("candidate:replace-{}-db-read", id_suffix(json_id(from_cell))),
            candidate_type: "proposed_refactor_action",
            title: format!("Replace {from_title} direct DB read with {domain_title} API call"),
            rationale: format!(
                "{from_title} should depend on {domain_title} Service interface instead of {to_title} ownership boundary."
            ),
            resolves_obstruction_ids: vec![obstruction_id],
            proposed_cell_ids: vec![format!("cell:action-replace-{}-direct-db-read", id_suffix(json_id(from_cell)))],
            source_ids,
            affected_invariant_ids: invariant_ids.to_vec(),
            witness_ids: obstruction_string_array(obstruction, "witness_ids"),
            blocked_ids: obstruction_string_array(obstruction, "blocked_ids"),
            proposed_incidence_ids: Vec::new(),
            confidence: 0.78,
            missing_type: MissingType::Cell,
            suggested_structure_type: "refactor_action_cell",
            metadata: json!({
                "specificity": "source_derived",
                "evidence_strength": evidence_strength,
                "precision_note": "Derived from boundary violation witness cells and obstruction evidence_ids.",
                "derived_from_hypothesis_id": h_implicit_interface,
                "from_cell_id": json_id(from_cell),
                "to_cell_id": json_id(to_cell),
                "incidence_id": obstruction.pointer("/metadata/incidence_id")
            }),
        })?,
    ])
}

pub(super) fn owner_completion_candidate(
    space: &AdvisorySpaceEnvelope,
    obstruction: &Value,
    invariant_ids: &[String],
) -> AdvisoryResult<Value> {
    let stem = json_id(obstruction).trim_start_matches("obstruction:");
    let h_unassigned = format!("hypothesis:{stem}-no-team-holds-action");
    let source_ids = completion_source_ids(space, obstruction);
    let blocked_cell = blocked_cell(space, obstruction);
    if let Some((blocked, owner)) = blocked_cell.and_then(|blocked| {
        best_related_cell(space, blocked, &["owner"]).map(|owner| (blocked, owner))
    }) {
        let blocked_suffix = id_suffix(json_id(blocked));
        let owner_suffix = id_suffix(json_id(owner));
        return completion_candidate(CandidateSpec {
            space,
            id: format!("candidate:{stem}-assign-{owner_suffix}"),
            candidate_type: "owner_assignment",
            title: format!("Assign {} as owner for {}", title(owner), title(blocked)),
            rationale: format!(
                "{} shares source or context with the unowned action {} and can be reviewed as the explicit owner.",
                title(owner),
                title(blocked)
            ),
            resolves_obstruction_ids: vec![json_id(obstruction).to_string()],
            proposed_cell_ids: Vec::new(),
            proposed_incidence_ids: vec![format!(
                "incidence:{owner_suffix}-owns-{blocked_suffix}"
            )],
            source_ids,
            affected_invariant_ids: invariant_ids.to_vec(),
            witness_ids: obstruction_string_array(obstruction, "witness_ids"),
            blocked_ids: obstruction_string_array(obstruction, "blocked_ids"),
            confidence: related_cell_confidence(blocked, owner, 0.76),
            missing_type: MissingType::Incidence,
            suggested_structure_type: "ownership_incidence",
            metadata: json!({
                "specificity": "source_derived",
                "evidence_strength": "related_owner_cell",
                "precision_note": "Derived by matching an existing owner cell to the blocked action through shared context or source IDs.",
                "derived_from_hypothesis_id": format!("hypothesis:{stem}-de-facto-owner-link-missing"),
                "owner_cell_id": json_id(owner),
                "blocked_cell_id": json_id(blocked)
            }),
        });
    }
    completion_candidate(CandidateSpec {
        space,
        id: format!("candidate:{stem}-owner"),
        candidate_type: "ownership_clarification",
        title: "Clarify action owner".to_string(),
        rationale: obstruction
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Action requires owner.")
            .to_string(),
        resolves_obstruction_ids: vec![json_id(obstruction).to_string()],
        proposed_cell_ids: Vec::new(),
        proposed_incidence_ids: Vec::new(),
        source_ids,
        affected_invariant_ids: invariant_ids.to_vec(),
        witness_ids: obstruction_string_array(obstruction, "witness_ids"),
        blocked_ids: obstruction_string_array(obstruction, "blocked_ids"),
        confidence: 0.7,
        missing_type: MissingType::Cell,
        suggested_structure_type: "owner_cell",
        metadata: json!({
            "specificity": "generic",
            "evidence_strength": "obstruction_message",
            "precision_note": "Identifies the missing owner relation but does not infer a specific owner.",
            "derived_from_hypothesis_id": h_unassigned
        }),
    })
}

pub(super) fn verification_completion_candidate(
    space: &AdvisorySpaceEnvelope,
    obstruction: &Value,
    invariant_ids: &[String],
) -> AdvisoryResult<Value> {
    let stem = json_id(obstruction).trim_start_matches("obstruction:");
    let h_genuinely_missing = format!("hypothesis:{stem}-verification-genuinely-missing");
    let source_ids = completion_source_ids(space, obstruction);
    let blocked_cell = blocked_cell(space, obstruction);
    if let Some((blocked, verification)) = blocked_cell.and_then(|blocked| {
        best_related_cell(space, blocked, &["test_or_verification", "metric"])
            .map(|verification| (blocked, verification))
    }) {
        let blocked_suffix = id_suffix(json_id(blocked));
        let verification_suffix = id_suffix(json_id(verification));
        return completion_candidate(CandidateSpec {
            space,
            id: format!("candidate:{stem}-link-{verification_suffix}"),
            candidate_type: "lift_verification_link",
            title: format!("Link {} as verification for {}", title(verification), title(blocked)),
            rationale: format!(
                "{} appears related to the unverified requirement {} and can be reviewed as its verifies relation.",
                title(verification),
                title(blocked)
            ),
            resolves_obstruction_ids: vec![json_id(obstruction).to_string()],
            proposed_cell_ids: Vec::new(),
            proposed_incidence_ids: vec![format!(
                "incidence:{verification_suffix}-verifies-{blocked_suffix}"
            )],
            source_ids,
            affected_invariant_ids: invariant_ids.to_vec(),
            witness_ids: obstruction_string_array(obstruction, "witness_ids"),
            blocked_ids: obstruction_string_array(obstruction, "blocked_ids"),
            confidence: related_cell_confidence(blocked, verification, 0.78),
            missing_type: MissingType::Incidence,
            suggested_structure_type: "verification_incidence",
            metadata: json!({
                "specificity": "source_derived",
                "evidence_strength": "related_verification_cell",
                "precision_note": "Derived by matching an existing test, metric, or verification cell to the blocked requirement through shared context or source IDs.",
                "derived_from_hypothesis_id": format!("hypothesis:{stem}-verification-link-not-lifted"),
                "verification_cell_id": json_id(verification),
                "blocked_cell_id": json_id(blocked)
            }),
        });
    }
    completion_candidate(CandidateSpec {
        space,
        id: format!("candidate:{stem}-verification"),
        candidate_type: "proposed_test",
        title: "Define verification method".to_string(),
        rationale: obstruction
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Requirement needs verification.")
            .to_string(),
        resolves_obstruction_ids: vec![json_id(obstruction).to_string()],
        proposed_cell_ids: Vec::new(),
        proposed_incidence_ids: Vec::new(),
        source_ids,
        affected_invariant_ids: invariant_ids.to_vec(),
        witness_ids: obstruction_string_array(obstruction, "witness_ids"),
        blocked_ids: obstruction_string_array(obstruction, "blocked_ids"),
        confidence: 0.7,
        missing_type: MissingType::Cell,
        suggested_structure_type: "verification_cell",
        metadata: json!({
            "specificity": "requirement_derived",
            "evidence_strength": "obstruction_message",
            "precision_note": "Identifies the verification gap but does not infer a concrete test implementation.",
            "derived_from_hypothesis_id": h_genuinely_missing
        }),
    })
}

pub(super) fn auth_guard_completion_candidate(
    space: &AdvisorySpaceEnvelope,
    obstruction: &Value,
    invariant_ids: &[String],
) -> AdvisoryResult<Value> {
    let route_path = obstruction
        .pointer("/metadata/route_path")
        .and_then(Value::as_str)
        .unwrap_or("API route");
    let source_ids = completion_source_ids(space, obstruction);
    let evidence_strength = if source_ids.is_empty() {
        "rule_derived_without_source_ids"
    } else {
        "source_backed_obstruction"
    };
    let stem = json_id(obstruction).trim_start_matches("obstruction:");
    let h_unprotected = format!("hypothesis:{stem}-truly-unprotected");
    completion_candidate(CandidateSpec {
        space,
        id: format!("candidate:{stem}-auth-guard"),
        candidate_type: "proposed_auth_guard",
        title: format!("Add authentication guard to {route_path}"),
        rationale: obstruction
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Database-touching API route requires authentication.")
            .to_string(),
        resolves_obstruction_ids: vec![json_id(obstruction).to_string()],
        proposed_cell_ids: Vec::new(),
        proposed_incidence_ids: Vec::new(),
        source_ids,
        affected_invariant_ids: invariant_ids.to_vec(),
        witness_ids: obstruction_string_array(obstruction, "witness_ids"),
        blocked_ids: obstruction_string_array(obstruction, "blocked_ids"),
        confidence: 0.72,
        missing_type: MissingType::Cell,
        suggested_structure_type: "auth_guard_cell",
        metadata: json!({
            "specificity": "code_derived",
            "evidence_strength": evidence_strength,
            "precision_note": "Derived from code snapshot route metadata. The candidate must be reviewed because lexical detection can miss shared middleware or dynamic auth wrappers.",
            "derived_from_hypothesis_id": h_unprotected,
            "route_path": route_path,
            "http_methods": obstruction.pointer("/metadata/http_methods").cloned().unwrap_or_else(|| json!([]))
        }),
    })
}
