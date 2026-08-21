use super::*;

pub(super) fn selection_score(
    candidate: &Value,
    gluing_kind: &str,
    witness_kinds: &[String],
    severities: &[String],
) -> (i64, Vec<String>) {
    let mut score = 0_i64;
    let mut reasons = Vec::new();
    match gluing_kind {
        "failure" => {
            score += 10_000;
            reasons.push("gluing_failure".to_owned());
        }
        "candidate" => {
            score += 9_000;
            reasons.push("gluing_review_candidate".to_owned());
        }
        "success" => {
            score += 100;
            reasons.push("gluing_success_context".to_owned());
        }
        other => {
            reasons.push(format!("gluing_{other}"));
        }
    }
    if severities.iter().any(|severity| severity == "blocking") {
        score += 8_000;
        reasons.push("blocking_difference".to_owned());
    }
    if severities.iter().any(|severity| severity == "major") {
        score += 6_000;
        reasons.push("major_difference".to_owned());
    }
    if !severities.is_empty() {
        score += 2_000;
        reasons.push("has_difference_witness".to_owned());
    }
    if witness_kinds.iter().any(|kind| {
        matches!(
            kind.as_str(),
            "PredicateSet" | "NormalizedClaim" | "ConstraintSet"
        )
    }) {
        score += 3_000;
        reasons.push("structural_or_constraint_overlap".to_owned());
    }
    let roles = participant_roles(candidate);
    if roles.iter().any(|role| role == "requirement") {
        score += 7_000;
        reasons.push("direct_requirement_participant".to_owned());
    }
    if roles.iter().any(|role| role == "obstruction") {
        score += 7_000;
        reasons.push("direct_obstruction_participant".to_owned());
    }
    if roles.iter().any(|role| role == "evidence") {
        score += 3_500;
        reasons.push("direct_evidence_participant".to_owned());
    }
    if roles.iter().any(|role| {
        matches!(
            role.as_str(),
            "obstruction" | "completion_candidate" | "hypothesis" | "falsifier"
        )
    }) {
        score += 1_000;
        reasons.push("review_relevant_participant_role".to_owned());
    }
    if is_generic_candidate_similarity(&roles, witness_kinds) {
        score -= 13_000;
        reasons.push("generic_candidate_similarity_deprioritized".to_owned());
    }
    (score, reasons)
}

pub(super) fn is_generic_candidate_similarity(roles: &[String], witness_kinds: &[String]) -> bool {
    roles.len() == 2
        && roles.iter().all(|role| role == "completion_candidate")
        && witness_kinds.len() == 1
        && witness_kinds[0] == "FeatureSet"
}
