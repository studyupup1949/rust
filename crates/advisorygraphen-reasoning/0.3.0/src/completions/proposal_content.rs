use super::*;

pub(super) fn proposal_content(id: &str, rationale: &str, spec: &CandidateSpec<'_>) -> Value {
    let stem = id_suffix(id);
    let scenario_id = format!("scenario:{stem}-planned");
    let morphism_id = format!("morphism:{stem}-as-is-to-proposed");
    let derivation_id = format!("derivation:{stem}-proposal");
    let valuation_id = format!("valuation:{stem}-proposal");
    let policy_id = format!("policy:{stem}-review-gate");
    let candidate_ref = json!({ "object_type": "completion_candidate", "id": id });
    let required_witnesses = required_witnesses(spec);
    let known_witnesses = known_witnesses(spec);
    let content_obstructions = proposal_content_obstructions(spec, &required_witnesses);

    json!({
        "schema": "advisorygraphen.proposal_content.v1",
        "review_status": "unreviewed",
        "scenario": {
            "id": scenario_id,
            "base_space": spec.space.space_id,
            "scenario_kind": "planned",
            "assumptions": hypothesis_assumptions(&spec.metadata),
            "changed_structures": {
                "added": proposed_structure_ids(spec),
                "removed": [],
                "modified": spec.blocked_ids
            },
            "reachable_from": {
                "ref": spec.space.space_id,
                "via_morphisms": [morphism_id.clone()]
            },
            "affected_invariants": spec.affected_invariant_ids,
            "expected_obstructions": spec.resolves_obstruction_ids,
            "required_witnesses": required_witnesses,
            "valuations": [valuation_id.clone()],
            "status": if content_obstructions.is_empty() { "candidate" } else { "blocked" },
            "review_status": "candidate"
        },
        "morphism": {
            "id": morphism_id,
            "morphism_type": "as_is_to_to_be",
            "source_space": spec.space.space_id,
            "target_scenario": scenario_id,
            "repairs_obstructions": spec.resolves_obstruction_ids,
            "preserved_invariants": spec.affected_invariant_ids,
            "changed_cell_ids": spec.proposed_cell_ids,
            "changed_incidence_ids": spec.proposed_incidence_ids,
            "distortion": proposal_distortion(spec),
            "composition_constraints": [
                "candidate_to_accepted_structure requires an explicit completion review event",
                "accepted completion must be materialized before the blocker is treated as resolved"
            ],
            "review_status": "unreviewed"
        },
        "invariant_checks": proposal_invariant_checks(spec),
        "derivation": {
            "id": derivation_id,
            "conclusion": id,
            "premises": proposal_premises(spec),
            "inference_rule": {
                "id": format!("rule:completion-{}", spec.candidate_type),
                "name": format!("Generate {} completion candidate from obstruction", spec.candidate_type),
                "interpretation_package": "technical_advisory_mvp"
            },
            "warrants": known_witnesses,
            "excluded_premises": [],
            "counterexamples": [],
            "verifier": null,
            "verification_status": "unverified",
            "failure_mode": if spec.source_ids.is_empty() { "missing_premise" } else { "none" },
            "rationale": rationale,
            "review_status": "candidate"
        },
        "witnesses": witness_records(spec),
        "valuation": {
            "id": valuation_id,
            "target": candidate_ref,
            "valuation_context": spec.space.space_id,
            "order_type": "partial_order",
            "criteria": [
                { "criterion_id": "obstruction_resolution", "name": "Obstruction resolution", "direction": "maximize", "required": true },
                { "criterion_id": "evidence_backing", "name": "Evidence backing", "direction": "maximize", "required": true },
                { "criterion_id": "review_safety", "name": "Review safety", "direction": "preserve", "required": true }
            ],
            "values": [
                { "criterion_id": "obstruction_resolution", "value": spec.resolves_obstruction_ids.len(), "evidence": known_or_synthetic_witness(spec) },
                { "criterion_id": "evidence_backing", "value": spec.source_ids.len(), "evidence": known_or_synthetic_witness(spec) },
                { "criterion_id": "review_safety", "value": true, "evidence": policy_id }
            ],
            "tradeoffs": proposal_tradeoffs(spec),
            "confidence": spec.confidence,
            "review_status": "candidate"
        },
        "policy": {
            "id": policy_id,
            "policy_type": "completion_review_gate",
            "target": candidate_ref,
            "rules": [
                "AI agents may propose this content but must not accept it as current state",
                "Acceptance requires explicit review and blocker application requirements",
                "Missing required witnesses remain proposal-content obstructions"
            ],
            "required_witnesses": required_witnesses,
            "review_status": "candidate"
        },
        "content_obstructions": content_obstructions
    })
}

