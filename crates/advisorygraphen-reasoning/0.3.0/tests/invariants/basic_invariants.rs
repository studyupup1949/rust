use super::*;

#[test]
fn action_without_owner_emits_missing_owner_obstruction() {
    let space = base_space(vec![action_cell("cell:ship-action")], vec![]);

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();

    assert_obstruction(&report.result, "missing_owner");
}

#[test]
fn requirement_marked_verification_required_emits_obstruction() {
    let requirement = json!({
        "id": "cell:requirement",
        "cell_type": "requirement",
        "title": "Requirement",
        "summary": null,
        "context_ids": [],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": { "require_verification": true }
    });
    let space = base_space(vec![requirement], vec![]);

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();

    assert_obstruction(&report.result, "requirement_unverified");
}

#[test]
fn accepted_inferred_action_emits_insufficient_evidence() {
    let action = json!({
        "id": "cell:inferred-action",
        "cell_type": "action",
        "title": "Inferred action",
        "summary": null,
        "context_ids": [],
        "source_ids": [],
        "structure_refs": [],
        "provenance": provenance("inferred", "accepted"),
        "metadata": {}
    });
    let space = base_space(vec![action], vec![]);

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();

    assert_obstruction(&report.result, "insufficient_evidence");
}

#[test]
fn boundary_completion_candidates_are_derived_from_witness_cells() {
    let mut space = base_space(
        vec![
            component_cell(
                "cell:inventory-service",
                "Inventory Service",
                "context:inventory",
            ),
            data_store_cell("cell:pricing-db", "Pricing DB", "context:pricing"),
        ],
        vec![json!({
            "id": "incidence:inventory-service-accesses-pricing-db",
            "relation_type": "accesses",
            "from_id": "cell:inventory-service",
            "to_id": "cell:pricing-db",
            "source_ids": ["source:pricing-note"],
            "evidence_ids": ["source:pricing-note"],
            "provenance": provenance("source_backed", "accepted"),
            "metadata": { "access_type": "direct_database_read" }
        })],
    );
    space.contexts = vec![
        context("context:inventory", "Inventory"),
        context("context:pricing", "Pricing"),
    ];

    let check_report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let completion_report =
        propose_completions(&space, &check_report, "check-report.json", None).unwrap();
    let candidates = completion_report.result["completion_candidates"]
        .as_array()
        .unwrap();

    assert!(check_report.result["obstructions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == "obstruction:inventory-service-direct-pricing-db-access"));
    assert!(candidates
        .iter()
        .any(|item| item["id"] == "candidate:pricing-status-api"));
    assert!(candidates.iter().any(|item| {
        item["metadata"]["specificity"] == "source_derived"
            && item["source_ids"] == json!(["source:pricing-note"])
    }));
    let status_api = candidates
        .iter()
        .find(|item| item["id"] == "candidate:pricing-status-api")
        .expect("pricing status API candidate");
    assert_eq!(
        status_api["proposal_content"]["scenario"]["scenario_kind"],
        "planned"
    );
    assert_eq!(
        status_api["application_plan"]["schema"],
        "advisorygraphen.completion.application_plan.v1"
    );
    assert_eq!(status_api["application_plan"]["dry_run_supported"], true);
    assert!(status_api["application_plan"]["operations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|operation| {
            operation["operation"] == "remove_incidence"
                && operation["incidence_id"] == "incidence:inventory-service-accesses-pricing-db"
        }));
    assert_eq!(
        status_api["proposal_content"]["morphism"]["morphism_type"],
        "as_is_to_to_be"
    );
    assert_eq!(
        status_api["proposal_content"]["scenario"]["affected_invariants"],
        json!(["invariant:architecture_no_cross_context_direct_database_access"])
    );
    assert_eq!(
        status_api["proposal_content"]["derivation"]["verification_status"],
        "hypothesis_not_supported"
    );
    assert_eq!(status_api["recommendation_role"], "follow_up_observation");
    assert_eq!(status_api["supported_hypothesis_ids"], json!([]));
    assert!(status_api["proposal_content"]["content_obstructions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["obstruction_type"] == "proposal_depends_on_unsupported_hypothesis"));
    assert_eq!(
        status_api["proposal_content"]["valuation"]["order_type"],
        "partial_order"
    );
    assert_eq!(
        status_api["proposal_content"]["policy"]["policy_type"],
        "completion_review_gate"
    );
    assert!(!candidates
        .iter()
        .any(|item| item["id"] == "candidate:billing-status-api"));
}
