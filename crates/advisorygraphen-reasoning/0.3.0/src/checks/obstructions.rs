use super::*;

pub(super) fn push_hypothesis_quality_obstruction(
    space: &AdvisorySpaceEnvelope,
    invariant_results: &mut Vec<Value>,
    obstructions: &mut Vec<Value>,
    hypothesis: &Value,
    obstruction_type: &str,
    resolution: &'static str,
    message_suffix: &'static str,
) -> AdvisoryResult<()> {
    let obstruction_id = format!(
        "obstruction:{}-{obstruction_type}",
        json_id(hypothesis).trim_start_matches("cell:")
    );
    let finding = violation_finding(FindingInput {
        space_id: &space.space_id,
        invariant_id: HYPOTHESIS_QUALITY_INVARIANT,
        obstruction_id: &obstruction_id,
        obstruction_type,
        severity: "medium",
        message: format!("{} {}", title(hypothesis), message_suffix),
        witness_ids: vec![json_id(hypothesis).to_string()],
        blocked_ids: vec![hypothesis["id"].clone()],
        evidence_ids: hypothesis
            .get("source_ids")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        recommended_completion_types: vec!["hypothesis_observation_model"],
        resolution,
        metadata: json!({
            "rule_precision": "explicit_hypothesis_workflow_quality_gate",
            "hypothesis_status": hypothesis_status(hypothesis),
            "specificity": "hypothesis_derived"
        }),
    })?;
    invariant_results.push(finding.invariant_result);
    obstructions.push(finding.obstruction);
    Ok(())
}
pub(super) fn push_proposal_trace_obstruction(
    space: &AdvisorySpaceEnvelope,
    invariant_results: &mut Vec<Value>,
    obstructions: &mut Vec<Value>,
    input: ProposalTraceObstruction<'_>,
) -> AdvisoryResult<()> {
    let obstruction_id = format!(
        "obstruction:{}-{obstruction_type}",
        json_id(input.action).trim_start_matches("cell:"),
        obstruction_type = input.obstruction_type
    );
    let finding = violation_finding(FindingInput {
        space_id: &space.space_id,
        invariant_id: PROPOSAL_TRACE_INVARIANT,
        obstruction_id: &obstruction_id,
        obstruction_type: input.obstruction_type,
        severity: "medium",
        message: format!("{} {}", title(input.action), input.message_suffix),
        witness_ids: vec![json_id(input.action).to_string()],
        blocked_ids: vec![input.action["id"].clone()],
        evidence_ids: input
            .action
            .get("source_ids")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        recommended_completion_types: vec!["proposal_trace_completion"],
        resolution: input.resolution,
        metadata: merge_json_objects(
            json!({
                "rule_precision": "explicit_hypothesis_workflow_proposal_trace_gate",
                "specificity": "proposal_trace_derived"
            }),
            input.metadata,
        ),
    })?;
    invariant_results.push(finding.invariant_result);
    obstructions.push(finding.obstruction);
    Ok(())
}

pub(super) fn merge_json_objects(mut left: Value, right: Value) -> Value {
    if let (Some(left), Some(right)) = (left.as_object_mut(), right.as_object()) {
        for (key, value) in right {
            left.insert(key.clone(), value.clone());
        }
    }
    left
}
