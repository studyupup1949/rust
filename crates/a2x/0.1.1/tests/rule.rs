//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use a2x::xacml::xpolicyentry::XPolicyEntry;
use common::compile_alfa_src;
use common::get_nth_policy;
use common::get_nth_policyset;
use pretty_assertions::assert_eq;
mod common;

// Integration tests for rules.
//
// This primarily handles sections 4.7 (rules) of
// alfa-for-xacml-v1.0-csd01

// This is mostly minimal rule structure within a policy, not so much
// conditions/targets/etc.

/// Simplest Possible Rule
#[test]
fn minimal_rule() {
    // simple rule with an effect.  Will not be serialized since there is no policy.
    let x = compile_alfa_src(
        r"
namespace main {
  rule {
    permit
  }
}
",
    );
    assert_eq!(x.len(), 0);
}

/// Simplest Possible Named Rule
#[test]
fn minimal_named_rule() {
    // simple rule with an effect.  Will not be serialized since there is no policy.
    let x = compile_alfa_src(
        r"
namespace main {
  rule R1 {
    permit
  }
}
",
    );
    assert_eq!(x.len(), 0);
}

/// Rule Defined in Policy
#[test]
fn minimal_rule_in_policy() {
    // simple rule with an effect.  Will not be serialized since there is no policy.
    let x = compile_alfa_src(
        r"
namespace main {
  policy foo {
    apply firstApplicable
    rule R1 {
      permit
    }
  }
}
",
    );
    let p = get_nth_policy(0, x);
    // there is exactly one rule in the foo policy
    assert_eq!(p.rules.len(), 1);
}

// Rule with Description
#[test]
fn rule_with_description() {
    // simple rule with a description
    let x = compile_alfa_src(
        r"
namespace main {
  policy foo {
    apply firstApplicable
    /** This rule permits **/
    rule R1 {
      permit
    }
  }
}
",
    );
    let p = get_nth_policy(0, x);
    // there is exactly one rule in the foo policy
    assert_eq!(p.rules.len(), 1);
    let r = p.rules.first().unwrap();
    assert_eq!(r.description, Some("This rule permits".to_owned()));
}

/// Rule with Effect
#[test]
fn rule_effects() {
    // simple rule with a description
    let x = compile_alfa_src(
        r"
namespace main {
  policy foo {
    apply firstApplicable
    /** This rule permits **/
    rule R1 {
      permit
    }
    /** This rule denies **/
    rule R2 {
      deny
    }
  }
}
",
    );
    let p = get_nth_policy(0, x);
    // there are exactly two rules in the foo policy
    assert_eq!(p.rules.len(), 2);
    let r_permit = p.rules.first().unwrap();
    let r_deny = p.rules.get(1).unwrap();
    assert_eq!(r_permit.effect, "Permit");
    assert_eq!(r_deny.effect, "Deny");
}

/// Rule Missing Effect
#[test]
#[should_panic(expected = "compile failed")]
fn rule_without_effect() {
    // two rules with same name, will not compile.
    compile_alfa_src(
        r"
namespace main {
  policy foo {
    apply firstApplicable
    rule R1 {
      // rules must have effects.
    }
  }
}
",
    );
}

/// Rule Missing Effect
#[test]
#[should_panic(expected = "compile failed")]
fn rule_dupe_effect() {
    // two rules with same name, will not compile.
    compile_alfa_src(
        r"
namespace main {
  policy foo {
    apply firstApplicable
    rule R1 {
      // rules must have only one effect.
      permit
      permit
    }
  }
}
",
    );
}

/// Rule Missing Effect
#[test]
#[should_panic(expected = "compile failed")]
fn rule_dupe_effect2() {
    // two rules with same name, will not compile.
    compile_alfa_src(
        r"
namespace main {
  policy foo {
    apply firstApplicable
    rule R1 {
      // rules must have only one effect.
      permit
      // not two.
      deny
    }
  }
}
",
    );
}

/// Duplicate Rule Name
#[test]
#[should_panic(expected = "compile failed")]
fn dupe_rule_name() {
    // two rules with same name, will not compile.
    compile_alfa_src(
        r"
namespace main {
  policy foo {
    apply firstApplicable
    /** This rule permits **/
    rule R1 {
      permit
    }
    /** This rule is a dupe **/
    rule R1 {
      permit
    }
  }
}
",
    );
}

/// Rule Reference
#[test]
fn rule_reference() {
    // rule referenced from same namespace
    let x = compile_alfa_src(
        r"
namespace main {
  /** This rule permits **/
  rule R1 {
    permit
  }
  policy foo {
    apply firstApplicable
    R1
  }
}
",
    );
    let p = get_nth_policy(0, x);
    // there is exactly one rule in the foo policy
    assert_eq!(p.rules.len(), 1);
    let r = p.rules.first().unwrap();
    assert_eq!(r.description, Some("This rule permits".to_owned()));
}

