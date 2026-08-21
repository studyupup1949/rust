use super::*;

pub fn hypothesis_support_workflow(options: &HypothesisFalsifyOptions) -> AdvisoryResult<Value> {
    hypothesis_lifecycle_event(options, "supported")
}

pub fn hypothesis_accept_workflow(options: &HypothesisFalsifyOptions) -> AdvisoryResult<Value> {
    hypothesis_lifecycle_event(options, "accepted")
}

pub fn hypothesis_reject_workflow(options: &HypothesisFalsifyOptions) -> AdvisoryResult<Value> {
    hypothesis_lifecycle_event(options, "rejected")
}

#[derive(Debug, Clone)]
pub(super) struct HypothesisAutonomyPolicy {
    pub(super) allowed_outcomes: Vec<String>,
    pub(super) min_confidence: f64,
    pub(super) allowed_trust_levels: Vec<String>,
    pub(super) max_events: usize,
    pub(super) require_candidate_status: bool,
    pub(super) allow_review_conflict: bool,
}

impl HypothesisAutonomyPolicy {
    fn default_conservative() -> Self {
        Self {
            allowed_outcomes: vec!["supported".to_string(), "falsified".to_string()],
            min_confidence: 0.7,
            allowed_trust_levels: vec![
                "reviewed_or_source_backed".to_string(),
                "test_passed".to_string(),
                "runtime_observed".to_string(),
            ],
            max_events: 3,
            require_candidate_status: true,
            allow_review_conflict: false,
        }
    }

    pub(super) fn as_json(&self) -> Value {
        json!({
            "allowed_outcomes": self.allowed_outcomes,
            "min_confidence": self.min_confidence,
            "allowed_trust_levels": self.allowed_trust_levels,
            "max_events": self.max_events,
            "require_candidate_status": self.require_candidate_status,
            "allow_review_conflict": self.allow_review_conflict
        })
    }
}

pub(super) struct AutonomyDecision {
    pub(super) allowed: bool,
    pub(super) reason: String,
}

pub(super) fn read_autonomy_policy(
    path: Option<&Path>,
) -> AdvisoryResult<HypothesisAutonomyPolicy> {
    let Some(path) = path else {
        return Ok(HypothesisAutonomyPolicy::default_conservative());
    };
    let value = read_json(path)?;
    let default = HypothesisAutonomyPolicy::default_conservative();
    Ok(HypothesisAutonomyPolicy {
        allowed_outcomes: optional_string_vec(&value, "allowed_outcomes")
            .unwrap_or(default.allowed_outcomes),
        min_confidence: value
            .get("min_confidence")
            .and_then(Value::as_f64)
            .unwrap_or(default.min_confidence),
        allowed_trust_levels: optional_string_vec(&value, "allowed_trust_levels")
            .unwrap_or(default.allowed_trust_levels),
        max_events: value
            .get("max_events")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(default.max_events),
        require_candidate_status: value
            .get("require_candidate_status")
            .and_then(Value::as_bool)
            .unwrap_or(default.require_candidate_status),
        allow_review_conflict: value
            .get("allow_review_conflict")
            .and_then(Value::as_bool)
            .unwrap_or(default.allow_review_conflict),
    })
}

pub(super) fn optional_string_vec(value: &Value, key: &str) -> Option<Vec<String>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty())
}

pub(super) fn autonomy_decision(
    proposal: &Value,
    policy: &HypothesisAutonomyPolicy,
) -> AutonomyDecision {
    let outcome = proposal
        .get("proposed_outcome")
        .and_then(Value::as_str)
        .unwrap_or("");
    if outcome == "review_conflict" && !policy.allow_review_conflict {
        return denied("review_conflict proposals require human review");
    }
    if !policy
        .allowed_outcomes
        .iter()
        .any(|allowed| allowed == outcome)
    {
        return denied(format!("outcome {outcome} is not policy-allowed"));
    }
    if policy.require_candidate_status
        && proposal
            .get("target_hypothesis_status")
            .and_then(Value::as_str)
            != Some("candidate")
    {
        return denied("target hypothesis is not in candidate lifecycle status");
    }
    let confidence = proposal
        .get("confidence")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    if confidence < policy.min_confidence {
        return denied(format!(
            "confidence {confidence} is below policy minimum {}",
            policy.min_confidence
        ));
    }
    let signal_pointer = match outcome {
        "supported" => "/supporting_signals",
        "falsified" => "/falsifying_signals",
        _ => "/supporting_signals",
    };
    let signals = proposal
        .pointer(signal_pointer)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if signals.is_empty() {
        return denied("proposal has no outcome-specific evidence signals");
    }
    let has_allowed_trust = signals.iter().any(|signal| {
        signal
            .get("trust_level")
            .and_then(Value::as_str)
            .is_some_and(|trust| {
                policy
                    .allowed_trust_levels
                    .iter()
                    .any(|allowed| allowed == trust)
            })
    });
    if !has_allowed_trust {
        return denied("proposal has no evidence signal with policy-allowed trust level");
    }
    AutonomyDecision {
        allowed: true,
        reason: "policy allowed".to_string(),
    }
}

pub(super) fn denied(reason: impl Into<String>) -> AutonomyDecision {
    AutonomyDecision {
        allowed: false,
        reason: reason.into(),
    }
}

