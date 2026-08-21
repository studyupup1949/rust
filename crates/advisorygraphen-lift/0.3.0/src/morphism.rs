use advisorygraphen_core::{array_field, record_to_cell_id, string_field, AdvisoryResult};
use higher_graphen_core::{
    Confidence, Id, Provenance, ReviewStatus, Severity, SourceKind, SourceRef,
};
use higher_graphen_structure::morphism::{
    CellMapping, LostStructure, Morphism, MorphismType, RelationMapping,
};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::{hg_id, higher_error};

pub(crate) fn source_to_space_morphism(
    snapshot: &Value,
    records: &[Value],
    space_id: &str,
) -> AdvisoryResult<Morphism> {
    let source_space_id = hg_id(string_field(snapshot, "snapshot_id")?)?;
    let mut cell_mapping: CellMapping = BTreeMap::new();
    let mut relation_mapping: RelationMapping = BTreeMap::new();

    for source in array_field(snapshot, "sources")? {
        let Some(source_id) = source.get("id").and_then(Value::as_str) else {
            continue;
        };
        cell_mapping.insert(
            hg_id(source_id)?,
            hg_id(&format!(
                "cell:evidence-{}",
                source_id.trim_start_matches("source:")
            ))?,
        );
    }

    for record in records {
        let Some(record_id) = record.get("id").and_then(Value::as_str) else {
            continue;
        };
        if record
            .get("relation")
            .is_some_and(|relation| !relation.is_null())
        {
            relation_mapping.insert(
                hg_id(record_id)?,
                hg_id(&format!(
                    "incidence:{}",
                    record_id.trim_start_matches("record:")
                ))?,
            );
        } else {
            cell_mapping.insert(hg_id(record_id)?, hg_id(&record_to_cell_id(record_id))?);
        }
    }

    let lost_structure = lost_structure_from_boundary(snapshot, &source_space_id)?;
    Ok(Morphism {
        id: hg_id("morphism:source-to-advisory-space")?,
        source_space_id,
        target_space_id: hg_id(space_id)?,
        name: "JSON snapshot to AdvisoryGraphen space".to_string(),
        morphism_type: MorphismType::Lift,
        cell_mapping,
        relation_mapping,
        preserved_invariant_ids: Vec::new(),
        lost_structure,
        distortion: Vec::new(),
        composable_with: Vec::new(),
        provenance: hg_source_adapter_provenance()?,
    })
}

fn lost_structure_from_boundary(
    snapshot: &Value,
    source_space_id: &Id,
) -> AdvisoryResult<Vec<LostStructure>> {
    let Some(boundary) = snapshot.get("source_boundary") else {
        return Ok(Vec::new());
    };
    let mut lost = Vec::new();
    for reason in boundary_strings(boundary, "extraction_loss") {
        lost.push(lost_structure(source_space_id, reason, Severity::Low)?);
    }
    for reason in boundary_strings(boundary, "excluded_summary") {
        lost.push(lost_structure(source_space_id, reason, Severity::Medium)?);
    }
    lost.sort_by(|left, right| {
        left.severity
            .cmp(&right.severity)
            .then_with(|| left.reason.cmp(&right.reason))
    });
    Ok(lost)
}

fn boundary_strings(boundary: &Value, field: &str) -> Vec<String> {
    boundary
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn lost_structure(
    source_space_id: &Id,
    reason: String,
    severity: Severity,
) -> AdvisoryResult<LostStructure> {
    Ok(LostStructure {
        source_element_id: source_space_id.clone(),
        reason,
        severity,
    })
}

fn hg_source_adapter_provenance() -> AdvisoryResult<Provenance> {
    Ok(Provenance::new(
        SourceRef::new(SourceKind::Document)
            .with_title("source-adapter:json")
            .map_err(higher_error)?,
        Confidence::new(1.0).map_err(higher_error)?,
    )
    .with_review_status(ReviewStatus::Accepted))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn lost_structure_uses_source_boundary_origin_severity() {
        let snapshot = json!({
            "source_boundary": {
                "extraction_loss": ["summarized prose"],
                "excluded_summary": ["source code omitted"]
            }
        });
        let source_space_id = hg_id("snapshot:test").unwrap();

        let lost = lost_structure_from_boundary(&snapshot, &source_space_id).unwrap();

        assert_eq!(lost.len(), 2);
        assert_eq!(lost[0].source_element_id.as_str(), "snapshot:test");
        assert_eq!(lost[0].reason, "summarized prose");
        assert_eq!(lost[0].severity, Severity::Low);
        assert_eq!(lost[1].reason, "source code omitted");
        assert_eq!(lost[1].severity, Severity::Medium);
    }
}
