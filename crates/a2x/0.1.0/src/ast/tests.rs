//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::ast::condition::CondExpression;
use crate::AlfaDocParser;
use pest::Parser;
#[test]
fn test_empty() {
    let root = Namespace::new_root("foo".to_string(), Rc::new(Context::default()));
    assert!(root.is_root());
}

#[test]
fn test_single_line_comment_cleanup() {
    let c = "// foo\n";
    assert_eq!(comment_cleanup(c), "foo");
}

#[test]
fn test_multi_line_comment_cleanup() {
    let c = "/* foo bar */";
    assert_eq!(comment_cleanup(c), "foo bar");
}

#[test]
fn test_multi_line_nl_comment_cleanup() {
    let c = "/*\n foo\n new line\n */";
    assert_eq!(comment_cleanup(c), "foo\n new line");
}

#[test]
fn test_multi_line_star_comment_cleanup() {
    let c = "/**** foo ********/";
    assert_eq!(comment_cleanup(c), "foo");
}
#[test]
fn test_multi_line_nl_star_comment_cleanup() {
    let c = "/**** foo\n* bar ********/";
    assert_eq!(comment_cleanup(c), "foo\n* bar");
}

/// Parsing a double-quoted string into a constant
#[test]
fn string_const_from_token() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#""test""#;
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(c, Constant::String("test".to_string()));
    Ok(())
}

/// Parsing a double-quoted string into a constant
#[test]
fn double_esc_quote_string_const_from_token() -> Result<(), Box<dyn std::error::Error>> {
    // is the problem here that we aren't un-escaping the quotes?
    let input = "\"t\\\"est\"";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(c, Constant::String("t\"est".to_string()));
    Ok(())
}

/// Parsing a single-quoted string into a constant
#[test]
fn single_esc_quote_string_const_from_token() -> Result<(), Box<dyn std::error::Error>> {
    // is the problem here that we aren't un-escaping the quotes?
    let input = "'t\\'est'";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(c, Constant::String("t'est".to_string()));
    Ok(())
}

/// Parsing a single-quoted string into a constant
#[test]
fn string_sq_const_from_token() -> Result<(), Box<dyn std::error::Error>> {
    let input = r"'test'";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(c, Constant::String("test".to_string()));
    Ok(())
}

/// Parsing a boolean into a constant
#[test]
fn bool_const_from_token() -> Result<(), Box<dyn std::error::Error>> {
    let input = r"true";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(c, Constant::Boolean(true));
    let input = r"false";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(c, Constant::Boolean(false));
    Ok(())
}

/// Parsing an integer into a constant
#[test]
fn int_const_from_token() -> Result<(), Box<dyn std::error::Error>> {
    let input = r"42";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(c, Constant::Integer("42".to_string()));
    let input = r"-42";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(c, Constant::Integer("-42".to_string()));
    Ok(())
}

/// Parsing a double into a constant
#[test]
fn double_const_from_token() -> Result<(), Box<dyn std::error::Error>> {
    let input = r"0.42";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(c, Constant::Double("0.42".to_string()));
    let input = r"-.42";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(c, Constant::Double("-.42".to_string()));
    let input = r"-0.42";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(c, Constant::Double("-0.42".to_string()));
    Ok(())
}

/// Parsing a custom datatype into a constant
#[test]
fn custom_const_from_token() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#""127.0.0.1":ipAddress"#;
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(
        c,
        Constant::Custom(
            CustomType {
                name: "ipAddress".to_string()
            },
            "127.0.0.1".to_string()
        )
    );
    Ok(())
}

/// Parsing a single-quoted custom datatype into a constant
#[test]
fn custom_sq_const_from_token() -> Result<(), Box<dyn std::error::Error>> {
    let input = r"'127.0.0.1':ipAddress";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(
        c,
        Constant::Custom(
            CustomType {
                name: "ipAddress".to_string()
            },
            "127.0.0.1".to_string()
        )
    );
    Ok(())
}

