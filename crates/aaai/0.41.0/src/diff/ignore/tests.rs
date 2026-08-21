use super::*;

fn rules(text: &str) -> IgnoreRules {
    IgnoreRules::from_str(text).unwrap()
}

#[test]
fn simple_glob_ignores() {
    let r = rules("target/**\n*.lock");
    assert!(r.is_ignored("target/debug/aaai"));
    assert!(r.is_ignored("Cargo.lock"));
    assert!(!r.is_ignored("src/main.rs"));
}

#[test]
fn negation_un_ignores() {
    let r = rules("*.lock\n!Cargo.lock");
    assert!(r.is_ignored("some.lock"));
    assert!(!r.is_ignored("Cargo.lock"), "negation should un-ignore");
}

#[test]
fn comments_and_blanks_are_skipped() {
    let r = rules("# comment\n\n*.tmp");
    assert!(r.is_ignored("file.tmp"));
    assert!(!r.is_ignored("file.rs"));
}

#[test]
fn empty_ruleset_ignores_nothing() {
    let r = rules("");
    assert!(!r.is_ignored("anything"));
}

#[test]
fn last_rule_wins() {
    // Pattern says ignore all .yaml, then un-ignore audit.yaml, then re-ignore it.
    let r = rules("*.yaml\n!audit.yaml\n*.yaml");
    assert!(r.is_ignored("audit.yaml"), "last *.yaml should win");
}
