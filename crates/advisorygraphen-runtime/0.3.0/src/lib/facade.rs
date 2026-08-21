use super::*;

pub fn project_workflow(options: &ProjectOptions) -> AdvisoryResult<String> {
    let space = read_space(&options.space)?;
    let report = read_projection_report(&options.report, options.completions_report.as_deref())?;
    let rendered = project(&space, &report, &options.audience, options.format)?;
    write_string_if_requested(&options.output, &rendered)?;
    Ok(rendered)
}

pub fn facade_propose_workflow(options: &FacadeProposeOptions) -> AdvisoryResult<ReportEnvelope> {
    ensure_new_case_dir(&options.case_dir)?;

    let input_rel = PathBuf::from("input/advisory.input.json");
    let space_rel = PathBuf::from("artifacts/advisory.space.json");
    let check_rel = PathBuf::from("artifacts/advisory.check.json");
    let completions_rel = PathBuf::from("artifacts/advisory.completions.json");
    let hypothesis_rel = PathBuf::from("artifacts/advisory.hypothesis.json");
    let ai_agent_rel = PathBuf::from("artifacts/projections/ai-agent.json");
    let store_rel = PathBuf::from("store");

    let input_path = options.case_dir.join(&input_rel);
    if let Some(parent) = input_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&options.input, &input_path)?;

    let validation = validate_workflow(&ValidateOptions {
        input: input_path.clone(),
        schema: None,
    })?;
    let space_path = options.case_dir.join(&space_rel);
    let space = lift_workflow(&LiftOptions {
        input: input_path.clone(),
        package: options.package.clone(),
        output: Some(space_path.clone()),
        command: options.command.clone(),
    })?;
    let check_path = options.case_dir.join(&check_rel);
    let check = check_workflow(&CheckOptions {
        space: space_path.clone(),
        ruleset: options.ruleset.clone(),
        output: Some(check_path.clone()),
        fail_on: None,
        command: options.command.clone(),
    })?;
    let completions_path = options.case_dir.join(&completions_rel);
    let completions = completions_propose_workflow(&CompletionProposeOptions {
        space: space_path.clone(),
        from_report: check_path.clone(),
        output: Some(completions_path.clone()),
        command: options.command.clone(),
    })?;
    let hypothesis_path = options.case_dir.join(&hypothesis_rel);
    let hypothesis = hypothesis_propose_workflow(&HypothesisProposeOptions {
        space: space_path.clone(),
        from_report: check_path.clone(),
        output: Some(hypothesis_path.clone()),
        command: options.command.clone(),
    })?;
    let ai_agent_path = options.case_dir.join(&ai_agent_rel);
    let ai_agent_rendered = project_workflow(&ProjectOptions {
        space: space_path.clone(),
        report: check_path.clone(),
        completions_report: Some(completions_path.clone()),
        audience: options.audience.clone(),
        format: advisorygraphen_projection::OutputFormat::Json,
        output: Some(ai_agent_path.clone()),
    })?;
    let ai_agent_projection: Value = serde_json::from_str(&ai_agent_rendered)?;

    let store_path = options.case_dir.join(&store_rel);
    let import = case_import_workflow(&CaseImportOptions {
        store: store_path.clone(),
        space: space_path.clone(),
        revision_id: DEFAULT_FACADE_REVISION.to_string(),
    })?;
    let reasoned = case_reason_workflow(&CaseReasonOptions {
        store: store_path,
        space_id: space.space_id.clone(),
    })?;

    let now = Utc::now().to_rfc3339();
    let manifest = CaseManifest {
        schema: "advisorygraphen.case.manifest.v1".to_string(),
        space_id: space.space_id.clone(),
        package: options.package.clone(),
        ruleset: options.ruleset.clone(),
        store_path: path_to_manifest_string(&store_rel),
        artifacts: CaseArtifacts {
            input: path_to_manifest_string(&input_rel),
            space: path_to_manifest_string(&space_rel),
            check_report: path_to_manifest_string(&check_rel),
            completions_report: path_to_manifest_string(&completions_rel),
            hypothesis_report: path_to_manifest_string(&hypothesis_rel),
            ai_agent_projection: path_to_manifest_string(&ai_agent_rel),
        },
        head_revision: DEFAULT_FACADE_REVISION.to_string(),
        created_at: now.clone(),
        updated_at: now,
    };
    write_case_manifest(&options.case_dir, &manifest)?;

    Ok(ReportEnvelope::new(
        "facade_propose",
        options.command.as_deref(),
        json!({
            "case_dir": options.case_dir,
            "input": options.input,
            "package": options.package,
            "ruleset": options.ruleset,
            "audience": options.audience
        }),
        json!({
            "case_manifest": CASE_MANIFEST_FILE,
            "space_id": space.space_id,
            "head_revision": DEFAULT_FACADE_REVISION,
            "artifacts": manifest.artifacts,
            "validation": validation,
            "check_obstruction_count": check.result.get("obstructions").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "completion_candidate_count": completions.result.get("completion_candidates").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "hypothesis_lifecycle_proposal_count": hypothesis.result.get("lifecycle_proposals").and_then(Value::as_array).map(Vec::len).unwrap_or(0),
            "recommendation_summary": ai_agent_projection.get("recommendation_trace"),
            "close_status": reasoned.pointer("/result/close_status"),
            "waiting_items": reasoned.pointer("/result/waiting_items"),
            "next_commands": [
                format!("advisorygraphen status --case {}", options.case_dir.display()),
                format!("advisorygraphen report --case {} --audience ai_agent", options.case_dir.display())
            ],
            "case_import": import
        }),
    ))
}

