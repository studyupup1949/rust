use super::*;

#[test]
fn recommendation_trace_separates_primary_from_follow_up_observations() {
    let space = empty_space();
    let report = check_report(
        vec![],
        vec![primary_candidate(), unsupported_follow_up_candidate()],
    );

    let projection = build_projection(&space, &report, "executive").unwrap();

    let trace = projection
        .pointer("/summary/recommendation_trace")
        .expect("recommendation_trace present");
    assert_eq!(trace["primary_count"], json!(1));
    assert_eq!(trace["follow_up_observation_count"], json!(1));
    assert_eq!(
        trace["primary_recommendations"][0]["candidate_id"],
        "candidate:supported-action"
    );
    assert_eq!(
        trace["follow_up_observations"][0]["unsupported_hypothesis_ids"],
        json!(["hypothesis:unreviewed"])
    );
    assert_eq!(
        trace["follow_up_observations"][0]["ranked_observation_tasks"][0]["observation_type"],
        json!("hypothesis_support")
    );
    assert_eq!(
        trace["follow_up_observations"][0]["ranked_observation_tasks"][0]["source_ids_to_inspect"],
        json!(["source:test"])
    );
    assert_eq!(
        trace["follow_up_observations"][0]["ranked_observation_tasks"][0]["output_schema"]
            ["required"][0],
        json!("observation_status")
    );
    assert!(
        trace["follow_up_observations"][0]["ranked_observation_tasks"][0]["command_template"]
            .as_str()
            .unwrap()
            .contains("verification method")
    );
    assert_eq!(
        trace["follow_up_observations"][0]["ranked_observation_tasks"][0]["pass_fail_extraction"]
            ["review_required"],
        json!(true)
    );
    let actions = projection
        .pointer("/summary/observation_actions")
        .expect("observation_actions present");
    assert_eq!(actions["count"], json!(1));
    assert_eq!(
        actions["actions"][0]["id"],
        json!("observation-action:follow-up-support-1")
    );
    assert_eq!(
        actions["actions"][0]["target_claim_ids"],
        json!(["hypothesis:unreviewed"])
    );
    assert_eq!(
        actions["actions"][0]["expected_evidence_kind"],
        json!("support_or_falsification_witness")
    );
    assert_eq!(actions["actions"][0]["estimated_cost"], json!("low"));
    assert_eq!(
        actions["actions"][0]["policy_blockers"],
        json!(["review_required"])
    );
}

#[test]
fn ai_agent_projection_exposes_hypothesis_promotion_workflow() {
    let space = empty_space();
    let report = check_report(vec![], vec![unsupported_follow_up_candidate()]);

    let projection = build_projection(&space, &report, "ai_agent").unwrap();

    let workflow = projection
        .pointer("/hypothesis_promotion_workflow")
        .expect("hypothesis_promotion_workflow present");
    assert_eq!(workflow["item_count"], json!(1));
    assert_eq!(
        workflow["items"][0]["blocking_hypothesis_ids"],
        json!(["hypothesis:unreviewed"])
    );
    assert_eq!(
        workflow["items"][0]["promotion_steps"][0],
        json!("Run the ranked observation tasks against the bounded source snapshot.")
    );
    assert!(projection["agent_operation_contract"]["resume_protocol"]
        .as_array()
        .unwrap()
        .contains(&json!(
            "inspect observation_actions before promoting unsupported hypotheses"
        )));
    assert!(projection["agent_operation_contract"]["resume_protocol"]
        .as_array()
        .unwrap()
        .contains(&json!(
            "inspect projection_loss_metrics and schema_morphisms before summarizing"
        )));
    assert!(
        projection["agent_operation_contract"]["forbidden_operations"]
            .as_array()
            .unwrap()
            .contains(&json!("hide projection_loss_metrics"))
    );
}
