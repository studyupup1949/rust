use advisorygraphen_core::{AdvisoryError, AdvisoryResult, AdvisorySpaceEnvelope};
use higher_graphen_core::{Id, Severity};
use higher_graphen_projection::{
    measure_projection_loss, InformationLoss, OutputSchema, Projection, ProjectionAudience,
    ProjectionEntry, ProjectionLossReport, ProjectionOutput, ProjectionPurpose, ProjectionResult,
    ProjectionSelector, RendererKind,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub struct ProjectionArtifacts {
    pub result_json: Value,
    pub loss_metrics: Value,
}

struct ProjectionTrace {
    entries: Vec<ProjectionEntry>,
    source_trace_gap_ids: Vec<String>,
}

pub fn projection_artifacts(
    space: &AdvisorySpaceEnvelope,
    report: &Value,
    audience: &str,
    omitted_ids: Vec<String>,
) -> AdvisoryResult<ProjectionArtifacts> {
    let trace = projection_trace(space, report)?;
    let output_source_ids = output_source_ids(&trace.entries, space)?;
    let (output_schema, output) = projection_output(&trace.entries)?;
    let projection = Projection::new(
        id(&format!(
            "projection:higher:{}:{}",
            audience,
            space.space_id.trim_start_matches("space:")
        ))?,
        id(&space.space_id)?,
        format!("AdvisoryGraphen {audience} projection"),
        audience_for(audience)?,
        purpose_for(audience),
        selector(report)?,
        output_schema,
        [information_loss(&omitted_ids, space)?],
    )
    .map_err(hg_err)?
    .with_renderer(renderer_for(audience)?);

    let result = ProjectionResult::from_projection(
        &projection,
        projection
            .renderer
            .clone()
            .unwrap_or(RendererKind::Structured),
        output,
        output_source_ids,
        projection.information_loss.clone(),
    )
    .map_err(hg_err)?;
    let eligible_source_ids = source_ids_from_space(space)?;
    let report = measure_projection_loss(&result, &eligible_source_ids);
    let loss_metrics = projection_loss_metrics(space, &report, &trace.source_trace_gap_ids);
    Ok(ProjectionArtifacts {
        loss_metrics,
        result_json: serde_json::to_value(result)?,
    })
}

fn selector(report: &Value) -> AdvisoryResult<ProjectionSelector> {
    let obstruction_ids = report
        .pointer("/result/obstructions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("id").and_then(Value::as_str))
        .map(id)
        .collect::<AdvisoryResult<Vec<_>>>()?;
    Ok(ProjectionSelector::all()
        .with_obstruction_ids(obstruction_ids)
        .with_min_severity(Severity::Low))
}

fn information_loss(
    omitted_ids: &[String],
    space: &AdvisorySpaceEnvelope,
) -> AdvisoryResult<InformationLoss> {
    let source_ids = omitted_ids
        .iter()
        .map(|source_id| id(source_id))
        .collect::<AdvisoryResult<Vec<_>>>()?;
    let source_ids = if source_ids.is_empty() {
        let space_source_ids = source_ids_from_space(space)?;
        if space_source_ids.is_empty() {
            vec![id(&space.space_id)?]
        } else {
            space_source_ids
        }
    } else {
        source_ids
    };
    InformationLoss::declared(
        "Projection omits or summarizes source material from the advisory space.",
        source_ids,
    )
    .map_err(hg_err)
}

fn output_source_ids(
    entries: &[ProjectionEntry],
    space: &AdvisorySpaceEnvelope,
) -> AdvisoryResult<Vec<Id>> {
    let ids = entries
        .iter()
        .flat_map(|entry| entry.source_ids.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if ids.is_empty() {
        Ok(vec![id(&space.space_id)?])
    } else {
        Ok(ids)
    }
}

fn projection_output(
    entries: &[ProjectionEntry],
) -> AdvisoryResult<(OutputSchema, ProjectionOutput)> {
    if entries.is_empty() {
        return Ok((
            OutputSchema::text(),
            ProjectionOutput::text("No represented projection items carry source attribution.")
                .map_err(hg_err)?,
        ));
    }
    // HigherGraphen key_value output requires at least one key.
    let keys = entries
        .iter()
        .map(|entry| entry.key.clone())
        .collect::<Vec<_>>();
    Ok((
        OutputSchema::key_value(keys).map_err(hg_err)?,
        ProjectionOutput::key_value(entries.iter().cloned()).map_err(hg_err)?,
    ))
}

fn projection_trace(
    space: &AdvisorySpaceEnvelope,
    report: &Value,
) -> AdvisoryResult<ProjectionTrace> {
    let mut entries = Vec::new();
    let mut source_trace_gap_ids = Vec::new();
    for item in represented_items(report) {
        let source_ids = source_ids_from_item(space, &item.2)?;
        if source_ids.is_empty() {
            source_trace_gap_ids.push(item.0);
        } else {
            entries.push(ProjectionEntry::new(item.0, item.1, source_ids).map_err(hg_err)?);
        }
    }
    Ok(ProjectionTrace {
        entries,
        source_trace_gap_ids,
    })
}

type RepresentedItem = (String, String, Value);

fn represented_items(report: &Value) -> Vec<RepresentedItem> {
    let mut items = tagged_items(report, "/result/obstructions", "obstruction");
    items.extend(tagged_items(
        report,
        "/result/completion_candidates",
        "completion_candidate",
    ));
    items.extend(tagged_items(
        report,
        "/related_reports/completions/result/completion_candidates",
        "completion_candidate",
    ));
    items.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
    items
}

fn tagged_items(report: &Value, pointer: &str, kind: &str) -> Vec<RepresentedItem> {
    report
        .pointer(pointer)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            Some((
                item.get("id").and_then(Value::as_str)?.to_string(),
                kind.to_string(),
                item.clone(),
            ))
        })
        .collect()
}

fn source_ids_from_item(space: &AdvisorySpaceEnvelope, item: &Value) -> AdvisoryResult<Vec<Id>> {
    let mut source_ids = advisorygraphen_core::optional_string_array(item, "source_ids")
        .into_iter()
        .collect::<BTreeSet<_>>();
    for evidence_id in advisorygraphen_core::optional_string_array(item, "evidence_ids") {
        if let Some(cell) = space
            .cells
            .iter()
            .find(|cell| cell.get("id").and_then(Value::as_str) == Some(evidence_id.as_str()))
        {
            source_ids.extend(advisorygraphen_core::optional_string_array(
                cell,
                "source_ids",
            ));
        } else {
            source_ids.insert(evidence_id);
        }
    }
    source_ids
        .into_iter()
        .map(|source_id| id(&source_id))
        .collect()
}

fn source_ids_from_space(space: &AdvisorySpaceEnvelope) -> AdvisoryResult<Vec<Id>> {
    space
        .cells
        .iter()
        .flat_map(|cell| advisorygraphen_core::optional_string_array(cell, "source_ids"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|source_id| id(&source_id))
        .collect()
}

fn projection_loss_metrics(
    space: &AdvisorySpaceEnvelope,
    report: &ProjectionLossReport,
    source_trace_gap_ids: &[String],
) -> Value {
    let metric = &report.metric;
    let ambiguity = &report.ambiguity;
    let measurable_loss = metric.collapsed_pair_count > 0
        || !metric.omitted_source_ids.is_empty()
        || metric.ambiguity_score > 0.0;
    json!({
        "id": format!("projection-loss-metric:{}", space.space_id.trim_start_matches("space:")),
        "metric_type": "projection_loss_metric",
        "source_cardinality_basis": metric.source_cardinality_basis,
        "source_cardinality": metric.source_cardinality,
        "projected_cardinality": metric.projected_cardinality,
        "omitted_source_count": metric.omitted_source_ids.len(),
        "omitted_source_ids": metric.omitted_source_ids.clone(),
        "collapsed_source_distinction_count": metric.collapsed_pair_count,
        "collapsed_pair_count": metric.collapsed_pair_count,
        "distinguished_pair_count": metric.distinguished_pair_count,
        "source_trace_gap_count": source_trace_gap_ids.len(),
        "source_trace_gap_ids": source_trace_gap_ids,
        "loss_declaration_count": metric.declared_loss_source_ids.len(),
        "declared_loss_source_ids": metric.declared_loss_source_ids.clone(),
        "missing_loss_declaration": !ambiguity.missing_loss_declarations.is_empty(),
        "missing_loss_declarations": ambiguity.missing_loss_declarations.clone(),
        "ambiguity": if !ambiguity.missing_loss_declarations.is_empty() {
            "undeclared_loss"
        } else if measurable_loss {
            "declared_loss"
        } else {
            "none_detected"
        },
        "ambiguity_score": metric.ambiguity_score,
        "ambiguous_output_ids": ambiguity.ambiguous_output_ids.clone(),
        "collapsed_source_groups": ambiguity.collapsed_source_groups.clone(),
        "risk_severity": ambiguity.risk_severity,
        "review_signals": ambiguity.obstructions.clone(),
        "review_status": "unreviewed",
        "rule": "Finite metric for what the projection collapses, omits, or leaves without source trace."
    })
}

fn audience_for(value: &str) -> AdvisoryResult<ProjectionAudience> {
    match value {
        "executive" | "client_review" | "cli" => Ok(ProjectionAudience::Executive),
        "developer_action" => Ok(ProjectionAudience::Developer),
        "audit_trace" => Ok(ProjectionAudience::Audit),
        "ai_agent" => Ok(ProjectionAudience::AiAgent),
        other => Err(AdvisoryError::UnsupportedAudience(other.to_string())),
    }
}

fn purpose_for(value: &str) -> ProjectionPurpose {
    match value {
        "developer_action" | "ai_agent" => ProjectionPurpose::ActionPlan,
        "audit_trace" => ProjectionPurpose::Review,
        _ => ProjectionPurpose::Report,
    }
}

fn renderer_for(value: &str) -> AdvisoryResult<RendererKind> {
    match value {
        "executive" | "client_review" | "cli" => Ok(RendererKind::Markdown),
        "developer_action" | "audit_trace" | "ai_agent" => Ok(RendererKind::Structured),
        other => RendererKind::custom(format!("advisory-{other}")).map_err(hg_err),
    }
}

fn id(value: &str) -> AdvisoryResult<Id> {
    Id::new(value).map_err(hg_err)
}

fn hg_err(error: higher_graphen_core::CoreError) -> AdvisoryError {
    AdvisoryError::Validation(format!("higher-graphen projection: {error}"))
}
