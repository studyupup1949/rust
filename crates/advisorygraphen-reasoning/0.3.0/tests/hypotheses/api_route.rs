use super::*;

#[test]
fn api_route_missing_auth_emits_three_competing_hypotheses() {
    let route = json!({
        "id": "cell:api-route-test",
        "cell_type": "component",
        "title": "API route /api/widget",
        "summary": null,
        "context_ids": ["context:application"],
        "source_ids": ["source:route"],
        "structure_refs": [],
        "provenance": provenance(),
        "metadata": {
            "component_type": "api_endpoint",
            "route_path": "/api/widget",
            "http_methods": ["GET"],
            "db_access_detected": true,
            "auth_detected": false,
            "public_endpoint": false
        }
    });
    let space = AdvisorySpaceEnvelope {
        schema: "advisorygraphen.space.v1".to_string(),
        space_id: "space:test-auth".to_string(),
        engagement_id: "engagement:test".to_string(),
        snapshot_id: "snapshot:test".to_string(),
        package_id: "package:technical_advisory_mvp".to_string(),
        cells: vec![route],
        contexts: vec![],
        incidences: vec![],
        morphisms: vec![],
        invariants: vec![],
        policies: vec![],
        metadata: serde_json::Map::new(),
    };

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let hypotheses = report.result["hypotheses"].as_array().unwrap();
    let falsifiers = report.result["falsifiers"].as_array().unwrap();

    assert_eq!(hypotheses.len(), 3);
    assert_eq!(falsifiers.len(), 3);
    assert!(hypotheses
        .iter()
        .any(|h| h["id"].as_str().unwrap().ends_with("-truly-unprotected")));
    assert!(hypotheses.iter().any(|h| h["id"]
        .as_str()
        .unwrap()
        .ends_with("-shared-middleware-auth")));
    assert!(hypotheses
        .iter()
        .any(|h| h["id"].as_str().unwrap().ends_with("-intentionally-public")));
}

#[test]
fn api_route_hypotheses_attach_unreviewed_agent_observation_support() {
    let route = json!({
        "id": "cell:api-route-test",
        "cell_type": "component",
        "title": "API route /api/widget",
        "summary": null,
        "context_ids": ["context:application"],
        "source_ids": ["source:route"],
        "structure_refs": [],
        "provenance": provenance(),
        "metadata": {
            "component_type": "api_endpoint",
            "route_path": "/api/widget",
            "http_methods": ["GET"],
            "db_access_detected": true,
            "auth_detected": false,
            "public_endpoint": false
        }
    });
    let observation = json!({
        "id": "cell:agent-observation-route-auth",
        "cell_type": "claim",
        "title": "Agent observed auth context before database access",
        "summary": "An AI agent read the route and observed a framework auth context check before database access.",
        "context_ids": ["context:application"],
        "source_ids": ["source:route"],
        "structure_refs": [],
        "provenance": {
            "origin": "inferred",
            "actor": "ai-agent",
            "confidence": 0.72,
            "review_status": "unreviewed"
        },
        "metadata": {
            "supports_hypothesis_type": "shared_middleware_auth"
        }
    });
    let support = json!({
        "id": "incidence:agent-observation-supports-route",
        "relation_type": "supports",
        "from_id": "cell:agent-observation-route-auth",
        "to_id": "cell:api-route-test",
        "context_ids": ["context:application"],
        "evidence_ids": ["source:route"],
        "strength": "soft",
        "provenance": {
            "origin": "inferred",
            "actor": "ai-agent",
            "confidence": 0.72,
            "review_status": "unreviewed"
        },
        "metadata": {}
    });
    let space = AdvisorySpaceEnvelope {
        schema: "advisorygraphen.space.v1".to_string(),
        space_id: "space:test-auth".to_string(),
        engagement_id: "engagement:test".to_string(),
        snapshot_id: "snapshot:test".to_string(),
        package_id: "package:technical_advisory_mvp".to_string(),
        cells: vec![route, observation],
        contexts: vec![],
        incidences: vec![support],
        morphisms: vec![],
        invariants: vec![],
        policies: vec![],
        metadata: serde_json::Map::new(),
    };

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let hypotheses = report.result["hypotheses"].as_array().unwrap();
    let shared_middleware = hypotheses
        .iter()
        .find(|h| {
            h["id"]
                .as_str()
                .unwrap()
                .ends_with("-shared-middleware-auth")
        })
        .expect("shared middleware hypothesis");
    assert_eq!(
        shared_middleware["metadata"]["evidence_strength"],
        "agent_observation_unreviewed"
    );
    assert_eq!(
        shared_middleware["metadata"]["supported_by"],
        json!(["cell:agent-observation-route-auth"])
    );
    assert!(report.result["argumentation_incidences"]
        .as_array()
        .unwrap()
        .iter()
        .any(|incidence| incidence["relation_type"] == "supported_by"
            && incidence["to_id"] == "cell:agent-observation-route-auth"));
}
