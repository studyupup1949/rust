use super::*;

pub(super) fn executive_projection(
    space: &AdvisorySpaceEnvelope,
    report: &Value,
    audience: &str,
) -> AdvisoryResult<Value> {
    let obstructions = obstructions(report);
    let represented_ids = represented_ids(report);
    let omitted_ids = source_ids(space);
    let high_severity_obstructions = obstructions_by_severity(&obstructions, "high");
    let medium_severity_obstructions = obstructions_by_severity(&obstructions, "medium");
    let close_status = close_status_value(space, report);
    let candidates = completion_candidates(report);
    let candidate_quality = candidate_quality_summary(&candidates);
    let proposal_content_summary = proposal_content_summary(&candidates);
    let recommendation_trace = recommendation_trace(&candidates);
    let observation_actions = observation_actions(&recommendation_trace);
    let falsifiers = falsifiers(report);
    let explicit_hypothesis_matrix = explicit_hypothesis_matrix(space);
    let hypotheses = merged_hypotheses(hypotheses(report), &explicit_hypothesis_matrix);
    let hypothesis_summary = hypothesis_summary(&hypotheses);
    let explicit_proposal_trace = explicit_proposal_trace(space);
    let projection_loss = projection_loss(space, report);
    let schema_morphisms = schema_morphisms(space);
    let higher_graphen =
        higher::projection_artifacts(space, report, audience, omitted_ids.clone())?;
    Ok(json!({
        "schema": "advisorygraphen.projection.v1",
        "projection_id": format!("projection:executive:{}", space.space_id.trim_start_matches("space:advisory:")),
        "audience": "executive",
        "space_id": space.space_id,
        "represented_ids": represented_ids,
        "omitted_ids": omitted_ids,
        "summary": {
            "closeable": close_status["closeable"].clone(),
            "blocking_threshold": close_status["blocking_threshold"].clone(),
            "blocking_obstruction_ids": close_status["blocking_obstruction_ids"].clone(),
            "obstruction_counts": obstruction_counts(&obstructions),
            "high_severity_obstructions": high_severity_obstructions,
            "medium_severity_obstructions": medium_severity_obstructions,
            "unreviewed_candidates_are_not_accepted": true,
            "candidate_quality": candidate_quality,
            "proposal_content_summary": proposal_content_summary,
            "recommendation_trace": recommendation_trace,
            "observation_actions": observation_actions,
            "explicit_hypothesis_matrix": explicit_hypothesis_matrix,
            "explicit_proposal_trace": explicit_proposal_trace,
            "hypothesis_summary": hypothesis_summary
        },
        "hypotheses": hypotheses,
        "falsifiers": falsifiers,
        "source_boundary": space.metadata.get("source_boundary").cloned().unwrap_or_else(|| json!({})),
        "projection_loss": projection_loss,
        "projection_loss_metrics": higher_graphen.loss_metrics,
        "schema_morphisms": schema_morphisms,
        "higher_graphen": higher_graphen.result_json
    }))
}

pub(super) fn developer_projection(
    space: &AdvisorySpaceEnvelope,
    report: &Value,
    audience: &str,
) -> AdvisoryResult<Value> {
    let represented_ids = represented_ids(report);
    let omitted_ids = source_ids(space);
    let projection_loss = projection_loss(space, report);
    let higher_graphen =
        higher::projection_artifacts(space, report, audience, omitted_ids.clone())?;
    Ok(json!({
        "schema": "advisorygraphen.projection.v1",
        "projection_id": format!("projection:developer-action:{}", space.space_id.trim_start_matches("space:advisory:")),
        "audience": "developer_action",
        "space_id": space.space_id,
        "represented_ids": represented_ids,
        "omitted_ids": omitted_ids,
        "actions": completion_candidates(report),
        "projection_loss": projection_loss,
        "projection_loss_metrics": higher_graphen.loss_metrics,
        "schema_morphisms": schema_morphisms(space),
        "higher_graphen": higher_graphen.result_json
    }))
}

pub(super) fn audit_projection(
    space: &AdvisorySpaceEnvelope,
    report: &Value,
    audience: &str,
) -> AdvisoryResult<Value> {
    let represented_ids = represented_ids(report);
    let omitted_ids = Vec::new();
    let projection_loss = projection_loss(space, report);
    let higher_graphen =
        higher::projection_artifacts(space, report, audience, omitted_ids.clone())?;
    Ok(json!({
        "schema": "advisorygraphen.projection.v1",
        "projection_id": format!("projection:audit:{}", space.space_id.trim_start_matches("space:advisory:")),
        "audience": "audit_trace",
        "space_id": space.space_id,
        "represented_ids": represented_ids,
        "omitted_ids": omitted_ids,
        "source_boundary": space.metadata.get("source_boundary").cloned().unwrap_or_else(|| json!({})),
        "report": report,
        "projection_loss": projection_loss,
        "projection_loss_metrics": higher_graphen.loss_metrics,
        "schema_morphisms": schema_morphisms(space),
        "higher_graphen": higher_graphen.result_json
    }))
}

