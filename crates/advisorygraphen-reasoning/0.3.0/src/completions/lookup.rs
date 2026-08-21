use super::*;

pub(super) fn find_cell<'a>(
    space: &'a AdvisorySpaceEnvelope,
    id: Option<&str>,
) -> Option<&'a Value> {
    let id = id?;
    space.cells.iter().find(|cell| json_id(cell) == id)
}

pub(super) fn blocked_cell<'a>(
    space: &'a AdvisorySpaceEnvelope,
    obstruction: &Value,
) -> Option<&'a Value> {
    obstruction
        .get("blocked_ids")
        .and_then(Value::as_array)?
        .iter()
        .filter_map(Value::as_str)
        .find_map(|id| find_cell(space, Some(id)))
}

pub(super) fn best_related_cell<'a>(
    space: &'a AdvisorySpaceEnvelope,
    blocked: &Value,
    cell_types: &[&str],
) -> Option<&'a Value> {
    space
        .cells
        .iter()
        .filter(|cell| {
            let cell_type = cell.get("cell_type").and_then(Value::as_str);
            cell_type.is_some_and(|value| cell_types.contains(&value))
        })
        .filter(|cell| related_cell_score(blocked, cell) > 0)
        .max_by_key(|cell| related_cell_score(blocked, cell))
}

pub(super) fn related_cell_confidence(blocked: &Value, candidate: &Value, base: f64) -> f64 {
    let score = related_cell_score(blocked, candidate);
    let bump = if score >= 3 {
        0.08
    } else if score >= 2 {
        0.04
    } else {
        0.0
    };
    (base + bump).min(0.88)
}

pub(super) fn related_cell_score(blocked: &Value, candidate: &Value) -> usize {
    let context_overlap = overlap_count(
        &optional_strings(blocked, "context_ids"),
        &optional_strings(candidate, "context_ids"),
    );
    let source_overlap = overlap_count(
        &optional_strings(blocked, "source_ids"),
        &optional_strings(candidate, "source_ids"),
    );
    (context_overlap * 2) + source_overlap
}

pub(super) fn overlap_count(left: &[String], right: &[String]) -> usize {
    left.iter().filter(|item| right.contains(item)).count()
}

pub(super) fn optional_strings(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

pub(super) fn witness_cell_id<'a>(
    space: &'a AdvisorySpaceEnvelope,
    obstruction: &'a Value,
    predicate: impl Fn(&Value) -> bool,
) -> Option<&'a str> {
    obstruction
        .get("witness_ids")
        .and_then(Value::as_array)?
        .iter()
        .filter_map(Value::as_str)
        .find(|id| find_cell(space, Some(id)).map(&predicate).unwrap_or(false))
}

pub(super) fn title(value: &Value) -> &str {
    value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_else(|| json_id(value))
}

pub(super) fn data_store_domain_title(title: &str) -> String {
    title
        .trim_end_matches(" Database")
        .trim_end_matches(" database")
        .trim_end_matches(" DB")
        .trim_end_matches(" db")
        .to_string()
}

pub(super) fn completion_source_ids(
    space: &AdvisorySpaceEnvelope,
    obstruction: &Value,
) -> Vec<String> {
    let mut source_ids = obstruction
        .get("evidence_ids")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .flat_map(|id| {
            if id.starts_with("source:") {
                vec![id.to_string()]
            } else {
                evidence_cell_source_ids(space, id)
            }
        })
        .collect::<Vec<_>>();
    source_ids.sort();
    source_ids.dedup();
    source_ids
}

pub(super) fn obstruction_string_array(obstruction: &Value, field: &str) -> Vec<String> {
    match obstruction.get(field) {
        Some(Value::String(value)) => vec![value.to_string()],
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

pub(super) fn obstruction_invariant_ids(
    check_report: &ReportEnvelope,
    obstruction_id: &str,
) -> Vec<String> {
    let mut invariant_ids = check_report
        .result
        .get("invariant_results")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|result| {
            result
                .get("obstruction_ids")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|id| id.as_str() == Some(obstruction_id))
        })
        .filter_map(|result| result.get("invariant_id").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    invariant_ids.sort();
    invariant_ids.dedup();
    invariant_ids
}

pub(super) fn evidence_cell_source_ids(
    space: &AdvisorySpaceEnvelope,
    evidence_id: &str,
) -> Vec<String> {
    let Some(cell) = find_cell(space, Some(evidence_id)) else {
        return Vec::new();
    };
    cell.pointer("/metadata/source_id")
        .and_then(Value::as_str)
        .map(|id| vec![id.to_string()])
        .unwrap_or_else(|| {
            cell.get("source_ids")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
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

pub(super) fn hg_id(value: &str) -> AdvisoryResult<Id> {
    Id::new(value).map_err(hg_err)
}

pub(super) fn hg_err(error: higher_graphen_core::CoreError) -> advisorygraphen_core::AdvisoryError {
    advisorygraphen_core::AdvisoryError::Validation(format!("higher-graphen completion: {error}"))
}
