//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use crate::context::Context;

use super::{constant::Constant, designator::AttributeDesignator, rule::Effect, PrettyPrint};
use std::{fmt, rc::Weak};

// A "Prescription" is just our term for the block of obligation and
// advice statements that can be associated with single effect within a
// rule/policy/policyset.

// A prescription is for a single "on <effect>" statement, so all
// child obligations/advice expressions will be for the same effect.

/// Obligation/advice assignment statements for an effect
#[derive(Debug, Clone)]
pub struct Prescription {
    pub effect: Effect, // permit/deny
    pub ns: Vec<String>,
    pub expressions: Vec<PrescriptionExpr>,
    /// Context for conversion
    pub ctx: Weak<Context>,
}

/// Target equality, ignoring context
impl PartialEq for Prescription {
    fn eq(&self, other: &Self) -> bool {
        self.effect == other.effect && self.expressions == other.expressions
    }
}

impl PrettyPrint for Prescription {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

impl fmt::Display for Prescription {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Prescription")
    }
}

/// An obligation/advice assignment
#[derive(Debug, PartialEq, Clone)]
pub struct PrescriptionExpr {
    pub ptype: PrescriptionType,
    pub id: String,                            // obligation/advice ALFA id
    pub assignments: Vec<AttributeAssignment>, // attribute assignment expressions, attribute destinations and targets
}

/// An assignment of some source value to a destination attribute ALFA ID
#[derive(Debug, PartialEq, Clone)]
pub struct AttributeAssignment {
    pub destination_id: String,
    pub source: AttrAssignmentSource,
}

/// Right-hand of an attribute assignment expression.
#[derive(Debug, PartialEq, Clone)]
pub enum AttrAssignmentSource {
    // use the designator from Conditions (AttributeDesignator)
    Attribute(AttributeDesignator), // an attribute identifier
    Value(Constant),                // some constant value
}

/// The effect of a rule (or obligation/advice)
#[derive(Debug, PartialEq, Clone)]
pub enum PrescriptionType {
    Obligation,
    Advice,
}

impl fmt::Display for PrescriptionType {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrescriptionType::Obligation => write!(f, "Obligation"),
            PrescriptionType::Advice => write!(f, "Advice"),
        }
    }
}