pub(super) fn proposal_premises(spec: &CandidateSpec<'_>) -> Vec<String> {
    let mut premises = spec.resolves_obstruction_ids.clone();
    premises.extend(spec.proposed_cell_ids.clone());
    premises.extend(spec.proposed_incidence_ids.clone());
    premises.extend(spec.witness_ids.clone());
    premises.extend(spec.source_ids.clone());
    premises.sort();
    premises.dedup();
    premises
}

pub(super) fn known_witnesses(spec: &CandidateSpec<'_>) -> Vec<String> {
    let mut witnesses = spec.source_ids.clone();
    witnesses.extend(spec.witness_ids.clone());
    witnesses.sort();
    witnesses.dedup();
    witnesses
}

pub(super) fn required_witnesses(spec: &CandidateSpec<'_>) -> Vec<String> {
    let mut witnesses = vec![format!("witness:{}-review", id_suffix(&spec.id))];
    match spec.candidate_type {
        "proposed_interface" => witnesses.push(format!(
            "witness:{}-interface-contract",
            id_suffix(&spec.id)
        )),
        "proposed_refactor_action" => {
            witnesses.push(format!("witness:{}-migration-plan", id_suffix(&spec.id)))
        }
        "ownership_clarification" | "owner_assignment" => witnesses.push(format!(
            "witness:{}-owner-confirmation",
            id_suffix(&spec.id)
        )),
        "proposed_test" | "lift_verification_link" => witnesses.push(format!(
            "witness:{}-verification-method",
            id_suffix(&spec.id)
        )),
        "proposed_auth_guard" => witnesses.push(format!(
            "witness:{}-auth-behavior-check",
            id_suffix(&spec.id)
        )),
        _ => {}
    }
    witnesses
}

pub(super) fn proposal_invariant_checks(spec: &CandidateSpec<'_>) -> Vec<Value> {
    let mut checks = spec
        .affected_invariant_ids
        .iter()
        .map(|id| {
            json!({
                "invariant_id": id,
                "status": "candidate_repair",
                "target_candidate_id": spec.id,
                "review_status": "unreviewed"
            })
        })
        .collect::<Vec<_>>();
    checks.push(json!({
        "invariant_id": "invariant:completion-candidate-review-gated",
        "status": "preserved",
        "target_candidate_id": spec.id,
        "review_status": "unreviewed"
    }));
    checks
}

pub(super) fn witness_records(spec: &CandidateSpec<'_>) -> Vec<Value> {
    known_witnesses(spec)
        .into_iter()
        .map(|id| {
            let witness_type = if id.starts_with("source:") {
                "source_reference"
            } else {
                "structure_reference"
            };
            json!({
                "id": id,
                "witness_type": witness_type,
                "supports": [spec.id],
                "validity_contexts": [spec.space.space_id],
                "review_status": "candidate"
            })
        })
        .collect()
}

