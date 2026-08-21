use super::*;

pub(super) fn find_cell<'a>(
    space: &'a AdvisorySpaceEnvelope,
    id: Option<&str>,
) -> Option<&'a Value> {
    let id = id?;
    space.cells.iter().find(|cell| json_id(cell) == id)
}

pub(super) fn is_cross_context(left: &[&str], right: &[&str]) -> bool {
    !left.is_empty() && !right.is_empty() && left.iter().all(|id| !right.contains(id))
}

pub(super) fn boundary_obstruction_id(from_id: &str, to_id: &str, access_type: &str) -> String {
    let access = match access_type {
        "direct_database_read" => "direct".to_string(),
        other => id_suffix(other),
    };
    format!(
        "obstruction:{}-{access}-{}-access",
        id_suffix(from_id),
        id_suffix(to_id)
    )
}

pub(super) fn title(value: &Value) -> &str {
    value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_else(|| json_id(value))
}

pub(super) fn id_suffix(id: &str) -> String {
    id.rsplit_once(':')
        .map(|(_, suffix)| suffix)
        .unwrap_or(id)
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

pub(super) fn has_incoming_owner(
    higher_space: &HigherGraphenAdvisorySpace,
    action_id: &str,
) -> bool {
    higher_space.incidence_records().iter().any(|incidence| {
        incidence.relation_type == "owns" && incidence.to_cell_id.as_str() == action_id
    })
}

pub(super) fn has_verification(
    higher_space: &HigherGraphenAdvisorySpace,
    requirement_id: &str,
) -> bool {
    higher_space.incidence_records().iter().any(|incidence| {
        matches!(incidence.relation_type.as_str(), "verifies" | "implements")
            && (incidence.from_cell_id.as_str() == requirement_id
                || incidence.to_cell_id.as_str() == requirement_id)
    })
}

pub(super) fn requires_verification(requirement: &Value) -> bool {
    requirement
        .pointer("/metadata/require_verification")
        .and_then(Value::as_bool)
        == Some(true)
        || requirement
            .pointer("/metadata/verification_required")
            .and_then(Value::as_bool)
            == Some(true)
}

pub(super) fn explicit_hypothesis_workflow(space: &AdvisorySpaceEnvelope) -> bool {
    space
        .metadata
        .get("method")
        .and_then(Value::as_str)
        .is_some_and(|method| method.contains("hypothesis"))
        || space.cells.iter().any(is_hypothesis_cell)
}

pub(super) fn is_hypothesis_cell(cell: &Value) -> bool {
    cell["cell_type"] == "hypothesis"
        || cell
            .pointer("/metadata/hypothesis")
            .and_then(Value::as_bool)
            == Some(true)
        || cell.pointer("/metadata/hypothesis_status").is_some()
        || cell.get("lifecycle_status").is_some()
}

pub(super) fn hypothesis_status(hypothesis: &Value) -> &str {
    hypothesis
        .pointer("/metadata/hypothesis_status")
        .and_then(Value::as_str)
        .or_else(|| hypothesis.get("lifecycle_status").and_then(Value::as_str))
        .unwrap_or("candidate")
}

pub(super) fn supported_status(status: &str) -> bool {
    matches!(
        status,
        "supported" | "strongly_supported" | "supported_needs_followup" | "plausible_secondary"
    )
}

pub(super) fn primary_action_status_supported(status: &str) -> bool {
    matches!(
        status,
        "accepted"
            | "supported"
            | "strongly_supported"
            | "supported_needs_followup"
            | "plausible_secondary"
    )
}

pub(super) fn metadata_array_non_empty(cell: &Value, pointer: &str) -> bool {
    cell.pointer(pointer)
        .and_then(Value::as_array)
        .is_some_and(|items| !items.is_empty())
}

pub(super) fn has_argumentation_relation(
    space: &AdvisorySpaceEnvelope,
    hypothesis_id: &str,
    relation_types: &[&str],
) -> bool {
    space.incidences.iter().any(|incidence| {
        let relation_type = incidence
            .get("relation_type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !relation_types.contains(&relation_type) {
            return false;
        }
        let from_id = incidence.get("from_id").and_then(Value::as_str);
        let to_id = incidence.get("to_id").and_then(Value::as_str);
        match relation_type {
            "supports" | "falsifies" => to_id == Some(hypothesis_id),
            "supported_by" | "falsified_by" => from_id == Some(hypothesis_id),
            _ => from_id == Some(hypothesis_id) || to_id == Some(hypothesis_id),
        }
    })
}

pub(super) fn has_competing_hypothesis_relation(
    space: &AdvisorySpaceEnvelope,
    hypothesis_id: &str,
) -> bool {
    space.incidences.iter().any(|incidence| {
        incidence.get("relation_type").and_then(Value::as_str) == Some("competes_with")
            && (incidence.get("from_id").and_then(Value::as_str) == Some(hypothesis_id)
                || incidence.get("to_id").and_then(Value::as_str) == Some(hypothesis_id))
    })
}

pub(super) fn hypothesis_has_refinement_context(
    space: &AdvisorySpaceEnvelope,
    hypothesis_id: &str,
    hypothesis: &Value,
) -> bool {
    hypothesis
        .pointer("/metadata/hypothesis_refinement")
        .and_then(Value::as_bool)
        == Some(true)
        || hypothesis
            .pointer("/metadata/refinement_iteration")
            .and_then(Value::as_u64)
            .is_some_and(|iteration| iteration > 1)
        || space.incidences.iter().any(|incidence| {
            matches!(
                incidence.get("relation_type").and_then(Value::as_str),
                Some("refines" | "refined_from" | "revises" | "revised_from")
            ) && (incidence.get("from_id").and_then(Value::as_str) == Some(hypothesis_id)
                || incidence.get("to_id").and_then(Value::as_str) == Some(hypothesis_id))
        })
}

pub(super) fn derived_hypothesis_ids(
    space: &AdvisorySpaceEnvelope,
    action_id: &str,
    action: &Value,
) -> Vec<String> {
    let mut ids = Vec::new();
    if let Some(id) = action
        .pointer("/metadata/derived_from_hypothesis")
        .and_then(Value::as_str)
    {
        ids.push(normalize_cell_id(id));
    }
    if let Some(id) = action
        .pointer("/metadata/derived_from_hypothesis_id")
        .and_then(Value::as_str)
    {
        ids.push(normalize_cell_id(id));
    }
    if let Some(values) = action
        .pointer("/metadata/derived_from_hypotheses")
        .and_then(Value::as_array)
    {
        ids.extend(
            values
                .iter()
                .filter_map(Value::as_str)
                .map(normalize_cell_id),
        );
    }
    ids.extend(
        space
            .incidences
            .iter()
            .filter(|incidence| {
                incidence.get("relation_type").and_then(Value::as_str) == Some("derives_from")
                    && incidence.get("from_id").and_then(Value::as_str) == Some(action_id)
            })
            .filter_map(|incidence| incidence.get("to_id").and_then(Value::as_str))
            .map(normalize_cell_id),
    );
    ids.sort();
    ids.dedup();
    ids
}

pub(super) fn normalize_cell_id(id: &str) -> String {
    if id.starts_with("record:") {
        format!("cell:{}", id.trim_start_matches("record:"))
    } else {
        id.to_string()
    }
}

pub(super) fn p0_or_p1(action: &Value) -> bool {
    action
        .pointer("/metadata/priority")
        .and_then(Value::as_str)
        .map(|priority| matches!(priority.to_ascii_lowercase().as_str(), "p0" | "p1"))
        .unwrap_or(false)
}

pub(super) fn action_has_required_verification(
    space: &AdvisorySpaceEnvelope,
    higher_space: &HigherGraphenAdvisorySpace,
    action: &Value,
) -> bool {
    action
        .pointer("/metadata/required_verification")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty())
        || action
            .pointer("/metadata/verification")
            .and_then(Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
        || has_verification(higher_space, json_id(action))
        || space.incidences.iter().any(|incidence| {
            matches!(
                incidence.get("relation_type").and_then(Value::as_str),
                Some("verifies" | "verified_by")
            ) && (incidence.get("from_id").and_then(Value::as_str) == Some(json_id(action))
                || incidence.get("to_id").and_then(Value::as_str) == Some(json_id(action)))
        })
}