/// Parsing a single-quoted custom datatype into a constant with spaces
#[test]
fn custom_sq_spaced_const_from_token() -> Result<(), Box<dyn std::error::Error>> {
    let input = r"'127.0.0.1' :   ipAddress";
    let mut pairs = AlfaDocParser::parse(Rule::literal, input)?;
    let c = constant_from_token(pairs.next().unwrap())?;
    assert_eq!(
        c,
        Constant::Custom(
            CustomType {
                name: "ipAddress".to_string()
            },
            "127.0.0.1".to_string()
        )
    );
    Ok(())
}

/// Parse Advice declaration
#[test]
fn parse_advice_decl() {
    let input = r#"advice myAdvice1 = "http://example.com/advice/my-advice-1""#;
    assert!(AlfaDocParser::parse(Rule::advice_decl, input).is_ok());
    // comments
    let input =
        r#"advice /* comment */ myAdvice1 /* comment */ = "http://example.com/advice/my-advice-1""#;
    assert!(AlfaDocParser::parse(Rule::advice_decl, input).is_ok());
}

/// Parse Obligation declaration
#[test]
fn parse_oblig_decl() {
    let input = r#"obligation myOblig1 = "http://example.com/obligation/my-obligation-1""#;
    assert!(AlfaDocParser::parse(Rule::obligation_decl, input).is_ok());
    // comments
    let input = r#"obligation /* comment */ myOblig1 /* comment */ = /* comment */ "http://example.com/obligation/my-obligation-1""#;
    assert!(AlfaDocParser::parse(Rule::obligation_decl, input).is_ok());
}

/// Advice and Obligation valid names
#[test]
fn reject_invalid_advice_oblg_names() {
    // invalid name (cannot start with number)
    let adv_input = r#"advice 1myAdvice /* comment */ = "http://example.com/advice/my-advice-1""#;
    assert!(AlfaDocParser::parse(Rule::advice_decl, adv_input).is_err());
    let obl_input =
        r#"obligation 1myOblig /* comment */ = "http://example.com/obligation/my-obligation-1""#;
    assert!(AlfaDocParser::parse(Rule::obligation_decl, obl_input).is_err());
    // invalid name (cannot start with character)
    let adv_input = r#"advice -myAdvice /* comment */ = "http://example.com/advice/my-advice-1""#;
    assert!(AlfaDocParser::parse(Rule::advice_decl, adv_input).is_err());
    let obl_input =
        r#"obligation -myOblig /* comment */ = "http://example.com/obligation/my-obligation-1""#;
    assert!(AlfaDocParser::parse(Rule::obligation_decl, obl_input).is_err());
}

/// Parse simple condition
#[test]
fn parse_condition_literal() {
    let input = "condition true";
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());
}

/// Parse simple condition
#[test]
fn parse_condition_simple_op() {
    // Test with an extra hanging operator which should not be included in the rule.
    let input = "condition 3 + 3";
    assert!(AlfaDocParser::parse(Rule::condition_stmt, input).is_ok());
}

/// Parse simple condition, but not too far
#[test]
fn parse_condition_stop_at_extra_op() {
    // Test with an extra hanging operator which should not be included in the rule.
    let input = "condition 3 + 3 +";
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!("condition 3 + 3", c.as_str());
}

/// Parse simple identifier with parens
#[test]
fn parse_condition_simple_paren() {
    let input = "condition (3)";
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());
}

/// Parse simple identifier with parens and ops
#[test]
fn parse_condition_paren_ops() {
    let input = "condition (3)+(3)";
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());
}

/// Parse simple condition with function
#[test]
fn parse_condition_simple_fun() {
    let input = "condition fun(3)";
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());
}

/// Parse simple condition with function and multiple args
#[test]
fn parse_condition_simple_fun_many_args() {
    let input = "condition fun(1,2,3,4,5)";
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());
}

/// Parse simple condition with function and nested operators
#[test]
fn parse_condition_simple_fun_nested_ops() {
    let input = "condition fun(1+1)";
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());

    let input = "condition fun(1+1,2)";
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());

    let input = "condition fun(1,2-3,4-2)";
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());
}

#[test]
fn parse_condition_logical_and() {
    let input = "condition true && true && true";
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());
}

