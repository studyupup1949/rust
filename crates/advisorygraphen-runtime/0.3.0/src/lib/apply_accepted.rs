use super::*;

pub fn completions_apply_accepted_workflow(
    options: &CompletionApplyAcceptedOptions,
) -> AdvisoryResult<Value> {
    fs::create_dir_all(&options.store)?;
    let head = read_imported_space_head(&options.store, &options.space_id)?;
    ensure_base_revision(Some(&head), options.base_revision.as_deref())?;
    let mut space = read_materialized_space(&options.store, &options.space_id)?;
    let check = check_space(&space, "technical_advisory_mvp", None, None)?;
    let blockers = check
        .result
        .get("obstructions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut completions = propose_completions(&space, &check, "apply_accepted", None)?;
    let mut candidates = completions
        .result
        .get("completion_candidates")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    apply_candidate_reviews(&options.store, &options.space_id, &mut candidates)?;
    completions.result["completion_candidates"] = json!(candidates.clone());
    let resolution_state = blocker_resolution_state(&blockers, &candidates);
    let mut applied_structures = Vec::new();
    let mut skipped_candidates = Vec::new();

    for item in &resolution_state {
        if item.get("resolution_status").and_then(Value::as_str)
            != Some("accepted_candidate_pending_application")
        {
            continue;
        }
        let obstruction_id = item
            .get("obstruction_id")
            .and_then(Value::as_str)
            .unwrap_or("obstruction:unknown");
        let Some(blocker) = blockers
            .iter()
            .find(|blocker| blocker.get("id").and_then(Value::as_str) == Some(obstruction_id))
        else {
            continue;
        };
        for candidate_id in item
            .get("accepted_candidate_ids")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            let Some(candidate) = candidates.iter().find(|candidate| {
                candidate.get("id").and_then(Value::as_str) == Some(candidate_id)
            }) else {
                continue;
            };
            match materialize_candidate_structure(&space, blocker, candidate, &options.reviewer) {
                Materialization::Applied { cells, incidences } => {
                    let gluing_review = application_gluing_review(
                        &space,
                        candidate,
                        &blockers,
                        &cells,
                        &incidences,
                    )?;
                    let policy_blockers = dry_run_gluing::policy_blockers(&gluing_review);
                    let policy_override = reviewed_policy_override(candidate, &policy_blockers)?;
                    if options.dry_run {
                        applied_structures.push(json!({
                            "candidate_id": candidate_id,
                            "dry_run": true,
                            "cells": cells,
                            "incidences": incidences,
                            "higher_graphen_gluing_review": gluing_review,
                            "policy_blockers": policy_blockers,
                            "policy_override": policy_override
                        }));
                    } else {
                        for cell in &cells {
                            upsert_by_id(&mut space.cells, cell.clone());
                        }
                        for incidence in &incidences {
                            upsert_by_id(&mut space.incidences, incidence.clone());
                        }
                        applied_structures.push(json!({
                            "candidate_id": candidate_id,
                            "dry_run": false,
                            "cell_ids": ids_of(&cells),
                            "incidence_ids": ids_of(&incidences),
                            "higher_graphen_gluing_review": gluing_review,
                            "policy_blockers": policy_blockers,
                            "policy_override": policy_override
                        }));
                    }
                }
                Materialization::Skipped { reason } => {
                    skipped_candidates.push(json!({
                        "candidate_id": candidate_id,
                        "candidate_type": candidate.get("candidate_type"),
                        "reason": reason
                    }));
                }
            }
        }
    }

    let mut current_head = head.clone();
    if !options.dry_run && !applied_structures.is_empty() {
        advisorygraphen_core::validate_space(&space)?;
        let sequence = next_sequence(&options.store, &options.space_id);
        let target_revision = format!("revision:completion-apply-{sequence:06}");
        let event = json!({
            "schema": "advisorygraphen.completion.application.v1",
            "application_event_id": format!("completion-application:{sequence:06}"),
            "engagement_id": space.engagement_id,
            "reviewer_id": options.reviewer,
            "reviewed_at": Utc::now().to_rfc3339(),
            "reason": options.reason,
            "base_revision_id": options.base_revision,
            "applied_structures": applied_structures,
            "skipped_candidates": skipped_candidates
        });
        append_store_event(
            &options.store,
            &json!({
                "schema": "advisorygraphen.case.log.entry.v1",
                "case_space_id": options.space_id.clone(),
                "sequence": sequence,
                "entry_id": format!("log:{sequence:06}"),
                "morphism_id": format!("morphism:completion-apply-{sequence:06}"),
                "source_revision_id": head,
                "target_revision_id": target_revision.clone(),
                "actor": options.reviewer,
                "recorded_at": Utc::now().to_rfc3339(),
                "previous_entry_hash": null,
                "entry_hash": null,
                "payload": event
            }),
        )?;
        fs::write(
            space_dir(&options.store, &options.space_id).join("materialized/space.json"),
            serde_json::to_vec_pretty(&space)?,
        )?;
        fs::write(
            space_dir(&options.store, &options.space_id).join("HEAD"),
            &target_revision,
        )?;
        current_head = target_revision;
    }

    let post_apply_case_reason = if options.dry_run || applied_structures.is_empty() {
        json!(null)
    } else {
        let reasoned = case_reason_workflow(&CaseReasonOptions {
            store: options.store.clone(),
            space_id: options.space_id.clone(),
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
        "report_type": "completion_apply_accepted",
        "report_version": 1,
        "tool": advisorygraphen_core::tool_metadata(None),
        "input": {
            "space_id": options.space_id,
            "base_revision": options.base_revision,
            "dry_run": options.dry_run
        },
        "result": {
            "applied_count": applied_structures.len(),
            "skipped_count": skipped_candidates.len(),
            "applied_structures": applied_structures,
            "skipped_candidates": skipped_candidates,
            "initial_head_revision": head,
            "case_head_revision": current_head,
            "post_apply_case_reason": post_apply_case_reason,
            "supported_candidate_types": [
                "ownership_clarification",
                "proposed_test"
            ]
        },
        "projection": {},
        "warnings": []
    }))
}
