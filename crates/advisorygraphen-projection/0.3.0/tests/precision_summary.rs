use advisorygraphen_core::{AdvisorySpaceEnvelope, ReportEnvelope};
use advisorygraphen_projection::build_projection;
use serde_json::{from_value, json, Value};

#[path = "precision_summary/correspondence.rs"]
mod correspondence;
#[path = "precision_summary/loss_and_quality.rs"]
mod loss_and_quality;
#[path = "precision_summary/recommendation_trace.rs"]
mod recommendation_trace;

fn empty_space() -> AdvisorySpaceEnvelope {
    from_value(json!({
        "schema": "advisorygraphen.space.v1",
        "space_id": "space:advisory:precision-test",
        "engagement_id": "engagement:precision-test",
        "snapshot_id": "snapshot:precision-test",
        "package_id": "technical_advisory_mvp",
        "cells": [],
        "contexts": [],
        "incidences": [],
        "morphisms": [],
        "invariants": [],
        "policies": [],
        "metadata": {}
    }))
    .unwrap()
}

fn explicit_hypothesis_space() -> AdvisorySpaceEnvelope {
    from_value(json!({
        "schema": "advisorygraphen.space.v1",
        "space_id": "space:advisory:precision-test",
        "engagement_id": "engagement:precision-test",
        "snapshot_id": "snapshot:precision-test",
        "package_id": "technical_advisory_mvp",
        "cells": [
            {
                "id": "cell:hypothesis-local-install",
                "cell_type": "hypothesis",
                "title": "Local install state is missing elkjs",
                "summary": null,
                "context_ids": [],
                "source_ids": ["source:test"],
                "structure_refs": [],
                "provenance": { "derivation": "source_backed", "review_status": "accepted" },
                "metadata": {
                    "hypothesis_status": "strongly_supported",
                    "expected_observations": ["unit test import resolution fails when node_modules is absent"],
                    "falsifiers": ["clean install still cannot resolve elkjs"]
                }
            },
            {
                "id": "cell:hypothesis-lockfile",
                "cell_type": "hypothesis",
                "title": "Lockfile pins an incompatible elkjs version",
                "summary": null,
                "context_ids": [],
                "source_ids": ["source:test"],
                "structure_refs": [],
                "provenance": { "derivation": "source_backed", "review_status": "accepted" },
                "metadata": {
                    "hypothesis_status": "falsified",
                    "expected_observations": ["lockfile contains a broken elkjs resolution"],
                    "falsifiers": ["lockfile resolution is internally consistent"]
                }
            },
            {
                "id": "cell:evidence-install",
                "cell_type": "evidence",
                "title": "node_modules lacks elkjs",
                "summary": null,
                "context_ids": [],
                "source_ids": ["source:test"],
                "structure_refs": [],
                "provenance": { "derivation": "source_backed", "review_status": "accepted" },
                "metadata": {}
            },
            {
                "id": "cell:proposal-install-deps",
                "cell_type": "action",
                "title": "Reinstall dependencies before judging test failures",
                "summary": null,
                "context_ids": [],
                "source_ids": ["source:test"],
                "structure_refs": [],
                "provenance": { "derivation": "source_backed", "review_status": "accepted" },
                "metadata": {
                    "priority": "P0",
                    "required_verification": "Run unit tests after reinstalling dependencies."
                }
            },
            {
                "id": "cell:owner-test",
                "cell_type": "owner",
                "title": "Test owner",
                "summary": null,
                "context_ids": [],
                "source_ids": ["source:test"],
                "structure_refs": [],
                "provenance": { "derivation": "source_backed", "review_status": "accepted" },
                "metadata": {}
            }
        ],
        "contexts": [],
        "incidences": [
            {
                "id": "incidence:evidence-supports-local-install",
                "relation_type": "supports",
                "from_id": "cell:evidence-install",
                "to_id": "cell:hypothesis-local-install",
                "source_ids": ["source:test"],
                "evidence_ids": ["source:test"],
                "provenance": { "derivation": "source_backed", "review_status": "accepted" },
                "metadata": {}
            },
            {
                "id": "incidence:evidence-falsifies-lockfile",
                "relation_type": "falsifies",
                "from_id": "cell:evidence-install",
                "to_id": "cell:hypothesis-lockfile",
                "source_ids": ["source:test"],
                "evidence_ids": ["source:test"],
                "provenance": { "derivation": "source_backed", "review_status": "accepted" },
                "metadata": {}
            },
            {
                "id": "incidence:hypotheses-compete",
                "relation_type": "competes_with",
                "from_id": "cell:hypothesis-local-install",
                "to_id": "cell:hypothesis-lockfile",
                "source_ids": ["source:test"],
                "evidence_ids": ["source:test"],
                "provenance": { "derivation": "source_backed", "review_status": "accepted" },
                "metadata": {}
            },
            {
                "id": "incidence:proposal-derives-from-hypothesis",
                "relation_type": "derives_from",
                "from_id": "cell:proposal-install-deps",
                "to_id": "cell:hypothesis-local-install",
                "source_ids": ["source:test"],
                "evidence_ids": ["source:test"],
                "provenance": { "derivation": "source_backed", "review_status": "accepted" },
                "metadata": {}
            },
            {
                "id": "incidence:owner-owns-proposal",
                "relation_type": "owns",
                "from_id": "cell:owner-test",
                "to_id": "cell:proposal-install-deps",
                "source_ids": ["source:test"],
                "evidence_ids": ["source:test"],
                "provenance": { "derivation": "source_backed", "review_status": "accepted" },
                "metadata": {}
            }
        ],
        "morphisms": [],
        "invariants": [],
        "policies": [],
        "metadata": { "method": "one-problem-multiple-hypotheses-observe-classify-propose" }
    }))
    .unwrap()
}

