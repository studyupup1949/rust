use super::*;

#[test]
fn ai_agent_projection_exposes_higher_graphen_correspondence_gluing() {
    let space = correspondence_space();
    let report = check_report(
        vec![correspondence_obstruction_with_failed_invariant()],
        vec![correspondence_candidate_with_satisfied_invariant()],
    );

    let projection = build_projection(&space, &report, "ai_agent").unwrap();

    let analysis = projection
        .pointer("/correspondence_analysis")
        .expect("correspondence analysis present");
    assert_eq!(
        analysis["source"],
        json!("highergraphen_0_5_correspondence_overlap_gluing")
    );
    assert!(
        analysis["candidate_count"].as_u64().unwrap_or(0) > 0,
        "expected at least one correspondence candidate: {analysis:#}"
    );
    assert!(
        analysis["emitted_candidate_count"].as_u64().unwrap_or(0)
            <= analysis["max_emitted_candidates"].as_u64().unwrap_or(0),
        "AI-agent projection should emit only bounded review-focus correspondence candidates: {analysis:#}"
    );
    assert!(
        analysis["review_focus_candidates"].as_array().unwrap()[0]["selection_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason == "gluing_failure"),
        "review focus should prioritize gluing failures: {analysis:#}"
    );
    assert!(
        analysis["gluing_summary"]["failure"].as_u64().unwrap_or(0) > 0,
        "falsifier/hypothesis mismatch should block silent gluing: {analysis:#}"
    );
    assert_eq!(
        analysis["candidates"][0]["reviewStatus"],
        json!("candidate"),
        "HG correspondence must stay reviewable"
    );
    assert!(
        analysis["ai_agent_projections"][0]["projectionLoss"].is_object(),
        "HG projection should preserve explicit projection loss"
    );
    assert!(
        analysis["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|candidate| candidate["overlapWitnesses"].as_array().unwrap())
            .all(|witness| witness["witnessKind"] != json!("PredicateSet")),
        "a single direct incidence must not be treated as a shared predicate: {analysis:#}"
    );
}

#[test]
fn projections_expose_explicit_hypothesis_matrix_and_proposal_trace() {
    let space = explicit_hypothesis_space();
    let report = check_report(vec![], vec![]);

    let executive = build_projection(&space, &report, "executive").unwrap();
    let matrix = executive
        .pointer("/summary/explicit_hypothesis_matrix")
        .expect("explicit_hypothesis_matrix present");
    assert_eq!(matrix["count"], json!(2));
    assert_eq!(matrix["status_counts"]["strongly_supported"], json!(1));
    assert_eq!(matrix["status_counts"]["falsified"], json!(1));
    assert_eq!(
        matrix["hypotheses"][0]["expected_observations"],
        json!(["unit test import resolution fails when node_modules is absent"])
    );
    assert_eq!(
        matrix["hypotheses"][0]["falsifiers"],
        json!(["clean install still cannot resolve elkjs"])
    );
    assert_eq!(
        matrix["hypotheses"][0]["supporting_incidence_ids"],
        json!(["incidence:evidence-supports-local-install"])
    );
    assert_eq!(
        matrix["hypotheses"][0]["competing_hypothesis_ids"],
        json!(["cell:hypothesis-lockfile"])
    );

    let trace = executive
        .pointer("/summary/explicit_proposal_trace")
        .expect("explicit_proposal_trace present");
    assert_eq!(trace["count"], json!(1));
    assert_eq!(
        trace["proposals"][0]["derived_hypothesis_ids"],
        json!(["cell:hypothesis-local-install"])
    );
    assert_eq!(
        trace["proposals"][0]["derived_hypothesis_statuses"][0]["status"],
        json!("strongly_supported")
    );
    assert_eq!(
        trace["proposals"][0]["required_verification"],
        json!("Run unit tests after reinstalling dependencies.")
    );
    assert_eq!(trace["proposals"][0]["owner_state"], json!("present"));
    assert_eq!(trace["proposals"][0]["proposal_quality_notes"], json!([]));

    let ai_agent = build_projection(&space, &report, "ai_agent").unwrap();
    assert_eq!(ai_agent["explicit_hypothesis_matrix"]["count"], json!(2));
    assert_eq!(ai_agent["explicit_proposal_trace"]["count"], json!(1));
    assert_eq!(ai_agent["hypothesis_summary"]["total"], json!(2));
    assert_eq!(
        ai_agent["hypothesis_summary"]["strongly_supported"],
        json!(1)
    );
    assert_eq!(ai_agent["hypothesis_summary"]["falsified"], json!(1));
    assert_eq!(
        ai_agent["hypotheses"][0]["source"],
        json!("explicit_advisory_space")
    );
    assert_eq!(
        ai_agent["hypotheses"][0]["refinement_status"],
        json!("seed")
    );
    let metrics = &ai_agent["projection_loss_metrics"];
    assert_eq!(metrics["source_cardinality"], json!(1));
    assert_eq!(metrics["projected_cardinality"], json!(1));
    assert_eq!(metrics["collapsed_source_distinction_count"], json!(0));
    assert_eq!(metrics["missing_loss_declaration"], json!(false));
    assert_eq!(metrics["risk_severity"], json!("medium"));
    assert_eq!(
        metrics["review_signals"],
        json!(["source_trace_missing", "unsupported_loss_metric"])
    );
}