pub(super) fn application_skip(proposal: &Value, reason: impl Into<String>) -> Value {
    json!({
        "proposal_id": proposal.get("id"),
        "target_hypothesis_id": proposal.get("target_hypothesis_id"),
        "proposed_outcome": proposal.get("proposed_outcome"),
        "reason": reason.into()
    })
}

pub(super) fn hypothesis_event_from_proposal(
    engagement_id: &str,
    proposal: &Value,
    reviewer: &str,
    reason: &str,
    from_report: &Path,
    base_revision: Option<&str>,
    ordinal: usize,
) -> AdvisoryResult<Value> {
    let hypothesis_id = proposal
        .get("target_hypothesis_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AdvisoryError::Validation("lifecycle proposal missing target_hypothesis_id".to_string())
        })?;
    let outcome = proposal
        .get("proposed_outcome")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AdvisoryError::Validation("lifecycle proposal missing proposed_outcome".to_string())
        })?;
    let event_outcome = match outcome {
        "supported" | "falsified" => outcome,
        other => {
            return Err(AdvisoryError::Validation(format!(
                "cannot apply lifecycle proposal outcome {other}"
            )))
        }
    };
    let evidence_ids = proposal
        .get("evidence_ids")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let hypothesis_slug = hypothesis_id.trim_start_matches("hypothesis:");
    let event = json!({
        "schema": HYPOTHESIS_EVENT_SCHEMA,
        "hypothesis_event_id": format!("hypothesis-event:auto-{event_outcome}:{hypothesis_slug}-{ordinal:06}"),
        "engagement_id": engagement_id,
        "target_hypothesis_id": hypothesis_id,
        "outcome": event_outcome,
        "reviewer_id": reviewer,
        "reviewed_at": Utc::now().to_rfc3339(),
        "reason": reason,
        "evidence_ids": evidence_ids,
        "base_revision_id": base_revision,
        "metadata": {
            "from_report": from_report.display().to_string(),
            "proposal_id": proposal.get("id"),
            "autonomy": {
                "applied_from_proposal": true,
                "proposal_confidence": proposal.get("confidence"),
                "supporting_signals": proposal.get("supporting_signals"),
                "falsifying_signals": proposal.get("falsifying_signals")
            }
        }
    });
    validate_document(&event, Some(HYPOTHESIS_EVENT_SCHEMA))?;
    Ok(event)
}

pub(super) fn hypothesis_lifecycle_event(
    options: &HypothesisFalsifyOptions,
    outcome: &str,
) -> AdvisoryResult<Value> {
    fs::create_dir_all(&options.store)?;
    let report = read_json(&options.from_report)?;
    let space_id = report
        .pointer("/input/space_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            AdvisoryError::Validation(
                "from-report must contain input.space_id for hypothesis events".to_string(),
            )
        })?
        .to_string();
    let hypothesis = report
        .pointer("/result/hypotheses")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|item| item.get("id").and_then(Value::as_str) == Some(&options.hypothesis_id))
        .ok_or_else(|| {
            AdvisoryError::Validation(format!(
                "hypothesis {} not found in from-report",
                options.hypothesis_id
            ))
        })?
        .clone();
    let head = read_imported_space_head(&options.store, &space_id)?;
    let materialized_space = read_materialized_space(&options.store, &space_id)?;
    ensure_base_revision(Some(&head), options.base_revision.as_deref())?;
    let sequence = next_sequence(&options.store, &space_id);
    let target_revision = format!("revision:hypothesis-{sequence:06}");
    let hypothesis_slug = options.hypothesis_id.trim_start_matches("hypothesis:");
    let hypothesis_event_id = format!("hypothesis-event:{outcome}:{hypothesis_slug}-{sequence:06}");
    let evidence_ids: Vec<Value> = options
        .evidence_ids
        .iter()
        .map(|id| Value::String(id.clone()))
        .collect();
    let event = json!({
        "schema": HYPOTHESIS_EVENT_SCHEMA,
        "hypothesis_event_id": hypothesis_event_id,
        "engagement_id": materialized_space.engagement_id,
        "target_hypothesis_id": options.hypothesis_id,
        "outcome": outcome,
        "reviewer_id": options.reviewer,
        "reviewed_at": Utc::now().to_rfc3339(),
        "reason": options.reason,
        "evidence_ids": evidence_ids,
        "base_revision_id": options.base_revision,
        "metadata": {
            "from_report": options.from_report.display().to_string(),
            "competes_with": hypothesis
                .pointer("/metadata/competes_with")
                .cloned()
                .unwrap_or_else(|| json!([])),
            "falsified_by": hypothesis
                .pointer("/metadata/falsified_by")
                .cloned()
                .unwrap_or_else(|| json!([]))
        }
    });
    validate_document(&event, Some(HYPOTHESIS_EVENT_SCHEMA))?;
    append_store_event(
        &options.store,
        &json!({
            "schema": "advisorygraphen.case.log.entry.v1",
            "case_space_id": space_id.clone(),
            "sequence": sequence,
            "entry_id": format!("log:{sequence:06}"),
            "morphism_id": format!("morphism:hypothesis-{outcome}-{hypothesis_slug}"),
            "source_revision_id": head,
            "target_revision_id": target_revision.clone(),
            "actor": event["reviewer_id"],
            "recorded_at": Utc::now().to_rfc3339(),
            "previous_entry_hash": null,
            "entry_hash": null,
            "payload": event
        }),
    )?;
    fs::write(
        space_dir(&options.store, &space_id).join("HEAD"),
        &target_revision,
    )?;
    Ok(event)
}
