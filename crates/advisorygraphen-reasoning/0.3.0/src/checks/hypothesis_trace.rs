use super::*;

pub(super) fn evaluate_hypothesis_quality(
    space: &AdvisorySpaceEnvelope,
    invariant_results: &mut Vec<Value>,
    obstructions: &mut Vec<Value>,
) -> AdvisoryResult<()> {
    for hypothesis in space.cells.iter().filter(|cell| is_hypothesis_cell(cell)) {
        let hypothesis_id = json_id(hypothesis);
        let status = hypothesis_status(hypothesis);
        if !metadata_array_non_empty(hypothesis, "/metadata/expected_observations") {
            push_hypothesis_quality_obstruction(
                space,
                invariant_results,
                obstructions,
                hypothesis,
                "hypothesis_missing_expected_observations",
                "record expected observations for the hypothesis",
                "Hypothesis lacks expected observations.",
            )?;
        }
        if !metadata_array_non_empty(hypothesis, "/metadata/falsifiers") {
            push_hypothesis_quality_obstruction(
                space,
                invariant_results,
                obstructions,
                hypothesis,
                "hypothesis_missing_falsifiers",
                "record falsifier observations for the hypothesis",
                "Hypothesis lacks falsifier observations.",
            )?;
        }
        if supported_status(status)
            && !has_argumentation_relation(space, hypothesis_id, &["supports", "supported_by"])
        {
            push_hypothesis_quality_obstruction(
                space,
                invariant_results,
                obstructions,
                hypothesis,
                "supported_hypothesis_missing_support",
                "attach a support incidence from source-backed evidence or downgrade the hypothesis",
                "Hypothesis is marked supported but has no support incidence.",
            )?;
        }
        if status == "falsified"
            && !has_argumentation_relation(space, hypothesis_id, &["falsifies", "falsified_by"])
        {
            push_hypothesis_quality_obstruction(
                space,
                invariant_results,
                obstructions,
                hypothesis,
                "falsified_hypothesis_missing_falsifier",
                "attach a falsifies incidence or change the lifecycle classification",
                "Hypothesis is marked falsified but has no falsifying incidence.",
            )?;
        }
        if status == "strongly_supported"
            && !has_competing_hypothesis_relation(space, hypothesis_id)
        {
            push_hypothesis_quality_obstruction(
                space,
                invariant_results,
                obstructions,
                hypothesis,
                "strong_hypothesis_missing_competition",
                "record at least one competes_with relation or explain why no alternative hypothesis exists",
                "Strongly supported hypothesis has no competing-hypothesis relation.",
            )?;
        }
        if hypothesis
            .pointer("/metadata/refinement_required")
            .and_then(Value::as_bool)
            == Some(true)
            && !hypothesis_has_refinement_context(space, hypothesis_id, hypothesis)
        {
            push_hypothesis_quality_obstruction(
                space,
                invariant_results,
                obstructions,
                hypothesis,
                "hypothesis_refinement_required",
                "create a refined hypothesis with a refines relation before deriving proposals",
                "Hypothesis is marked refinement_required but has no refinement lineage.",
            )?;
        }
    }
    Ok(())
}

