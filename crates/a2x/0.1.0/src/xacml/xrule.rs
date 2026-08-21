//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML Rules

use super::xcondition::XCondition;
use super::xprescription::XPrescriptionByType;
use super::xprescription::XPrescriptionExpr;
use super::xprescription::XPrescriptions;
use super::xtarget::XTarget;
use crate::ast::rule::RuleDef;
use crate::errors::ParseError;
use log::info;
use std::io::Write;
use xml::writer::EventWriter;
use xml::writer::XmlEvent;

/// `<Rule>` elements within an [`XPolicy`].
#[derive(Debug, PartialEq, Default)]
pub struct XRule {
    pub id: String,
    pub description: Option<String>,
    pub effect: String,
    pub target: XTarget,
    pub condition: Option<XCondition>,
    pub prescriptions: XPrescriptions,
}

/// Conversion of Alfa Rule to XACML Rule
impl TryFrom<&RuleDef> for XRule {
    type Error = ParseError;
    fn try_from(r: &RuleDef) -> Result<Self, Self::Error> {
        let _ctx = r.ctx.upgrade().ok_or(ParseError::ContextMissing)?;
        let target = r
            .target
            .as_ref()
            .map(XTarget::try_from)
            .transpose()? // swap option/result
            .unwrap_or_default();

        // we need to convert the condition using try_from
        let condition = r.condition.as_ref().map(XCondition::try_from).transpose()?;

        // each prescription converts to zero-or-many xprescrexprs, combine them all together
        let prescriptions = XPrescriptions {
            exprs: r
                .prescriptions
                .iter()
                .map(Vec::<XPrescriptionExpr>::try_from)
                .collect::<Result<Vec<Vec<XPrescriptionExpr>>, ParseError>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<XPrescriptionExpr>>(),
        };

        Ok(XRule {
            id: r.get_id(),
            description: r.description.clone(),
            effect: format!("{}", r.effect),
            target,
            condition,
            prescriptions,
        })
    }
}

impl XRule {
    /// Write an XML (XACML) representation of an `XRule` to a
    /// stream.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the writer fails.
    pub fn write_xml<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), xml::writer::Error> {
        // <xacml3:Rule Effect="Permit" RuleId="main.main/main.main.internalRule">
        writer.write(
            XmlEvent::start_element("xacml3:Rule")
                .attr("Effect", &self.effect)
                .attr("RuleId", &self.id),
        )?;
        // Write a description
        if let Some(d) = self.description.as_ref() {
            writer.write(XmlEvent::start_element("xacml3:Description"))?;
            writer.write(XmlEvent::characters(d))?;
            writer.write(XmlEvent::end_element())?;
        }
        // Serialize the <Target> element
        self.target.write_xml(writer)?;
        if let Some(c) = self.condition.as_ref() {
            // Serialize the condition, if it exists
            c.write_xml(writer)?;
        }

        // convert prescriptions into obligations/advice
        let xpt = XPrescriptionByType::from(self.prescriptions.clone());

        // Serialize Obligations
        if !xpt.obligations.is_empty() {
            writer.write(XmlEvent::start_element("xacml3:ObligationExpressions"))?;
            // serialize each prescription (obligation) expression
            for expr in xpt.obligations {
                info!("serializing obligation expression");
                expr.write_xml(writer)?;
            }
            writer.write(XmlEvent::end_element())?;
        }
        // Serialize Advice
        if !xpt.associated_advice.is_empty() {
            writer.write(XmlEvent::start_element("xacml3:AdviceExpressions"))?;
            // serialize each prescription (advice) expression
            for expr in xpt.associated_advice {
                expr.write_xml(writer)?;
                info!("serializing advice expression");
            }
            writer.write(XmlEvent::end_element())?;
        }

        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}
