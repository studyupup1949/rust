use super::*;

pub(super) fn render_markdown(audience: &str, projection: &Value) -> AdvisoryResult<String> {
    let mut lines = vec![
        format!(
            "# AdvisoryGraphen {} Projection",
            audience.replace('_', " ")
        ),
        String::new(),
        format!(
            "Space: `{}`",
            projection["space_id"].as_str().unwrap_or("unknown")
        ),
        String::new(),
    ];
    if let Some(obstructions) = projection
        .pointer("/summary/high_severity_obstructions")
        .and_then(Value::as_array)
    {
        if let Some(closeable) = projection
            .pointer("/summary/closeable")
            .and_then(Value::as_bool)
        {
            lines.push("## Close status".to_string());
            lines.push(format!("- Closeable: `{closeable}`"));
            if let Some(blocking_ids) = projection
                .pointer("/summary/blocking_obstruction_ids")
                .and_then(Value::as_array)
            {
                lines.push(format!("- Blocking obstructions: {}", blocking_ids.len()));
            }
            lines.push(String::new());
        }
        if let Some(counts) = projection
            .pointer("/summary/obstruction_counts")
            .and_then(Value::as_object)
        {
            lines.push("## Obstruction summary".to_string());
            for severity in ["high", "medium", "low", "unknown"] {
                let count = counts.get(severity).and_then(Value::as_u64).unwrap_or(0);
                lines.push(format!("- {severity}: {count}"));
            }
            lines.push(String::new());
        }
        if let Some(quality) = projection.pointer("/summary/candidate_quality") {
            lines.push("## Candidate quality".to_string());
            lines.push(format!(
                "- Source-derived: {}",
                quality["source_derived"].as_u64().unwrap_or(0)
            ));
            lines.push(format!(
                "- Requirement-derived: {}",
                quality["requirement_derived"].as_u64().unwrap_or(0)
            ));
            lines.push(format!(
                "- Code-derived: {}",
                quality["code_derived"].as_u64().unwrap_or(0)
            ));
            lines.push(format!(
                "- Generic: {}",
                quality["generic"].as_u64().unwrap_or(0)
            ));
            lines.push(format!(
                "- Missing precision metadata: {}",
                quality["missing_precision_metadata"].as_u64().unwrap_or(0)
            ));
            lines.push(String::new());
        }
        if let Some(summary) = projection.pointer("/summary/proposal_content_summary") {
            lines.push("## Proposal content".to_string());
            lines.push(format!(
                "- With structured content: {}",
                summary["with_structured_content"].as_u64().unwrap_or(0)
            ));
            lines.push(format!(
                "- Blocked content: {}",
                summary["blocked_content"].as_u64().unwrap_or(0)
            ));
            lines.push(format!(
                "- Content obstructions: {}",
                summary["content_obstruction_count"].as_u64().unwrap_or(0)
            ));
            lines.push(String::new());
        }
        if let Some(trace) = projection.pointer("/summary/recommendation_trace") {
            lines.push("## Recommendation trace".to_string());
            lines.push(format!(
                "- Primary recommendations: {}",
                trace["primary_count"].as_u64().unwrap_or(0)
            ));
            lines.push(format!(
                "- Alternatives: {}",
                trace["alternative_count"].as_u64().unwrap_or(0)
            ));
            lines.push(format!(
                "- Follow-up observations: {}",
                trace["follow_up_observation_count"].as_u64().unwrap_or(0)
            ));
            if let Some(items) = trace
                .get("follow_up_observations")
                .and_then(Value::as_array)
            {
                for item in items.iter().take(5) {
                    lines.push(format!(
                        "- Follow-up: `{}` from `{}`: {}",
                        item["candidate_id"].as_str().unwrap_or("unknown"),
                        item["derived_hypothesis_id"].as_str().unwrap_or("missing"),
                        item["title"].as_str().unwrap_or("Untitled follow-up")
                    ));
                    for task in item
                        .get("ranked_observation_tasks")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .take(2)
                    {
                        lines.push(format!(
                            "  - Observation {}: {}",
                            task["rank"].as_u64().unwrap_or(0),
                            task["expected_observation"]
                                .as_str()
                                .unwrap_or("Collect evidence before promotion.")
                        ));
                    }
                }
            }
            lines.push(String::new());
        }
        lines.push("## High-severity obstructions".to_string());
        if obstructions.is_empty() {
            lines.push("- None".to_string());
        } else {
            for obstruction in obstructions {
                lines.push(format!(
                    "- `{}`: {}",
                    obstruction["id"].as_str().unwrap_or("unknown"),
                    obstruction["message"].as_str().unwrap_or("No message.")
                ));
            }
        }
        lines.push(String::new());
    }
    if let Some(obstructions) = projection
        .pointer("/summary/medium_severity_obstructions")
        .and_then(Value::as_array)
    {
        lines.push("## Medium-severity obstructions".to_string());
        if obstructions.is_empty() {
            lines.push("- None".to_string());
        } else {
            for obstruction in obstructions {
                lines.push(format!(
                    "- `{}`: {}",
                    obstruction["id"].as_str().unwrap_or("unknown"),
                    obstruction["message"].as_str().unwrap_or("No message.")
                ));
            }
        }
        lines.push(String::new());
    }
    let mut reframed_obstructions: Vec<&Value> = Vec::new();
    for obstruction in projection
        .pointer("/summary/high_severity_obstructions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .chain(
            projection
                .pointer("/summary/medium_severity_obstructions")
                .and_then(Value::as_array)
                .into_iter()
                .flatten(),
        )
    {
        if obstruction.pointer("/metadata/reframe").is_some() {
            reframed_obstructions.push(obstruction);
        }
    }
    if !reframed_obstructions.is_empty() {
        lines.push("## Reframed obstructions (primary hypothesis falsified)".to_string());
        for obstruction in reframed_obstructions {
            let id = obstruction["id"].as_str().unwrap_or("unknown");
            let original = obstruction
                .pointer("/metadata/reframe/original_severity")
                .and_then(Value::as_str)
                .unwrap_or("?");
            let effective = obstruction
                .pointer("/metadata/reframe/effective_severity")
                .and_then(Value::as_str)
                .unwrap_or("?");
            lines.push(format!(
                "- `{id}`: severity {original} → effective {effective}"
            ));
            if let Some(types) = obstruction
                .pointer("/metadata/reframe/effective_completion_types")
                .and_then(Value::as_array)
            {
                let names: Vec<&str> = types.iter().filter_map(Value::as_str).collect();
                lines.push(format!("  - now suggests: {}", names.join(", ")));
            }
        }
        lines.push(String::new());
    }
    if let Some(summary) = projection.pointer("/summary/hypothesis_summary") {
        let total = summary["total"].as_u64().unwrap_or(0);
        if total > 0 {
            lines.push("## Hypotheses".to_string());
            lines.push(format!("- Total: {}", total));
            for status in [
                "candidate",
                "supported",
                "accepted",
                "rejected",
                "falsified",
            ] {
                let count = summary[status].as_u64().unwrap_or(0);
                if count > 0 {
                    lines.push(format!("- {status}: {count}"));
                }
            }
            lines.push(String::new());
        }
    }
    if let Some(boundary) = projection.get("source_boundary") {
        lines.push("## Source boundary".to_string());
        if let Some(included) = boundary
            .get("included_source_ids")
            .and_then(Value::as_array)
        {
            lines.push(format!("- Included sources: {}", included.len()));
        }
        if let Some(excluded) = boundary.get("excluded_summary").and_then(Value::as_array) {
            for item in excluded {
                lines.push(format!(
                    "- Excluded: {}",
                    item.as_str().unwrap_or("unknown")
                ));
            }
        }
        lines.push(String::new());
    }
    lines.push("## Projection loss".to_string());
    for loss in projection["projection_loss"]
        .as_array()
        .into_iter()
        .flatten()
    {
        lines.push(format!(
            "- `{}`: {}",
            loss["loss_type"].as_str().unwrap_or("loss"),
            loss["description"]
                .as_str()
                .unwrap_or("Projection omitted or compressed information.")
        ));
    }
    Ok(lines.join("\n"))
}
