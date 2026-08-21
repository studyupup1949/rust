use super::*;

#[test]
fn hypothesis_seed_lift_drives_hypothesis_first_proposal_trace() {
    let temp = TempDir::new().unwrap();
    let snapshot_path = temp.path().join("hypothesis-first.snapshot.json");
    let space_path = temp.path().join("space.json");
    let check_path = temp.path().join("check.json");
    let projection_path = temp.path().join("ai-agent.json");

    fs::write(
        &snapshot_path,
        serde_json::to_vec_pretty(&json!({
            "schema": "advisorygraphen.engagement.snapshot.v1",
            "snapshot_id": "snapshot:hypothesis-first-structuring",
            "engagement_id": "engagement:advisorygraphen-self-review",
            "captured_at": "2026-05-08T00:00:00Z",
            "source_boundary": {
                "included_source_ids": ["source:runtime-design-note", "source:case-store-observation"],
                "excluded_summary": [],
                "extraction_loss": ["Synthetic regression fixture records only the discriminating design signal."],
                "trust_notes": ["AI-inferred hypothesis remains unreviewed until observation support is recorded."],
                "adapter_version": "json_snapshot:0.1.0"
            },
            "metadata": {},
            "sources": [
                {
                    "id": "source:runtime-design-note",
                    "source_type": "repository_note",
                    "title": "Runtime design note",
                    "uri": null,
                    "captured_at": "2026-05-08T00:00:00Z",
                    "classification": "internal",
                    "metadata": {}
                },
                {
                    "id": "source:case-store-observation",
                    "source_type": "code_observation",
                    "title": "Case store observation",
                    "uri": null,
                    "captured_at": "2026-05-08T00:00:00Z",
                    "classification": "internal",
                    "metadata": {}
                }
            ],
            "records": [
                {
                    "id": "record:case-log-source-of-truth-drift",
                    "record_type": "hypothesis_seed",
                    "title": "Case log source-of-truth drift",
                    "summary": "Case reasoning may rely on materialized space state instead of replaying the append-only event log.",
                    "source_ids": ["source:runtime-design-note", "source:case-store-observation"],
                    "context_hints": ["case-reasoning", "storage"],
                    "relation": null,
                    "provenance": {
                        "origin": "inferred",
                        "actor": "ai-agent:design-scan",
                        "confidence": 0.64,
                        "review_status": "unreviewed"
                    },
                    "metadata": {
                        "expected_observations": [
                            "case reason reads materialized/space.json without replaying morphism-log.jsonl"
                        ],
                        "falsifiers": [
                            "case reason reconstructs the advisory space by replaying the append-only log before projection"
                        ],
                        "candidate_structure_types": ["obstruction", "invariant", "refactor_action"]
                    }
                },
                {
                    "id": "record:case-log-replay-gap-refined",
                    "record_type": "hypothesis_refinement",
                    "title": "Case log replay gap after import",
                    "summary": "The likely design issue is narrower: imported materialized state and later log events can diverge unless all case reasoning uses one replay path.",
                    "source_ids": ["source:runtime-design-note", "source:case-store-observation"],
                    "context_hints": ["case-reasoning", "storage"],
                    "relation": null,
                    "provenance": {
                        "origin": "inferred",
                        "actor": "ai-agent:design-scan",
                        "confidence": 0.68,
                        "review_status": "unreviewed"
                    },
                    "metadata": {
                        "refinement_iteration": 2,
                        "expected_observations": [
                            "case import writes materialized state and later review events mutate the same case through a separate log path"
                        ],
                        "falsifiers": [
                            "all case reasoning reconstructs state exclusively from the append-only log"
                        ],
                        "candidate_structure_types": ["invariant", "refactor_action"]
                    }
                },
                {
                    "id": "record:unify-case-store-replay-action",
                    "record_type": "structure_proposal",
                    "title": "Unify case store replay path",
                    "summary": "Make case reason and close checks rebuild state through the same append-only replay path before projecting recommendations.",
                    "source_ids": ["source:runtime-design-note"],
                    "context_hints": ["case-reasoning", "storage"],
                    "relation": null,
                    "provenance": {
                        "origin": "inferred",
                        "actor": "ai-agent:design-scan",
                        "confidence": 0.58,
                        "review_status": "unreviewed"
                    },
                    "metadata": {
                        "priority": "p1",
                        "derived_from_hypothesis_id": "cell:case-log-replay-gap-refined",
                        "required_verification": "case reason and close check produce identical state from materialized import plus replayed log entries"
                    }
                },
                {
                    "id": "record:architecture-maintainer",
                    "record_type": "owner",
                    "title": "Architecture maintainer",
                    "summary": "Maintains AdvisoryGraphen runtime architecture.",
                    "source_ids": ["source:runtime-design-note"],
                    "context_hints": ["case-reasoning"],
                    "relation": null,
                    "provenance": {
                        "origin": "source_backed",
                        "actor": "source-adapter:json",
                        "confidence": 1.0,
                        "review_status": "accepted"
                    },
                    "metadata": {}
                },
                {
                    "id": "record:refined-hypothesis-narrows-seed",
                    "record_type": "refinement_relation",
                    "title": "Replay-gap hypothesis refines source-of-truth drift",
                    "summary": "The refined replay-gap hypothesis narrows the broader source-of-truth drift hypothesis.",
                    "source_ids": ["source:case-store-observation"],
                    "context_hints": ["case-reasoning", "storage"],
                    "relation": {
                        "relation_type": "refines",
                        "from_record_id": "record:case-log-replay-gap-refined",
                        "to_record_id": "record:case-log-source-of-truth-drift"
                    },
                    "provenance": {
                        "origin": "inferred",
                        "actor": "ai-agent:design-scan",
                        "confidence": 0.68,
                        "review_status": "unreviewed"
                    },
                    "metadata": {}
                },
                {
                    "id": "record:proposal-derived-from-hypothesis",
                    "record_type": "derivation_relation",
                    "title": "Proposal derives from source-of-truth hypothesis",
                    "summary": "The replay-path action is only justified if the refined replay-gap hypothesis remains plausible.",
                    "source_ids": ["source:runtime-design-note"],
                    "context_hints": ["case-reasoning", "storage"],
                    "relation": {
                        "relation_type": "derives_from",
                        "from_record_id": "record:unify-case-store-replay-action",
                        "to_record_id": "record:case-log-replay-gap-refined"
                    },
                    "provenance": {
                        "origin": "inferred",
                        "actor": "ai-agent:design-scan",
                        "confidence": 0.58,
                        "review_status": "unreviewed"
                    },
                    "metadata": {}
                },
                {
                    "id": "record:architecture-maintainer-owns-proposal",
                    "record_type": "ownership_relation",
                    "title": "Architecture maintainer owns replay proposal",
                    "summary": "The maintainer owns verification of the replay-path proposal.",
                    "source_ids": ["source:runtime-design-note"],
                    "context_hints": ["case-reasoning"],
                    "relation": {
                        "relation_type": "owns",
                        "from_record_id": "record:architecture-maintainer",
                        "to_record_id": "record:unify-case-store-replay-action"
                    },
                    "provenance": {
                        "origin": "source_backed",
                        "actor": "source-adapter:json",
                        "confidence": 1.0,
                        "review_status": "accepted"
                    },
                    "metadata": {}
                }
            ]
        }))
        .unwrap(),
    )
    .unwrap();

    lift_workflow(&LiftOptions {
        input: snapshot_path,
        package: "technical_advisory_mvp".to_string(),
        output: Some(space_path.clone()),
        command: None,
    })
    .unwrap();

    let space: serde_json::Value = serde_json::from_slice(&fs::read(&space_path).unwrap()).unwrap();
    let schema_morphism = &space["morphisms"][0]["schema_morphism"];
    assert_eq!(
        schema_morphism["id"],
        "schema-morphism:engagement-snapshot-to-advisory-space"
    );
    assert_eq!(
        schema_morphism["compatibility"],
        "compatible_with_declared_loss"
    );
    assert_eq!(
        space["metadata"]["schema_morphisms"][0]["verification"]["status"],
        "checked_by_lift_validation"
    );
    let hypothesis = space["cells"]
        .as_array()
        .unwrap()
        .iter()
        .find(|cell| cell["id"] == "cell:case-log-source-of-truth-drift")
        .unwrap();
    assert_eq!(hypothesis["cell_type"], "hypothesis");
    assert_eq!(hypothesis["metadata"]["hypothesis"], true);
    assert_eq!(hypothesis["metadata"]["hypothesis_status"], "candidate");
    assert_eq!(
        hypothesis["metadata"]["structuring_phase"],
        "hypothesis_first"
    );
    let refined = space["cells"]
        .as_array()
        .unwrap()
        .iter()
        .find(|cell| cell["id"] == "cell:case-log-replay-gap-refined")
        .unwrap();
    assert_eq!(refined["cell_type"], "hypothesis");
    assert_eq!(refined["metadata"]["hypothesis_refinement"], true);
    assert_eq!(
        refined["metadata"]["structuring_phase"],
        "hypothesis_refinement"
    );

    let proposal = space["cells"]
        .as_array()
        .unwrap()
        .iter()
        .find(|cell| cell["id"] == "cell:unify-case-store-replay-action")
        .unwrap();
    assert_eq!(proposal["cell_type"], "action");
    assert_eq!(proposal["metadata"]["structure_proposal"], true);
    assert_eq!(
        proposal["metadata"]["structuring_phase"],
        "derived_from_hypothesis"
    );
    assert!(space["incidences"]
        .as_array()
        .unwrap()
        .iter()
        .any(|incidence| {
            incidence["relation_type"] == "derives_from"
                && incidence["from_id"] == "cell:unify-case-store-replay-action"
                && incidence["to_id"] == "cell:case-log-replay-gap-refined"
        }));
    assert!(space["incidences"]
        .as_array()
        .unwrap()
        .iter()
        .any(|incidence| {
            incidence["relation_type"] == "refines"
                && incidence["from_id"] == "cell:case-log-replay-gap-refined"
                && incidence["to_id"] == "cell:case-log-source-of-truth-drift"
        }));

    check_workflow(&CheckOptions {
        space: space_path.clone(),
        ruleset: "technical_advisory_mvp".to_string(),
        output: Some(check_path.clone()),
        fail_on: None,
        command: None,
    })
    .unwrap();
    let check: serde_json::Value = serde_json::from_slice(&fs::read(&check_path).unwrap()).unwrap();
    assert!(check["result"]["obstructions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|obstruction| {
            obstruction["obstruction_type"] == "proposal_derived_from_unsupported_hypothesis"
                && obstruction["metadata"]["action_id"] == "cell:unify-case-store-replay-action"
                && obstruction["metadata"]["hypothesis_id"] == "cell:case-log-replay-gap-refined"
        }));

    project_workflow(&ProjectOptions {
        space: space_path,
        report: check_path,
        completions_report: None,
        audience: "ai_agent".to_string(),
        format: OutputFormat::Json,
        output: Some(projection_path.clone()),
    })
    .unwrap();
    let projection: serde_json::Value =
        serde_json::from_slice(&fs::read(&projection_path).unwrap()).unwrap();
    assert!(projection["explicit_hypothesis_matrix"]["hypotheses"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["hypothesis_id"] == "cell:case-log-source-of-truth-drift"));
    let refined_projection = projection["explicit_hypothesis_matrix"]["hypotheses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["hypothesis_id"] == "cell:case-log-replay-gap-refined")
        .unwrap();
    assert_eq!(refined_projection["refinement_status"], "refined");
    assert_eq!(refined_projection["refinement_depth"], 1);
    assert!(refined_projection["refinement_parent_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id == "cell:case-log-source-of-truth-drift"));
    let seed_projection = projection["explicit_hypothesis_matrix"]["hypotheses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["hypothesis_id"] == "cell:case-log-source-of-truth-drift")
        .unwrap();
    assert_eq!(seed_projection["refinement_status"], "has_refinements");
    assert!(seed_projection["refinement_child_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id == "cell:case-log-replay-gap-refined"));
    assert!(projection["explicit_proposal_trace"]["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| {
            entry["action_id"] == "cell:unify-case-store-replay-action"
                && entry["derived_hypothesis_ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|id| id == "cell:case-log-replay-gap-refined")
        }));
}
