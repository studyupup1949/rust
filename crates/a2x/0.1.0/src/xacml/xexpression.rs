//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML Conditions

use super::xapply::XApply;
use super::xattr_designator::XAttrDesignator;
use super::xfunction::XFunction;
use super::XAttrValue;
use std::io::Write;
use xml::writer::EventWriter;

/// An expression within a `<Condition>` element.
#[derive(Debug, PartialEq, Clone)]
pub enum XExpression {
    Apply(XApply),
    Function(XFunction),
    Value(XAttrValue),
    Attrib(XAttrDesignator),
    // Apply is used to represent function application
    // Function is used to represent a function-as-argument
    // AttributeValue is a literal value
    // AttributeDesignator is an attribute in context
    // (VariableReference and AttributeSelector are not used.)
}

impl XExpression {
    /// Write an XML (XACML) representation of an `XExpression` to a
    /// stream.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the writer fails.
    pub fn write_xml<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), xml::writer::Error> {
        match self {
            XExpression::Value(v) => {
                v.write_xml(writer)?;
            }
            XExpression::Attrib(a) => {
                a.write_xml(writer)?;
            }
            XExpression::Apply(a) => {
                a.write_xml(writer)?;
            }
            XExpression::Function(f) => {
                f.write_xml(writer)?;
            }
        }
        Ok(())
    }
}
