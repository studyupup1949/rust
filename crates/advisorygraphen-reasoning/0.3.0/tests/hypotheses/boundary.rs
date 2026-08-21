use super::*;

#[test]
fn boundary_violation_emits_three_competing_hypotheses_and_falsifiers() {
    let space = boundary_space();

    let report = check_space(&space, "technical_advisory_mvp", None, None).unwrap();
    let hypotheses = report.result["hypotheses"].as_array().unwrap();
    let falsifiers = report.result["falsifiers"].as_array().unwrap();
    let incidences = report.result["argumentation_incidences"]
        .as_array()
        .unwrap();

    assert_eq!(
        hypotheses.len(),
        3,
        "primary + 2 alternatives expected, got {hypotheses:#?}"
    );
    assert!(
        hypotheses
            .iter()
            .all(|hypothesis| hypothesis["lifecycle_status"] == "candidate"
                && hypothesis["cell_type"] == "hypothesis"),
        "all hypotheses begin as candidates"
    );
    assert_eq!(falsifiers.len(), 2, "primary + misclassified falsifiers");
    assert!(falsifiers
        .iter()
        .all(|falsifier| falsifier["cell_type"] == "falsifier"),);
    let primary = hypotheses
        .iter()
        .find(|hypothesis| {
            hypothesis["id"]
                .as_str()
                .unwrap_or("")
                .ends_with("-implicit-interface")
        })
        .expect("primary implicit-interface hypothesis");
    let competes_with = primary["metadata"]["competes_with"].as_array().unwrap();
    assert_eq!(
        competes_with.len(),
        2,
        "primary competes with 2 alternatives"
    );
    assert!(
        incidences
            .iter()
            .any(|incidence| incidence["relation_type"] == "competes_with"),
        "competes_with argumentation incidence emitted"
    );
    assert!(
        incidences
            .iter()
            .any(|incidence| incidence["relation_type"] == "explains"),
        "explains argumentation incidence emitted"
    );
    assert!(
        incidences
            .iter()
            .any(|incidence| incidence["relation_type"] == "falsified_by"),
        "falsified_by argumentation incidence emitted"
    );
}