#[test]
fn parse_condition_complex_min() {
    let input = "condition booleanFromString(1)++bar";
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());
}
#[test]
fn parse_condition_complex() {
    let input = r#"condition booleanFromString ( stringSubString("foo", 4+3, 3--4)) && true"#;
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());
}

/// Varieties of attribute designator options, with everything else
#[test]
fn parse_condition_designator_attrib() {
    let input = r#"condition AttribDesig[mustbepresent issuer="foo"] +/ booleanFromString ( stringSubString("foo", AttribDesig[mustbepresent], AttribDesig[issuer="foo"],4+3, 3--4)) && true"#;
    let pairs = AlfaDocParser::parse(Rule::condition_stmt, input);
    assert!(pairs.is_ok());
    let c = pairs.unwrap().next().unwrap();
    assert_eq!(input, c.as_str());
}

#[test]
fn test_process_operator() -> Result<(), Box<dyn std::error::Error>> {
    let input = "main.foo.bar.+/";
    let mut pairs = AlfaDocParser::parse(Rule::operator_identifier, input)?;
    let op_pair = pairs.next().expect("no operator found");
    assert_eq!(
        process_operator(&op_pair)?,
        Operator {
            ns: vec!["main".to_string(), "foo".to_string(), "bar".to_string()],
            operator: "+/".to_string()
        }
    );
    Ok(())
}

#[test]
fn test_cond_display_literal_string() -> Result<(), Box<dyn std::error::Error>> {
    let input = r#"condition "foo""#;
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: \"foo\"");
    Ok(())
}

#[test]
fn test_cond_display_literal_int() -> Result<(), Box<dyn std::error::Error>> {
    let input = "condition 3";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: 3");
    Ok(())
}

#[test]
fn test_cond_display_literal_float() -> Result<(), Box<dyn std::error::Error>> {
    let input = "condition 3.99";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: 3.99");
    Ok(())
}

#[test]
fn test_cond_display_literal_bool() -> Result<(), Box<dyn std::error::Error>> {
    let input = "condition true";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: true");
    Ok(())
}

#[test]
fn test_cond_display_expr() -> Result<(), Box<dyn std::error::Error>> {
    let input = "condition 3 + 3";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: (3 + 3)");
    Ok(())
}

#[test]
fn test_cond_display_expr_paren() -> Result<(), Box<dyn std::error::Error>> {
    let input = "condition (3 + 3)";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: (3 + 3)");
    Ok(())
}

#[test]
fn test_cond_display_expr_mult_op_left_assoc() -> Result<(), Box<dyn std::error::Error>> {
    // left associate operator
    let input = "condition 1 + 2 + 3";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: ((1 + 2) + 3)");
    Ok(())
}

#[test]
fn test_cond_display_expr_mult_op_right_assoc() -> Result<(), Box<dyn std::error::Error>> {
    // left associate operator
    let input = "condition 1 | 2 | 3";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: (1 | (2 | 3))");
    Ok(())
}

#[test]
fn test_cond_display_expr_fn() -> Result<(), Box<dyn std::error::Error>> {
    // left associate operator
    let input = r#"condition foo(1,2,"foo")"#;
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), r#"Condition: foo(1, 2, "foo")"#);
    Ok(())
}

#[test]
fn test_cond_display_empty() {
    // left associate operator
    let input = "condition ()";
    let parse = AlfaDocParser::parse(Rule::condition_stmt, input);
    // empty condition is not valid per parse rules
    assert!(parse.is_err());
}

#[test]
fn test_cond_display_expr_fn_nested() -> Result<(), Box<dyn std::error::Error>> {
    // left associate operator
    let input = "condition add(1,subtract(7,9))";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: add(1, subtract(7, 9))");
    Ok(())
}

#[test]
fn test_cond_display_ops_and_fns() -> Result<(), Box<dyn std::error::Error>> {
    // left associate operator
    let input = "condition foo(1) + bar(2)";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: (foo(1) + bar(2))");
    Ok(())
}

