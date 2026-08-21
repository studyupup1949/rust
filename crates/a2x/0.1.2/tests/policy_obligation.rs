//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use common::{compile_alfa_src, get_nth_policy};
use pretty_assertions::assert_eq;
mod common;

// Integration tests for policy obligations.
//
// This primarily handles sections 4.15 (obligations and advice) of
// alfa-for-xacml-v1.0-csd01

// Here we focus on the obligations.

/// Simple Obligation
#[test]
fn simple_obligation() {
    let x = compile_alfa_src(
        r#"
namespace main {
  obligation o1 = "urn:example:oblig"
  policy p {
    apply firstApplicable
    rule {
      permit
    }
    on permit {
      obligation o1
    }
  }
}
"#,
    );
    // there will be a single policy container returned.
    assert_eq!(x.len(), 1);
}

/// Complex Obligations
#[test]
fn complex_obligation() {
    // Simple rule with a target, outside a policy.  Compiles, but
    // will not be serialized without a policy that uses it.
    let x = compile_alfa_src(
        r#"
namespace main {
  obligation o1 = "urn:example:oblig"
  policy p {
    apply firstApplicable
    rule {
      permit
    }
    on permit {
      obligation o1 {
        actionId = "foo"
        resourceId = resourceLocation[mustbepresent issuer="bar"]
      }
    }
  }
}
"#,
    );
    // there will be a single policy container returned.
    assert_eq!(x.len(), 1);
    let p = get_nth_policy(0, x);
    let expressions = p.prescriptions.exprs;
    // there should be an obligation expression
    assert_eq!(expressions.len(), 1);
}