/// Rule Reference (Separate Namespace)
#[test]
fn rule_reference_other_ns() {
    // rule referenced from another namespace
    let x = compile_alfa_src(
        r"
namespace rules {
  /** This rule permits **/
  rule R1 {
    permit
  }
}
namespace main {
  policy foo {
    apply firstApplicable
    rules.R1
  }
}
",
    );
    let p = get_nth_policy(0, x);
    // there is exactly one rule in the foo policy
    assert_eq!(p.rules.len(), 1);
    let r = p.rules.first().unwrap();
    assert_eq!(r.description, Some("This rule permits".to_owned()));
}

/// Invalid Rule Name (dot)
#[test]
#[should_panic(expected = "compile failed")]
fn invalid_rule_name_dot() {
    // rule with dot in name
    compile_alfa_src(
        r"
namespace main {
  policy foo {
    apply firstApplicable
    /** This rule permits **/
    rule bar.R1 {
      permit
    }
  }
}
",
    );
}

/// Invalid Rule Name (leading number)
#[test]
#[should_panic(expected = "compile failed")]
fn invalid_rule_name_leading_digit() {
    // rule with leading number
    compile_alfa_src(
        r"
namespace main {
  policy foo {
    apply firstApplicable
    /** This rule permits **/
    rule 1R {
      permit
    }
  }
}
",
    );
}

/// Invalid Rule Name (dash)
#[test]
#[should_panic(expected = "compile failed")]
fn invalid_rule_name_leading_dash() {
    // rule with leading number
    compile_alfa_src(
        r"
namespace main {
  policy foo {
    apply firstApplicable
    /** This rule permits **/
    rule R-1 {
      permit
    }
  }
}
",
    );
}

/// Valid Rule Names
#[test]
#[should_panic(expected = "compile failed")]
fn valid_rule_names() {
    // variety of legal rule names.
    // bad idea to use keywords such as "rule", "policy", etc. as names, but not technically invalid.
    let x = compile_alfa_src(
        r"
namespace main {
  policy foo {
    apply firstApplicable
    rule {
      permit
    }
    rule R_1 {
      permit
    }
    rule _R_2 {
      permit
    }
    rule R_3__ {
      permit
    }
    rule policy {
      permit
    }
    rule rule {
      permit
    }
    rule policyset {
      permit
    }
    rule namespace {
      permit
    }
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 7);
}

/// Rule named rule
#[test]
fn rule_named_rule() {
    // A bad idea, but legal.
    let x = compile_alfa_src(
        r"
namespace main {
  rule rule {
    permit
  }

  policy foo {
    apply firstApplicable
    rule
  }
}
",
    );
    let p = get_nth_policy(0, x);
    assert_eq!(p.rules.len(), 1);
}

/// Rule Reference (Separate Namespace + Policy)
#[test]
fn child_rule_reference_other_ns() {
    // policy child rule referenced from another namespace and policy
    let x = compile_alfa_src(
        r"
namespace foo {
  policy p {
    apply firstApplicable
    /** This rule permits **/
    rule R1 {
      permit
    }
  }
}
namespace main {
  policy bar {
    apply firstApplicable
    foo.p.R1
  }
}
",
    );
    // get the 2nd policy
    let p = get_nth_policy(1, x);
    // there is exactly one rule in the foo policy
    assert_eq!(p.rules.len(), 1);
    let r = p.rules.first().unwrap();
    assert_eq!(r.description, Some("This rule permits".to_owned()));
}

/// Rule Deconditioning
#[test]
fn child_rule_decondition() {
    // this checks some more complicated relative references for rules
    // inside policies that are subject to deconditioning.
    let x = compile_alfa_src(
        r#"
namespace foo {
  attribute baz {
    id = "urn:baz"
    type = string
    category = subjectCat
  }
  policy p {
    target clause baz == "test"
    condition baz == "test"
    apply firstApplicable
    /** This rule permits **/
    rule R1 {
      permit
      target clause baz == "test"
    }
  }
}
namespace main {
  policy bar {
    condition foo.baz == "test"
    apply firstApplicable
    foo.p.R1
  }
}
"#,
    );
    // get the 2nd policy entry, which is a policyset
    let p = get_nth_policyset(1, x);
    // there should be two child policies
    assert_eq!(p.children.len(), 2);
    // the second policy should have our R1 rule
    if let Some(cond_policy) = p.children.get(1) {
        match cond_policy {
            XPolicyEntry::Policy(a) => {
                assert_eq!(a.rules.len(), 1);
                let r = a.rules.first().unwrap();
                assert_eq!(r.description, Some("This rule permits".to_owned()));
            }
            _ => {
                panic!("child should not be a policyset");
            }
        }
    } else {
        panic!("there must be a child policy containing the rule R1");
    }
}
