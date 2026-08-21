use super::*;

#[test]
fn accepted_completion_candidate_materializes_required_owner_structure() {
    let temp = TempDir::new().unwrap();
    let space_path = temp.path().join("space.json");
    let check_path = temp.path().join("check.json");
    let completions_path = temp.path().join("completions.json");
    let store_path = temp.path().join("store");
    let space = json!({
        "schema": "advisorygraphen.space.v1",
        "space_id": "space:completion-application-test",
        "engagement_id": "engagement:completion-application-test",
        "snapshot_id": "snapshot:completion-application-test",
        "package_id": "package:technical_advisory_mvp",
        "cells": [
            {
                "id": "cell:ship-release-action",
                "cell_type": "action",
                "title": "Ship release action",
                "summary": "Release action needs an owner before the case can close.",
                "context_ids": [],
                "source_ids": ["source:test-plan"],
                "structure_refs": [],
                "provenance": {
                    "origin": "source_backed",
                    "actor": "test",
                    "confidence": 1.0,
                    "review_status": "accepted"
                },
                "metadata": {}
            }
        ],
        "contexts": [],
        "incidences": [],
        "morphisms": [],
        "invariants": [],
        "policies": [],
        "metadata": {}
    });
    fs::write(&space_path, serde_json::to_vec_pretty(&space).unwrap()).unwrap();

    let check = check_workflow(&CheckOptions {
        space: space_path.clone(),
        ruleset: "technical_advisory_mvp".to_string(),
        output: Some(check_path.clone()),
        fail_on: None,
        command: None,
    })
    .unwrap();
    assert_eq!(
        check.result["obstructions"][0]["id"],
        "obstruction:ship-release-action-missing-owner"
    );

    let completions = completions_propose_workflow(&CompletionProposeOptions {
        space: space_path.clone(),
        from_report: check_path.clone(),
        output: Some(completions_path.clone()),
        command: None,
    })
    .unwrap();
    let candidate_id = completions.result["completion_candidates"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        candidate_id,
        "candidate:ship-release-action-missing-owner-owner"
    );
    let dry_run = completions_dry_run_workflow(&CompletionDryRunOptions {
        space: space_path.clone(),
        from_report: completions_path.clone(),
        candidate_ids: vec![candidate_id.clone()],
        output: None,
        command: None,
    })
    .unwrap();
    assert_eq!(dry_run.report_type, "completion_dry_run");
    let run = &dry_run.result["dry_runs"][0];
    assert_eq!(run["status"], "applied_to_dry_run_space");
    assert_eq!(
        run["applied_structure"]["cell_ids"][0],
        "cell:auto-owner-ship-release-action"
    );
    assert!(run["check_delta"]["resolved_obstruction_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id == "obstruction:ship-release-action-missing-owner"));
    assert_eq!(run["after_close_status"]["closeable"], true);
    let gluing = &run["higher_graphen_gluing_review"];
    assert_eq!(
        gluing["schema"],
        "advisorygraphen.completion_dry_run.gluing_review.v1"
    );
    assert_eq!(
        gluing["source"],
        "highergraphen_0_5_correspondence_overlap_gluing"
    );
    assert!(
        gluing["correspondence_count"].as_u64().unwrap_or(0) > 0,
        "dry-run should emit candidate-specific HG correspondence: {gluing:#}"
    );
    assert!(
        gluing["gluing_summary"]["failure"].as_u64().unwrap_or(0) > 0,
        "pre-apply obstruction should block silent gluing: {gluing:#}"
    );
    assert!(gluing["preserved_structure_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id == "candidate:ship-release-action-missing-owner-owner"));

    case_import_workflow(&CaseImportOptions {
        store: store_path.clone(),
        space: space_path,
        revision_id: "revision:initial".to_string(),
    })
    .unwrap();
    let review = review_workflow(&ReviewOptions {
        store: store_path.clone(),
        candidate_id,
        from_report: Some(completions_path),
        reviewer: "reviewer:test".to_string(),
        reason: "Owner placeholder is acceptable for materialization test.".to_string(),
        outcome: "accepted".to_string(),
        base_revision: Some("revision:initial".to_string()),
    })
    .unwrap();
    let review_policy = &review["metadata"]["higher_graphen_gluing_policy"];
    assert!(review_policy["policy_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "gluing_failure_requires_explicit_review"));
    assert_eq!(
        review_policy["policy_override"],
        "explicit_completion_review"
    );
    let review_head = fs::read_to_string(
        store_path
            .join("spaces")
            .join("space-completion-application-test")
            .join("HEAD"),
    )
    .unwrap();

    let apply = completions_apply_accepted_workflow(&CompletionApplyAcceptedOptions {
        store: store_path.clone(),
        space_id: "space:completion-application-test".to_string(),
        reviewer: "ai-agent:test".to_string(),
        reason: "Apply reviewed accepted completion.".to_string(),
        base_revision: Some(review_head),
        dry_run: false,
    })
    .unwrap();
    assert_eq!(apply["result"]["applied_count"], 1);
    assert_eq!(
        apply["result"]["applied_structures"][0]["cell_ids"][0],
        "cell:auto-owner-ship-release-action"
    );
    assert!(apply["result"]["applied_structures"][0]["policy_blockers"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "gluing_failure_requires_explicit_review"));
    assert_eq!(
        apply["result"]["applied_structures"][0]["policy_override"],
        "explicit_completion_review"
    );
    assert_eq!(
        apply["result"]["post_apply_case_reason"]["close_status"]["closeable"],
        true
    );

    let reason = case_reason_workflow(&CaseReasonOptions {
        store: store_path,
        space_id: "space:completion-application-test".to_string(),
    })
    .unwrap();
    assert!(reason["result"]["blockers"].as_array().unwrap().is_empty());
}
