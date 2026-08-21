use super::*;

pub fn validate_workflow(options: &ValidateOptions) -> AdvisoryResult<Value> {
    let value = read_json(&options.input)?;
    let schema = options.schema.as_deref().map(canonical_schema_name);
    let report = validate_document(&value, schema.as_deref())?;
    Ok(serde_json::to_value(report)?)
}
pub fn lift_workflow(options: &LiftOptions) -> AdvisoryResult<AdvisorySpaceEnvelope> {
    let snapshot = read_json(&options.input)?;
    let package = InterpretationPackage::load(&options.package)?;
    let space = lift_snapshot(&snapshot, &package)?;
    write_json_if_requested(&options.output, &space)?;
    let _ = &options.command;
    Ok(space)
}
pub fn check_workflow(options: &CheckOptions) -> AdvisoryResult<ReportEnvelope> {
    let space = read_space(&options.space)?;
    let report = check_space(
        &space,
        &options.ruleset,
        options.fail_on,
        options.command.as_deref(),
    )?;
    write_json_if_requested(&options.output, &report)?;
    Ok(report)
}

pub fn micro_review_workflow(options: &MicroReviewOptions) -> AdvisoryResult<ReportEnvelope> {
    let request = read_json(&options.input)?;
    advisorygraphen_core::validate_document(
        &request,
        Some(advisorygraphen_core::MICRO_REVIEW_REQUEST_SCHEMA),
    )?;
    let result = micro_review::analyze(&request);
    let report = ReportEnvelope::new(
        "micro_review",
        options.command.as_deref(),
        json!({
            "input": options.input,
            "mode": "small_scope_ai_answer_review"
        }),
        result,
    );
    write_json_if_requested(&options.output, &report)?;
    Ok(report)
}

pub fn completions_propose_workflow(
    options: &CompletionProposeOptions,
) -> AdvisoryResult<ReportEnvelope> {
    let space = read_space(&options.space)?;
    let check_report: ReportEnvelope = serde_json::from_value(read_json(&options.from_report)?)?;
    let report = propose_completions(
        &space,
        &check_report,
        file_name(&options.from_report),
        options.command.as_deref(),
    )?;
    write_json_if_requested(&options.output, &report)?;
    Ok(report)
}

pub fn completions_dry_run_workflow(
    options: &CompletionDryRunOptions,
) -> AdvisoryResult<ReportEnvelope> {
    let space = read_space(&options.space)?;
    let completion_report: ReportEnvelope =
        serde_json::from_value(read_json(&options.from_report)?)?;
    if completion_report.report_type != "completion_proposal" {
        return Err(AdvisoryError::Validation(
            "from-report must be a completion_proposal report".to_string(),
        ));
    }
    let before_check = check_space(&space, "technical_advisory_mvp", None, None)?;
    let before_obstructions = before_check
        .result
        .get("obstructions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let candidates = completion_report
        .result
        .get("completion_candidates")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let selected = candidates
        .iter()
        .filter(|candidate| {
            options.candidate_ids.is_empty()
                || options
                    .candidate_ids
                    .iter()
                    .any(|id| candidate.get("id").and_then(Value::as_str) == Some(id.as_str()))
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut dry_runs = Vec::new();

    for candidate in selected {
        let candidate_id = json_id(&candidate).to_string();
        let mut dry_space = space.clone();
        let materialization =
            materialize_candidate_dry_run(&dry_space, &before_obstructions, &candidate);
        match materialization {
            DryRunMaterialization::Applied {
                cells,
                incidences,
                removed_incidence_ids,
            } => {
                for incidence_id in &removed_incidence_ids {
                    dry_space
                        .incidences
                        .retain(|incidence| json_id(incidence) != incidence_id);
                }
                for cell in &cells {
                    upsert_by_id(&mut dry_space.cells, cell.clone());
                }
                for incidence in &incidences {
                    upsert_by_id(&mut dry_space.incidences, incidence.clone());
                }
                advisorygraphen_core::validate_space(&dry_space)?;
                let after_check = check_space(&dry_space, "technical_advisory_mvp", None, None)?;
                let before_ids = obstruction_ids(&before_check);
                let after_ids = obstruction_ids(&after_check);
                let resolved_ids = before_ids
                    .iter()
                    .filter(|id| !after_ids.contains(*id))
                    .cloned()
                    .collect::<Vec<_>>();
                let introduced_ids = after_ids
                    .iter()
                    .filter(|id| !before_ids.contains(*id))
                    .cloned()
                    .collect::<Vec<_>>();
                dry_runs.push(json!({
                    "candidate_id": candidate_id,
                    "candidate_type": candidate.get("candidate_type"),
                    "status": "applied_to_dry_run_space",
                    "application_plan": candidate.get("application_plan"),
                    "applied_structure": {
                        "cell_ids": ids_of(&cells),
                        "incidence_ids": ids_of(&incidences),
                        "removed_incidence_ids": removed_incidence_ids
                    },
                    "check_delta": {
                        "before_obstruction_ids": before_ids,
                        "after_obstruction_ids": after_ids,
                        "resolved_obstruction_ids": resolved_ids,
                        "introduced_obstruction_ids": introduced_ids
                    },
                    "after_close_status": close_status(&dry_space, &after_check),
                    "higher_graphen_gluing_review": dry_run_gluing::candidate_gluing_review(
                        &space,
                        &candidate,
                        &before_obstructions,
                        after_check
                            .result
                            .get("obstructions")
                            .and_then(Value::as_array)
                            .map(Vec::as_slice)
                            .unwrap_or(&[]),
                        &cells,
                        &incidences,
                        &removed_incidence_ids
                    )?
                }));
            }
            DryRunMaterialization::Skipped { reason } => {
                dry_runs.push(json!({
                    "candidate_id": candidate_id,
                    "candidate_type": candidate.get("candidate_type"),
                    "status": "skipped",
                    "reason": reason,
                    "application_plan": candidate.get("application_plan"),
                    "higher_graphen_gluing_review": dry_run_gluing::skipped_candidate_gluing_review(
                        &candidate,
                        &reason
                    )
                }));
            }
        }
    }

    let report = ReportEnvelope::new(
        "completion_dry_run",
        options.command.as_deref(),
        json!({
            "space_id": space.space_id,
            "from_report": file_name(&options.from_report),
            "candidate_ids": options.candidate_ids
        }),
        json!({
            "dry_runs": dry_runs,
            "candidate_count": candidates.len()
        }),
    );
    write_json_if_requested(&options.output, &report)?;
    Ok(report)
}
