//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML Functions

use std::io::Write;
use xml::writer::EventWriter;
use xml::writer::XmlEvent;

/// `<Function>` element, used to reference a XACML function as an argument.
#[derive(Debug, PartialEq, Clone)]
pub struct XFunction {
    pub function_uri: String,
}

impl XFunction {
    /// Write an XML (XACML) representation of an `XFunction` to a
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
            XmlEvent::start_element("xacml3:Function").attr("FunctionId", &self.function_uri),
        )?;
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}
