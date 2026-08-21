use super::*;

pub fn hypothesis_falsify_workflow(options: &HypothesisFalsifyOptions) -> AdvisoryResult<Value> {
    hypothesis_lifecycle_event(options, "falsified")
}

pub fn observation_record_workflow(options: &ObservationRecordOptions) -> AdvisoryResult<Value> {
    let head = read_imported_space_head(&options.store, &options.space_id)?;
    ensure_base_revision(Some(&head), options.base_revision.as_deref())?;
    let mut space = read_materialized_space(&options.store, &options.space_id)?;
    let projection = read_json(&options.from_projection)?;
    let task = find_observation_task(&projection, &options.task_id).ok_or_else(|| {
        AdvisoryError::Validation(format!(
            "observation task {} not found in from-projection",
            options.task_id
        ))
    })?;
    let result = read_json(&options.result)?;
    validate_observation_result(&task, &result)?;

    let sequence = next_sequence(&options.store, &options.space_id);
    let target_revision = format!("revision:observation-{sequence:06}");
    let task_slug = advisorygraphen_core::slugify_id(&options.task_id);
    let evidence_id = format!("cell:observation-{task_slug}-{sequence:06}");
    let hypothesis_id = task
        .get("hypothesis_id")
        .and_then(Value::as_str)
        .unwrap_or("hypothesis:unknown");
    let supports = result
        .get("supports_hypothesis")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let falsifies = result
        .get("falsifies_hypothesis")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut metadata = json!({
        "observation_task_id": options.task_id,
        "candidate_id": task.get("candidate_id").cloned().unwrap_or(Value::Null),
        "hypothesis_id": hypothesis_id,
        "observation_status": result.get("observation_status").cloned().unwrap_or(Value::Null),
        "observation_result": result,
        "reviewer": options.reviewer,
        "reason": options.reason
    });
    if supports {
        metadata["supports_hypothesis_id"] = json!(hypothesis_id);
    }
    if falsifies {
        metadata["falsifies_hypothesis_id"] = json!(hypothesis_id);
    }
    let evidence_cell = json!({
        "id": evidence_id,
        "cell_type": "evidence",
        "title": format!("Observation result for {}", options.task_id),
        "summary": result.get("summary").and_then(Value::as_str).unwrap_or("Recorded observation result."),
        "context_ids": [],
        "source_ids": result.get("evidence_ids").cloned().unwrap_or_else(|| json!([])),
        "structure_refs": [options.task_id],
        "provenance": {
            "origin": "source_backed",
            "actor": options.reviewer,
            "confidence": 0.8,
            "review_status": "unreviewed"
        },
        "metadata": metadata
    });
    upsert_by_id(&mut space.cells, evidence_cell.clone());
    advisorygraphen_core::validate_space(&space)?;
    fs::write(
        space_dir(&options.store, &options.space_id).join("materialized/space.json"),
        serde_json::to_vec_pretty(&space)?,
    )?;
    append_store_event(
        &options.store,
        &json!({
            "schema": "advisorygraphen.case.log.entry.v1",
            "case_space_id": options.space_id,
            "sequence": sequence,
            "entry_id": format!("log:{sequence:06}"),
            "morphism_id": format!("morphism:observation-record-{task_slug}"),
            "source_revision_id": head,
            "target_revision_id": target_revision,
            "actor": options.reviewer,
            "recorded_at": Utc::now().to_rfc3339(),
            "previous_entry_hash": null,
            "entry_hash": null,
            "payload": {
                "schema": "advisorygraphen.observation.result.v1",
                "task_id": options.task_id,
                "evidence_cell_id": evidence_id,
                "hypothesis_id": hypothesis_id,
                "result": evidence_cell.pointer("/metadata/observation_result").cloned().unwrap_or(Value::Null)
            }
        }),
    )?;
    fs::write(
        space_dir(&options.store, &options.space_id).join("HEAD"),
        &target_revision,
    )?;
    Ok(json!({
        "schema": "advisorygraphen.report.v1",
        "report_type": "observation_record",
        "report_version": 1,
        "tool": advisorygraphen_core::tool_metadata(None),
        "input": {
            "space_id": options.space_id,
            "task_id": options.task_id,
            "base_revision": options.base_revision
        },
        "result": {
            "recorded": true,
            "case_head_revision": target_revision,
            "evidence_cell": evidence_cell,
            "promotion_gate": observation_promotion_gate(
                hypothesis_id,
                &evidence_id,
                &target_revision,
                supports,
                falsifies,
                options,
            ),
            "suggested_next_commands": observation_next_commands(
                hypothesis_id,
                &evidence_id,
                &target_revision,
                options,
            )
        },
        "projection": {},
        "warnings": []
    }))
}

