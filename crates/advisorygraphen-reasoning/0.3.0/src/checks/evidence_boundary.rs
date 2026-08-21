use super::*;

pub(super) fn evaluate_recommendation_evidence(
    space: &AdvisorySpaceEnvelope,
    invariant_results: &mut Vec<Value>,
    obstructions: &mut Vec<Value>,
) -> AdvisoryResult<()> {
    for cell in space
        .cells
        .iter()
        .filter(|cell| matches!(cell["cell_type"].as_str(), Some("action" | "decision")))
    {
        let review_status = cell
            .pointer("/provenance/review_status")
            .and_then(Value::as_str);
        if review_status != Some("accepted") || has_accepted_supporting_evidence(cell)? {
            continue;
        }
        let obstruction_id = format!(
            "obstruction:{}-insufficient-evidence",
            json_id(cell).trim_start_matches("cell:")
        );
        let finding = violation_finding(FindingInput {
            space_id: &space.space_id,
            invariant_id: EVIDENCE_INVARIANT,
            obstruction_id: &obstruction_id,
            obstruction_type: "insufficient_evidence",
            severity: "high",
            message: format!(
                "{} is accepted without source-backed or review-promoted evidence.",
                title(cell)
            ),
            witness_ids: vec![json_id(cell).to_string()],
            blocked_ids: vec![cell["id"].clone()],
            evidence_ids: cell
                .get("source_ids")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            recommended_completion_types: vec!["review_promote_evidence", "source_backed_evidence"],
            resolution: "attach source-backed or review-promoted evidence",
            metadata: json!({
                "rule_precision": "review_status_and_supporting_evidence",
                "evidence_strength": "cell_source_ids",
                "specificity": "source_derived"
            }),
        })?;
        invariant_results.push(finding.invariant_result);
        obstructions.push(finding.obstruction);
    }
    Ok(())
}
pub(super) fn evaluate_boundary(
    space: &AdvisorySpaceEnvelope,
    higher_space: &HigherGraphenAdvisorySpace,
    invariant_results: &mut Vec<Value>,
    obstructions: &mut Vec<Value>,
) -> AdvisoryResult<()> {
    for incidence in &space.incidences {
        let Some(higher_incidence) = higher_space.incidence(json_id(incidence)) else {
            continue;
        };
        if higher_incidence.relation_type != "accesses" {
            continue;
        }
        if incidence
            .pointer("/metadata/access_type")
            .and_then(Value::as_str)
            != Some("direct_database_read")
        {
            continue;
        }
        let Some(from) = higher_space.cell(higher_incidence.from_cell_id.as_str()) else {
            continue;
        };
        let Some(to) = higher_space.cell(higher_incidence.to_cell_id.as_str()) else {
            continue;
        };
        if to.cell_type != "data_store" {
            continue;
        }
        let from_contexts = from
            .context_ids
            .iter()
            .map(HigherId::as_str)
            .collect::<Vec<_>>();
        let to_contexts = to
            .context_ids
            .iter()
            .map(HigherId::as_str)
            .collect::<Vec<_>>();
        if !is_cross_context(&from_contexts, &to_contexts) {
            continue;
        }
        let Some(from_advisory) = find_cell(space, Some(higher_incidence.from_cell_id.as_str()))
        else {
            continue;
        };
        let Some(to_advisory) = find_cell(space, Some(higher_incidence.to_cell_id.as_str())) else {
            continue;
        };
        let obstruction_id = boundary_obstruction_id(
            json_id(from_advisory),
            json_id(to_advisory),
            incidence
                .pointer("/metadata/access_type")
                .and_then(Value::as_str)
                .unwrap_or("access"),
        );
        let blocked_ids = incidence
            .pointer("/metadata/blocked_ids")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_else(|| vec![json!("decision:approve-current-architecture")]);
        let finding = violation_finding(FindingInput {
            space_id: &space.space_id,
            invariant_id: BOUNDARY_INVARIANT,
            obstruction_id: &obstruction_id,
            obstruction_type: "boundary_violation",
            severity: "high",
            message: format!(
                "{} directly reads {} across ownership boundary.",
                title(from_advisory),
                title(to_advisory)
            ),
            witness_ids: vec![
                json_id(from_advisory).to_string(),
                json_id(to_advisory).to_string(),
                json_id(incidence).to_string(),
            ],
            blocked_ids,
            evidence_ids: incidence
                .get("evidence_ids")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
            recommended_completion_types: vec!["proposed_interface", "proposed_refactor_action"],
            resolution: "replace cross-context direct database access with an explicit interface",
            metadata: json!({
                "rule_precision": "cross_context_accesses_data_store_with_direct_database_read",
                "evidence_strength": "source_backed_incidence_when_evidence_ids_present",
                "specificity": "source_derived",
                "from_cell_id": json_id(from_advisory),
                "to_cell_id": json_id(to_advisory),
                "incidence_id": json_id(incidence),
                "from_context_ids": from_contexts,
                "to_context_ids": to_contexts
            }),
        })?;
        invariant_results.push(finding.invariant_result);
        obstructions.push(finding.obstruction);
    }
    Ok(())
}
