use super::*;

pub fn facade_report_workflow(options: &FacadeReportOptions) -> AdvisoryResult<String> {
    let manifest = read_case_manifest(&options.case_dir)?;
    let rendered = if options.audience == "ai_agent" {
        let reasoned = case_reason_workflow(&CaseReasonOptions {
            store: options.case_dir.join(&manifest.store_path),
            space_id: manifest.space_id.clone(),
        })?;
        let projection = reasoned
            .get("projection")
            .cloned()
            .unwrap_or_else(|| json!({}));
        match options.format {
            advisorygraphen_projection::OutputFormat::Json => {
                serde_json::to_string_pretty(&projection)?
            }
            advisorygraphen_projection::OutputFormat::Markdown => {
                return Err(AdvisoryError::Validation(
                    "ai_agent facade report supports json format only".to_string(),
                ))
            }
        }
    } else {
        project_workflow(&ProjectOptions {
            space: options.case_dir.join(&manifest.artifacts.space),
            report: options.case_dir.join(&manifest.artifacts.check_report),
            completions_report: Some(
                options
                    .case_dir
                    .join(&manifest.artifacts.completions_report),
            ),
            audience: options.audience.clone(),
            format: options.format,
            output: None,
        })?
    };
    write_string_if_requested(&options.output, &rendered)?;
    Ok(rendered)
}

pub fn facade_completion_review_workflow(
    options: &FacadeCompletionReviewOptions,
) -> AdvisoryResult<Value> {
    let mut manifest = read_case_manifest(&options.case_dir)?;
    let store = options.case_dir.join(&manifest.store_path);
    let head = read_imported_space_head(&store, &manifest.space_id)?;
    let event = review_workflow(&ReviewOptions {
        store: store.clone(),
        candidate_id: options.candidate_id.clone(),
        from_report: Some(
            options
                .case_dir
                .join(&manifest.artifacts.completions_report),
        ),
        reviewer: options.reviewer.clone(),
        reason: options.reason.clone(),
        outcome: options.outcome.clone(),
        base_revision: Some(head),
    })?;
    sync_manifest_head(&options.case_dir, &mut manifest)?;
    Ok(json!({
        "schema": "advisorygraphen.report.v1",
        "report_type": "facade_completion_review",
        "report_version": 1,
        "tool": advisorygraphen_core::tool_metadata(None),
        "input": {
            "case_dir": options.case_dir,
            "candidate_id": options.candidate_id,
            "outcome": options.outcome
        },
        "result": {
            "review_event": event,
            "case_head_revision": manifest.head_revision,
            "next_commands": [
                "advisorygraphen status --case <case>",
                "advisorygraphen report --case <case> --audience ai_agent"
            ]
        },
        "projection": {},
        "warnings": []
    }))
}

pub fn facade_hypothesis_review_workflow(
    options: &FacadeHypothesisReviewOptions,
) -> AdvisoryResult<Value> {
    let mut manifest = read_case_manifest(&options.case_dir)?;
    let store = options.case_dir.join(&manifest.store_path);
    let head = read_imported_space_head(&store, &manifest.space_id)?;
    let review_options = HypothesisFalsifyOptions {
        store,
        from_report: options.case_dir.join(&manifest.artifacts.check_report),
        hypothesis_id: options.hypothesis_id.clone(),
        evidence_ids: options.evidence_ids.clone(),
        reviewer: options.reviewer.clone(),
        reason: options.reason.clone(),
        base_revision: Some(head),
    };
    let event = match options.outcome.as_str() {
        "support" => hypothesis_support_workflow(&review_options)?,
        "falsify" => hypothesis_falsify_workflow(&review_options)?,
        "accept" => hypothesis_accept_workflow(&review_options)?,
        "reject" => hypothesis_reject_workflow(&review_options)?,
        other => {
            return Err(AdvisoryError::Validation(format!(
                "unsupported hypothesis review outcome: {other}"
            )))
        }
    };
    sync_manifest_head(&options.case_dir, &mut manifest)?;
    Ok(json!({
        "schema": "advisorygraphen.report.v1",
        "report_type": "facade_hypothesis_review",
        "report_version": 1,
        "tool": advisorygraphen_core::tool_metadata(None),
        "input": {
            "case_dir": options.case_dir,
            "hypothesis_id": options.hypothesis_id,
            "outcome": options.outcome
        },
        "result": {
            "review_event": event,
            "case_head_revision": manifest.head_revision,
            "next_commands": [
                "advisorygraphen status --case <case>",
                "advisorygraphen report --case <case> --audience ai_agent"
            ]
        },
        "projection": {},
        "warnings": []
    }))
}
