use super::constant::Constant;
use super::designator::AttributeDesignator;
use super::operator::Operator;
use super::PrettyPrint;
use crate::errors::ParseError;
use crate::Context;
use std::fmt;
use std::iter::Peekable;
use std::rc::Weak;

// need converters from ConditionUnparsed -> Condition
// CondExpressionUnparsed -> CondExpression
// CondAtomUnparsed -> CondAtom
// CondFunctionCallUnparsed -> CondFunctionCall

/// A condition, with unparsed expressions
#[derive(Debug, Default, Clone)]
pub struct Condition {
    /// conditions contain expressions, which can be nested, this form
    /// has no associativity applied.
    pub cond_expr: CondExpression,
    /// The namespace this target is located in
    pub ns: Vec<String>,
    /// Context for conversion
    pub ctx: Weak<Context>,
}

/// Target equality, ignoring context
impl PartialEq for Condition {
    fn eq(&self, other: &Self) -> bool {
        self.cond_expr == other.cond_expr && self.ns == other.ns
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Condition: {}", self.cond_expr)
    }
}

impl PrettyPrint for Condition {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

/// Condition Terms and Operators
#[derive(Debug, Default, Clone, PartialEq)]
pub enum CondExpression {
    Infix(Box<CondExpression>, Operator, Box<CondExpression>),
    Fn(CondFunctionCall),
    Attr(AttributeDesignator),
    FnRef(FunctionReference),
    Lit(Constant),
    #[default]
    Empty,
}

impl fmt::Display for CondExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CondExpression::Infix(a, op, b) => {
                write!(f, "({a} {op} {b})")
            }
            CondExpression::Lit(c) => {
                write!(f, "{c}")
            }
            CondExpression::Fn(n) => {
                write!(f, "{n}")
            }
            CondExpression::FnRef(n) => {
                write!(f, "{n}")
            }
            CondExpression::Attr(a) => {
                write!(f, "{a}")
            }
            CondExpression::Empty => {
                panic!("empty expressions are not expected")
            }
        }
    }
}

// Function calls which can have unparsed arguments.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CondFunctionCall {
    pub identifier: Vec<String>, // qualified name
    pub arguments: Vec<CondExpression>,
}

impl CondFunctionCall {
    #[must_use]
    pub fn fully_qualified_name(&self) -> String {
        self.identifier.join(".")
    }
}

impl fmt::Display for CondFunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.identifier.join("."),
            self.arguments
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Conversion of `ConditionUnparsed` to `Condition`
impl TryFrom<&ConditionUnparsed> for Condition {
    type Error = ParseError;
    fn try_from(c: &ConditionUnparsed) -> Result<Self, Self::Error> {
        // just convert the inner CondExpression.
        Ok(Condition {
            cond_expr: CondExpression::try_from(&c.cond_expr)?,
            ns: c.ns.clone(),
            ctx: c.ctx.clone(),
        })
    }
}

/// Conversion of `ConditionExpressionUnparsed` to `CondExpression`
impl TryFrom<&CondExpressionUnparsed> for CondExpression {
    type Error = ParseError;
    fn try_from(c: &CondExpressionUnparsed) -> Result<Self, Self::Error> {
        expr_bp(&mut c.items.clone().into_iter().peekable(), 0)
    }
}

// inspired by https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
fn expr_bp<I>(items: &mut Peekable<I>, min_binding_power: u8) -> Result<CondExpression, ParseError>
where
    I: Iterator<Item = CondItemUnparsed>,
{
    // Get the first argument
    let mut lhs = match items.next() {
        Some(CondItemUnparsed::Atom(a)) => Ok(CondExpression::try_from(&a)?),
        _ => Err(ParseError::AstConvertError),
    }?;

    // consume operators and atoms to build up an expression tree.
    loop {
        // Get the operator
        let op = match items.peek() {
            None => break, // nothing left
            Some(CondItemUnparsed::Op(op)) => Ok(op.clone()),
            Some(CondItemUnparsed::Atom(_) | CondItemUnparsed::Empty) => {
                Err(ParseError::AstConvertError)
            }
        }?;
        let (l_bp, r_bp) = operator_bp(&op.operator);
        // if the next operator has lower binding power, this expression is complete.
        if l_bp < min_binding_power {
            break;
        }
        // consume the operator
        items.next();
        // complete the right-hand side of the expression
        let rhs = expr_bp(items, r_bp)?;
        // construct a new expression node, and make it the new
        // left-hand side for the next loop.
        lhs = CondExpression::Infix(Box::new(lhs), op.clone(), Box::new(rhs));
    }
    Ok(lhs)
}

