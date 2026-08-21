use super::*;

pub fn case_import_workflow(options: &CaseImportOptions) -> AdvisoryResult<Value> {
    let space = read_space(&options.space)?;
    let dir = space_dir(&options.store, &space.space_id);
    fs::create_dir_all(dir.join("materialized"))?;
    fs::create_dir_all(dir.join("logs"))?;
    fs::write(
        dir.join("materialized/space.json"),
        serde_json::to_vec_pretty(&space)?,
    )?;
    fs::write(dir.join("HEAD"), &options.revision_id)?;
    let log_entry = json!({
        "schema": "advisorygraphen.case.log.entry.v1",
        "case_space_id": space.space_id,
        "sequence": 1,
        "entry_id": "log:000001",
        "morphism_id": "morphism:import",
        "source_revision_id": null,
        "target_revision_id": options.revision_id,
        "actor": "advisorygraphen",
        "recorded_at": Utc::now().to_rfc3339(),
        "previous_entry_hash": null,
        "entry_hash": null,
        "payload": { "space_id": space.space_id }
    });
    append_log_line(&dir.join("logs/morphism-log.jsonl"), &log_entry)?;
    Ok(json!({
        "schema": "advisorygraphen.report.v1",
        "report_type": "case_import",
        "report_version": 1,
        "tool": advisorygraphen_core::tool_metadata(None),
        "input": {
            "store": options.store,
            "space_id": space.space_id,
            "revision_id": options.revision_id
        },
        "result": {
            "imported": true,
            "revision_id": options.revision_id,
            "log_entry_id": "log:000001"
        },
        "projection": {},
        "warnings": []
    }))
}

pub fn case_reason_workflow(options: &CaseReasonOptions) -> AdvisoryResult<Value> {
    let head = read_space_head_revision(&options.store, &options.space_id)?;
    let space = read_materialized_space(&options.store, &options.space_id)?;
    let mut check = check_space(&space, "technical_advisory_mvp", None, None)?;
    let mut hypotheses = check
        .result
        .get("hypotheses")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    apply_hypothesis_events(&options.store, &options.space_id, &mut hypotheses)?;
    check.result["hypotheses"] = json!(hypotheses.clone());
    let mut obstructions = check
        .result
        .get("obstructions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    reframe_obstructions(&mut obstructions, &hypotheses);
    check.result["obstructions"] = json!(obstructions.clone());
    let mut completions = propose_completions(&space, &check, "case_reason", None)?;
    let mut candidates = completions
        .result
        .get("completion_candidates")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    extend_candidates_from_supported_hypotheses(&mut candidates, &hypotheses, &obstructions);
    mark_orphaned_candidates(&mut candidates, &hypotheses);
    apply_candidate_reviews(&options.store, &options.space_id, &mut candidates)?;
    completions.result["completion_candidates"] = json!(candidates.clone());
    let blockers = obstructions.clone();
    let resolution_state = blocker_resolution_state(&blockers, &candidates);
    let frontier = frontier_items(&resolution_state);
    let waiting = waiting_items(&resolution_state);
    let agent_report = attach_completion_report(
        serde_json::to_value(&check)?,
        serde_json::to_value(&completions)?,
    )?;
    let mut projection = build_projection(&space, &agent_report, "ai_agent")?;
    projection["case_head_revision"] = json!(head.clone());
    Ok(json!({
        "schema": "advisorygraphen.report.v1",
        "report_type": "case_reason",
        "report_version": 1,
        "tool": advisorygraphen_core::tool_metadata(None),
        "input": {
            "space_id": options.space_id,
            "case_head_revision": head
        },
        "result": {
            "space_id": options.space_id,
            "case_head_revision": head,
            "blockers": blockers,
            "candidate_review_state": candidates,
            "blocker_resolution_state": resolution_state,
            "close_status": close_status(&space, &check),
            "frontier_items": frontier,
            "waiting_items": waiting
        },
        "projection": projection,
        "warnings": []
    }))
}

pub fn case_close_check_workflow(options: &CaseCloseCheckOptions) -> AdvisoryResult<Value> {
    let head = read_space_head_revision(&options.store, &options.space_id)?;
    ensure_base_revision(Some(&head), options.base_revision.as_deref())?;
    let space = read_materialized_space(&options.store, &options.space_id)?;
    let check = check_space(&space, "technical_advisory_mvp", None, None)?;
    let status = close_status(&space, &check);
    Ok(json!({
        "schema": "advisorygraphen.report.v1",
        "report_type": "case_close_check",
        "report_version": 1,
        "tool": advisorygraphen_core::tool_metadata(None),
        "input": {
            "space_id": options.space_id,
            "base_revision": options.base_revision
        },
        "result": status,
        "projection": build_projection(&space, &serde_json::to_value(&check)?, "audit_trace")?,
        "warnings": []
    }))
}

pub(super) fn ensure_new_case_dir(case_dir: &Path) -> AdvisoryResult<()> {
    if case_dir.exists() {
        let mut entries = fs::read_dir(case_dir)?;
        if entries.next().is_some() {
            return Err(AdvisoryError::Validation(format!(
                "case directory must be empty or nonexistent: {}",
                case_dir.display()
            )));
        }
    }
    fs::create_dir_all(case_dir)?;
    fs::create_dir_all(case_dir.join("input"))?;
    fs::create_dir_all(case_dir.join("artifacts/projections"))?;
    Ok(())
}

pub(super) fn read_case_manifest(case_dir: &Path) -> AdvisoryResult<CaseManifest> {
    let path = case_dir.join(CASE_MANIFEST_FILE);
    let manifest: CaseManifest = serde_json::from_slice(&fs::read(&path)?)?;
    if manifest.schema != "advisorygraphen.case.manifest.v1" {
        return Err(AdvisoryError::Validation(format!(
            "unsupported case manifest schema: {}",
            manifest.schema
        )));
    }
    Ok(manifest)
}

pub(super) fn write_case_manifest(case_dir: &Path, manifest: &CaseManifest) -> AdvisoryResult<()> {
    fs::write(
        case_dir.join(CASE_MANIFEST_FILE),
        serde_json::to_vec_pretty(manifest)?,
    )?;
    Ok(())
}

pub(super) fn sync_manifest_head(
    case_dir: &Path,
    manifest: &mut CaseManifest,
) -> AdvisoryResult<()> {
    manifest.head_revision =
        read_space_head_revision(&case_dir.join(&manifest.store_path), &manifest.space_id)?;
    manifest.updated_at = Utc::now().to_rfc3339();
    write_case_manifest(case_dir, manifest)
}

pub(super) fn path_to_manifest_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