#[test]
fn test_cond_display_simple_attr() -> Result<(), Box<dyn std::error::Error>> {
    // left associate operator
    let input = "condition subjectName";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: subjectName");
    Ok(())
}

#[test]
fn test_cond_display_simple_ns_attr() -> Result<(), Box<dyn std::error::Error>> {
    // left associate operator
    let input = "condition main.subjectName";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(format!("{cond}"), "Condition: main.subjectName");
    Ok(())
}

#[test]
fn test_cond_display_ns_attr_issuer() -> Result<(), Box<dyn std::error::Error>> {
    // left associate operator
    let input = r#"condition main.subjectName[issuer="foo"]"#;
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(
        format!("{cond}"),
        "Condition: main.subjectName[issuer=\"foo\"]"
    );
    Ok(())
}

#[test]
fn test_cond_display_ns_attr_mustbepresent() -> Result<(), Box<dyn std::error::Error>> {
    // left associate operator
    let input = "condition main.subjectName[mustbepresent]";
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(
        format!("{cond}"),
        "Condition: main.subjectName[mustbepresent]"
    );
    Ok(())
}

#[test]
fn test_cond_display_ns_attr_allopts() -> Result<(), Box<dyn std::error::Error>> {
    // left associate operator
    let input = r#"condition main.subjectName[mustbepresent issuer="foo"]"#;
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    let cond = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    assert_eq!(
        format!("{cond}"),
        r#"Condition: main.subjectName[mustbepresent issuer="foo"]"#
    );
    Ok(())
}

#[test]
fn test_cond_pratt_simple() -> Result<(), Box<dyn std::error::Error>> {
    // 3 + 3
    let input = "condition 3 + 3";
    // parse the statement.
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    // process the statement
    let parsed = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    // build the expected parse tree
    let arg3 = CondExpression::Lit(Constant::Integer("3".to_string()));
    let plus = Operator {
        ns: vec![],
        operator: "+".to_string(),
    };
    let exp = CondExpression::Infix(Box::new(arg3.clone()), plus, Box::new(arg3));
    let ctx = Rc::new(Context::default());
    assert_eq!(
        parsed,
        Condition {
            cond_expr: exp,
            ns: vec![],
            ctx: Rc::<Context>::downgrade(&ctx)
        }
    );
    Ok(())
}

// test associativity of a single operator (&).
// this tests against a full parse tree just to be very precise.
#[test]
fn test_cond_pratt_single_right_assoc() -> Result<(), Box<dyn std::error::Error>> {
    // 3 + 3
    let input = "condition 1 & 2 & 3 & 4";
    // parse the statement.
    let mut pairs = AlfaDocParser::parse(Rule::condition_stmt, input)?;
    let pair = pairs.next().expect("parsed condition has a first member");
    // process the statement
    let parsed = process_condition(pair.into_inner(), vec![], &Rc::new(Context::default()))?;
    // build the expected parse tree
    let arg1 = CondExpression::Lit(Constant::Integer("1".to_string()));
    let arg2 = CondExpression::Lit(Constant::Integer("2".to_string()));
    let arg3 = CondExpression::Lit(Constant::Integer("3".to_string()));
    let arg4 = CondExpression::Lit(Constant::Integer("4".to_string()));
    let amp = Operator {
        ns: vec![],
        operator: "&".to_string(),
    };
    let exp = CondExpression::Infix(
        Box::new(arg1.clone()),
        amp.clone(),
        Box::new(CondExpression::Infix(
            Box::new(arg2.clone()),
            amp.clone(),
            Box::new(CondExpression::Infix(
                Box::new(arg3.clone()),
                amp.clone(),
                Box::new(arg4.clone()),
            )),
        )),
    );
    let ctx = Rc::new(Context::default());
    assert_eq!(
        parsed,
        Condition {
            cond_expr: exp,
            ns: vec![],
            ctx: Rc::<Context>::downgrade(&ctx)
        }
    );
    Ok(())
}

