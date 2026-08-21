use advisorygraphen_core::{
    json_id, sorted_values_by_id, AdvisoryError, AdvisoryResult, AdvisorySpaceEnvelope,
    HigherGraphenAdvisorySpace, ReportEnvelope, Severity,
};
use advisorygraphen_interpretation::load_ruleset;
use higher_graphen_core::Id as HigherId;
use serde_json::{json, Value};

mod completions;
mod cycles;
mod higher;
mod hypotheses;
mod hypothesis_lifecycle;
mod resolution;
pub use completions::propose_completions;
use cycles::evaluate_dependency_cycles;
pub use cycles::CYCLE_INVARIANT;
use higher::{has_accepted_supporting_evidence, violation_finding, FindingInput};
use hypotheses::build_hypotheses;
pub use hypotheses::{
    HYPOTHESIS_LIFECYCLE_ACCEPTED, HYPOTHESIS_LIFECYCLE_CANDIDATE, HYPOTHESIS_LIFECYCLE_FALSIFIED,
    HYPOTHESIS_LIFECYCLE_REJECTED, HYPOTHESIS_LIFECYCLE_SUPPORTED,
};
pub use hypothesis_lifecycle::propose_hypothesis_lifecycle;
pub use resolution::{blocker_resolution_state, frontier_items, waiting_items};

pub const BOUNDARY_INVARIANT: &str =
    "invariant:architecture_no_cross_context_direct_database_access";
pub const EVIDENCE_INVARIANT: &str = "invariant:recommendation_requires_evidence";
pub const OWNER_INVARIANT: &str = "invariant:action_requires_owner";
pub const REQUIREMENT_VERIFICATION_INVARIANT: &str = "invariant:requirement_requires_verification";
pub const API_ROUTE_AUTH_INVARIANT: &str =
    "invariant:api_route_database_access_requires_auth_guard";
pub const HYPOTHESIS_QUALITY_INVARIANT: &str = "invariant:hypothesis_requires_observation_model";
pub const PROPOSAL_TRACE_INVARIANT: &str = "invariant:proposal_requires_supported_hypothesis_trace";

pub fn check_space(
    space: &AdvisorySpaceEnvelope,
    ruleset: &str,
    fail_on: Option<Severity>,
    command: Option<&str>,
) -> AdvisoryResult<ReportEnvelope> {
    let _package = load_ruleset(ruleset)?;
    let higher_space = space.to_higher_graphen()?;
    let mut invariant_results = Vec::new();
    let mut obstructions = Vec::new();

    evaluate_boundary(
        space,
        &higher_space,
        &mut invariant_results,
        &mut obstructions,
    )?;
    evaluate_recommendation_evidence(space, &mut invariant_results, &mut obstructions)?;
    evaluate_action_owners(
        space,
        &higher_space,
        &mut invariant_results,
        &mut obstructions,
    )?;
    evaluate_required_verification(
        space,
        &higher_space,
        &mut invariant_results,
        &mut obstructions,
    )?;
    evaluate_api_route_auth(space, &mut invariant_results, &mut obstructions)?;
    evaluate_dependency_cycles(
        space,
        &higher_space,
        &mut invariant_results,
        &mut obstructions,
    )?;
    if explicit_hypothesis_workflow(space) {
        evaluate_hypothesis_quality(space, &mut invariant_results, &mut obstructions)?;
        evaluate_proposal_hypothesis_trace(
            space,
            &higher_space,
            &mut invariant_results,
            &mut obstructions,
        )?;
    }

    invariant_results = sorted_values_by_id(invariant_results);
    obstructions = sorted_values_by_id(obstructions);
    if let Some(threshold) = fail_on {
        let triggered = obstructions
            .iter()
            .filter_map(|item| item.get("severity").and_then(Value::as_str))
            .filter_map(Severity::parse)
            .any(|severity| severity >= threshold);
        if triggered {
            return Err(AdvisoryError::FailOnThreshold(format!("{threshold:?}")));
        }
    }

    let hypothesis_bundle = build_hypotheses(space, &obstructions)?;

    Ok(ReportEnvelope::new(
        "check",
        command,
        json!({
            "space_id": space.space_id,
            "ruleset": ruleset
        }),
        json!({
            "invariant_results": invariant_results,
            "obstructions": obstructions,
            "hypotheses": hypothesis_bundle.hypotheses,
            "falsifiers": hypothesis_bundle.falsifiers,
            "argumentation_incidences": hypothesis_bundle.incidences,
            "higher_graphen": higher_space.summary_json()
        }),
    ))
}

pub fn close_status(space: &AdvisorySpaceEnvelope, check_report: &ReportEnvelope) -> Value {
    let blocking = check_report
        .result
        .get("obstructions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|obstruction| {
            let effective = obstruction
                .pointer("/metadata/effective_severity")
                .and_then(Value::as_str)
                .or_else(|| obstruction.get("severity").and_then(Value::as_str));
            effective
                .and_then(Severity::parse)
                .is_some_and(|severity| severity >= Severity::Medium)
                && obstruction.get("review_status").and_then(Value::as_str) != Some("waived")
        })
        .cloned()
        .collect::<Vec<_>>();
    json!({
        "space_id": space.space_id,
        "blocking_threshold": "medium",
        "closeable": blocking.is_empty(),
        "blocking_obstruction_ids": blocking.iter().filter_map(|item| item.get("id").and_then(Value::as_str)).collect::<Vec<_>>(),
        "blocking_obstructions": blocking
    })
}

struct ProposalTraceObstruction<'a> {
    action: &'a Value,
    obstruction_type: &'static str,
    resolution: &'static str,
    message_suffix: &'static str,
    metadata: Value,
}

#[path = "checks/evidence_boundary.rs"]
mod evidence_boundary;
#[path = "checks/helpers.rs"]
mod helpers;
#[path = "checks/hypothesis_trace.rs"]
mod hypothesis_trace;
#[path = "checks/obstructions.rs"]
mod obstructions;
#[path = "checks/ownership_verification_auth.rs"]
mod ownership_verification_auth;

use evidence_boundary::*;
use helpers::*;
use hypothesis_trace::*;
use obstructions::*;
use ownership_verification_auth::*;
