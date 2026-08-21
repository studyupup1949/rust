use super::*;

#[test]
fn database_touching_api_route_without_auth_emits_obstruction_and_completion() {
    let route = api_route_cell(
        "cell:api-route-src-app-api-public-data-route-ts-abc123",
        "/api/public-data",
        true,
        false,
        false,
    );
    let space = base_space(vec![route], vec![]);

    let check_report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let completion_report =
        propose_completions(&space, &check_report, "check-report.json", None).unwrap();
    let obstructions = check_report.result["obstructions"].as_array().unwrap();
    let candidates = completion_report.result["completion_candidates"]
        .as_array()
        .unwrap();

    assert!(obstructions.iter().any(|item| {
        item["obstruction_type"] == "api_route_missing_auth"
            && item["severity"] == "high"
            && item["metadata"]["specificity"] == "code_derived"
            && item["metadata"]["route_path"] == "/api/public-data"
            && item["evidence_ids"] == json!(["source:route"])
            && item["metadata"].get("confidence").is_none()
    }));
    assert!(candidates.iter().any(|item| {
        item["candidate_type"] == "proposed_auth_guard"
            && item["metadata"]["specificity"] == "code_derived"
            && item["source_ids"] == json!(["source:route"])
    }));
}

#[test]
fn generic_completion_content_records_missing_structure_obstructions() {
    let space = base_space(vec![action_cell("cell:ship-action")], vec![]);
    let check_report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let completion_report =
        propose_completions(&space, &check_report, "check-report.json", None).unwrap();
    let candidate = completion_report.result["completion_candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["candidate_type"] == "ownership_clarification")
        .expect("ownership clarification candidate");

    assert_eq!(
        candidate["proposal_content"]["scenario"]["status"],
        "blocked"
    );
    assert!(candidate["proposal_content"]["content_obstructions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["obstruction_type"] == "proposal_content_underspecified"));
    assert!(!candidate["proposal_content"]["content_obstructions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["obstruction_type"] == "proposal_content_missing_source_witness"));
    assert_eq!(candidate["source_ids"], json!(["source:test"]));
    assert!(candidate["proposal_content"]["policy"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|rule| rule
            .as_str()
            .unwrap()
            .contains("must not accept it as current state")));
}

#[test]
fn owner_completion_uses_related_owner_cell_when_available() {
    let mut action = action_cell("cell:ship-action");
    action["context_ids"] = json!(["context:release"]);
    let owner = owner_cell("cell:release-team", "Release Team", "context:release");
    let space = base_space(vec![action, owner], vec![]);

    let check_report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let completion_report =
        propose_completions(&space, &check_report, "check-report.json", None).unwrap();
    let candidate = completion_report.result["completion_candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["candidate_type"] == "owner_assignment")
        .expect("owner assignment candidate");

    assert_eq!(candidate["metadata"]["owner_cell_id"], "cell:release-team");
    assert_eq!(
        candidate["proposal_content"]["scenario"]["status"],
        "blocked"
    );
    assert_eq!(candidate["recommendation_role"], "follow_up_observation");
    assert!(candidate["proposed_incidence_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id.as_str().unwrap().contains("-owns-")));
    assert!(candidate["proposal_content"]["content_obstructions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["obstruction_type"] == "proposal_depends_on_unsupported_hypothesis"));
}

#[test]
fn verification_completion_links_related_test_when_available() {
    let mut requirement = json!({
        "id": "cell:requirement",
        "cell_type": "requirement",
        "title": "Requirement",
        "summary": null,
        "context_ids": ["context:release"],
        "source_ids": ["source:test"],
        "structure_refs": [],
        "provenance": provenance("source_backed", "accepted"),
        "metadata": { "require_verification": true }
    });
    requirement["context_ids"] = json!(["context:release"]);
    let verification = verification_cell(
        "cell:release-smoke-test",
        "Release smoke test",
        "context:release",
    );
    let space = base_space(vec![requirement, verification], vec![]);

    let check_report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let completion_report =
        propose_completions(&space, &check_report, "check-report.json", None).unwrap();
    let candidate = completion_report.result["completion_candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["candidate_type"] == "lift_verification_link")
        .expect("verification link candidate");

    assert_eq!(
        candidate["metadata"]["verification_cell_id"],
        "cell:release-smoke-test"
    );
    assert_eq!(
        candidate["proposal_content"]["scenario"]["status"],
        "blocked"
    );
    assert_eq!(candidate["recommendation_role"], "follow_up_observation");
    assert!(candidate["proposed_incidence_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id.as_str().unwrap().contains("-verifies-")));
    assert!(candidate["proposal_content"]["content_obstructions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["obstruction_type"] == "proposal_depends_on_unsupported_hypothesis"));
}
