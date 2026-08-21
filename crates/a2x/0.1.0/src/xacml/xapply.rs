//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML Apply

use super::xcondition::FunctionTypeResolved;
use super::xexpression::XExpression;
use std::io::Write;
use xml::writer::EventWriter;
use xml::writer::XmlEvent;

/// An `<Apply>` element that applies a function to arguments.
#[derive(Debug, PartialEq, Clone)]
pub struct XApply {
    // Apply must have a function identifier, and then arguments, which themselves can be expressions.
    pub function_uri: String,
    pub arguments: Vec<XExpression>,
    pub return_type: FunctionTypeResolved,
}

impl XApply {
    /// Write an XML (XACML) representation of an `XApply` to a
    /// stream.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the writer fails.
    pub fn write_xml<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), xml::writer::Error> {
        writer.write(
            XmlEvent::start_element("xacml3:Apply").attr("FunctionId", &self.function_uri),
        )?;
        // write out the arguments
        for a in &self.arguments {
            a.write_xml(writer)?;
        }
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}
