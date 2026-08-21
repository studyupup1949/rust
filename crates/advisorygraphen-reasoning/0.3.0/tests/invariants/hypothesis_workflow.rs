use super::*;

#[test]
fn hypothesis_lifecycle_proposal_uses_agent_observation_without_applying_event() {
    let action = action_cell("cell:ship-action");
    let check_space_without_observation = base_space(vec![action.clone()], vec![]);
    let check_report = check_space(
        &check_space_without_observation,
        "technical_advisory_mvp",
        None,
        None,
    )
    .unwrap();
    let hypothesis_id = "hypothesis:ship-action-missing-owner-no-team-holds-action";
    let observation = json!({
        "id": "cell:owner-observation",
        "cell_type": "evidence",
        "title": "Agent observed no owner metadata",
        "summary": "An AI agent inspected the action record and found no owner cell or owns incidence.",
        "context_ids": [],
        "source_ids": [],
        "structure_refs": [],
        "provenance": provenance("inferred", "unreviewed"),
        "metadata": {
            "supports_hypothesis_id": hypothesis_id
        }
    });
    let space_with_observation = base_space(vec![action, observation], vec![]);

    let proposal = propose_hypothesis_lifecycle(
        &space_with_observation,
        &check_report,
        "check-report.json",
        None,
    )
    .unwrap();
    let proposals = proposal.result["lifecycle_proposals"].as_array().unwrap();

    assert_eq!(proposal.report_type, "hypothesis_lifecycle_proposal");
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0]["target_hypothesis_id"], hypothesis_id);
    assert_eq!(proposals[0]["proposed_outcome"], "supported");
    assert_eq!(proposals[0]["review_status"], "unreviewed");
    assert_eq!(
        proposal.result["authority_boundary"]["may_apply_events"],
        json!(false)
    );
}

#[test]
fn explicit_hypothesis_workflow_checks_hypothesis_and_proposal_quality() {
    let hypothesis = json!({
        "id": "cell:hypothesis-install-drift",
        "cell_type": "claim",
        "title": "Local install drift",
        "summary": "Install drift explains collection failure.",
        "context_ids": [],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {
            "hypothesis": true,
            "hypothesis_status": "strongly_supported",
            "expected_observations": ["module missing locally"]
        }
    });
    let action = json!({
        "id": "cell:repair-install",
        "cell_type": "action",
        "title": "Repair install",
        "summary": "Repair local install state.",
        "context_ids": [],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {
            "priority": "P0",
            "derived_from_hypothesis": "cell:hypothesis-install-drift"
        }
    });
    let mut space = base_space(vec![hypothesis, action], vec![]);
    space.metadata.insert(
        "method".to_string(),
        json!("one-problem-multiple-hypotheses-observe-classify-propose"),
    );

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();

    assert_obstruction(&report.result, "hypothesis_missing_falsifiers");
    assert_obstruction(&report.result, "supported_hypothesis_missing_support");
    assert_obstruction(&report.result, "strong_hypothesis_missing_competition");
    assert_obstruction(
        &report.result,
        "proposal_derived_from_unsupported_hypothesis",
    );
    assert_obstruction(&report.result, "proposal_missing_verification");
}

#[test]
fn explicit_hypothesis_workflow_accepts_supported_trace_with_verification() {
    let hypothesis = json!({
        "id": "cell:hypothesis-install-drift",
        "cell_type": "claim",
        "title": "Local install drift",
        "summary": "Install drift explains collection failure.",
        "context_ids": [],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {
            "hypothesis": true,
            "hypothesis_status": "strongly_supported",
            "expected_observations": ["module missing locally"],
            "falsifiers": ["fresh install still fails"]
        }
    });
    let competing = json!({
        "id": "cell:hypothesis-import-path",
        "cell_type": "claim",
        "title": "Import path issue",
        "summary": "Import path explains collection failure.",
        "context_ids": [],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {
            "hypothesis": true,
            "hypothesis_status": "falsified",
            "expected_observations": ["package installed but import fails"],
            "falsifiers": ["package absent locally"]
        }
    });
    let evidence = json!({
        "id": "cell:evidence-install-missing",
        "cell_type": "evidence",
        "title": "Install missing",
        "summary": "Package is absent locally.",
        "context_ids": [],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {}
    });
    let action = json!({
        "id": "cell:repair-install",
        "cell_type": "action",
        "title": "Repair install",
        "summary": "Repair local install state.",
        "context_ids": [],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {
            "priority": "P0",
            "required_verification": "full collection exits 0"
        }
    });
    let mut space = base_space(
        vec![hypothesis, competing, evidence, action],
        vec![
            relation(
                "incidence:evidence-supports-install",
                "supports",
                "cell:evidence-install-missing",
                "cell:hypothesis-install-drift",
            ),
            relation(
                "incidence:evidence-falsifies-import",
                "falsifies",
                "cell:evidence-install-missing",
                "cell:hypothesis-import-path",
            ),
            relation(
                "incidence:install-competes-import",
                "competes_with",
                "cell:hypothesis-install-drift",
                "cell:hypothesis-import-path",
            ),
            relation(
                "incidence:repair-derives-install",
                "derives_from",
                "cell:repair-install",
                "cell:hypothesis-install-drift",
            ),
        ],
    );
    space.metadata.insert(
        "method".to_string(),
        json!("one-problem-multiple-hypotheses-observe-classify-propose"),
    );

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let obstruction_types = report.result["obstructions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["obstruction_type"].as_str())
        .collect::<Vec<_>>();

    assert!(!obstruction_types.contains(&"hypothesis_missing_falsifiers"));
    assert!(!obstruction_types.contains(&"supported_hypothesis_missing_support"));
    assert!(!obstruction_types.contains(&"strong_hypothesis_missing_competition"));
    assert!(!obstruction_types.contains(&"proposal_derived_from_unsupported_hypothesis"));
    assert!(!obstruction_types.contains(&"proposal_missing_verification"));
}
