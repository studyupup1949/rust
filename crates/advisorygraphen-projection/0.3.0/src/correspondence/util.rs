use super::*;

pub(super) fn participant_ids(candidate: &Value) -> Vec<String> {
    candidate
        .get("participants")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|participant| participant.pointer("/ref/id").and_then(Value::as_str))
        .map(str::to_owned)
        .collect()
}

pub(super) fn participant_roles(candidate: &Value) -> Vec<String> {
    candidate
        .get("participants")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|participant| participant.get("role").and_then(Value::as_str))
        .map(str::to_owned)
        .collect()
}

pub(super) fn unique_sorted(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn normalized_label(value: &Value) -> Option<String> {
    ["title", "summary", "message", "rationale"]
        .into_iter()
        .filter_map(|field| value.get(field).and_then(Value::as_str))
        .map(|text| text.trim().to_ascii_lowercase())
        .find(|text| !text.is_empty())
}

pub(super) fn extend_strings(target: &mut BTreeSet<String>, value: &Value, fields: &[&str]) {
    for field in fields {
        target.extend(string_values(value, field));
    }
}

pub(super) fn string_values(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

pub(super) fn item_id<'a>(value: &'a Value, fields: &[&str]) -> Option<&'a str> {
    fields
        .iter()
        .find_map(|field| value.get(*field).and_then(Value::as_str))
}

pub(super) fn cell_type(value: &Value) -> &str {
    value
        .get("cell_type")
        .and_then(Value::as_str)
        .unwrap_or("cell")
}

pub(super) fn cell_modality(value: &Value) -> Option<&str> {
    match cell_type(value) {
        "hypothesis" => hypothesis_status(value),
        "falsifier" => Some("falsifier"),
        _ => None,
    }
}

pub(super) fn hypothesis_status(value: &Value) -> Option<&str> {
    value
        .pointer("/metadata/hypothesis_status")
        .and_then(Value::as_str)
        .or_else(|| value.get("lifecycle_status").and_then(Value::as_str))
        .or_else(|| value.get("status").and_then(Value::as_str))
}

pub(super) fn ids(values: BTreeSet<String>) -> AdvisoryResult<Vec<Id>> {
    values.into_iter().map(|value| id(&value)).collect()
}

pub(super) fn id(value: &str) -> AdvisoryResult<Id> {
    Id::new(value).map_err(hg_err)
}

pub(super) fn hg_err(error: higher_graphen_core::CoreError) -> AdvisoryError {
    AdvisoryError::Validation(format!("higher-graphen correspondence: {error}"))
}
