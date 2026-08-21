use super::*;

pub(super) fn evaluate_action_owners(
    space: &AdvisorySpaceEnvelope,
    higher_space: &HigherGraphenAdvisorySpace,
    invariant_results: &mut Vec<Value>,
    obstructions: &mut Vec<Value>,
) -> AdvisoryResult<()> {
    for action in space
        .cells
        .iter()
        .filter(|cell| cell["cell_type"] == "action")
    {
        if has_incoming_owner(higher_space, json_id(action)) {
            continue;
        }
        let obstruction_id = format!(
            "obstruction:{}-missing-owner",
            json_id(action).trim_start_matches("cell:")
        );
        let finding = violation_finding(FindingInput {
            space_id: &space.space_id,
            invariant_id: OWNER_INVARIANT,
            obstruction_id: &obstruction_id,
            obstruction_type: "missing_owner",
            severity: "medium",
            message: format!("{} has no owner.", title(action)),
            witness_ids: vec![json_id(action).to_string()],
            blocked_ids: vec![action["id"].clone()],
            evidence_ids: action
                .get("source_ids")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            recommended_completion_types: vec!["ownership_clarification"],
            resolution: "clarify the action owner",
            metadata: json!({
                "rule_precision": "action_cell_without_incoming_owns_relation",
                "evidence_strength": "cell_source_ids",
                "specificity": "generic"
            }),
        })?;
        invariant_results.push(finding.invariant_result);
        obstructions.push(finding.obstruction);
    }
    Ok(())
}

pub(super) fn evaluate_required_verification(
    space: &AdvisorySpaceEnvelope,
    higher_space: &HigherGraphenAdvisorySpace,
    invariant_results: &mut Vec<Value>,
    obstructions: &mut Vec<Value>,
) -> AdvisoryResult<()> {
    for requirement in space
        .cells
        .iter()
        .filter(|cell| cell["cell_type"] == "requirement")
    {
        if !requires_verification(requirement)
            || has_verification(higher_space, json_id(requirement))
        {
            continue;
        }
        let obstruction_id = format!(
            "obstruction:{}-missing-verification",
            json_id(requirement).trim_start_matches("cell:")
        );
        let finding = violation_finding(FindingInput {
            space_id: &space.space_id,
            invariant_id: REQUIREMENT_VERIFICATION_INVARIANT,
            obstruction_id: &obstruction_id,
            obstruction_type: "requirement_unverified",
            severity: "medium",
            message: format!("{} has no verification method.", title(requirement)),
            witness_ids: vec![json_id(requirement).to_string()],
            blocked_ids: vec![requirement["id"].clone()],
            evidence_ids: requirement
                .get("source_ids")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            recommended_completion_types: vec![
                "proposed_test",
                "proposed_metric",
                "requirement_review",
            ],
            resolution: "define a test, metric, or review path for the requirement",
            metadata: json!({
                "rule_precision": "requirement_marked_verification_required_without_verifies_or_implements_relation",
                "evidence_strength": "cell_source_ids",
                "specificity": "requirement_derived"
            }),
        })?;
        invariant_results.push(finding.invariant_result);
        obstructions.push(finding.obstruction);
    }
    Ok(())
}

pub(super) fn evaluate_api_route_auth(
    space: &AdvisorySpaceEnvelope,
    invariant_results: &mut Vec<Value>,
    obstructions: &mut Vec<Value>,
) -> AdvisoryResult<()> {
    for route in space.cells.iter().filter(|cell| {
        cell["cell_type"] == "component"
            && cell
                .pointer("/metadata/component_type")
                .and_then(Value::as_str)
                == Some("api_endpoint")
    }) {
        if route
            .pointer("/metadata/db_access_detected")
            .and_then(Value::as_bool)
            != Some(true)
            || route
                .pointer("/metadata/auth_detected")
                .and_then(Value::as_bool)
                == Some(true)
            || trusted_route_exception(route, "/metadata/public_endpoint")
            || trusted_route_exception(route, "/metadata/anonymous_allowed")
        {
            continue;
        }
        let obstruction_id = format!(
            "obstruction:{}-missing-auth-guard",
            json_id(route).trim_start_matches("cell:")
        );
        let route_path = route
            .pointer("/metadata/route_path")
            .and_then(Value::as_str)
            .unwrap_or_else(|| title(route));
        let finding = violation_finding(FindingInput {
            space_id: &space.space_id,
            invariant_id: API_ROUTE_AUTH_INVARIANT,
            obstruction_id: &obstruction_id,
            obstruction_type: "api_route_missing_auth",
            severity: "high",
            message: format!(
                "{} touches the database without a detected authentication guard.",
                title(route)
            ),
            witness_ids: vec![json_id(route).to_string()],
            blocked_ids: vec![route["id"].clone()],
            evidence_ids: route
                .get("source_ids")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            recommended_completion_types: vec![
                "proposed_auth_guard",
                "route_security_review",
                "source_backed_evidence",
            ],
            resolution: "add an authentication guard, explicitly mark the endpoint public, or attach reviewed evidence explaining the exception",
            metadata: json!({
                "rule_precision": "api_endpoint_with_db_access_without_detected_auth_guard",
                "evidence_strength": "code_source_ids",
                "specificity": "code_derived",
                "precision_note": "Derived from lexical code snapshot metadata; review is required for dynamic auth wrappers or route-level public exceptions.",
                "route_path": route_path,
                "http_methods": route.pointer("/metadata/http_methods").cloned().unwrap_or_else(|| json!([])),
                "db_access_detected": true,
                "auth_detected": false
            }),
        })?;
        invariant_results.push(finding.invariant_result);
        obstructions.push(finding.obstruction);
    }
    Ok(())
}

pub(super) fn trusted_route_exception(route: &Value, pointer: &str) -> bool {
    route.pointer(pointer).and_then(Value::as_bool) == Some(true)
        && route
            .pointer("/provenance/review_status")
            .and_then(Value::as_str)
            == Some("accepted")
        && route.pointer("/provenance/origin").and_then(Value::as_str) != Some("inferred")
}
