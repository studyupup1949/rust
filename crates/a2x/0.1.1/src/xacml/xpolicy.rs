//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML Policy Sets

use super::xprescription::XPrescriptions;
use super::xrule::XRule;
use super::xtarget::XTarget;
use super::XacmlWriter;
use crate::ast::policy::Policy;
use crate::ast::rule::RuleEntry;
use crate::ast::QualifiedName;
use crate::errors::ParseError;
use crate::xacml::xprescription::XPrescriptionByType;
use crate::xacml::xprescription::XPrescriptionExpr;
use log::debug;
use log::info;
use std::io::Write;
use xml::writer::EventWriter;
use xml::writer::XmlEvent;

/// `<Policy>` element and all children.
#[derive(Debug, PartialEq, Default)]
pub struct XPolicy {
    /// Unique identifier for the policy.
    pub id: String,
    pub filename: Option<String>, // Todo: make non-optional
    pub combining_alg: String,
    pub description: Option<String>,
    pub target: XTarget,
    pub prescriptions: XPrescriptions,
    pub rules: Vec<XRule>,
}

// this is a nice structure in that the event writer can be passed along for serialization.
impl XacmlWriter for XPolicy {
    /// Write an XML (XACML) representation of an `XPolicy` to a
    /// stream.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the writer fails.
    fn write_xml<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), xml::writer::Error> {
        debug!("about to write xml for a policy");
        // Define the Policy element
        writer.write(
            XmlEvent::start_element("xacml3:Policy")
                .attr("PolicyId", &self.id)
                .attr("RuleCombiningAlgId", &self.combining_alg)
                .attr("Version", "1.0")
                .ns("xacml3", "urn:oasis:names:tc:xacml:3.0:core:schema:wd-17"),
        )?;
        // Add description
        if let Some(d) = self.description.as_ref() {
            writer.write(XmlEvent::start_element("xacml3:Description"))?;
            writer.write(XmlEvent::characters(d))?;
            writer.write(XmlEvent::end_element())?;
        }
        debug!("writing target...");
        self.target.write_xml(writer)?;
        // Write Rules
        // TODO: rule references broken?
        for rule in &self.rules {
            rule.write_xml(writer)?;
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
        // End the Policy
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

/// Conversion of Alfa Policy to XACML Policy
impl TryFrom<&Policy> for XPolicy {
    type Error = ParseError;
    fn try_from(p: &Policy) -> Result<Self, Self::Error> {
        info!("running try_from for policy -> XPolicy");
        // if there is a condition for the policy, we cannot convert
        // to an XPolicy.
        if p.condition.is_some() {
            info!("this policy has a condition, cannot convert to XPolicy");
            return Err(ParseError::PolicyNoCondition);
        }

        p.finalize_id();
        // TODO: why does looking up combining algorithm fail with an import?

        // ID and description are straightforward copies.
        // the rule combining algorithm needs to be resolved.
        let combining_alg = p
            .ctx
            .lookup_rule_combinator(&p.apply.id, &p.ns, &p.apply.src_loc)?
            .uri
            .to_string();

        let target = p
            .target
            .as_ref()
            .map(XTarget::try_from)
            .transpose()? // swap option/result
            .unwrap_or_default();

        let mut rules: Vec<XRule> = vec![];
        for r in &p.rules {
            match r {
                RuleEntry::Ref(rr) => {
                    // lookup this rule
                    info!("trying to lookup....{}, from namespace {:?}", &rr.id, &p.ns);
                    // the above shows up we are NOT getting the full reference.
		    // we need the source location of the rule.
                    let resolved_r = p.ctx.lookup_rule(
                        &rr.fully_qualified_name()
                            .ok_or(ParseError::AstConvertError)?,
                        &p.ns,
			&rr.src_loc
                    )?;
                    // The Rule ID here needs to be made unique.  Only
                    // the Original RuleDef can use the rule path.
                    let mut xr = XRule::try_from(resolved_r.as_ref())?;
                    xr.id.push_str(&format!(
                        "#rule_{}",
                        &p.ctx.get_next_rule_id(&resolved_r.ns.join(".")).to_string()
                    ));
                    rules.push(xr);
                }
                RuleEntry::Def(d) => rules.push(XRule::try_from(d.as_ref())?),
            }
        }
        let prescriptions = XPrescriptions {
            exprs: p
                .prescriptions
                .iter()
                .map(Vec::<XPrescriptionExpr>::try_from)
                .collect::<Result<Vec<Vec<XPrescriptionExpr>>, ParseError>>()?
                .into_iter()
                .flatten()
                .collect::<Vec<XPrescriptionExpr>>(),
        };
        let filename = p.get_filename();
        info!("creating an xpolicy with filename: {filename:?}");
        Ok(XPolicy {
            id: p.get_id(),
            filename,
            combining_alg,
            description: p.description.clone(),
            target,
            prescriptions,
            rules,
        })
    }
}
