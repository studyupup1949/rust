//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use a2x::xacml::xexpression::XExpression;
use common::compile_alfa_src;
use common::get_nth_policy;
use pretty_assertions::assert_eq;
mod common;

// Integration tests for conditions inside rules
//
// This primarily handles sections 4.11 (conditions) of
// alfa-for-xacml-v1.0-csd01

// Here we focus on targets within rules.

/// Simplest Possible Condition
#[test]
fn minimal_condition() {
    // Simple policy + rule with a trivial condition

    // While this works and is trivial to convert to XACML, this seems
    // to be outside the ALFA 1.0 CSD01 spec, which requires that the
    // condition is composed of an expression which must be an
    // operator or function.  Since XACML 3 allows
    // AttributeValues/AttributeDesignators at the top-level of a
    // Condition, we maintain this capability.
    let x = compile_alfa_src(
        r"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition true
    }
  }
}
",
    );
    assert_eq!(x.len(), 1);
}

/// Simple Condition with Parens
#[test]
fn minimal_condition_parens() {
    let x = compile_alfa_src(
        r"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition (true)
    }
  }
}
",
    );
    assert_eq!(x.len(), 1);
}

/// Simple Condition with Multi-Parens
#[test]
fn minimal_condition_multiparens() {
    let x = compile_alfa_src(
        r"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition ((((((true))))))
    }
  }
}
",
    );
    assert_eq!(x.len(), 1);
}

/// Empty Condition
#[test]
#[should_panic(expected = "compile failed")]
fn empty_condition() {
    // Conditions must have an expression
    let _x = compile_alfa_src(
        r"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition
    }
  }
}
",
    );
}

/// Condition with many operators
#[test]
fn condition_operators() {
    // Multiple operators
    let _x = compile_alfa_src(
        r"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition (3>3) && true && (4>5) || (3 > (1 + 2 / 3 * 2))
    }
  }
}
",
    );
}

/// Condition with non-boolean type
#[test]
#[should_panic(expected = "compile failed")]
fn condition_value_non_bool() {
    // Multiple operators
    let _x = compile_alfa_src(
        r#"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition "this is not a boolean"
    }
  }
}
"#,
    );
}

/// Condition with non-boolean type
#[test]
#[should_panic(expected = "compile failed")]
fn condition_operators_non_bool() {
    // Multiple operators
    let _x = compile_alfa_src(
        r"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition (3 + 4)
    }
  }
}
",
    );
}

/// Condition with a bag type
#[test]
#[should_panic(expected = "compile failed")]
fn condition_bag_result() {
    // This fails because the type should be boolean.
    let _x = compile_alfa_src(
        r#"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition stringBag("foo", "bar")
    }
  }
}
"#,
    );
}

/// Condition with a computed boolean type
#[test]
fn condition_function_bool() {
    let _x = compile_alfa_src(
        r#"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition booleanOneAndOnly(booleanBag("true"))
    }
  }
}
"#,
    );
}

/// Condition with an infix function
#[test]
fn condition_infix_bool() {
    let _x = compile_alfa_src(
        r#"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition resourceId == stringBag("foo", "bar")
    }
  }
}
"#,
    );
}

/// Condition with a function that does not resolve
#[test]
#[should_panic(expected = "compile failed")]
fn condition_unknown_fn() {
    let _x = compile_alfa_src(
        r#"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition booleanOneAndOnly(fakeFunction("true"))
    }
  }
}
"#,
    );
}

/// Condition with a function of type bag
#[test]
#[should_panic(expected = "compile failed")]
fn condition_boolean_bag() {
    // since the bag is not an atomic boolean, this should fail.
    let _x = compile_alfa_src(
        r#"
namespace main {
  policy {
    apply firstApplicable
    rule {
      permit
      condition booleanBag("true")
    }
  }
}
"#,
    );
}

/// Condition with an attribute designator
#[test]
#[should_panic(expected = "compile failed")]
fn condition_attribute_designator() {
    // since AttributeDesignators resolve to bags, this should fail due to the type not being boolean.
    let _x = compile_alfa_src(
        r#"
namespace main {
  attribute boolAttr {
    id = "test:boolean"
    type = boolean
    category = subjectCat
  }
  policy {
    apply firstApplicable
    rule {
      permit
      condition boolAttr
    }
  }
}
"#,
    );
}

/// Condition with an attribute designator wrapped in function
#[test]
fn condition_attribute_design_fn_wrapped() {
    let _x = compile_alfa_src(
        r#"
namespace main {
  attribute boolAttr {
    id = "test:boolean"
    type = boolean
    category = subjectCat
  }
  policy {
    apply firstApplicable
    rule {
      permit
      condition booleanOneAndOnly(boolAttr)
    }
  }
}
"#,
    );
}

/// Condition with an attribute designator and mustbepresent/issuer
#[test]
fn condition_attribute_design_mustbepresent() {
    let x = compile_alfa_src(
        r#"
namespace main {
  attribute boolAttr {
    id = "test:boolean"
    type = boolean
    category = subjectCat
  }
  policy {
    apply firstApplicable
    rule {
      permit
      condition booleanOneAndOnly(boolAttr[mustbepresent issuer="foo"])
    }
  }
}
"#,
    );
    // Check that the top-level function is correct
    let p = get_nth_policy(0, x);
    let parsed_rule = p
        .rules
        .first() // rule
        .unwrap();
    let expr = &parsed_rule.condition.as_ref().unwrap().expr;
    match expr {
        XExpression::Apply(a) => {
            assert_eq!(
                a.function_uri,
                "urn:oasis:names:tc:xacml:1.0:function:boolean-one-and-only"
            );
            // check the first argument
            let first_arg = a.arguments.first().unwrap();
            match first_arg {
                XExpression::Attrib(ad) => {
                    assert!(ad.must_be_present);
                    assert_eq!(ad.issuer, Some("foo".to_owned()));
                }
                _ => {
                    panic!("first argument was not an attribute designator");
                }
            }
        }
        _ => {
            panic!("expression was not a function");
        }
    }
}