fn correspondence_space() -> AdvisorySpaceEnvelope {
    from_value(json!({
        "schema": "advisorygraphen.space.v1",
        "space_id": "space:advisory:correspondence-test",
        "engagement_id": "engagement:correspondence-test",
        "snapshot_id": "snapshot:correspondence-test",
        "package_id": "technical_advisory_mvp",
        "cells": [
            {
                "id": "cell:hypothesis-missing-auth",
                "cell_type": "hypothesis",
                "title": "The route is genuinely missing authentication",
                "summary": null,
                "context_ids": ["context:route"],
                "source_ids": ["source:route"],
                "structure_refs": [],
                "lifecycle_status": "candidate",
                "provenance": { "derivation": "inferred", "review_status": "unreviewed" },
                "metadata": { "hypothesis_status": "candidate" }
            },
            {
                "id": "cell:falsifier-shared-middleware",
                "cell_type": "falsifier",
                "title": "Shared middleware already authenticates the route",
                "summary": null,
                "context_ids": ["context:route"],
                "source_ids": ["source:route"],
                "structure_refs": [],
                "provenance": { "derivation": "inferred", "review_status": "unreviewed" },
                "metadata": { "falsifies": "cell:hypothesis-missing-auth" }
            }
        ],
        "contexts": [],
        "incidences": [
            {
                "id": "incidence:falsifier-shared-middleware-falsifies-hypothesis",
                "relation_type": "falsifies",
                "from_id": "cell:falsifier-shared-middleware",
                "to_id": "cell:hypothesis-missing-auth",
                "source_ids": ["source:route"],
                "evidence_ids": ["source:route"],
                "provenance": { "derivation": "inferred", "review_status": "unreviewed" },
                "metadata": {}
            }
        ],
        "morphisms": [],
        "invariants": [],
        "policies": [],
        "metadata": {}
    }))
    .unwrap()
}

fn correspondence_obstruction_with_failed_invariant() -> Value {
    json!({
        "id": "obstruction:auth-invariant-failed",
        "severity": "high",
        "message": "Route auth invariant is currently violated",
        "blocked_ids": ["cell:hypothesis-missing-auth"],
        "source_ids": ["source:route"],
        "violated_invariant_id": "invariant:route-auth"
    })
}

