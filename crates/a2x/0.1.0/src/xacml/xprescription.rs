//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML Prescriptions; Obligations and Advice

use super::xattr_designator::XAttrDesignator;
use crate::ast::prescription::{AttrAssignmentSource, Prescription, PrescriptionType};
use crate::ast::rule::Effect;
use crate::context::TypedLiteral;
use crate::errors::ParseError;
use log::info;
use std::io::Write;
use xml::writer::EventWriter;
use xml::writer::XmlEvent;

/// `<AttributeValue>` element, found in [`XExpression`] and
/// [`XAttributeAssignment`].
#[derive(Debug, PartialEq, Clone)]
pub struct XAttrValue {
    // Attribute values consist of a type URI and value
    pub v: TypedLiteral,
}

impl XAttrValue {
    /// Write an XML (XACML) representation of an `XAttrValue` to a
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
            XmlEvent::start_element("xacml3:AttributeValue").attr("DataType", &self.v.type_uri),
        )?;
        writer.write(XmlEvent::characters(&self.v.value))?;
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

/// Container for prescriptions that have been separated into
/// obligations and advice.
#[derive(Debug, PartialEq, Clone)]
pub struct XPrescriptionByType {
    pub obligations: Vec<XPrescriptionExpr>,
    pub associated_advice: Vec<XPrescriptionExpr>,
}

/// Container for Obligations and Advice.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct XPrescriptions {
    pub exprs: Vec<XPrescriptionExpr>,
}

/// Separate prescriptions by type.
impl From<XPrescriptions> for XPrescriptionByType {
    fn from(ps: XPrescriptions) -> Self {
        let mut obligations = vec![];
        let mut associated_advice = vec![];
        for p in ps.exprs {
            match p.ptype {
                PrescriptionType::Advice => associated_advice.push(p),
                PrescriptionType::Obligation => obligations.push(p),
            }
        }
        XPrescriptionByType {
            obligations,
            associated_advice,
        }
    }
}

/// An `<ObligationExpression>` or `<AdviceExpression>` element.
#[derive(Debug, PartialEq, Clone)]
pub struct XPrescriptionExpr {
    pub ptype: PrescriptionType,
    pub id: String,
    pub fulfill_on: Effect, // permit or deny
    pub assignments: Vec<XAttributeAssignment>,
}

impl XPrescriptionExpr {
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
        match self.ptype {
            PrescriptionType::Advice => {
                writer.write(
                    XmlEvent::start_element("xacml3:AdviceExpression")
                        .attr("AdviceId", &self.id)
                        .attr("AppliesTo", &self.fulfill_on.to_string()),
                )?;
            }
            PrescriptionType::Obligation => {
                writer.write(
                    XmlEvent::start_element("xacml3:ObligationExpression")
                        .attr("ObligationId", &self.id)
                        .attr("FulfillOn", &self.fulfill_on.to_string()),
                )?;
            }
        };

        // write the attribute assignment expression
        for a in &self.assignments {
            writer.write(
                XmlEvent::start_element("xacml3:AttributeAssignmentExpression")
                    .attr("AttributeId", &a.id)
                    .attr("Category", &a.category),
            )?;
            match &a.arg {
                XAttributeAssignmentArgument::Value(v) => v.write_xml(writer)?,
                XAttributeAssignmentArgument::Attrib(ad) => ad.write_xml(writer)?,
            };
            writer.write(XmlEvent::end_element())?;
        }
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

/// Assignments to Prescription Attributes.
#[derive(Debug, PartialEq, Clone)]
pub enum XAttributeAssignmentArgument {
    /// Constant/Literal value.
    Value(XAttrValue),
    /// Use an existing Attribute Designator.
    Attrib(XAttrDesignator),
}

/// An individual attribute assignment within an [`XPrescriptionExpr`].
#[derive(Debug, PartialEq, Clone)]
pub struct XAttributeAssignment {
    /// The identifier for the attribute value returned within this
    /// obligation/advice.
    pub id: String,
    /// The category URI for the returned attribute.
    pub category: String,
    /// The right-hand side of an obligation/advice assignment
    /// expression.
    pub arg: XAttributeAssignmentArgument,
}

/// Conversion of Alfa Rule to XACML Rule
impl TryFrom<&Prescription> for Vec<XPrescriptionExpr> {
    type Error = ParseError;
    fn try_from(p: &Prescription) -> Result<Self, Self::Error> {
        let ctx = p.ctx.upgrade().ok_or(ParseError::ContextMissing)?;
        info!("attempting to convert a prescription: {p:?}");
        // for every item, we will have the same effect.
        let effect = p.effect.clone();
        let mut xpe: Vec<XPrescriptionExpr> = vec![];
        for e in &p.expressions {
            let ptype = e.ptype.clone();
            // lookup the ID, which depends on the type
            let xid = match ptype {
                PrescriptionType::Advice => ctx.lookup_advice(&e.id, &p.ns)?.uri.clone(),
                PrescriptionType::Obligation => ctx.lookup_obligation(&e.id, &p.ns)?.uri.clone(),
            };

            // attribute assignments
            let mut xaa: Vec<XAttributeAssignment> = vec![];

            info!("converting expression (uri = {xid}, effect={effect}, type={ptype})");
            for assignment in &e.assignments {
                info!(
                    "found assignment: {} <- {:?}",
                    assignment.destination_id, assignment.source
                );
                // convert destination to a URI
                let dest_attr = ctx.lookup_attribute(&assignment.destination_id, &p.ns)?;
                let dest_attr_id = dest_attr.uri.clone();
                let dest_attr_category =
                    ctx.lookup_category(&dest_attr.category, &p.ns)?.uri.clone();
                info!("destination:  {dest_attr_id:?}");
                // if source is a literal, we can store it as-is.
                // if source is an attribute, look it up.
                let xarg = match &assignment.source {
                    AttrAssignmentSource::Attribute(a) => {
                        // we have the designator;
                        // lookup attribute, cat, type
                        let attr = ctx.lookup_attribute(&a.fully_qualified_name(), &p.ns)?;
                        let cat = ctx.lookup_category(&attr.category, &attr.ns)?;
                        let atype = ctx.lookup_type(&attr.typedef, &attr.ns)?;
                        // construct XAttrDesignator
                        let xattr = XAttrDesignator {
                            uri: attr.uri.clone(),
                            category: cat.uri.clone(),
                            type_uri: atype.uri.clone(),
                            must_be_present: a.mustbepresent,
                            issuer: a.issuer.clone(),
                        };
                        XAttributeAssignmentArgument::Attrib(xattr)
                    }
                    AttrAssignmentSource::Value(v) => {
                        XAttributeAssignmentArgument::Value(XAttrValue {
                            v: ctx.constant_to_typedliteral(v.clone(), &p.ns)?,
                        })
                    }
                };
                xaa.push(XAttributeAssignment {
                    id: dest_attr_id,
                    category: dest_attr_category,
                    arg: xarg,
                });
            }
            xpe.push(XPrescriptionExpr {
                ptype,
                id: xid,
                fulfill_on: effect.clone(),
                assignments: xaa,
            });
        }
        Ok(xpe)
    }
}
