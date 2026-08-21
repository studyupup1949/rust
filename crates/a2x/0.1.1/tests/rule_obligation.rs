//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use a2x::ast::prescription::PrescriptionType;
use a2x::ast::rule::Effect;
use a2x::context::TypedLiteral;
use a2x::xacml::xprescription::XAttrValue;
use a2x::xacml::xprescription::XAttributeAssignmentArgument;
use common::compile_alfa_src;
use common::get_nth_policy;
use pretty_assertions::assert_eq;
use unwrap::unwrap;
mod common;

// Integration tests for rule obligations.
//
// This primarily handles sections 4.15 (obligations and advice) of
// alfa-for-xacml-v1.0-csd01

// Here we focus on the obligations.

/// Simple Obligation
#[test]
fn simple_obligation() {
    // Simple rule in a policy.
    let x = compile_alfa_src(
        r#"
namespace main {
  obligation o1 = "urn:example:oblig"
  policy p {
    apply firstApplicable
    rule {
      permit
      on permit {
        obligation o1 {
        }
      }
    }
  }
}
"#,
    );
    // there will be a single policy container returned.
    assert_eq!(x.len(), 1);
    let p = get_nth_policy(0, x);
    let r = unwrap!(p.rules.first(), "expected a single rule");
    let expressions = &r.prescriptions.exprs;
    // there should be an obligation expression
    let exp = unwrap!(
        expressions.first(),
        "expected a single obligation expression"
    );
    assert_eq!(exp.ptype, PrescriptionType::Obligation);
    assert_eq!(exp.fulfill_on, Effect::Permit);
}

/// Simple Obligation (Deny)
#[test]
fn simple_obligation_deny() {
    // Simple rule in a policy.
    let x = compile_alfa_src(
        r#"
namespace main {
  obligation o1 = "urn:example:oblig"
  policy p {
    apply firstApplicable
    rule {
      permit
      on deny {
        obligation o1 {
        }
      }
    }
  }
}
"#,
    );
    // there will be a single policy container returned.
    assert_eq!(x.len(), 1);
    let p = get_nth_policy(0, x);
    let r = unwrap!(p.rules.first(), "expected a single rule");
    let expressions = &r.prescriptions.exprs;
    // there should be an obligation expression
    let exp = unwrap!(
        expressions.first(),
        "expected a single obligation expression"
    );
    assert_eq!(exp.ptype, PrescriptionType::Obligation);
    assert_eq!(exp.fulfill_on, Effect::Deny);
}

/// Complex Obligations
#[test]
fn complex_obligation() {
    // Simple rule in a policy.
    let x = compile_alfa_src(
        r#"
namespace main {
  obligation o1 = "urn:example:oblig"
  policy p {
    apply firstApplicable
    rule {
      permit
      on permit {
        obligation o1 {
          actionId = "foo"
          resourceId = resourceLocation[mustbepresent issuer="bar"]
        }
      }
    }
  }
}
"#,
    );
    // there will be a single policy container returned.
    assert_eq!(x.len(), 1);
    let p = get_nth_policy(0, x);
    let r = unwrap!(p.rules.first(), "expected a single rule");
    let expressions = &r.prescriptions.exprs;
    // there should be an obligation expression
    let exp = unwrap!(
        expressions.first(),
        "expected a single obligation expression"
    );
    assert_eq!(exp.ptype, PrescriptionType::Obligation);
    assert_eq!(exp.fulfill_on, Effect::Permit);
    assert_eq!(exp.id, "urn:example:oblig");
    // two assignments
    assert_eq!(exp.assignments.len(), 2);
    let first_assignment = unwrap!(
        exp.assignments.first(),
        "expect a first obligation assignment"
    );
    // Assert details of the assignment
    assert_eq!(
        first_assignment.id,
        "urn:oasis:names:tc:xacml:1.0:action:action-id"
    );
    assert_eq!(
        (first_assignment.arg),
        XAttributeAssignmentArgument::Value(XAttrValue {
            v: TypedLiteral {
                type_uri: "http://www.w3.org/2001/XMLSchema#string".to_owned(),
                value: "foo".to_owned()
            }
        })
    );
}