impl TryFrom<&CondAtomUnparsed> for CondExpression {
    type Error = ParseError;
    fn try_from(c: &CondAtomUnparsed) -> Result<Self, Self::Error> {
        match c {
            CondAtomUnparsed::Expr(e) => Ok(CondExpression::try_from(e)?),
            CondAtomUnparsed::Fn(f) => Ok(CondExpression::Fn(CondFunctionCall::try_from(f)?)),
            CondAtomUnparsed::Attr(a) => Ok(CondExpression::Attr(a.clone())),
            CondAtomUnparsed::Lit(l) => Ok(CondExpression::Lit(l.clone())),
            CondAtomUnparsed::FnRef(f) => Ok(CondExpression::FnRef(f.clone())),
            CondAtomUnparsed::Empty => Ok(CondExpression::Empty),
        }
    }
}

impl TryFrom<&CondFunctionCallUnparsed> for CondFunctionCall {
    type Error = ParseError;
    fn try_from(c: &CondFunctionCallUnparsed) -> Result<Self, Self::Error> {
        Ok(CondFunctionCall {
            identifier: c.identifier.clone(),
            arguments: c
                .arguments
                .iter()
                .map(CondExpression::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

// Unparsed variants below, which are used for the initial parsing.
// Then we convert into forms that have operator precedence applied.

/// A condition, with unparsed expressions
#[derive(Debug, Default, Clone)]
pub struct ConditionUnparsed {
    /// conditions contain expressions, which can be nested, this form
    /// has no associativity applied.
    pub cond_expr: CondExpressionUnparsed,
    /// The namespace this target is located in
    pub ns: Vec<String>,
    /// Context for conversion
    pub ctx: Weak<Context>,
}

/// Unparsed expression
#[derive(Debug, Default, Clone)]
pub struct CondExpressionUnparsed {
    pub items: Vec<CondItemUnparsed>,
}

/// Condition Terms and Operators, To-Be-Pratt-Parsed.
#[derive(Debug, Default, Clone)]
pub enum CondItemUnparsed {
    Atom(CondAtomUnparsed),
    Op(Operator),
    #[default]
    Empty,
}

/// Atoms that can contain unparsed expressions
#[derive(Debug, Default, Clone)]
pub enum CondAtomUnparsed {
    Expr(CondExpressionUnparsed), // from a parenthesed expr
    Fn(CondFunctionCallUnparsed),
    Attr(AttributeDesignator),
    FnRef(FunctionReference),
    Lit(Constant),
    #[default]
    Empty,
}

// Function calls which can have unparsed arguments.
#[derive(Debug, Default, Clone)]
pub struct CondFunctionCallUnparsed {
    pub identifier: Vec<String>, // qualified name
    pub arguments: Vec<CondExpressionUnparsed>,
}

/// A function reference (function-as-argument)
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FunctionReference {
    pub identifier: Vec<String>, // qualified name
}

impl FunctionReference {
    #[must_use]
    pub fn fully_qualified_name(&self) -> String {
        self.identifier.join(".")
    }
}

impl fmt::Display for FunctionReference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "fn[{}]", self.identifier.join("."))
    }
}

/// Operator Binding Power
fn operator_bp(symbol: &str) -> (u8, u8) {
    if symbol.starts_with('|') {
        // Operators starting with ‘|’. These are right associative.
        (12, 11)
    } else if symbol.starts_with('&') {
        // Operators starting with ‘&’. These are right associative.
        (10, 9)
    } else if symbol.starts_with('=')
        || symbol.starts_with('<')
        || symbol.starts_with('>')
        || symbol.starts_with('$')
    {
        // Operators starting with ‘=’, ‘<’, ‘>’ or ‘$’. These are left associative.
        (7, 8)
    } else if symbol.starts_with('@') || symbol.starts_with('^') {
        // Operators starting with ‘@’ or ‘^’. These are right associative.
        (6, 5)
    } else if symbol.starts_with('+') || symbol.starts_with('-') {
        // Operators starting with ‘+’ or ‘-‘. These are left associative.
        (3, 4)
    } else if symbol.starts_with('*') || symbol.starts_with('/') || symbol.starts_with('%') {
        // Operators starting with ‘*’, ‘/’ or ‘%’. These are left associative.
        (1, 2)
    } else {
        (0, 0)
    }
}
