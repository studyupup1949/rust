use super::*;

#[test]
fn code_derived_candidates_count_into_their_own_bucket() {
    let space = empty_space();
    let report = check_report(
        vec![code_derived_obstruction()],
        vec![code_derived_candidate(), source_derived_candidate()],
    );

    let projection = build_projection(&space, &report, "executive").unwrap();

    let quality = projection
        .pointer("/summary/candidate_quality")
        .expect("candidate_quality summary present");
    assert_eq!(quality["code_derived"].as_u64(), Some(1));
    assert_eq!(quality["source_derived"].as_u64(), Some(1));
    assert_eq!(quality["missing_precision_metadata"].as_u64(), Some(0));
    assert_eq!(quality["total"].as_u64(), Some(2));
}

#[test]
fn code_derived_obstructions_emit_lexical_detection_loss_entry() {
    let space = empty_space();
    let report = check_report(vec![code_derived_obstruction()], vec![]);

    let projection = build_projection(&space, &report, "executive").unwrap();

    let losses = projection
        .pointer("/projection_loss")
        .and_then(Value::as_array)
        .expect("projection_loss array present");
    let lexical = losses
        .iter()
        .find(|entry| entry["loss_type"] == "lexical_detection_caveat")
        .expect("lexical_detection_caveat entry emitted");
    assert_eq!(lexical["severity"], json!("medium"));
    assert_eq!(
        lexical["omitted_ids"],
        json!(["obstruction:route-missing-auth-guard"])
    );
}

#[test]
fn projection_loss_omits_lexical_caveat_when_no_code_derived_finding() {
    let space = empty_space();
    let report = check_report(vec![], vec![source_derived_candidate()]);

    let projection = build_projection(&space, &report, "executive").unwrap();

    let losses = projection
        .pointer("/projection_loss")
        .and_then(Value::as_array)
        .expect("projection_loss array present");
    assert!(losses
        .iter()
        .all(|entry| entry["loss_type"] != "lexical_detection_caveat"));
}

#[test]
fn projection_loss_metrics_report_source_trace_gaps() {
    let space = empty_space();
    let report = check_report(
        vec![],
        vec![source_derived_candidate(), untraced_candidate()],
    );

    let projection = build_projection(&space, &report, "ai_agent").unwrap();

    let metrics = projection
        .pointer("/projection_loss_metrics")
        .expect("projection_loss_metrics present");
    assert_eq!(metrics["projected_cardinality"], json!(1));
    assert_eq!(metrics["source_trace_gap_count"], json!(1));
    assert_eq!(
        metrics["source_trace_gap_ids"],
        json!(["candidate:untraced"])
    );
}

#[test]
fn proposal_content_summary_counts_blocked_content_obstructions() {
    let space = empty_space();
    let report = check_report(
        vec![],
        vec![
            source_derived_candidate(),
            blocked_proposal_content_candidate(),
        ],
    );

    let projection = build_projection(&space, &report, "ai_agent").unwrap();

    let summary = projection
        .pointer("/proposal_content_summary")
        .expect("proposal_content_summary present");
    assert_eq!(summary["with_structured_content"], json!(1));
    assert_eq!(summary["blocked_content"], json!(1));
    assert_eq!(summary["content_obstruction_count"], json!(1));
    assert_eq!(
        summary["content_obstruction_types"]["proposal_content_underspecified"],
        json!(1)
    );
}