pub fn facade_status_workflow(options: &FacadeStatusOptions) -> AdvisoryResult<Value> {
    let manifest = read_case_manifest(&options.case_dir)?;
    let reasoned = case_reason_workflow(&CaseReasonOptions {
        store: options.case_dir.join(&manifest.store_path),
        space_id: manifest.space_id.clone(),
    })?;
    let decision_surface = facade_status_decision_surface(&reasoned);
    let next_commands = json!([
        "advisorygraphen report --case <case> --audience ai_agent",
        "advisorygraphen review completion accept|reject --case <case> --candidate-id <id> --reviewer <id> --reason <reason>"
    ]);
    let result = if options.brief {
        json!({
            "case_head_revision": reasoned.pointer("/result/case_head_revision"),
            "summary": decision_surface.get("summary"),
            "top_blockers": decision_surface.get("top_blockers"),
            "next_best_action": decision_surface.get("next_best_action"),
            "next_commands": next_commands
        })
    } else {
        json!({
            "case_head_revision": reasoned.pointer("/result/case_head_revision"),
            "summary": decision_surface.get("summary"),
            "top_blockers": decision_surface.get("top_blockers"),
            "next_best_action": decision_surface.get("next_best_action"),
            "close_status": reasoned.pointer("/result/close_status"),
            "blockers": reasoned.pointer("/result/blockers"),
            "frontier_items": reasoned.pointer("/result/frontier_items"),
            "waiting_items": reasoned.pointer("/result/waiting_items"),
            "next_commands": next_commands
        })
    };
    Ok(json!({
        "schema": "advisorygraphen.report.v1",
        "report_type": "facade_status",
        "report_version": 1,
        "tool": advisorygraphen_core::tool_metadata(None),
        "input": {
            "case_dir": options.case_dir,
            "space_id": manifest.space_id,
            "manifest": CASE_MANIFEST_FILE,
            "brief": options.brief
        },
        "result": result,
        "projection": {},
        "warnings": []
    }))
}

