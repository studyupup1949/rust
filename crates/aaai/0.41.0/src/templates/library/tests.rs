use super::*;

#[test]
fn all_templates_produce_valid_strategies() {
    for tmpl in TEMPLATES {
        let strat = (tmpl.strategy)();
        // None and LineMatch with placeholder are valid (or expected to need filling)
        // Just ensure no panics
        let _ = strat.label();
    }
}

#[test]
fn find_by_id_works() {
    assert!(find("version_bump").is_some());
    assert!(find("nonexistent").is_none());
}

#[test]
fn no_duplicate_ids() {
    let mut ids: Vec<&str> = TEMPLATES.iter().map(|t| t.id).collect();
    let original_len = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), original_len, "duplicate template IDs found");
}
