//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML Attribute Designators

use std::io::Write;
use xml::writer::EventWriter;
use xml::writer::XmlEvent;

/// An `<AttributeDesignator>` element, referencing a XACML attribute.
#[derive(Debug, PartialEq, Clone)]
pub struct XAttrDesignator {
    pub uri: String,
    pub category: String,
    pub type_uri: String,
    pub must_be_present: bool,
    pub issuer: Option<String>,
}

impl XAttrDesignator {
    /// Write an XML (XACML) representation of an `XAttrDesignator` to
    /// a stream.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the writer fails.
    pub fn write_xml<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), xml::writer::Error> {
        let mbp = &self.must_be_present.to_string();
        let mut attrdesig = XmlEvent::start_element("xacml3:AttributeDesignator")
            .attr("AttributeId", &self.uri)
            .attr("Category", &self.category)
            .attr("DataType", &self.type_uri);
        // if issuer exists, add it
        if let Some(i) = &self.issuer {
            attrdesig = attrdesig.attr("Issuer", i);
        }
        attrdesig = attrdesig.attr("MustBePresent", mbp);
        writer.write(attrdesig)?;
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}
