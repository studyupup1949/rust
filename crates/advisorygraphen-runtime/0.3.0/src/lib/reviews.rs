use super::*;

pub fn review_workflow(options: &ReviewOptions) -> AdvisoryResult<Value> {
    fs::create_dir_all(&options.store)?;
    let from_report = review_report_path(options)?;
    let space_id = review_space_id(from_report)?;
    let reviewed_at = Utc::now().to_rfc3339();
    let higher_graphen_review =
        higher_graphen_completion_review(options, from_report, &reviewed_at)?;
    let head = read_imported_space_head(&options.store, &space_id)?;
    let materialized_space = read_materialized_space(&options.store, &space_id)?;
    ensure_base_revision(Some(&head), options.base_revision.as_deref())?;
    let gluing_policy = review_gluing_policy(
        &materialized_space,
        from_report,
        &options.candidate_id,
        &options.outcome,
    )?;
    let sequence = next_sequence(&options.store, &space_id);
    let target_revision = format!("revision:review-{sequence:06}");
    let candidate_slug = options.candidate_id.trim_start_matches("candidate:");
    let review_event_id = format!("review:{}:{candidate_slug}-{sequence:06}", options.outcome);
    let event = json!({
        "schema": REVIEW_EVENT_SCHEMA,
        "review_event_id": review_event_id,
        "engagement_id": materialized_space.engagement_id,
        "target_ids": [options.candidate_id],
        "outcome": options.outcome,
        "reviewer_id": options.reviewer,
        "reviewed_at": reviewed_at,
        "reason": options.reason,
        "evidence_ids": [],
        "base_revision_id": options.base_revision,
        "metadata": {
            "from_report": options.from_report.as_ref().map(|path| path.display().to_string()),
            "higher_graphen": higher_graphen_review,
            "higher_graphen_gluing_policy": gluing_policy
        }
    });
    validate_document(&event, Some(REVIEW_EVENT_SCHEMA))?;
    append_store_event(
        &options.store,
        &json!({
            "schema": "advisorygraphen.case.log.entry.v1",
            "case_space_id": space_id.clone(),
            "sequence": sequence,
            "entry_id": format!("log:{sequence:06}"),
            "morphism_id": format!("morphism:{}-{candidate_slug}", options.outcome),
            "source_revision_id": head,
            "target_revision_id": target_revision.clone(),
            "actor": event["reviewer_id"],
            "recorded_at": Utc::now().to_rfc3339(),
            "previous_entry_hash": null,
            "entry_hash": null,
            "payload": event
        }),
    )?;
    fs::write(
        space_dir(&options.store, &space_id).join("HEAD"),
        &target_revision,
    )?;
    Ok(event)
}

pub(super) fn review_gluing_policy(
    space: &AdvisorySpaceEnvelope,
    from_report: &Path,
    candidate_id: &str,
    outcome: &str,
) -> AdvisoryResult<Value> {
    let report = read_json(from_report)?;
    let candidate = candidate_from_report(&report, candidate_id)?.clone();
    let before_check = check_space(space, "technical_advisory_mvp", None, None)?;
    let before_obstructions = before_check
        .result
        .get("obstructions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let review = match materialize_candidate_dry_run(space, &before_obstructions, &candidate) {
        DryRunMaterialization::Applied {
            cells,
            incidences,
            removed_incidence_ids,
        } => {
            let mut dry_space = space.clone();
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
            dry_run_gluing::candidate_gluing_review(
                space,
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
                &removed_incidence_ids,
            )?
        }
        DryRunMaterialization::Skipped { reason } => {
            dry_run_gluing::skipped_candidate_gluing_review(&candidate, &reason)
        }
    };
    let policy_blockers = dry_run_gluing::policy_blockers(&review);
    Ok(json!({
        "schema": "advisorygraphen.completion_review.gluing_policy.v1",
        "outcome": outcome,
        "explicit_review_recorded": true,
        "policy_blockers": policy_blockers,
        "policy_override": if outcome == "accepted" && !policy_blockers.is_empty() {
            json!("explicit_completion_review")
        } else {
            json!(null)
        },
        "dry_run_gluing_review": review
    }))
}

pub(super) fn application_gluing_review(
    space: &AdvisorySpaceEnvelope,
    candidate: &Value,
    before_obstructions: &[Value],
    cells: &[Value],
    incidences: &[Value],
) -> AdvisoryResult<Value> {
    let mut dry_space = space.clone();
    for cell in cells {
        upsert_by_id(&mut dry_space.cells, cell.clone());
    }
    for incidence in incidences {
        upsert_by_id(&mut dry_space.incidences, incidence.clone());
    }
    advisorygraphen_core::validate_space(&dry_space)?;
    let after_check = check_space(&dry_space, "technical_advisory_mvp", None, None)?;
    dry_run_gluing::candidate_gluing_review(
        space,
        candidate,
        before_obstructions,
        after_check
            .result
            .get("obstructions")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or(&[]),
        cells,
        incidences,
        &[],
    )
}

pub(super) fn reviewed_policy_override(
    candidate: &Value,
    policy_blockers: &[Value],
) -> AdvisoryResult<Value> {
    if policy_blockers.is_empty() {
        return Ok(json!(null));
    }
    let candidate_id = json_id(candidate);
    let Some(latest_review) = candidate.pointer("/metadata/latest_review") else {
        return Err(AdvisoryError::Validation(format!(
            "candidate {candidate_id} has gluing policy blockers but no review event summary"
        )));
    };
    if latest_review.get("outcome").and_then(Value::as_str) != Some("accepted") {
        return Err(AdvisoryError::Validation(format!(
            "candidate {candidate_id} has gluing policy blockers but was not accepted"
        )));
    }
    let override_value = latest_review
        .pointer("/higher_graphen_gluing_policy/policy_override")
        .cloned()
        .unwrap_or(Value::Null);
    if override_value != json!("explicit_completion_review") {
        return Err(AdvisoryError::Validation(format!(
            "candidate {candidate_id} has gluing policy blockers but review event did not record policy_override explicit_completion_review"
        )));
    }
    Ok(override_value)
}

pub(super) fn candidate_from_report<'a>(
    report: &'a Value,
    candidate_id: &str,
) -> AdvisoryResult<&'a Value> {
    let candidates = report
        .pointer("/result/completion_candidates")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AdvisoryError::Validation(
                "from-report must contain result.completion_candidates".to_string(),
            )
        })?;
    candidates
        .iter()
        .find(|candidate| candidate.get("id").and_then(Value::as_str) == Some(candidate_id))
        .ok_or_else(|| {
            AdvisoryError::Validation(format!("candidate {candidate_id} not found in from-report"))
        })
}