pub(super) fn facade_status_decision_surface(reasoned: &Value) -> Value {
    let close_status = reasoned
        .pointer("/result/close_status")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let closeable = close_status
        .get("closeable")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let blockers = reasoned
        .pointer("/result/blockers")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let frontier_items = reasoned
        .pointer("/result/frontier_items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let waiting_items = reasoned
        .pointer("/result/waiting_items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut ranked_blockers = blockers
        .iter()
        .enumerate()
        .collect::<Vec<(usize, &Value)>>();
    ranked_blockers.sort_by(|(left_index, left), (right_index, right)| {
        blocker_severity_rank(right)
            .cmp(&blocker_severity_rank(left))
            .then_with(|| left_index.cmp(right_index))
    });
    let top_blockers = ranked_blockers
        .into_iter()
        .take(3)
        .map(|(_, blocker)| summarize_blocker(blocker))
        .collect::<Vec<_>>();
    let status_label = if closeable {
        "closeable"
    } else if !waiting_items.is_empty() {
        "blocked_waiting_on_review"
    } else if !frontier_items.is_empty() {
        "blocked_agent_actionable"
    } else if !blockers.is_empty() {
        "blocked_needs_source_or_review"
    } else {
        "unknown"
    };
    json!({
        "summary": {
            "status_label": status_label,
            "case_head_revision": reasoned.pointer("/result/case_head_revision"),
            "closeable": closeable,
            "blocking_threshold": close_status.get("blocking_threshold"),
            "blocker_count": blockers.len(),
            "waiting_count": waiting_items.len(),
            "frontier_count": frontier_items.len(),
            "top_blocker_count": top_blockers.len()
        },
        "top_blockers": top_blockers,
        "next_best_action": next_best_facade_action(
            closeable,
            waiting_items.first(),
            frontier_items.first(),
            blockers.first()
        )
    })
}

pub(super) fn blocker_severity_rank(blocker: &Value) -> u8 {
    match blocker.get("severity").and_then(Value::as_str) {
        Some("critical") => 5,
        Some("high") => 4,
        Some("medium") => 3,
        Some("low") => 2,
        Some("info") => 1,
        _ => 0,
    }
}

pub(super) fn summarize_blocker(blocker: &Value) -> Value {
    json!({
        "id": blocker.get("id"),
        "severity": blocker.get("severity"),
        "type": blocker.get("obstruction_type"),
        "message": blocker.get("message"),
        "blocked_ids": blocker.get("blocked_ids").cloned().unwrap_or_else(|| json!([])),
        "recommended_completion_types": blocker
            .get("recommended_completion_types")
            .cloned()
            .unwrap_or_else(|| json!([])),
        "review_status": blocker.get("review_status")
    })
}

pub(super) fn next_best_facade_action(
    closeable: bool,
    first_waiting: Option<&Value>,
    first_frontier: Option<&Value>,
    first_blocker: Option<&Value>,
) -> Value {
    if closeable {
        return json!({
            "action_type": "report_or_close",
            "reason": "No blocking obstructions remain at the configured threshold.",
            "command": "advisorygraphen report --case <case> --audience ai_agent",
            "target_ids": []
        });
    }

    if let Some(waiting) = first_waiting {
        let candidate_ids = waiting
            .get("candidate_ids")
            .cloned()
            .unwrap_or_else(|| json!([]));
        let target_ids = if candidate_ids.as_array().is_some_and(|ids| !ids.is_empty()) {
            candidate_ids.clone()
        } else {
            waiting
                .get("obstruction_id")
                .map(|id| json!([id]))
                .unwrap_or_else(|| json!([]))
        };
        return json!({
            "action_type": "review_pending_candidate",
            "reason": waiting
                .get("waiting_on")
                .and_then(Value::as_str)
                .unwrap_or("A blocker has candidate structure waiting on explicit review."),
            "command": "advisorygraphen review completion accept|reject --case <case> --candidate-id <candidate-id> --reviewer <id> --reason <reason>",
            "target_ids": target_ids,
            "candidate_ids": candidate_ids,
            "obstruction_id": waiting.get("obstruction_id")
        });
    }

    if let Some(frontier) = first_frontier {
        return json!({
            "action_type": "advance_frontier",
            "reason": frontier
                .get("next_operation")
                .and_then(Value::as_str)
                .unwrap_or("Agent-actionable frontier work is available."),
            "command": "advisorygraphen report --case <case> --audience ai_agent",
            "target_ids": frontier
                .get("candidate_ids")
                .cloned()
                .unwrap_or_else(|| json!([])),
            "obstruction_id": frontier.get("obstruction_id")
        });
    }

    if let Some(blocker) = first_blocker {
        return json!({
            "action_type": "inspect_blocker",
            "reason": blocker
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("A blocker remains but no candidate or frontier action is available."),
            "command": "advisorygraphen report --case <case> --audience ai_agent",
            "target_ids": blocker
                .get("blocked_ids")
                .cloned()
                .unwrap_or_else(|| json!([])),
            "obstruction_id": blocker.get("id")
        });
    }

    json!({
        "action_type": "inspect_report",
        "reason": "Status did not expose a closeable state, blocker, waiting item, or frontier item.",
        "command": "advisorygraphen report --case <case> --audience ai_agent",
        "target_ids": []
    })
}
