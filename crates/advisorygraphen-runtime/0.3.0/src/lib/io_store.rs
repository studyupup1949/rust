use super::*;

pub fn read_json(path: &Path) -> AdvisoryResult<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub fn write_json_if_requested<T: serde::Serialize>(
    path: &Option<PathBuf>,
    value: &T,
) -> AdvisoryResult<()> {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(value)?)?;
    }
    Ok(())
}

pub fn write_string_if_requested(path: &Option<PathBuf>, value: &str) -> AdvisoryResult<()> {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, value)?;
    }
    Ok(())
}

pub(super) fn read_space(path: &Path) -> AdvisoryResult<AdvisorySpaceEnvelope> {
    let space: AdvisorySpaceEnvelope = serde_json::from_value(read_json(path)?)?;
    advisorygraphen_core::validate_space(&space)?;
    Ok(space)
}

pub(super) fn read_materialized_space(
    store: &Path,
    space_id: &str,
) -> AdvisoryResult<AdvisorySpaceEnvelope> {
    read_space(&space_dir(store, space_id).join("materialized/space.json"))
}

pub(super) fn read_space_head_revision(store: &Path, space_id: &str) -> AdvisoryResult<String> {
    Ok(fs::read_to_string(space_dir(store, space_id).join("HEAD"))?)
}

pub(super) fn read_imported_space_head(store: &Path, space_id: &str) -> AdvisoryResult<String> {
    read_space_head_revision(store, space_id).map_err(|error| match error {
        AdvisoryError::Io(_) => AdvisoryError::Validation(format!(
            "case space {space_id} must be imported before review"
        )),
        other => other,
    })
}

pub(super) fn ensure_base_revision(head: Option<&str>, base: Option<&str>) -> AdvisoryResult<()> {
    let Some(head) = head.map(str::trim) else {
        return Ok(());
    };
    let Some(base) = base else {
        return Err(AdvisoryError::StaleRevision {
            expected: head.to_string(),
            actual: "<missing>".to_string(),
        });
    };
    if head != base {
        return Err(AdvisoryError::StaleRevision {
            expected: head.to_string(),
            actual: base.to_string(),
        });
    }
    Ok(())
}

pub(super) fn space_dir(store: &Path, space_id: &str) -> PathBuf {
    store.join("spaces").join(space_id.replace([':', '/'], "-"))
}

pub(super) fn canonical_schema_name(schema: &str) -> String {
    match schema {
        "engagement_snapshot" | "snapshot" => advisorygraphen_core::SNAPSHOT_SCHEMA,
        "space" | "advisory_space" => advisorygraphen_core::SPACE_SCHEMA,
        "report" => advisorygraphen_core::REPORT_SCHEMA,
        "projection_request" => advisorygraphen_core::PROJECTION_REQUEST_SCHEMA,
        "review_event" => REVIEW_EVENT_SCHEMA,
        "micro_review_request" => advisorygraphen_core::MICRO_REVIEW_REQUEST_SCHEMA,
        other => other,
    }
    .to_string()
}

pub(super) fn file_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("report.json")
}

pub(super) fn append_store_event(store: &Path, value: &Value) -> AdvisoryResult<()> {
    fs::create_dir_all(store.join("logs"))?;
    append_log_line(&store.join("logs/morphism-log.jsonl"), value)
}

pub(super) fn append_log_line(path: &Path, value: &Value) -> AdvisoryResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

pub(super) fn next_sequence(store: &Path, space_id: &str) -> u64 {
    let path = store.join("logs/morphism-log.jsonl");
    fs::read_to_string(path)
        .ok()
        .map(|contents| {
            contents
                .lines()
                .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                .filter(|entry| {
                    entry.get("case_space_id").and_then(Value::as_str) == Some(space_id)
                })
                .count() as u64
                + 1
        })
        .unwrap_or(1)
}
