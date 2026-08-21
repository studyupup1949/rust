//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::constant::Constant;
use super::operator::Operator;
use super::PrettyPrint;
use crate::Context;
use std::fmt;
use std::rc::Weak;

/// A target statement
#[derive(Debug, Default, Clone)]
pub struct Target {
    // targets are made up of clauses (AND'd)
    pub clauses: Vec<DisjunctiveSeq>,
    /// The namespace this target is located in
    pub ns: Vec<String>,
    /// Context for conversion
    pub ctx: Weak<Context>,
}

/// Target equality, ignoring context
impl PartialEq for Target {
    fn eq(&self, other: &Self) -> bool {
        self.clauses == other.clauses && self.ns == other.ns
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Target: {} AnyOf", self.clauses.len())
    }
}

impl PrettyPrint for Target {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
        // print out all clauses
        for c in &self.clauses {
            c.pretty_print(indent_level + 1);
        }
    }
}

/// A disjunctive sequence
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DisjunctiveSeq {
    pub statements: Vec<ConjunctiveSeq>,
}

impl fmt::Display for DisjunctiveSeq {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AnyOf: {} stmts", self.statements.len())
    }
}

impl PrettyPrint for DisjunctiveSeq {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
        // print out all matches
        for m in &self.statements {
            m.pretty_print(indent_level + 1);
        }
    }
}

/// A conjunctive sequence
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ConjunctiveSeq {
    pub matches: Vec<Match>,
}

impl fmt::Display for ConjunctiveSeq {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AllOf: {} matches", self.matches.len())
    }
}

impl PrettyPrint for ConjunctiveSeq {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
        // print out all matches
        for m in &self.matches {
            m.pretty_print(indent_level + 1);
        }
    }
}

/// Match statement in a target
#[derive(Debug, Clone, PartialEq)]
pub enum Match {
    MatchFunc(MatchFunction),
    MatchOp(MatchOperation),
}

impl fmt::Display for Match {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Match::MatchFunc(x) => write!(f, "{x:?}"),
            Match::MatchOp(x) => write!(f, "{x}"),
        }
    }
}

impl PrettyPrint for Match {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

/// A match function application (target)
#[derive(Debug, Default, Clone, PartialEq)]
pub struct MatchFunction {
    /// fully qualified namespace for the function
    pub function_id: Vec<String>,
    pub literal: Constant,
    /// qualified name of the attribute
    pub attribute: Vec<String>,
    /// attribute issuer
    pub issuer: Option<String>,
    /// is the attribute required
    pub mustbepresent: bool,
}

/// A match operation (target)
#[derive(Debug, Default, Clone, PartialEq)]
pub struct MatchOperation {
    pub attribute: Vec<String>, //qualified name
    pub operator: Operator,     // ==, +, etc.
    pub literal: Constant,      // this could be a string/number/boolean
    pub reversed: bool, // true if this was attribute then literal (reverse of how XACML requires)
    /// attribute issuer
    pub issuer: Option<String>,
    /// is the attribute required
    pub mustbepresent: bool,
}

impl fmt::Display for MatchOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} {} {:?}",
            self.attribute.join("."),
            self.operator,
            self.literal
        )
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.ns.is_empty() {
            write!(f, "{}", self.operator)
        } else {
            write!(f, "{}.{}", self.ns.join("."), self.operator)
        }
    }
}
