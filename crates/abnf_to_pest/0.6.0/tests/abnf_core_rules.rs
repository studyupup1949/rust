use abnf_to_pest::{abnf_core_rules, parse_abnf, render_rules_to_pest};

/// The names of the RFC 5234 §B.1 core rules, in table order.
const CORE_RULE_NAMES: &[&str] = &[
    "ALPHA", "BIT", "CHAR", "CR", "CRLF", "DIGIT", "DQUOTE", "HEXDIG", "HTAB", "LF", "LWSP",
    "OCTET", "SP", "VCHAR", "WSP",
];

/// Lines of a whitespace-collapsed rendering.
fn rendered(abnf: &str) -> Vec<String> {
    let rules = parse_abnf(abnf).unwrap();
    render_rules_to_pest(rules)
        .pretty(80)
        .to_string()
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

/// First token of each line, i.e. the rule name.
fn rule_names(lines: &[String]) -> Vec<&str> {
    lines
        .iter()
        .map(|l| l.split_whitespace().next().unwrap_or(""))
        .collect()
}

#[test]
fn abnf_core_rules_exposes_all_core_rules() {
    let names: Vec<&str> = abnf_core_rules().map(|(name, _)| name).collect();
    assert_eq!(names, CORE_RULE_NAMES);
}

#[test]
fn referenced_but_undefined_core_rule_is_injected() {
    let lines = rendered("myrule = ALPHA SP\n");
    let names = rule_names(&lines);

    assert!(names.contains(&"myrule"));
    assert!(names.contains(&"ALPHA"));
    assert!(names.contains(&"SP"));
}

#[test]
fn unreferenced_core_rules_are_not_injected() {
    let lines = rendered("myrule = \"x\"\n");
    assert_eq!(rule_names(&lines), &["myrule"]);
}

#[test]
fn user_defined_rule_of_same_name_is_not_overridden() {
    let lines = rendered("ALPHA = \"x\"\nmyrule = ALPHA\n");

    let alpha_lines: Vec<&String> = lines.iter().filter(|l| l.starts_with("ALPHA ")).collect();
    assert_eq!(alpha_lines.len(), 1, "expected exactly one ALPHA rule");
    assert_eq!(*alpha_lines[0], "ALPHA = { ^\"x\" }");
}

#[test]
fn core_rule_dependencies_are_pulled_in_transitively() {
    // CRLF references CR and LF; referencing CRLF should pull in CR and LF too.
    let lines = rendered("myrule = CRLF\n");
    let names = rule_names(&lines);

    assert!(names.contains(&"myrule"));
    assert!(names.contains(&"CRLF"));
    assert!(names.contains(&"CR"));
    assert!(names.contains(&"LF"));
    // WSP is not referenced transitively here.
    assert!(!names.contains(&"WSP"));
}

#[test]
fn lwsp_pulls_in_its_full_dependency_chain() {
    // LWSP = *(WSP / CRLF WSP) → pulls in WSP, CRLF, CR, LF, SP, HTAB.
    let lines = rendered("myrule = LWSP\n");
    let names = rule_names(&lines);
    for expected in &["myrule", "LWSP", "WSP", "CRLF", "CR", "LF", "SP", "HTAB"] {
        assert!(
            names.contains(expected),
            "expected {} to be injected, got {:?}",
            expected,
            names
        );
    }
}

#[test]
fn hexdig_renders_as_expected() {
    let lines = rendered("myrule = HEXDIG\n");
    let hexdig = lines
        .iter()
        .find(|l| l.starts_with("HEXDIG "))
        .expect("HEXDIG should be injected");
    assert_eq!(*hexdig, "HEXDIG = { ASCII_HEX_DIGIT }");
}

#[test]
fn alpha_uses_pest_builtin() {
    let lines = rendered("myrule = ALPHA\n");
    let alpha = lines
        .iter()
        .find(|l| l.starts_with("ALPHA "))
        .expect("ALPHA should be injected");
    assert_eq!(*alpha, "ALPHA = { ASCII_ALPHA }");
}

#[test]
fn bit_uses_pest_builtin() {
    let lines = rendered("myrule = BIT\n");
    let bit = lines
        .iter()
        .find(|l| l.starts_with("BIT "))
        .expect("BIT should be injected");
    assert_eq!(*bit, "BIT = { ASCII_BIN_DIGIT }");
}

#[test]
fn digit_uses_pest_builtin() {
    let lines = rendered("myrule = DIGIT\n");
    let digit = lines
        .iter()
        .find(|l| l.starts_with("DIGIT "))
        .expect("DIGIT should be injected");
    assert_eq!(*digit, "DIGIT = { ASCII_DIGIT }");
}

#[test]
fn crlf_renders_as_concatenation() {
    let lines = rendered("myrule = CRLF\n");
    let crlf = lines
        .iter()
        .find(|l| l.starts_with("CRLF "))
        .expect("CRLF should be injected");
    assert_eq!(*crlf, "CRLF = { CR ~ LF }");
}
