use advisorygraphen_core::AdvisorySpaceEnvelope;
use advisorygraphen_reasoning::check_space;
use serde_json::{json, Value};

#[path = "hypotheses/api_route.rs"]
mod api_route;
#[path = "hypotheses/boundary.rs"]
mod boundary;
#[path = "hypotheses/other_obstructions.rs"]
mod other_obstructions;

fn component_cell(id: &str, title: &str, context_id: &str) -> Value {
    json!({
        "id": id,
        "cell_type": "component",
        "title": title,
        "summary": null,
        "context_ids": [context_id],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance(),
        "metadata": {}
    })
}

fn depends_on_incidence(id: &str, from: &str, to: &str) -> Value {
    json!({
        "id": id,
        "relation_type": "depends_on",
        "from_id": from,
        "to_id": to,
        "context_ids": [],
        "evidence_ids": [],
        "strength": "hard",
        "provenance": provenance(),
        "metadata": {}
    })
}

#[test]
fn space_without_boundary_obstruction_emits_no_hypotheses() {
    let space = AdvisorySpaceEnvelope {
        schema: "advisorygraphen.space.v1".to_string(),
        space_id: "space:test-empty".to_string(),
        engagement_id: "engagement:test".to_string(),
        snapshot_id: "snapshot:test".to_string(),
        package_id: "package:technical_advisory_mvp".to_string(),
        cells: vec![],
        contexts: vec![],
        incidences: vec![],
        morphisms: vec![],
        invariants: vec![],
        policies: vec![],
        metadata: serde_json::Map::new(),
    };

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();

    assert!(report.result["hypotheses"].as_array().unwrap().is_empty());
    assert!(report.result["falsifiers"].as_array().unwrap().is_empty());
}

fn boundary_space() -> AdvisorySpaceEnvelope {
    AdvisorySpaceEnvelope {
        schema: "advisorygraphen.space.v1".to_string(),
        space_id: "space:test-boundary".to_string(),
        engagement_id: "engagement:test".to_string(),
        snapshot_id: "snapshot:test".to_string(),
        package_id: "package:technical_advisory_mvp".to_string(),
        cells: vec![
            cell(
                "cell:inventory-service",
                "component",
                "Inventory Service",
                "context:inventory",
            ),
            cell(
                "cell:pricing-db",
                "data_store",
                "Pricing DB",
                "context:pricing",
            ),
        ],
        contexts: vec![
            context("context:inventory", "Inventory"),
            context("context:pricing", "Pricing"),
        ],
        incidences: vec![json!({
            "id": "incidence:inventory-service-accesses-pricing-db",
            "relation_type": "accesses",
            "from_id": "cell:inventory-service",
            "to_id": "cell:pricing-db",
            "source_ids": ["source:pricing-note"],
            "evidence_ids": ["source:pricing-note"],
            "provenance": provenance(),
            "metadata": { "access_type": "direct_database_read" }
        })],
        morphisms: vec![],
        invariants: vec![],
        policies: vec![],
        metadata: serde_json::Map::new(),
    }
}

fn cell(id: &str, cell_type: &str, title: &str, context_id: &str) -> Value {
    json!({
        "id": id,
        "cell_type": cell_type,
        "title": title,
        "summary": null,
        "context_ids": [context_id],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance(),
        "metadata": {}
    })
}

fn context(id: &str, title: &str) -> Value {
    json!({
        "id": id,
        "context_type": "bounded_context",
        "title": title,
        "provenance": provenance(),
        "metadata": {}
    })
}

fn provenance() -> Value {
    json!({
        "origin": "source_backed",
        "actor": "tester",
        "confidence": 1.0,
        "review_status": "accepted"
    })
}