#[test]
fn test_empty_permit_obligation() -> Result<(), Box<dyn std::error::Error>> {
    let input = "on permit { obligation foo { } }";
    // parse the statement.
    let mut pairs = AlfaDocParser::parse(Rule::on_effect, input)?;
    let pair = pairs
        .next()
        .expect("parsed prescription has a first member");
    let ctx = Rc::new(Context::default());
    let parsed = process_prescription(pair, vec![], &ctx)?;

    assert_eq!(parsed.effect, Effect::Permit);
    // one expression ("foo"), with no assignments
    assert_eq!(
        parsed.expressions.len(),
        1,
        "no expressions in this prescription"
    );
    assert_eq!(
        parsed
            .expressions
            .first()
            .expect("single expression")
            .assignments
            .len(),
        0,
        "no assignments in this expression"
    );
    Ok(())
}

#[test]
fn test_empty_deny_obligation() -> Result<(), Box<dyn std::error::Error>> {
    let input = "on deny { obligation foo { } }";
    // parse the statement.
    let mut pairs = AlfaDocParser::parse(Rule::on_effect, input)?;
    let pair = pairs
        .next()
        .expect("parsed prescription has a first member");
    let ctx = Rc::new(Context::default());
    let parsed = process_prescription(pair, vec![], &ctx)?;
    assert_eq!(parsed.effect, Effect::Deny);
    // one expression ("foo"), with no assignments
    assert_eq!(
        parsed.expressions.len(),
        1,
        "no expressions in this prescription"
    );
    assert_eq!(
        parsed
            .expressions
            .first()
            .expect("single expression")
            .assignments
            .len(),
        0,
        "no assignments in this expression"
    );
    Ok(())
}

#[test]
fn test_empty_permit_advice() -> Result<(), Box<dyn std::error::Error>> {
    let input = "on permit { advice foo { } }";
    // parse the statement.
    let mut pairs = AlfaDocParser::parse(Rule::on_effect, input)?;
    let pair = pairs
        .next()
        .expect("parsed prescription has a first member");
    let ctx = Rc::new(Context::default());
    let parsed = process_prescription(pair, vec![], &ctx)?;
    assert_eq!(parsed.effect, Effect::Permit);
    // one expression ("foo"), with no assignments
    assert_eq!(
        parsed.expressions.len(),
        1,
        "no expressions in this prescription"
    );
    assert_eq!(
        parsed
            .expressions
            .first()
            .expect("single expression")
            .assignments
            .len(),
        0,
        "no assignments in this expression"
    );
    Ok(())
}

#[test]
fn test_empty_deny_advice() -> Result<(), Box<dyn std::error::Error>> {
    let input = "on deny { advice foo { } }";
    // parse the statement.
    let mut pairs = AlfaDocParser::parse(Rule::on_effect, input)?;
    let pair = pairs
        .next()
        .expect("parsed prescription has a first member");
    let ctx = Rc::new(Context::default());
    let parsed = process_prescription(pair, vec![], &ctx)?;
    assert_eq!(parsed.effect, Effect::Deny);
    // one expression ("foo"), with no assignments
    assert_eq!(
        parsed.expressions.len(),
        1,
        "no expressions in this prescription"
    );
    assert_eq!(
        parsed
            .expressions
            .first()
            .expect("single expression")
            .assignments
            .len(),
        0,
        "no assignments in this expression"
    );
    Ok(())
}

#[test]
fn test_permit_advice_expression_var() -> Result<(), Box<dyn std::error::Error>> {
    // we'll assign an attribute the constant 42.
    let input = "on permit { advice foo { attributes.balance = 42 } }";
    // parse the statement.
    let mut pairs = AlfaDocParser::parse(Rule::on_effect, input)?;
    let pair = pairs
        .next()
        .expect("parsed prescription has a first member");
    let ctx = Rc::new(Context::default());
    let parsed = process_prescription(pair, vec![], &ctx)?;
    assert_eq!(parsed.effect, Effect::Permit);
    // one expression ("foo"), with one assignment
    assert_eq!(
        parsed.expressions.len(),
        1,
        "no expressions in this prescription"
    );
    assert_eq!(
        parsed
            .expressions
            .first()
            .expect("single expression")
            .assignments
            .len(),
        1,
        "no assignments in this expression"
    );
    Ok(())
}
