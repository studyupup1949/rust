use super::*;

#[test]
fn supported_high_priority_proposal_requires_hypothesis_refinement_lineage() {
    let temp = TempDir::new().unwrap();
    let space_path = temp.path().join("space.json");
    let check_path = temp.path().join("check.json");
    let space = json!({
        "schema": "advisorygraphen.space.v1",
        "space_id": "space:hypothesis-refinement-gate",
        "engagement_id": "engagement:hypothesis-refinement-gate",
        "snapshot_id": "snapshot:hypothesis-refinement-gate",
        "package_id": "package:technical_advisory_mvp",
        "cells": [
            {
                "id": "cell:evidence-runtime-observation",
                "cell_type": "evidence",
                "title": "Runtime observation",
                "summary": "Source-backed observation supports the hypothesis.",
                "context_ids": [],
                "source_ids": ["source:runtime-observation"],
                "structure_refs": [],
                "provenance": {
                    "origin": "source_backed",
                    "actor": "test",
                    "confidence": 1.0,
                    "review_status": "accepted"
                },
                "metadata": {}
            },
            {
                "id": "cell:case-store-drift",
                "cell_type": "hypothesis",
                "title": "Case store drift",
                "summary": "A supported but still unrefined hypothesis.",
                "context_ids": [],
                "source_ids": ["source:runtime-observation"],
                "structure_refs": [],
                "provenance": {
                    "origin": "source_backed",
                    "actor": "test",
                    "confidence": 1.0,
                    "review_status": "accepted"
                },
                "metadata": {
                    "hypothesis": true,
                    "hypothesis_status": "supported",
                    "expected_observations": ["case reason uses materialized state"],
                    "falsifiers": ["case reason always replays append-only log"]
                }
            },
            {
                "id": "cell:architecture-owner",
                "cell_type": "owner",
                "title": "Architecture owner",
                "summary": "Owns runtime design.",
                "context_ids": [],
                "source_ids": ["source:runtime-observation"],
                "structure_refs": [],
                "provenance": {
                    "origin": "source_backed",
                    "actor": "test",
                    "confidence": 1.0,
                    "review_status": "accepted"
                },
                "metadata": {}
            },
            {
                "id": "cell:rewrite-case-store",
                "cell_type": "action",
                "title": "Rewrite case store",
                "summary": "High-priority action should not be promoted from an unrefined hypothesis.",
                "context_ids": [],
                "source_ids": ["source:runtime-observation"],
                "structure_refs": [],
                "provenance": {
                    "origin": "source_backed",
                    "actor": "test",
                    "confidence": 1.0,
                    "review_status": "accepted"
                },
                "metadata": {
                    "priority": "p1",
                    "derived_from_hypothesis_id": "cell:case-store-drift",
                    "required_verification": "case replay parity test passes"
                }
            }
        ],
        "contexts": [],
        "incidences": [
            {
                "id": "incidence:evidence-supports-case-store-drift",
                "relation_type": "supports",
                "from_id": "cell:evidence-runtime-observation",
                "to_id": "cell:case-store-drift",
                "context_ids": [],
                "evidence_ids": ["cell:evidence-runtime-observation"],
                "strength": "hard",
                "provenance": {
                    "origin": "source_backed",
                    "actor": "test",
                    "confidence": 1.0,
                    "review_status": "accepted"
                },
                "metadata": {}
            },
            {
                "id": "incidence:rewrite-derived-from-case-store-drift",
                "relation_type": "derives_from",
                "from_id": "cell:rewrite-case-store",
                "to_id": "cell:case-store-drift",
                "context_ids": [],
                "evidence_ids": ["cell:evidence-runtime-observation"],
                "strength": "hard",
                "provenance": {
                    "origin": "source_backed",
                    "actor": "test",
                    "confidence": 1.0,
                    "review_status": "accepted"
                },
                "metadata": {}
            },
            {
                "id": "incidence:owner-owns-rewrite",
                "relation_type": "owns",
                "from_id": "cell:architecture-owner",
                "to_id": "cell:rewrite-case-store",
                "context_ids": [],
                "evidence_ids": ["cell:evidence-runtime-observation"],
                "strength": "hard",
                "provenance": {
                    "origin": "source_backed",
                    "actor": "test",
                    "confidence": 1.0,
                    "review_status": "accepted"
                },
                "metadata": {}
            }
        ],
        "morphisms": [],
        "invariants": [],
        "policies": [],
        "metadata": {}
    });
    fs::write(&space_path, serde_json::to_vec_pretty(&space).unwrap()).unwrap();

    check_workflow(&CheckOptions {
        space: space_path,
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
            obstruction["obstruction_type"]
                == "high_priority_proposal_missing_hypothesis_refinement"
                && obstruction["metadata"]["action_id"] == "cell:rewrite-case-store"
        }));
}
