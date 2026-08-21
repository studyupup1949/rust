use super::*;

#[test]
fn observation_support_promotes_follow_up_candidate_to_primary_after_case_reason() {
    let temp = TempDir::new().unwrap();
    let space_path = temp.path().join("space.json");
    let check_path = temp.path().join("check.json");
    let completions_path = temp.path().join("completions.json");
    let projection_path = temp.path().join("ai-agent.json");
    let observation_result_path = temp.path().join("observation-result.json");
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
    completions_propose_workflow(&CompletionProposeOptions {
        space: space_path.clone(),
        from_report: check_path.clone(),
        output: Some(completions_path.clone()),
        command: None,
    })
    .unwrap();
    project_workflow(&ProjectOptions {
        space: space_path.clone(),
        report: check_path.clone(),
        completions_report: Some(completions_path),
        audience: "ai_agent".to_string(),
        format: OutputFormat::Json,
        output: Some(projection_path.clone()),
    })
    .unwrap();
    let projection: serde_json::Value =
        serde_json::from_slice(&fs::read(&projection_path).unwrap()).unwrap();
    assert_eq!(
        projection["recommendation_trace"]["primary_count"], 0,
        "unsupported hypotheses should keep candidates out of primary recommendations"
    );
    let task = projection["recommendation_trace"]["follow_up_observations"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|item| item["ranked_observation_tasks"].as_array().unwrap().iter())
        .find(|task| {
            task["hypothesis_id"]
                == "hypothesis:order-service-direct-billing-db-access-implicit-interface"
        })
        .unwrap();
    let task_id = task["task_id"].as_str().unwrap().to_string();

    case_import_workflow(&CaseImportOptions {
        store: store_path.clone(),
        space: space_path,
        revision_id: "revision:initial".to_string(),
    })
    .unwrap();
    fs::write(
        &observation_result_path,
        serde_json::to_vec_pretty(&json!({
            "observation_status": "supports",
            "evidence_ids": ["source:architecture-note"],
            "summary": "Reviewed architecture evidence supports the implicit interface hypothesis.",
            "supports_hypothesis": true,
            "falsifies_hypothesis": false
        }))
        .unwrap(),
    )
    .unwrap();
    let observation = observation_record_workflow(&ObservationRecordOptions {
        store: store_path.clone(),
        space_id: space_id.clone(),
        from_projection: projection_path,
        task_id,
        result: observation_result_path,
        reviewer: "reviewer:test".to_string(),
        reason: "Observed source-backed support for the hypothesis.".to_string(),
        base_revision: Some("revision:initial".to_string()),
    })
    .unwrap();
    let evidence_id = observation["result"]["evidence_cell"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let observation_head = observation["result"]["case_head_revision"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        observation["result"]["promotion_gate"]["evidence_cell_id"],
        evidence_id
    );
    let support_command = observation["result"]["suggested_next_commands"]["support"]
        .as_str()
        .unwrap();
    assert!(support_command.contains(&evidence_id));
    assert!(support_command.contains(&observation_head));
    assert!(!support_command.contains("<evidence_cell_id>"));

    hypothesis_support_workflow(&HypothesisFalsifyOptions {
        store: store_path.clone(),
        from_report: check_path,
        hypothesis_id: "hypothesis:order-service-direct-billing-db-access-implicit-interface"
            .to_string(),
        evidence_ids: vec![evidence_id],
        reviewer: "reviewer:test".to_string(),
        reason: "Observation supports the implicit interface hypothesis.".to_string(),
        base_revision: Some(observation_head),
    })
    .unwrap();

    let report = case_reason_workflow(&CaseReasonOptions {
        store: store_path,
        space_id,
    })
    .unwrap();
    let trace = &report["projection"]["recommendation_trace"];
    assert!(
        trace["primary_count"].as_u64().unwrap() > 0,
        "supporting the hypothesis should allow derived candidates to become primary"
    );
    assert!(trace["primary_recommendations"]
        .as_array()
        .unwrap()
        .iter()
        .all(|candidate| candidate["supported_hypothesis_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |id| id == "hypothesis:order-service-direct-billing-db-access-implicit-interface"
            )));
}
