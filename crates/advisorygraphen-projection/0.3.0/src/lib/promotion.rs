use super::*;

pub(super) fn hypothesis_promotion_workflow(recommendation_trace: &Value) -> Value {
    let items: Vec<Value> = recommendation_trace
        .get("follow_up_observations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|item| {
            let observation_task_ids: Vec<Value> = item
                .get("ranked_observation_tasks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|task| task.get("task_id").cloned())
                .collect();
            json!({
                "candidate_id": item.get("candidate_id").cloned().unwrap_or(Value::Null),
                "current_role": item.get("recommendation_role").cloned().unwrap_or_else(|| json!("follow_up_observation")),
                "blocking_hypothesis_ids": item.get("unsupported_hypothesis_ids").cloned().unwrap_or_else(|| json!([])),
                "observation_task_ids": observation_task_ids,
                "next_command_drafts": promotion_command_drafts(item),
                "promotion_steps": [
                    "Run the ranked observation tasks against the bounded source snapshot.",
                    "Record source-backed evidence that supports, weakens, or falsifies the blocking hypothesis.",
                    "Use a review-gated hypothesis support or accept command only after evidence exists.",
                    "Rerun completions propose and project ai_agent; promote only if the candidate no longer has unsupported hypotheses or content obstructions."
                ]
            })
        })
        .collect();

    json!({
        "workflow_rule": "Follow-up observations become primary recommendations only through source-backed hypothesis review and a fresh projection.",
        "review_gated_commands": [
            "hypothesis support",
            "hypothesis accept",
            "completions accept"
        ],
        "item_count": items.len(),
        "items": items
    })
}

pub(super) fn promotion_command_drafts(item: &Value) -> Value {
    let task_ids: Vec<Value> = item
        .get("ranked_observation_tasks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|task| task.get("task_id").cloned())
        .collect();
    let hypothesis_id = item
        .get("unsupported_hypothesis_ids")
        .and_then(Value::as_array)
        .and_then(|ids| ids.first())
        .and_then(Value::as_str)
        .unwrap_or("<hypothesis-id>");
    json!({
        "record_observation": {
            "command": "advisorygraphen observation record --store STORE --space-id SPACE_ID --from-projection AI_AGENT.json --task-id TASK_ID --result OBSERVATION_RESULT.json --reviewer REVIEWER --reason REASON --base-revision REVISION --format json",
            "task_ids": task_ids
        },
        "support_hypothesis": {
            "command": format!("advisorygraphen hypothesis support --store STORE --from-report CHECK.json --hypothesis-id {hypothesis_id} --evidence EVIDENCE_CELL_ID --reviewer REVIEWER --reason REASON --base-revision REVISION --format json"),
            "requires": [
                "observation result with observation_status=supports",
                "evidence cell produced by observation record"
            ]
        },
        "falsify_hypothesis": {
            "command": format!("advisorygraphen hypothesis falsify --store STORE --from-report CHECK.json --hypothesis-id {hypothesis_id} --evidence EVIDENCE_CELL_ID --reviewer REVIEWER --reason REASON --base-revision REVISION --format json"),
            "requires": [
                "observation result with observation_status=falsifies",
                "evidence cell produced by observation record"
            ]
        }
    })
}

pub(super) fn id_tail(id: &str) -> String {
    id.rsplit(':').next().unwrap_or(id).replace('_', "-")
}

pub(super) fn id_fragment(id: &str) -> String {
    id.trim_start_matches("observation:")
        .replace([':', '_'], "-")
}

pub(super) fn close_status_value(space: &AdvisorySpaceEnvelope, report: &Value) -> Value {
    let envelope = serde_json::from_value(report.clone()).unwrap_or_else(|_| {
        advisorygraphen_core::ReportEnvelope::new("check", None, json!({}), json!({}))
    });
    close_status(space, &envelope)
}
