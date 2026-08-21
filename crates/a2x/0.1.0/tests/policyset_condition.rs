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

// Here we focus on policysets that have conditions.

#[test]
fn policyset_condition() {
    let x = compile_alfa_src(
        r#"
namespace main {
  policyset x = "foo-ID" {
    apply firstApplicable
    target clause resourceId == "res-test"
    condition subjectId == "sub-test"
    policy y {
    apply firstApplicable
      rule {
        deny
      }
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
        "https://sr.ht/~gheartsfield/a2x/alfa/ident/main/x/cond"
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
    let orig_policyset_ent = p.children.get(1).expect("conditional policy exists");
    // second policyset

    let XPolicyEntry::PolicySet(orig_policyset) = orig_policyset_ent else {
        panic!("Expected a PolicySet")
    };
    // original policy name is now the parent + "orig"
    assert_eq!(
        orig_policyset.id,
        "https://sr.ht/~gheartsfield/a2x/alfa/ident/main/x/orig"
    );
    // the original policy uses whatever we specified in the ALFA definition.
    assert_eq!(
        orig_policyset.combining_alg,
        "urn:oasis:names:tc:xacml:1.0:policy-combining-algorithm:first-applicable"
    );
    // Our target is present
    assert_eq!(
        orig_policyset.target.anyofs.len(),
        1,
        "Target should not be empty"
    );
    // Original policyset has a child policy
    let XPolicyEntry::Policy(orig_policy_child) = orig_policyset
        .children
        .first()
        .expect("original policyset has a child policy")
    else {
        panic!("Expected a Policy");
    };
    // Original rule is present, with effect
    let orig_rule = orig_policy_child
        .rules
        .first()
        .expect("original policy has rule");
    assert_eq!(
        orig_rule.effect, "Deny",
        "original policy effect is preserved"
    );
}
