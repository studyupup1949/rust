//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use a2x::xacml::xtarget::XMatch;
use common::compile_alfa_src;
use common::get_nth_policy;
use pretty_assertions::assert_eq;
mod common;

// Integration tests for rule targets.
//
// This primarily handles sections 4.10 (targets) of
// alfa-for-xacml-v1.0-csd01

// Here we focus on targets within rules.

/// Simplest Possible Target
#[test]
fn minimal_empty_target() {
    // Simple rule with a target, outside a policy.  Compiles, but
    // will not be serialized without a policy that uses it.
    let x = compile_alfa_src(
        r"
namespace main {
  rule {
    target
    permit
  }
}
",
    );
    // there will be no policy containers returned
    assert_eq!(x.len(), 0);
}

/// Simplest Possible Target
#[test]
fn minimal_empty_target_in_policy() {
    let x = compile_alfa_src(
        r"
namespace main {
  policy {
    apply firstApplicable
    rule {
      target
      permit
    }
  }
}
",
    );
    assert_eq!(x.len(), 1);
}

/// Single target clause and operator
#[test]
fn single_clause_operator() {
    // Target with single clause using an operator
    let x = compile_alfa_src(
        r#"
namespace main {
  policy {
    apply firstApplicable
    rule {
      target clause resourceId == "test.txt"
      permit
    }
  }
}
"#,
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
    let r = p.rules.first().unwrap();
    // single clause
    let anyofs = &r.target.anyofs;
    assert_eq!(anyofs.len(), 1);
    // single allof
    let anyof = anyofs.first().unwrap();
    let allofs = &anyof.allofs;
    assert_eq!(allofs.len(), 1);
    // single match
    let allof = allofs.first().unwrap();
    let mchs = &allof.matches;
    assert_eq!(mchs.len(), 1);
    let mch = mchs.first().unwrap();
    let xm = XMatch {
        matchid: "urn:oasis:names:tc:xacml:1.0:function:string-equal".to_owned(),
        value: "test.txt".to_owned(),
        value_type: "http://www.w3.org/2001/XMLSchema#string".to_owned(),
        designator_id: "urn:oasis:names:tc:xacml:1.0:resource:resource-id".to_owned(),
        designator_category: "urn:oasis:names:tc:xacml:3.0:attribute-category:resource".to_owned(),
        designator_type: "http://www.w3.org/2001/XMLSchema#string".to_owned(),
        must_be_present: false,
        issuer: None,
    };
    assert_eq!(mch, &xm);
}

/// Must Be Present
#[test]
fn must_be_present_operator() {
    // Target with single clause using mustbepresent
    let x = compile_alfa_src(
        r#"
namespace main {
  policy {
    apply firstApplicable
    rule {
      target clause resourceId[mustbepresent] == "test.txt"
      permit
    }
  }
}
"#,
    );
    let p = get_nth_policy(0, x);
    let parsed_match = p
        .rules
        .first()
        .unwrap()
        .target
        .anyofs
        .first()
        .unwrap()
        .allofs
        .first()
        .unwrap()
        .matches
        .first()
        .unwrap();
    assert!(parsed_match.must_be_present);
}

/// Must Be Present
#[test]
fn issuer_operator() {
    // Target with single clause using mustbepresent
    let x = compile_alfa_src(
        r#"
namespace main {
  policy {
    apply firstApplicable
    rule {
      target clause resourceId[mustbepresent issuer="urn:example:foobar"] == "test.txt"
      permit
    }
  }
}
"#,
    );
    let p = get_nth_policy(0, x);
    let parsed_match = p
        .rules
        .first()
        .unwrap()
        .target
        .anyofs
        .first()
        .unwrap()
        .allofs
        .first()
        .unwrap()
        .matches
        .first()
        .unwrap();
    assert!(parsed_match.must_be_present);
    assert_eq!(parsed_match.issuer, Some("urn:example:foobar".to_owned()));
}

/// Targets must compare values with Attributes
#[test]
#[should_panic(expected = "compile failed")]
fn target_two_values() {
    // Target with single clause
    compile_alfa_src(
        r"
namespace main {
  policy {
    apply firstApplicable
    rule {
      target clause true == true
      permit
    }
  }
}
",
    );
}

/// Targets must have an operation/function
#[test]
#[should_panic(expected = "compile failed")]
fn target_single_scalar() {
    // Target with single clause
    compile_alfa_src(
        r"
namespace main {
  policy {
    apply firstApplicable
    rule {
      target clause true
      permit
    }
  }
}
",
    );
}

/// Multiple clauses, "or" and "and" statements
#[test]
fn multi_clauses() {
    let x = compile_alfa_src(
        r#"
namespace main {
  policy {
    apply firstApplicable
    rule {
      target clause resourceId == "single"
             clause resourceId == "or1" or resourceId == "or2"
             clause resourceId == "and1" and resourceId == "and2"
                 or resourceId == "or-next-to-last"
                 or resourceId == "or-last"
      permit
    }
  }
}
"#,
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
    let r = p.rules.first().unwrap();
    let anyofs = &r.target.anyofs;
    assert_eq!(anyofs.len(), 3);

    // Clause 1
    let clause1 = anyofs.first().unwrap();
    let c1_allofs = &clause1.allofs;
    let c1_allof = c1_allofs.first().unwrap();
    {
        let mchs = &c1_allof.matches;
        assert_eq!(mchs.len(), 1);
    }
    // Clause 2
    let clause2 = anyofs.get(1).unwrap();
    let c2_allofs = &clause2.allofs;
    assert_eq!(c2_allofs.len(), 2);
    {
        let c2_allof_first = c2_allofs.first().unwrap();
        let mchs = &c2_allof_first.matches;
        assert_eq!(mchs.len(), 1);
        assert_eq!(mchs.first().unwrap().value, "or1");
        let c2_allof_snd = c2_allofs.get(1).unwrap();
        let mchs = &c2_allof_snd.matches;
        assert_eq!(mchs.len(), 1);
        assert_eq!(mchs.first().unwrap().value, "or2");
    }

    // Clause 3
    let clause3 = anyofs.get(2).unwrap();
    let c3_allofs = &clause3.allofs;
    assert_eq!(c3_allofs.len(), 3);
    {
        let c3_allof_first = c3_allofs.first().unwrap();
        let mchs = &c3_allof_first.matches;
        // matches in the first AllOf
        assert_eq!(mchs.len(), 2);
        assert_eq!(mchs.first().unwrap().value, "and1");
        assert_eq!(mchs.get(1).unwrap().value, "and2");
        // matches is the second AllOf
        let c3_allof_snd = c3_allofs.get(1).unwrap();
        let mchs = &c3_allof_snd.matches;
        assert_eq!(mchs.len(), 1);
        assert_eq!(mchs.first().unwrap().value, "or-next-to-last");
        // matches in the last AllOf
        let c3_allof_snd = c3_allofs.get(2).unwrap();
        let mchs = &c3_allof_snd.matches;
        assert_eq!(mchs.len(), 1);
        assert_eq!(mchs.first().unwrap().value, "or-last");
    }
}
