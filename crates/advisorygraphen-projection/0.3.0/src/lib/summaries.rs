use super::*;

pub(super) fn represented_ids(report: &Value) -> Vec<String> {
    obstructions(report)
        .into_iter()
        .chain(completion_candidates(report))
        .filter_map(|item| item.get("id").and_then(Value::as_str).map(str::to_string))
        .collect()
}

pub(super) fn source_ids(space: &AdvisorySpaceEnvelope) -> Vec<String> {
    let mut ids = space
        .cells
        .iter()
        .flat_map(|cell| advisorygraphen_core::optional_string_array(cell, "source_ids"))
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids
}

pub(super) fn projection_loss(space: &AdvisorySpaceEnvelope, report: &Value) -> Vec<Value> {
    let mut entries = vec![json!({
        "loss_type": "omitted_source_text",
        "description": "Source material is represented by structured records and summarized for this projection.",
        "omitted_ids": source_ids(space),
        "severity": "low"
    })];
    let code_derived_obstruction_ids: Vec<Value> = obstructions(report)
        .into_iter()
        .filter(|obstruction| {
            obstruction
                .pointer("/metadata/specificity")
                .and_then(Value::as_str)
                == Some("code_derived")
        })
        .filter_map(|obstruction| obstruction.get("id").cloned())
        .collect();
    if !code_derived_obstruction_ids.is_empty() {
        entries.push(json!({
            "loss_type": "lexical_detection_caveat",
            "description": "Code-derived findings are produced by lexical analysis and may miss shared middleware, dynamic wrappers, or framework-specific conventions; review is required before treating them as accepted fact.",
            "omitted_ids": code_derived_obstruction_ids,
            "severity": "medium"
        }));
    }
    entries
}

pub(super) fn schema_morphisms(space: &AdvisorySpaceEnvelope) -> Value {
    let mut morphisms = space
        .morphisms
        .iter()
        .filter_map(|morphism| morphism.get("schema_morphism").cloned())
        .collect::<Vec<_>>();
    morphisms.extend(
        space
            .metadata
            .get("schema_morphisms")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
    );
    morphisms.sort_by_key(|morphism| {
        morphism
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    });
    morphisms.dedup_by(|a, b| a.get("id") == b.get("id"));
    json!({
        "count": morphisms.len(),
        "morphisms": morphisms,
        "rule": "Schema morphisms describe contract evolution or lift mappings with compatibility, verification, and explicit loss claims."
    })
}

pub(super) fn obstructions(report: &Value) -> Vec<Value> {
    report
        .pointer("/result/obstructions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

pub(super) fn completion_candidates(report: &Value) -> Vec<Value> {
    let mut candidates = report
        .pointer("/result/completion_candidates")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    candidates.extend(
        report
            .pointer("/related_reports/completions/result/completion_candidates")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
    );
    candidates
}

pub(super) fn partition_candidates(candidates: &[Value]) -> (Vec<Value>, Vec<Value>) {
    let mut live = Vec::new();
    let mut superseded = Vec::new();
    for candidate in candidates {
        if candidate.get("review_status").and_then(Value::as_str) == Some("superseded") {
            superseded.push(candidate.clone());
        } else {
            live.push(candidate.clone());
        }
    }
    (live, superseded)
}

pub(super) fn obstructions_by_severity(obstructions: &[Value], severity: &str) -> Vec<Value> {
    obstructions
        .iter()
        .filter(|item| item["severity"] == severity)
        .cloned()
        .collect()
}

pub(super) fn obstruction_counts(obstructions: &[Value]) -> Value {
    let mut high = 0_u64;
    let mut medium = 0_u64;
    let mut low = 0_u64;
    let mut unknown = 0_u64;
    for obstruction in obstructions {
        match obstruction
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
        {
            "high" => high += 1,
            "medium" => medium += 1,
            "low" => low += 1,
            _ => unknown += 1,
        }
    }
    json!({
        "high": high,
        "medium": medium,
        "low": low,
        "unknown": unknown
    })
}

pub(super) fn candidate_quality_summary(candidates: &[Value]) -> Value {
    let mut source_derived = 0_u64;
    let mut requirement_derived = 0_u64;
    let mut code_derived = 0_u64;
    let mut topology_derived = 0_u64;
    let mut generic = 0_u64;
    let mut missing_precision_metadata = 0_u64;
    let mut source_backed = 0_u64;
    for candidate in candidates {
        match candidate
            .pointer("/metadata/specificity")
            .and_then(Value::as_str)
            .unwrap_or("missing")
        {
            "source_derived" => source_derived += 1,
            "requirement_derived" => requirement_derived += 1,
            "code_derived" => code_derived += 1,
            "topology_derived" => topology_derived += 1,
            "generic" => generic += 1,
            _ => missing_precision_metadata += 1,
        }
        if candidate
            .get("source_ids")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .next()
            .is_some()
        {
            source_backed += 1;
        }
    }
    json!({
        "total": candidates.len(),
        "source_derived": source_derived,
        "requirement_derived": requirement_derived,
        "code_derived": code_derived,
        "topology_derived": topology_derived,
        "generic": generic,
        "source_backed": source_backed,
        "missing_precision_metadata": missing_precision_metadata
    })
}

pub(super) fn proposal_content_summary(candidates: &[Value]) -> Value {
    let mut with_structured_content = 0_u64;
    let mut blocked_content = 0_u64;
    let mut candidate_content = 0_u64;
    let mut content_obstruction_count = 0_u64;
    let mut obstruction_types = serde_json::Map::new();

    for candidate in candidates {
        let Some(content) = candidate.get("proposal_content") else {
            continue;
        };
        with_structured_content += 1;
        match content
            .pointer("/scenario/status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
        {
            "blocked" => blocked_content += 1,
            "candidate" => candidate_content += 1,
            _ => {}
        }
        for obstruction in content
            .get("content_obstructions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            content_obstruction_count += 1;
            let key = obstruction
                .get("obstruction_type")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let count = obstruction_types
                .get(key)
                .and_then(Value::as_u64)
                .unwrap_or(0)
                + 1;
            obstruction_types.insert(key.to_string(), json!(count));
        }
    }

    json!({
        "with_structured_content": with_structured_content,
        "candidate_content": candidate_content,
        "blocked_content": blocked_content,
        "content_obstruction_count": content_obstruction_count,
        "content_obstruction_types": obstruction_types
    })
}
