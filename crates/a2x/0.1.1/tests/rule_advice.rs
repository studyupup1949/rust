//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use common::compile_alfa_src;
use pretty_assertions::assert_eq;
mod common;

// Integration tests for rule advice.
//
// This primarily handles sections 4.15 (obligations and advice) of
// alfa-for-xacml-v1.0-csd01

// Here we focus on the advice.

/// Simple Advice
#[test]
fn simple_advice() {
    let x = compile_alfa_src(
        r#"
namespace main {
  advice a1 = "urn:example:advice"
  policy p {
    apply firstApplicable
    rule {
      permit
      on permit {
        advice a1
      }
    }
  }
}
"#,
    );
    // there will be a single policy container returned.
    assert_eq!(x.len(), 1);
}

/// Complex Advice
#[test]
fn complex_advice() {
    let x = compile_alfa_src(
        r#"
namespace main {
  advice a1 = "urn:example:advice"
  policy p {
    apply firstApplicable
    rule {
      permit
      on permit {
        advice a1 {
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
}