pub(super) fn find_observation_task(projection: &Value, task_id: &str) -> Option<Value> {
    projection
        .pointer("/recommendation_trace/follow_up_observations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|item| {
            item.get("ranked_observation_tasks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .find(|task| task.get("task_id").and_then(Value::as_str) == Some(task_id))
        .cloned()
}

pub(super) fn validate_observation_result(task: &Value, result: &Value) -> AdvisoryResult<()> {
    let missing = task
        .pointer("/output_schema/required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|field| result.get(*field).is_none())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(AdvisoryError::Validation(format!(
            "observation result missing required fields: {}",
            missing.join(", ")
        )));
    }
    let status = result
        .get("observation_status")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AdvisoryError::Validation("observation_status must be a string".to_string())
        })?;
    if ![
        "supports",
        "falsifies",
        "insufficient_evidence",
        "requires_human_review",
    ]
    .contains(&status)
    {
        return Err(AdvisoryError::Validation(format!(
            "invalid observation_status: {status}"
        )));
    }
    for field in ["supports_hypothesis", "falsifies_hypothesis"] {
        if result.get(field).and_then(Value::as_bool).is_none() {
            return Err(AdvisoryError::Validation(format!(
                "{field} must be a boolean"
            )));
        }
    }
    Ok(())
}

pub(super) fn observation_promotion_gate(
    hypothesis_id: &str,
    evidence_id: &str,
    case_head_revision: &str,
    supports: bool,
    falsifies: bool,
    options: &ObservationRecordOptions,
) -> Value {
    let outcome = if supports {
        "support"
    } else if falsifies {
        "falsify"
    } else {
        "review"
    };
    json!({
        "outcome": outcome,
        "hypothesis_id": hypothesis_id,
        "evidence_cell_id": evidence_id,
        "case_head_revision": case_head_revision,
        "review_required": true,
        "next_command": match outcome {
            "support" => concrete_observation_next_command("support", hypothesis_id, evidence_id, case_head_revision, options),
            "falsify" => concrete_observation_next_command("falsify", hypothesis_id, evidence_id, case_head_revision, options),
            _ => "Review the observation result before supporting or falsifying the hypothesis.".to_string(),
        },
        "rerun_after_review": [
            "advisorygraphen case reason --store STORE --space-id SPACE_ID --format json",
            "advisorygraphen completions apply-accepted --store STORE --space-id SPACE_ID --reviewer REVIEWER --reason REASON --base-revision CASE_HEAD --format json"
        ]
    })
}

pub(super) fn observation_next_commands(
    hypothesis_id: &str,
    evidence_id: &str,
    case_head_revision: &str,
    options: &ObservationRecordOptions,
) -> Value {
    json!({
        "support": concrete_observation_next_command("support", hypothesis_id, evidence_id, case_head_revision, options),
        "falsify": concrete_observation_next_command("falsify", hypothesis_id, evidence_id, case_head_revision, options)
    })
}

pub(super) fn concrete_observation_next_command(
    action: &str,
    hypothesis_id: &str,
    evidence_id: &str,
    case_head_revision: &str,
    options: &ObservationRecordOptions,
) -> String {
    format!(
        "advisorygraphen hypothesis {action} --store {} --from-report CHECK.json --hypothesis-id {} --evidence {} --reviewer {} --reason '{}' --base-revision {} --format json",
        options.store.display(),
        hypothesis_id,
        evidence_id,
        options.reviewer,
        options.reason.replace('\'', " "),
        case_head_revision
    )
}
