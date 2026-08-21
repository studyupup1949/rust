use super::*;

pub fn hypothesis_propose_workflow(
    options: &HypothesisProposeOptions,
) -> AdvisoryResult<ReportEnvelope> {
    let space = read_space(&options.space)?;
    let check_report: ReportEnvelope = serde_json::from_value(read_json(&options.from_report)?)?;
    let report = propose_hypothesis_lifecycle(
        &space,
        &check_report,
        file_name(&options.from_report),
        options.command.as_deref(),
    )?;
    write_json_if_requested(&options.output, &report)?;
    Ok(report)
}

pub fn hypothesis_apply_proposals_workflow(
    options: &HypothesisApplyProposalsOptions,
) -> AdvisoryResult<Value> {
    fs::create_dir_all(&options.store)?;
    let proposal_report = read_json(&options.from_report)?;
    if proposal_report.get("report_type").and_then(Value::as_str)
        != Some("hypothesis_lifecycle_proposal")
    {
        return Err(AdvisoryError::Validation(
            "from-report must be a hypothesis_lifecycle_proposal report".to_string(),
        ));
    }
    let space_id = proposal_report
        .pointer("/input/space_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AdvisoryError::Validation(
                "from-report must contain input.space_id for hypothesis proposal application"
                    .to_string(),
            )
        })?
        .to_string();
    let policy = read_autonomy_policy(options.policy.as_deref())?;
    let initial_head = read_imported_space_head(&options.store, &space_id)?;
    let materialized_space = read_materialized_space(&options.store, &space_id)?;
    ensure_base_revision(Some(&initial_head), options.base_revision.as_deref())?;

    let proposals = proposal_report
        .pointer("/result/lifecycle_proposals")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut applied = Vec::new();
    let mut skipped = Vec::new();
    let mut current_head = initial_head.clone();

    for proposal in proposals {
        let decision = autonomy_decision(&proposal, &policy);
        if !decision.allowed {
            skipped.push(application_skip(&proposal, decision.reason));
            continue;
        }
        if applied.len() >= policy.max_events {
            skipped.push(application_skip(
                &proposal,
                format!("policy max_events {} reached", policy.max_events),
            ));
            continue;
        }
        let event = hypothesis_event_from_proposal(
            &materialized_space.engagement_id,
            &proposal,
            &options.reviewer,
            &options.reason,
            &options.from_report,
            Some(&current_head),
            applied.len() + 1,
        )?;
        if !options.dry_run {
            let sequence = next_sequence(&options.store, &space_id);
            let target_revision = format!("revision:hypothesis-auto-{sequence:06}");
            let hypothesis_slug = event["target_hypothesis_id"]
                .as_str()
                .unwrap_or("hypothesis:unknown")
                .trim_start_matches("hypothesis:")
                .to_string();
            append_store_event(
                &options.store,
                &json!({
                    "schema": "advisorygraphen.case.log.entry.v1",
                    "case_space_id": space_id.clone(),
                    "sequence": sequence,
                    "entry_id": format!("log:{sequence:06}"),
                    "morphism_id": format!("morphism:hypothesis-auto-{}-{hypothesis_slug}", event["outcome"].as_str().unwrap_or("unknown")),
                    "source_revision_id": current_head,
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
            current_head = target_revision;
        }
        applied.push(event);
    }
    let post_apply_case_reason = if options.dry_run || applied.is_empty() {
        json!(null)
    } else {
        let reasoned = case_reason_workflow(&CaseReasonOptions {
            store: options.store.clone(),
            space_id: space_id.clone(),
        })?;
        json!({
            "case_head_revision": reasoned.pointer("/result/case_head_revision"),
            "close_status": reasoned.pointer("/result/close_status"),
            "frontier_items": reasoned.pointer("/result/frontier_items"),
            "waiting_items": reasoned.pointer("/result/waiting_items")
        })
    };

    Ok(json!({
        "schema": "advisorygraphen.report.v1",
        "report_type": "hypothesis_lifecycle_apply_proposals",
        "report_version": 1,
        "tool": advisorygraphen_core::tool_metadata(None),
        "input": {
            "space_id": space_id,
            "from_report": options.from_report,
            "policy": options.policy,
            "base_revision": options.base_revision,
            "dry_run": options.dry_run
        },
        "result": {
            "applied_count": applied.len(),
            "skipped_count": skipped.len(),
            "applied_events": applied,
            "skipped_proposals": skipped,
            "initial_head_revision": initial_head,
            "case_head_revision": current_head,
            "policy": policy.as_json(),
            "post_apply_case_reason": post_apply_case_reason
        },
        "projection": {},
        "warnings": []
    }))
}
