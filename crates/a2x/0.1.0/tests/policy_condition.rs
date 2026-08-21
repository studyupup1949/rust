//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use a2x::xacml::xpolicyentry::XPolicyEntry;
use common::{compile_alfa_src, get_nth_policyset};
//use common::get_nth_policy;
use pretty_assertions::assert_eq;
mod common;

// Integration tests for conditions inside rules
//
// This primarily handles sections 4.11 (conditions) of
// alfa-for-xacml-v1.0-csd01

// Here we focus on policies that have conditions.

#[test]
fn policy_condition() {
    let x = compile_alfa_src(
        r#"
namespace main {
  policy c = "foo-ID" {
    apply firstApplicable
    target clause resourceId == "res-test"
    condition subjectId == "sub-test"
    rule {
      deny
    }
  }
}
"#,
    );
    // should have 1 policy file result
    assert_eq!(x.len(), 1);
    let p = get_nth_policyset(0, x);
    // the top level combining alg should be on-permit-apply-second
    assert_eq!(
        p.combining_alg,
        "urn:oasis:names:tc:xacml:3.0:policy-combining-algorithm:on-permit-apply-second"
    );
    // the name of the container policy should be the same as the original ALFA one.
    assert_eq!(p.id, "foo-ID", "ID matches ALFA");
    // expect two child policies
    assert_eq!(p.children.len(), 2);
    // first policy is for holding the rule with a condition
    let cond_policy_ent = p.children.first().expect("conditional policy exists");
    let XPolicyEntry::Policy(cond_policy) = cond_policy_ent else {
        panic!("Expected a Policy")
    };
    // cond policy name is now the parent + "cond"
    assert_eq!(
        cond_policy.id,
        "https://sr.ht/~gheartsfield/a2x/alfa/ident/main/c/cond"
    );

    // condition policy uses combining algorithm permit-overrides
    assert_eq!(
        cond_policy.combining_alg,
        "urn:oasis:names:tc:xacml:3.0:rule-combining-algorithm:permit-overrides"
    );
    // Policy contains a rule, with effect permit
    let cond_rule = cond_policy
        .rules
        .first()
        .expect("conditional policy contains a rule");
    // Rule effect
    assert_eq!(
        cond_rule.effect, "Permit",
        "conditional policy effect is permit"
    );
    assert_eq!(cond_rule.target.anyofs.len(), 0, "Target should be empty");
    // first policy is for holding the rule with a condition
    let orig_policy_ent = p.children.get(1).expect("conditional policy exists");
    // second policy
    let XPolicyEntry::Policy(orig_policy) = orig_policy_ent else {
        panic!("Expected a Policy")
    };
    // original policy name is now the parent + "orig"
    assert_eq!(
        orig_policy.id,
        "https://sr.ht/~gheartsfield/a2x/alfa/ident/main/c/orig"
    );
    // the original policy uses whatever we specified in the ALFA definition.
    assert_eq!(
        orig_policy.combining_alg,
        "urn:oasis:names:tc:xacml:1.0:rule-combining-algorithm:first-applicable"
    );
    // Our target is present
    assert_eq!(
        orig_policy.target.anyofs.len(),
        1,
        "Target should not be empty"
    );
    // Original rule is present, with effect
    let orig_rule = orig_policy.rules.first().expect("original policy has rule");
    assert_eq!(
        orig_rule.effect, "Deny",
        "original policy effect is preserved"
    );
}

// If referencing a policy with a condition from a policyset, ensure
// that the child entry in the policyset is referenced with
// PolicySetId, not PolicyId.
#[test]
fn policyset_policy_ref() {
    let x = compile_alfa_src(
        r#"
namespace main {
  policyset a = "a" {
    apply firstApplicable
    c
  }
  // this will be de-conditioned and transformed into a policyset
  policy c = "will-become-a-policyset" {
    apply firstApplicable
    condition subjectId == "sub-test"
  }
}
"#,
    );
    // should have 2 policy files
    assert_eq!(x.len(), 2);
    // the first is our policyset (a)
    let p = get_nth_policyset(0, x);
    assert_eq!(p.id, "a");
    // there is one child reference, and it is a policyset.
    assert_eq!(p.children.len(), 1);
    let policy_c = p.children.first().unwrap();
    if let XPolicyEntry::PolicySetIdRef(c) = policy_c {
	assert_eq!(c, "will-become-a-policyset");
    } else {
	panic!("Expected a policySetId, not a PolicyId");
    }
}