pub(super) fn ai_agent_projection(
    space: &AdvisorySpaceEnvelope,
    report: &Value,
    audience: &str,
) -> AdvisoryResult<Value> {
    let represented_ids = represented_ids(report);
    let omitted_ids = source_ids(space);
    let open_obstructions = obstructions(report);
    let candidates = completion_candidates(report);
    let resolution_state = blocker_resolution_state(&open_obstructions, &candidates);
    let candidate_quality = candidate_quality_summary(&candidates);
    let proposal_content_summary = proposal_content_summary(&candidates);
    let recommendation_trace = recommendation_trace(&candidates);
    let observation_actions = observation_actions(&recommendation_trace);
    let hypothesis_promotion_workflow = hypothesis_promotion_workflow(&recommendation_trace);
    let (live_candidates, superseded_candidates) = partition_candidates(&candidates);
    let falsifiers = falsifiers(report);
    let argumentation_incidences = argumentation_incidences(report);
    let explicit_hypothesis_matrix = explicit_hypothesis_matrix(space);
    let hypotheses = merged_hypotheses(hypotheses(report), &explicit_hypothesis_matrix);
    let hypothesis_summary = hypothesis_summary(&hypotheses);
    let explicit_proposal_trace = explicit_proposal_trace(space);
    let projection_loss = projection_loss(space, report);
    let schema_morphisms = schema_morphisms(space);
    let higher_graphen =
        higher::projection_artifacts(space, report, audience, omitted_ids.clone())?;
    let correspondence_analysis = correspondence::correspondence_analysis(
        space,
        &open_obstructions,
        &hypotheses,
        &falsifiers,
        &candidates,
        &argumentation_incidences,
    )?;
    Ok(json!({
        "schema": "advisorygraphen.projection.v1",
        "projection_id": format!("projection:ai-agent:{}", space.space_id.trim_start_matches("space:advisory:")),
        "audience": "ai_agent",
        "space_id": space.space_id,
        "represented_ids": represented_ids,
        "omitted_ids": omitted_ids,
        "hg_operation_model": {
            "primary_operator": "ai_agent",
            "human_role": "sets goals, reviews candidates, and accepts or rejects promotions",
            "human_ui_role": "projection_consumer",
            "source_of_truth": "advisory_space_case_log_and_review_events",
            "principle": "HigherGraphen structure is manipulated by agents; humans review projections and explicit promotion events."
        },
        "agent_operation_contract": {
            "allowed_commands": [
                "validate",
                "lift",
                "check",
                "completions propose",
                "hypothesis propose",
                "hypothesis apply-proposals with conservative policy",
                "project ai_agent",
                "project audit_trace",
                "case import",
                "case reason",
                "case close-check"
            ],
            "review_gated_commands": [
                "completions accept",
                "completions reject",
                "hypothesis falsify",
                "hypothesis support",
                "hypothesis accept",
                "hypothesis reject"
            ],
            "forbidden_operations": [
                "promote unreviewed candidate structure",
                "hide projection_loss",
                "hide projection_loss_metrics",
                "treat inferred evidence as accepted fact",
                "rewrite source material outside the bounded snapshot"
            ],
            "resume_protocol": [
                "read close_status",
                "inspect open_obstructions",
                "inspect candidate_review_state",
                "inspect correspondence_analysis for shared evidence, conflicts, and gluing failures",
                "inspect blocker_resolution_state.application_requirements when present",
                "inspect observation_actions before promoting unsupported hypotheses",
                "inspect projection_loss_metrics and schema_morphisms before summarizing",
                "propose missing owner or verification structure",
                "generate audit_trace before reporting final state"
            ]
        },
        "open_obstructions": open_obstructions,
        "hypotheses": hypotheses,
        "falsifiers": falsifiers,
        "argumentation_incidences": argumentation_incidences,
        "correspondence_analysis": correspondence_analysis,
        "hypothesis_summary": hypothesis_summary,
        "explicit_hypothesis_matrix": explicit_hypothesis_matrix,
        "explicit_proposal_trace": explicit_proposal_trace,
        "candidate_review_state": candidates,
        "live_candidates": live_candidates,
        "superseded_candidates": superseded_candidates,
        "candidate_quality": candidate_quality,
        "proposal_content_summary": proposal_content_summary,
        "recommendation_trace": recommendation_trace,
        "observation_actions": observation_actions,
        "hypothesis_promotion_workflow": hypothesis_promotion_workflow,
        "blocker_resolution_state": resolution_state,
        "frontier_items": frontier_items(&resolution_state),
        "waiting_items": waiting_items(&resolution_state),
        "next_safe_operations": [
            "review_obstructions",
            "inspect_application_requirements",
            "propose_or_review_candidates",
            "run_case_close_check_before_closure",
            "generate_audit_projection"
        ],
        "close_status": close_status_value(space, report),
        "projection_loss": projection_loss,
        "projection_loss_metrics": higher_graphen.loss_metrics,
        "schema_morphisms": schema_morphisms,
        "higher_graphen": higher_graphen.result_json
    }))
}
