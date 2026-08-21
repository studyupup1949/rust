//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use a2x::xacml::XTopPolicy;
use common::compile_alfa_src;
use common::xentry_to_str;
use pretty_assertions::assert_eq;
use unwrap::unwrap;
mod common;

// Integration tests for policies.
//
// This primarily handles sections 4.8 (policies) of
// alfa-for-xacml-v1.0-csd01

// This is mostly minimal policy contents & naming, not
// conditions/targets/advice/etc.

/// Simplest Possible Policy
#[test]
fn minimal_policy() {
    let x = compile_alfa_src(
        r"
namespace main {
  policy {
    apply firstApplicable
  }
}
",
    );
    // compile to xacml.
    // expect no top policies, since none were defined.
    assert_eq!(x.len(), 1);
    // name of this policy should be "master"
    let main_p = x.first().unwrap();
    let XTopPolicy::Policy(policy) = main_p else {
        panic!("Expected Policy")
    };
    // We should be using the firstApplicable XACML rule combining algorithm
    assert_eq!(
        policy.combining_alg,
        "urn:oasis:names:tc:xacml:1.0:rule-combining-algorithm:first-applicable"
    );
    // No description
    assert_eq!(policy.description, None);
    // Policy filename has been computed
    assert_eq!(policy.filename, Some("main.policy_0.xml".to_owned()));
    // No rules defined
    assert_eq!(policy.rules.len(), 0);
    // namespace + generated policy ID appended to basename
    assert_eq!(
        policy.id,
        "https://sr.ht/~gheartsfield/a2x/alfa/ident/main/policy_0"
    );
    // Nothing in the target
    assert!(policy.target.anyofs.is_empty());
}

/// Simplest Possible Policy with Name
#[test]
fn minimal_policy_named() {
    let x = compile_alfa_src(
        r"
namespace main {
  /** First Policy **/
  policy master {
    apply firstApplicable
  }
}
",
    );
    // compile to xacml.
    // expect no top policies, since none were defined.
    assert_eq!(x.len(), 1);
    // name of this policy should be "master"
    let main_p = x.first().unwrap();
    let XTopPolicy::Policy(policy) = main_p else {
        panic!("Expected Policy")
    };
    // We should be using the firstApplicable XACML rule combining algorithm
    assert_eq!(
        policy.combining_alg,
        "urn:oasis:names:tc:xacml:1.0:rule-combining-algorithm:first-applicable"
    );
    // No description
    assert_eq!(policy.description, Some("First Policy".to_owned()));
    // Policy filename was set to namespace + policy name
    assert_eq!(policy.filename, Some("main.master.xml".to_owned()));
    // No rules defined
    assert_eq!(policy.rules.len(), 0);
    // namespace + policy name appended to base namespace for IDs.
    assert_eq!(
        policy.id,
        "https://sr.ht/~gheartsfield/a2x/alfa/ident/main/master"
    );
    // Nothing in the target
    assert!(policy.target.anyofs.is_empty());
}

/// Simple Policy with Name and ID
#[test]
fn minimal_policy_name_and_id() {
    let x = compile_alfa_src(
        r#"
namespace main {
  /** First Policy **/
  policy master = "http://example.com/abac/main/master#v1" {
    apply firstApplicable
  }
}
"#,
    );
    // compile to xacml.
    // expect no top policies, since none were defined.
    assert_eq!(x.len(), 1);
    // name of this policy should be "master"
    let main_p = x.first().unwrap();
    let XTopPolicy::Policy(policy) = main_p else {
        panic!("Expected Policy")
    };
    // We should be using the firstApplicable XACML rule combining algorithm
    assert_eq!(
        policy.combining_alg,
        "urn:oasis:names:tc:xacml:1.0:rule-combining-algorithm:first-applicable"
    );
    // Description
    assert_eq!(policy.description, Some("First Policy".to_owned()));
    // Policy filename was set to namespace + policy name
    assert_eq!(policy.filename, Some("main.master.xml".to_owned()));
    // No rules defined
    assert_eq!(policy.rules.len(), 0);
    // namespace + policy name appended to base namespace for IDs.
    assert_eq!(policy.id, "http://example.com/abac/main/master#v1");
    // Nothing in the target
    assert!(policy.target.anyofs.is_empty());
}

// Policy Naming

/// Valid Policy Names
#[test]
fn valid_policy_names() {
    compile_alfa_src(
        r"
namespace main {
  policy foo9 {
    apply firstApplicable
  }
}
",
    );
    compile_alfa_src(
        r"
namespace main {
  policy Foo {
    apply firstApplicable
  }
}
",
    );
    compile_alfa_src(
        r"
namespace main {
  policy Foo_Bar {
    apply firstApplicable
  }
}
",
    );
    compile_alfa_src(
        r"
namespace main {
  policy Foo__Bar9_ {
    apply firstApplicable
  }
}
",
    );
}

/// Invalid Policy Names
#[test]
#[should_panic(expected = "compile failed")]
fn dotted_invalid_policy_name() {
    compile_alfa_src(
        r"
namespace main {
  policy master.foo {
    apply firstApplicable
  }
}
",
    );
}

#[test]
#[should_panic(expected = "compile failed")]
fn number_invalid_policy_name() {
    compile_alfa_src(
        r"
namespace main {
  policy 9foo {
    apply firstApplicable
  }
}
",
    );
}

#[test]
#[should_panic(expected = "compile failed")]
fn dash_invalid_policy_name() {
    compile_alfa_src(
        r"
namespace main {
  policy -foo {
    apply firstApplicable
  }
}
",
    );
}

/// Duplicate Policy Names
#[test]
#[should_panic(expected = "compile failed")]
fn dupe_policy_name() {
    compile_alfa_src(
        r"
namespace main {
  /** First Policy **/
  policy master {
    apply firstApplicable
  }
  /** Duplicate Policy **/
  policy master {
    apply firstApplicable
  }
}
",
    );
}

/// Duplicate Policy IDs
#[test]
#[should_panic(expected = "compile failed")]
fn dupe_policy_id() {
    // these duplicate policy IDs are in different namespaces.  Policy
    // IDs must be globally unique.
    compile_alfa_src(
        r#"
namespace foo {
  /** First Policy **/
  policy master = "master-id" {
    apply firstApplicable
  }
}
namespace bar {
  /** Duplicate Policy **/
  policy master2 = "master-id" {
    apply firstApplicable
  }
}
"#,
    );
}

/// End-to-End ALFA-to-XACML test
#[test]
fn xacml_policy_simple() {
    let xpolicies = compile_alfa_src(
        r#"
namespace foo {
  /** First Policy **/
  policy master = "master-id" {
    apply firstApplicable
  }
}
"#,
    );
    let xp = unwrap!(xpolicies.first(), "there is at least one policy");
    let xacml = xentry_to_str(xp);
    assert_eq!(
        xacml,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xacml3:Policy xmlns:xacml3="urn:oasis:names:tc:xacml:3.0:core:schema:wd-17" PolicyId="master-id" RuleCombiningAlgId="urn:oasis:names:tc:xacml:1.0:rule-combining-algorithm:first-applicable" Version="1.0">
  <xacml3:Description>First Policy</xacml3:Description>
  <xacml3:Target />
</xacml3:Policy>"#
    );
}