pub(super) fn proposal_tradeoffs(spec: &CandidateSpec<'_>) -> Vec<Value> {
    match spec.candidate_type {
        "proposed_interface" => vec![json!({
            "gains": "Replaces direct ownership-boundary access with an explicit interface.",
            "losses": "Adds an interface contract that still needs owner, compatibility, and verification witnesses.",
            "affected_invariants": spec.affected_invariant_ids
        })],
        "proposed_refactor_action" => vec![json!({
            "gains": "Moves the violating caller toward the proposed boundary-preserving structure.",
            "losses": "Requires migration sequencing and regression evidence before acceptance.",
            "affected_invariants": spec.affected_invariant_ids
        })],
        "owner_assignment" => vec![json!({
            "gains": "Reuses an existing owner cell and proposes only the missing ownership relation.",
            "losses": "The owner match still needs review because shared context or source IDs are suggestive, not proof of responsibility.",
            "affected_invariants": spec.affected_invariant_ids
        })],
        "ownership_clarification" => vec![json!({
            "gains": "Makes execution accountability explicit.",
            "losses": "Does not infer a concrete owner without a reviewed ownership witness.",
            "affected_invariants": spec.affected_invariant_ids
        })],
        "lift_verification_link" => vec![json!({
            "gains": "Reuses an existing verification structure and proposes the missing verifies relation.",
            "losses": "The verification match still needs review because shared context or source IDs do not prove coverage.",
            "affected_invariants": spec.affected_invariant_ids
        })],
        "proposed_test" => vec![json!({
            "gains": "Turns an unverified requirement into a checkable obligation.",
            "losses": "The concrete test or metric still needs design and review.",
            "affected_invariants": spec.affected_invariant_ids
        })],
        "proposed_auth_guard" => vec![json!({
            "gains": "Adds an explicit security control candidate for a database-touching route.",
            "losses": "Shared middleware and intended-public exceptions still need reviewed witnesses.",
            "affected_invariants": spec.affected_invariant_ids
        })],
        _ => vec![json!({
            "gains": "May resolve the linked obstruction.",
            "losses": "Proposal content requires review before acceptance.",
            "affected_invariants": spec.affected_invariant_ids
        })],
    }
}

pub(super) fn proposal_distortion(spec: &CandidateSpec<'_>) -> Vec<Value> {
    let mut distortion = Vec::new();
    if proposed_structure_ids(spec).is_empty() {
        distortion.push(json!({
            "distortion_type": "underspecified_structure",
            "summary": "Candidate describes a missing structure type but does not yet name the concrete cell to materialize."
        }));
    }
    if spec.source_ids.is_empty() {
        distortion.push(json!({
            "distortion_type": "weak_source_backing",
            "summary": "Candidate is derived from an obstruction without direct source IDs."
        }));
    }
    distortion
}

pub(super) fn proposal_content_obstructions(
    spec: &CandidateSpec<'_>,
    required_witnesses: &[String],
) -> Vec<Value> {
    let mut obstructions = Vec::new();
    if proposed_structure_ids(spec).is_empty() {
        obstructions.push(json!({
            "obstruction_type": "proposal_content_underspecified",
            "message": "Proposal content does not yet identify concrete structures to add.",
            "required_resolution": "Add concrete proposed cells or incidences before treating the candidate as structurally complete.",
            "review_status": "unreviewed"
        }));
    }
    if spec.source_ids.is_empty() {
        obstructions.push(json!({
            "obstruction_type": "proposal_content_missing_source_witness",
            "message": "Proposal content lacks source-backed witnesses.",
            "required_witnesses": required_witnesses,
            "review_status": "unreviewed"
        }));
    }
    obstructions
}

pub(super) fn proposed_structure_ids(spec: &CandidateSpec<'_>) -> Vec<String> {
    let mut ids = spec.proposed_cell_ids.clone();
    ids.extend(spec.proposed_incidence_ids.clone());
    ids
}

pub(super) fn hypothesis_assumptions(metadata: &Value) -> Vec<String> {
    metadata
        .get("derived_from_hypothesis_id")
        .and_then(Value::as_str)
        .map(|id| vec![id.to_string()])
        .unwrap_or_default()
}

pub(super) fn known_or_synthetic_witness(spec: &CandidateSpec<'_>) -> String {
    known_witnesses(spec)
        .into_iter()
        .next()
        .unwrap_or_else(|| format!("witness:{}-proposal-generated", id_suffix(&spec.id)))
}
