use super::*;

#[test]
fn policy_allowed_hypothesis_lifecycle_proposals_apply_as_events() {
    let temp = TempDir::new().unwrap();
    let space_path = temp.path().join("space.json");
    let check_path = temp.path().join("check.json");
    let proposals_path = temp.path().join("hypothesis-proposals.json");
    let store_path = temp.path().join("store");

    lift_workflow(&LiftOptions {
        input: fixture("advisory.input.json"),
        package: "technical_advisory_mvp".to_string(),
        output: Some(space_path.clone()),
        command: None,
    })
    .unwrap();
    let check = check_workflow(&CheckOptions {
        space: space_path.clone(),
        ruleset: "technical_advisory_mvp".to_string(),
        output: Some(check_path.clone()),
        fail_on: None,
        command: None,
    })
    .unwrap();
    let space_id = check.input["space_id"].as_str().unwrap().to_string();

    let mut observed_space: serde_json::Value =
        serde_json::from_slice(&fs::read(&space_path).unwrap()).unwrap();
    observed_space["cells"].as_array_mut().unwrap().push(json!({
        "id": "cell:reviewed-direct-db-observation",
        "cell_type": "evidence",
        "title": "Reviewed direct DB observation",
        "summary": "Reviewed evidence supports the implicit interface hypothesis.",
        "context_ids": [],
        "source_ids": ["source:architecture-note"],
        "structure_refs": [],
        "provenance": {
            "origin": "source_backed",
            "actor": "reviewer:test",
            "confidence": 0.9,
            "review_status": "accepted"
        },
        "metadata": {
            "supports_hypothesis_id": "hypothesis:order-service-direct-billing-db-access-implicit-interface"
        }
    }));
    fs::write(
        &space_path,
        serde_json::to_vec_pretty(&observed_space).unwrap(),
    )
    .unwrap();

    let proposals = hypothesis_propose_workflow(&HypothesisProposeOptions {
        space: space_path.clone(),
        from_report: check_path,
        output: Some(proposals_path.clone()),
        command: None,
    })
    .unwrap();
    assert_eq!(proposals.result["proposal_count"], 1);
    assert_eq!(
        proposals.result["lifecycle_proposals"][0]["proposed_outcome"],
        "supported"
    );

    case_import_workflow(&CaseImportOptions {
        store: store_path.clone(),
        space: space_path,
        revision_id: "revision:initial".to_string(),
    })
    .unwrap();

    let apply = hypothesis_apply_proposals_workflow(&HypothesisApplyProposalsOptions {
        store: store_path.clone(),
        from_report: proposals_path,
        policy: None,
        reviewer: "ai-agent:test".to_string(),
        reason: "Default conservative policy allowed source-backed support.".to_string(),
        base_revision: Some("revision:initial".to_string()),
        dry_run: false,
    })
    .unwrap();
    assert_eq!(apply["result"]["applied_count"], 1);
    assert_eq!(apply["result"]["skipped_count"], 0);
    assert_eq!(
        apply["result"]["post_apply_case_reason"]["case_head_revision"],
        apply["result"]["case_head_revision"]
    );

    let report = case_reason_workflow(&CaseReasonOptions {
        store: store_path,
        space_id,
    })
    .unwrap();
    let hypotheses = report["projection"]["hypotheses"].as_array().unwrap();
    let supported = hypotheses
        .iter()
        .find(|hypothesis| {
            hypothesis["id"]
                == "hypothesis:order-service-direct-billing-db-access-implicit-interface"
        })
        .unwrap();
    assert_eq!(supported["lifecycle_status"], "supported");
}
