use abnf::types::Node;
use abnf_to_pest::{parse_abnf, render_rules_to_pest, PestyRule};
use indexmap::IndexMap;

/// Whitespace-collapsed single-rule rendering.
fn rendered_single(rules: &IndexMap<String, PestyRule>, name: &str) -> String {
    let one = rules
        .get(name)
        .map(|r| PestyRule {
            silent: false,
            node: r.node.clone(),
        })
        .unwrap();
    let pretty = render_rules_to_pest(std::iter::once((name.to_string(), one)));
    pretty
        .pretty(80)
        .to_string()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn incremental_alternatives_collapse() {
    let abnf = "rule = A\nrule =/ B\nrule =/ C / D\n";
    let rules = parse_abnf(abnf).unwrap();

    let node = &rules.get("rule").unwrap().node;
    match node {
        Node::Alternatives(v) => {
            assert_eq!(v.len(), 4, "expected flat alternatives, got {:?}", node)
        }
        _ => panic!("expected an alternation, got {:?}", node),
    }
    assert_eq!(rendered_single(&rules, "rule"), "rule = { A | B | C | D }");
}

#[test]
fn incremental_without_base_is_initial() {
    // A stray `=/` with no preceding `=` is parsed as `Kind::Incremental`;
    // with nothing to augment, treat it as the initial definition.
    let abnf = "rule =/ A / B\n";
    let rules = parse_abnf(abnf).unwrap();
    assert!(rules.contains_key("rule"));
    assert_eq!(rendered_single(&rules, "rule"), "rule = { A | B }");
}