fn correspondence_candidate_with_satisfied_invariant() -> Value {
    json!({
        "id": "candidate:auth-invariant-satisfied",
        "title": "Treat the route auth invariant as satisfied",
        "candidate_type": "proposed_test",
        "confidence": 0.7,
        "source_ids": ["source:route"],
        "resolves_obstruction_ids": ["obstruction:auth-invariant-failed"],
        "affected_invariant_ids": ["invariant:route-auth"],
        "metadata": { "specificity": "requirement_derived" },
        "proposal_content": {}
    })
}

fn code_derived_obstruction() -> Value {
    json!({
        "id": "obstruction:route-missing-auth-guard",
        "obstruction_type": "api_route_missing_auth",
        "severity": "high",
        "review_status": "unreviewed",
        "message": "Route touches the database without an authentication guard.",
        "witness_ids": ["cell:route"],
        "blocked_ids": ["cell:route"],
        "evidence_ids": ["source:route"],
        "recommended_completion_types": ["proposed_auth_guard"],
        "metadata": { "specificity": "code_derived" }
    })
}

fn code_derived_candidate() -> Value {
    json!({
        "id": "candidate:route-auth-guard",
        "candidate_type": "proposed_auth_guard",
        "confidence": 0.72,
        "source_ids": ["source:route"],
        "metadata": { "specificity": "code_derived" }
    })
}

fn source_derived_candidate() -> Value {
    json!({
        "id": "candidate:billing-status-api",
        "candidate_type": "proposed_interface",
        "confidence": 0.82,
        "source_ids": ["source:architecture"],
        "metadata": { "specificity": "source_derived" }
    })
}

fn untraced_candidate() -> Value {
    json!({
        "id": "candidate:untraced",
        "candidate_type": "proposed_interface",
        "confidence": 0.5,
        "metadata": { "specificity": "generic" }
    })
}

fn blocked_proposal_content_candidate() -> Value {
    json!({
        "id": "candidate:missing-owner-owner",
        "candidate_type": "ownership_clarification",
        "confidence": 0.7,
        "source_ids": ["source:runbook"],
        "metadata": { "specificity": "generic" },
        "proposal_content": {
            "scenario": { "status": "blocked" },
            "content_obstructions": [
                { "obstruction_type": "proposal_content_underspecified" }
            ]
        }
    })
}

fn primary_candidate() -> Value {
    json!({
        "id": "candidate:supported-action",
        "title": "Supported action",
        "candidate_type": "proposed_test",
        "confidence": 0.8,
        "source_ids": ["source:test"],
        "recommendation_role": "primary",
        "supported_hypothesis_ids": ["hypothesis:supported"],
        "unsupported_hypothesis_ids": [],
        "hypothesis_trace": {
            "derived_hypothesis_id": "hypothesis:supported",
            "lifecycle_status": "supported",
            "supported": true
        },
        "metadata": { "specificity": "requirement_derived" }
    })
}

fn unsupported_follow_up_candidate() -> Value {
    json!({
        "id": "candidate:follow-up",
        "title": "Follow-up observation",
        "candidate_type": "proposed_test",
        "confidence": 0.6,
        "source_ids": ["source:test"],
        "recommendation_role": "follow_up_observation",
        "supported_hypothesis_ids": [],
        "unsupported_hypothesis_ids": ["hypothesis:unreviewed"],
        "hypothesis_trace": {
            "derived_hypothesis_id": "hypothesis:unreviewed",
            "lifecycle_status": "candidate",
            "supported": false
        },
        "metadata": { "specificity": "requirement_derived" },
        "proposal_content": {
            "content_obstructions": [
                {
                    "obstruction_type": "proposal_depends_on_unsupported_hypothesis",
                    "required_resolution": "Collect supporting observations."
                }
            ]
        }
    })
}

fn check_report(obstructions: Vec<Value>, candidates: Vec<Value>) -> Value {
    let envelope = ReportEnvelope::new(
        "check",
        Some("test"),
        json!({"space_id": "space:advisory:precision-test"}),
        json!({
            "obstructions": obstructions,
            "completion_candidates": candidates
        }),
    );
    serde_json::to_value(envelope).unwrap()
}
