use advisorygraphen_core::AdvisorySpaceEnvelope;
use advisorygraphen_reasoning::{
    blocker_resolution_state, check_space, frontier_items, propose_completions,
    propose_hypothesis_lifecycle, waiting_items,
};
use serde_json::{json, Value};

#[path = "invariants/basic_invariants.rs"]
mod basic_invariants;
#[path = "invariants/completion_content.rs"]
mod completion_content;
#[path = "invariants/hypothesis_workflow.rs"]
mod hypothesis_workflow;
#[path = "invariants/resolution.rs"]
mod resolution;

fn assert_obstruction(result: &Value, obstruction_type: &str) {
    let obstructions = result["obstructions"].as_array().unwrap();
    assert!(
        obstructions
            .iter()
            .any(|item| item["obstruction_type"] == obstruction_type),
        "expected obstruction_type {obstruction_type}, got {obstructions:#?}"
    );
}

fn relation(id: &str, relation_type: &str, from: &str, to: &str) -> Value {
    json!({
        "id": id,
        "relation_type": relation_type,
        "from_id": from,
        "to_id": to,
        "context_ids": [],
        "evidence_ids": [],
        "strength": "hard",
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {}
    })
}

fn base_space(cells: Vec<Value>, incidences: Vec<Value>) -> AdvisorySpaceEnvelope {
    AdvisorySpaceEnvelope {
        schema: "advisorygraphen.space.v1".to_string(),
        space_id: "space:test".to_string(),
        engagement_id: "engagement:test".to_string(),
        snapshot_id: "snapshot:test".to_string(),
        package_id: "package:technical_advisory_mvp".to_string(),
        cells,
        contexts: vec![],
        incidences,
        morphisms: vec![],
        invariants: vec![],
        policies: vec![],
        metadata: serde_json::Map::new(),
    }
}

fn action_cell(id: &str) -> Value {
    json!({
        "id": id,
        "cell_type": "action",
        "title": "Ship action",
        "summary": null,
        "context_ids": [],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
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
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {}
    })
}

fn component_cell(id: &str, title: &str, context_id: &str) -> Value {
    json!({
        "id": id,
        "cell_type": "component",
        "title": title,
        "summary": null,
        "context_ids": [context_id],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {}
    })
}

fn owner_cell(id: &str, title: &str, context_id: &str) -> Value {
    json!({
        "id": id,
        "cell_type": "owner",
        "title": title,
        "summary": null,
        "context_ids": [context_id],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {}
    })
}

fn verification_cell(id: &str, title: &str, context_id: &str) -> Value {
    json!({
        "id": id,
        "cell_type": "test_or_verification",
        "title": title,
        "summary": null,
        "context_ids": [context_id],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {}
    })
}

fn api_route_cell(
    id: &str,
    route_path: &str,
    db_access_detected: bool,
    auth_detected: bool,
    public_endpoint: bool,
) -> Value {
    json!({
        "id": id,
        "cell_type": "component",
        "title": format!("API route {route_path}"),
        "summary": null,
        "context_ids": ["context:application"],
        "source_ids": ["source:route"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {
            "component_type": "api_endpoint",
            "route_path": route_path,
            "http_methods": ["GET"],
            "db_access_detected": db_access_detected,
            "auth_detected": auth_detected,
            "public_endpoint": public_endpoint
        }
    })
}

fn data_store_cell(id: &str, title: &str, context_id: &str) -> Value {
    json!({
        "id": id,
        "cell_type": "data_store",
        "title": title,
        "summary": null,
        "context_ids": [context_id],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {}
    })
}

fn context(id: &str, title: &str) -> Value {
    json!({
        "id": id,
        "context_type": "bounded_context",
        "title": title,
        "provenance": provenance("source_backed", "accepted"),
        "metadata": {}
    })
}

fn provenance(origin: &str, review_status: &str) -> Value {
    json!({
        "origin": origin,
        "actor": "tester",
        "confidence": 1.0,
        "review_status": review_status
    })
}
