use super::*;

#[test]
fn explicitly_public_database_route_does_not_emit_auth_obstruction() {
    let route = api_route_cell(
        "cell:api-route-src-app-api-public-feed-route-ts-abc123",
        "/api/public-feed",
        true,
        false,
        true,
    );
    let space = base_space(vec![route], vec![]);

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();

    assert!(!report.result["obstructions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["obstruction_type"] == "api_route_missing_auth"));
}

#[test]
fn inferred_public_database_route_still_requires_reviewed_resolution() {
    let mut route = api_route_cell(
        "cell:api-route-src-app-api-public-feed-route-ts-abc123",
        "/api/public-feed",
        true,
        false,
        true,
    );
    route["provenance"] = provenance("inferred", "unreviewed");
    let space = base_space(vec![route], vec![]);

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();

    assert!(report.result["obstructions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["obstruction_type"] == "api_route_missing_auth"));
}

#[test]
fn directed_dependency_cycle_emits_circular_dependency_obstruction() {
    let cells = vec![
        component_cell("cell:service-a", "Service A", "context:platform"),
        component_cell("cell:service-b", "Service B", "context:platform"),
        component_cell("cell:service-c", "Service C", "context:platform"),
    ];
    let incidences = vec![
        depends_on_incidence("incidence:a-b", "cell:service-a", "cell:service-b"),
        depends_on_incidence("incidence:b-c", "cell:service-b", "cell:service-c"),
        depends_on_incidence("incidence:c-a", "cell:service-c", "cell:service-a"),
    ];
    let space = base_space(cells, incidences);

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let obstructions = report.result["obstructions"].as_array().unwrap();
    let cycle = obstructions
        .iter()
        .find(|item| item["obstruction_type"] == "circular_dependency")
        .expect("circular_dependency obstruction emitted");

    assert_eq!(cycle["severity"], "medium");
    assert_eq!(cycle["metadata"]["specificity"], "topology_derived");
    let participants = cycle["metadata"]["cycle_cell_ids"].as_array().unwrap();
    assert_eq!(participants.len(), 3);
}

#[test]
fn dag_dependencies_do_not_emit_cycle_obstruction() {
    let cells = vec![
        component_cell("cell:service-a", "Service A", "context:platform"),
        component_cell("cell:service-b", "Service B", "context:platform"),
    ];
    let incidences = vec![depends_on_incidence(
        "incidence:a-b",
        "cell:service-a",
        "cell:service-b",
    )];
    let space = base_space(cells, incidences);

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();

    assert!(report.result["obstructions"]
        .as_array()
        .unwrap()
        .iter()
        .all(|item| item["obstruction_type"] != "circular_dependency"));
}

#[test]
fn blocker_resolution_excludes_rejected_candidates_from_application_requirements() {
    let blockers = vec![json!({
        "id": "obstruction:missing-owner",
        "blocked_ids": ["cell:ship-action"]
    })];
    let candidates = vec![json!({
        "id": "candidate:missing-owner-owner",
        "candidate_type": "ownership_clarification",
        "review_status": "rejected",
        "resolves_obstruction_ids": ["obstruction:missing-owner"]
    })];

    let state = blocker_resolution_state(&blockers, &candidates);

    assert_eq!(state[0]["resolution_status"], "all_candidates_rejected");
    assert!(state[0]["application_requirements"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn blocker_resolution_describes_accepted_candidate_application_contract() {
    let blockers = vec![json!({
        "id": "obstruction:missing-verification",
        "blocked_ids": ["cell:requirement"]
    })];
    let candidates = vec![json!({
        "id": "candidate:missing-verification-test",
        "candidate_type": "proposed_test",
        "review_status": "accepted",
        "resolves_obstruction_ids": ["obstruction:missing-verification"]
    })];

    let state = blocker_resolution_state(&blockers, &candidates);
    let requirement = &state[0]["application_requirements"][0];

    assert_eq!(
        state[0]["resolution_status"],
        "accepted_candidate_pending_application"
    );
    assert_eq!(
        requirement["required_cell_types"],
        json!(["test_or_verification"])
    );
    assert_eq!(requirement["required_relation_types"], json!(["verifies"]));
}

#[test]
fn no_candidate_frontier_preserves_obstruction_completion_hints() {
    let blockers = vec![json!({
        "id": "obstruction:missing-owner",
        "obstruction_type": "missing_owner",
        "severity": "medium",
        "blocked_ids": ["cell:ship-action"],
        "recommended_completion_types": ["ownership_clarification"]
    })];
    let state = blocker_resolution_state(&blockers, &[]);
    let frontier = frontier_items(&state);

    assert_eq!(state[0]["resolution_status"], "no_candidate");
    assert_eq!(frontier[0]["item_type"], "propose_completion_candidate");
    assert_eq!(frontier[0]["blocked_ids"], json!(["cell:ship-action"]));
    assert_eq!(
        frontier[0]["recommended_completion_types"],
        json!(["ownership_clarification"])
    );
}

#[test]
fn waiting_items_preserve_rejected_candidate_completion_hints() {
    let blockers = vec![json!({
        "id": "obstruction:missing-owner",
        "obstruction_type": "missing_owner",
        "severity": "medium",
        "blocked_ids": ["cell:ship-action"],
        "recommended_completion_types": ["ownership_clarification"]
    })];
    let candidates = vec![json!({
        "id": "candidate:missing-owner-owner",
        "candidate_type": "ownership_clarification",
        "review_status": "rejected",
        "resolves_obstruction_ids": ["obstruction:missing-owner"]
    })];
    let state = blocker_resolution_state(&blockers, &candidates);
    let waiting = waiting_items(&state);

    assert_eq!(state[0]["resolution_status"], "all_candidates_rejected");
    assert_eq!(waiting[0]["item_type"], "all_candidates_rejected");
    assert_eq!(
        waiting[0]["candidate_ids"],
        json!(["candidate:missing-owner-owner"])
    );
    assert_eq!(
        waiting[0]["recommended_completion_types"],
        json!(["ownership_clarification"])
    );
}