pub(super) fn evaluate_proposal_hypothesis_trace(
    space: &AdvisorySpaceEnvelope,
    higher_space: &HigherGraphenAdvisorySpace,
    invariant_results: &mut Vec<Value>,
    obstructions: &mut Vec<Value>,
) -> AdvisoryResult<()> {
    let hypothesis_statuses = space
        .cells
        .iter()
        .filter(|cell| is_hypothesis_cell(cell))
        .map(|cell| {
            (
                json_id(cell).to_string(),
                hypothesis_status(cell).to_string(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    for action in space
        .cells
        .iter()
        .filter(|cell| cell["cell_type"] == "action")
    {
        let action_id = json_id(action);
        let derived_hypothesis_ids = derived_hypothesis_ids(space, action_id, action);
        if derived_hypothesis_ids.is_empty() {
            push_proposal_trace_obstruction(
                space,
                invariant_results,
                obstructions,
                ProposalTraceObstruction {
                    action,
                    obstruction_type: "proposal_missing_hypothesis_trace",
                    resolution: "connect the action to a hypothesis with derives_from before treating it as a recommendation",
                    message_suffix: "Action has no derives_from relation to a hypothesis.",
                    metadata: json!({ "action_id": action_id }),
                },
            )?;
            continue;
        }

        for hypothesis_id in &derived_hypothesis_ids {
            let status = hypothesis_statuses
                .get(hypothesis_id)
                .map(String::as_str)
                .unwrap_or("missing");
            if status == "falsified" || status == "rejected" {
                push_proposal_trace_obstruction(
                    space,
                    invariant_results,
                    obstructions,
                    ProposalTraceObstruction {
                        action,
                        obstruction_type: "proposal_derived_from_falsified_hypothesis",
                        resolution: "remove this primary proposal or reframe it from a non-falsified hypothesis",
                        message_suffix: "Action derives from a falsified or rejected hypothesis.",
                        metadata: json!({ "action_id": action_id, "hypothesis_id": hypothesis_id, "hypothesis_status": status }),
                    },
                )?;
            } else if !primary_action_status_supported(status)
                || (supported_status(status)
                    && !has_argumentation_relation(
                        space,
                        hypothesis_id,
                        &["supports", "supported_by"],
                    ))
            {
                push_proposal_trace_obstruction(
                    space,
                    invariant_results,
                    obstructions,
                    ProposalTraceObstruction {
                        action,
                        obstruction_type: "proposal_derived_from_unsupported_hypothesis",
                        resolution: "collect supporting observations before promoting this action as a primary proposal",
                        message_suffix: "Action derives from a hypothesis that is not supported enough for a primary recommendation.",
                        metadata: json!({ "action_id": action_id, "hypothesis_id": hypothesis_id, "hypothesis_status": status }),
                    },
                )?;
            } else if p0_or_p1(action)
                && matches!(status, "plausible_secondary" | "supported_needs_followup")
            {
                push_proposal_trace_obstruction(
                    space,
                    invariant_results,
                    obstructions,
                    ProposalTraceObstruction {
                        action,
                        obstruction_type: "high_priority_proposal_needs_stronger_hypothesis",
                        resolution: "downgrade the action to follow-up or collect decisive support before P0/P1 promotion",
                        message_suffix: "High-priority action derives from a secondary or follow-up hypothesis.",
                        metadata: json!({ "action_id": action_id, "hypothesis_id": hypothesis_id, "hypothesis_status": status }),
                    },
                )?;
            }
        }

        if p0_or_p1(action)
            && !derived_hypothesis_ids.iter().any(|hypothesis_id| {
                space
                    .cells
                    .iter()
                    .find(|cell| cell.get("id").and_then(Value::as_str) == Some(hypothesis_id))
                    .is_some_and(|hypothesis| {
                        hypothesis_has_refinement_context(space, hypothesis_id, hypothesis)
                    })
            })
        {
            push_proposal_trace_obstruction(
                space,
                invariant_results,
                obstructions,
                ProposalTraceObstruction {
                    action,
                    obstruction_type: "high_priority_proposal_missing_hypothesis_refinement",
                    resolution: "refine the source hypothesis with an explicit refines relation or lower the proposal priority",
                    message_suffix: "High-priority action derives from hypotheses without refinement lineage.",
                    metadata: json!({ "action_id": action_id, "derived_hypothesis_ids": derived_hypothesis_ids }),
                },
            )?;
        }

        if !action_has_required_verification(space, higher_space, action) {
            push_proposal_trace_obstruction(
                space,
                invariant_results,
                obstructions,
                ProposalTraceObstruction {
                    action,
                    obstruction_type: "proposal_missing_verification",
                    resolution: "attach required_verification metadata or a verifies incidence for the action",
                    message_suffix: "Action lacks explicit verification for the proposal.",
                    metadata: json!({ "action_id": action_id, "derived_hypothesis_ids": derived_hypothesis_ids }),
                },
            )?;
        }
    }
    Ok(())
}
