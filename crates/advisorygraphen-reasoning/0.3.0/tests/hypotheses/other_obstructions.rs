use super::*;

#[test]
fn missing_owner_emits_three_competing_hypotheses() {
    let action = json!({
        "id": "cell:ship-action",
        "cell_type": "action",
        "title": "Ship dashboard widget",
        "summary": null,
        "context_ids": [],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance(),
        "metadata": {}
    });
    let space = AdvisorySpaceEnvelope {
        schema: "advisorygraphen.space.v1".to_string(),
        space_id: "space:test-owner".to_string(),
        engagement_id: "engagement:test".to_string(),
        snapshot_id: "snapshot:test".to_string(),
        package_id: "package:technical_advisory_mvp".to_string(),
        cells: vec![action],
        contexts: vec![],
        incidences: vec![],
        morphisms: vec![],
        invariants: vec![],
        policies: vec![],
        metadata: serde_json::Map::new(),
    };

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let hypotheses = report.result["hypotheses"].as_array().unwrap();

    assert_eq!(hypotheses.len(), 3);
    assert!(hypotheses
        .iter()
        .any(|h| h["id"].as_str().unwrap().ends_with("-no-team-holds-action")));
    assert!(hypotheses.iter().any(|h| h["id"]
        .as_str()
        .unwrap()
        .ends_with("-de-facto-owner-link-missing")));
    assert!(hypotheses
        .iter()
        .any(|h| h["id"].as_str().unwrap().ends_with("-collective-ownership")));
}

#[test]
fn requirement_unverified_emits_three_competing_hypotheses() {
    let requirement = json!({
        "id": "cell:requirement",
        "cell_type": "requirement",
        "title": "Audit logs must capture user actions",
        "summary": null,
        "context_ids": [],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance(),
        "metadata": { "require_verification": true }
    });
    let space = AdvisorySpaceEnvelope {
        schema: "advisorygraphen.space.v1".to_string(),
        space_id: "space:test-requirement".to_string(),
        engagement_id: "engagement:test".to_string(),
        snapshot_id: "snapshot:test".to_string(),
        package_id: "package:technical_advisory_mvp".to_string(),
        cells: vec![requirement],
        contexts: vec![],
        incidences: vec![],
        morphisms: vec![],
        invariants: vec![],
        policies: vec![],
        metadata: serde_json::Map::new(),
    };

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let hypotheses = report.result["hypotheses"].as_array().unwrap();

    assert_eq!(hypotheses.len(), 3);
    assert!(hypotheses.iter().any(|h| h["id"]
        .as_str()
        .unwrap()
        .ends_with("-verification-genuinely-missing")));
    assert!(hypotheses.iter().any(|h| h["id"]
        .as_str()
        .unwrap()
        .ends_with("-test-exists-link-missing")));
    assert!(hypotheses.iter().any(|h| h["id"]
        .as_str()
        .unwrap()
        .ends_with("-requirement-is-exploratory")));
}

#[test]
fn insufficient_evidence_emits_three_competing_hypotheses() {
    let action = json!({
        "id": "cell:inferred-action",
        "cell_type": "action",
        "title": "Inferred recommendation",
        "summary": null,
        "context_ids": [],
        "source_ids": [],
        "structure_refs": [],
        "provenance": {
            "origin": "inferred",
            "actor": "tester",
            "confidence": 1.0,
            "review_status": "accepted"
        },
        "metadata": {}
    });
    let space = AdvisorySpaceEnvelope {
        schema: "advisorygraphen.space.v1".to_string(),
        space_id: "space:test-evidence".to_string(),
        engagement_id: "engagement:test".to_string(),
        snapshot_id: "snapshot:test".to_string(),
        package_id: "package:technical_advisory_mvp".to_string(),
        cells: vec![action],
        contexts: vec![],
        incidences: vec![],
        morphisms: vec![],
        invariants: vec![],
        policies: vec![],
        metadata: serde_json::Map::new(),
    };

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let hypotheses = report.result["hypotheses"].as_array().unwrap();
    let evidence_hypotheses: Vec<&Value> = hypotheses
        .iter()
        .filter(|h| {
            h["id"]
                .as_str()
                .unwrap_or("")
                .contains("insufficient-evidence")
        })
        .collect();

    assert_eq!(evidence_hypotheses.len(), 3);
    assert!(evidence_hypotheses.iter().any(|h| h["id"]
        .as_str()
        .unwrap()
        .ends_with("-source-evidence-genuinely-missing")));
    assert!(evidence_hypotheses.iter().any(|h| h["id"]
        .as_str()
        .unwrap()
        .ends_with("-evidence-exists-link-missing")));
    assert!(evidence_hypotheses.iter().any(|h| h["id"]
        .as_str()
        .unwrap()
        .ends_with("-accepted-as-judgment-call")));
}

#[test]
fn circular_dependency_emits_three_competing_hypotheses() {
    let cells = vec![
        component_cell("cell:service-a", "Service A", "context:platform"),
        component_cell("cell:service-b", "Service B", "context:platform"),
        component_cell("cell:service-c", "Service C", "context:platform"),
    ];
    let incidences = vec![
        depends_on_incidence("incidence:a-b", "cell:service-a", "cell:service-b"),
        depends_on_incidence("incidence:b-c", "cell:service-b", "cell:service-c"),
        depends_on_incidence("incidence:c-a", "cell:service-c", "cell:service-a"),
    ];
    let space = AdvisorySpaceEnvelope {
        schema: "advisorygraphen.space.v1".to_string(),
        space_id: "space:test-cycle".to_string(),
        engagement_id: "engagement:test".to_string(),
        snapshot_id: "snapshot:test".to_string(),
        package_id: "package:technical_advisory_mvp".to_string(),
        cells,
        contexts: vec![context("context:platform", "Platform")],
        incidences,
        morphisms: vec![],
        invariants: vec![],
        policies: vec![],
        metadata: serde_json::Map::new(),
    };

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let hypotheses = report.result["hypotheses"].as_array().unwrap();

    assert_eq!(hypotheses.len(), 3);
    assert!(hypotheses
        .iter()
        .any(|h| h["id"].as_str().unwrap().ends_with("-true-runtime-cycle")));
    assert!(hypotheses
        .iter()
        .any(|h| h["id"].as_str().unwrap().ends_with("-edge-misclassified")));
    assert!(hypotheses.iter().any(|h| h["id"]
        .as_str()
        .unwrap()
        .ends_with("-cycle-broken-by-runtime-mechanism")));
}
