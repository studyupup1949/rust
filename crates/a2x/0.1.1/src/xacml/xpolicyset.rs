//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML Policy Sets

use super::xpolicyentry::XPolicyEntry;
use super::xprescription::XPrescriptions;
use super::xtarget::XTarget;
use super::XacmlWriter;
use crate::ast::policy::Policy;
use crate::ast::policyset::{PolicyEntry, PolicySet};
use crate::errors::ParseError;
use crate::xacml::xprescription::{XPrescriptionByType, XPrescriptionExpr};
use log::debug;
use log::info;
use log::warn;
use std::io::Write;
use xml::writer::EventWriter;
use xml::writer::XmlEvent;

/// `<PolicySet>` element and all children.
#[derive(Debug, PartialEq, Default)]
pub struct XPolicySet {
    /// Unique identifier for the policy.
    pub id: String,
    pub filename: Option<String>, // Todo: make non-optional
    pub combining_alg: String,
    pub description: Option<String>,
    pub target: XTarget,
    pub prescriptions: XPrescriptions,
    pub children: Vec<XPolicyEntry>,
}

impl XacmlWriter for XPolicySet {
    /// Write an XML (XACML) representation of an `XPolicySet` to a
    /// stream.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the writer fails.
    fn write_xml<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), xml::writer::Error> {
        // Define the Policy element
        writer.write(
            XmlEvent::start_element("xacml3:PolicySet")
                .attr("PolicySetId", &self.id)
                .attr("PolicyCombiningAlgId", &self.combining_alg)
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
        warn!("child count is {}", self.children.len());
        for p in &self.children {
            warn!("writing child policy or policyset");
            match p {
                XPolicyEntry::PolicyIdRef(i) => {
                    writer.write(XmlEvent::start_element("xacml3:PolicyIdReference"))?;
                    writer.write(XmlEvent::characters(i))?;
                    writer.write(XmlEvent::end_element())?;
                }
                XPolicyEntry::PolicySetIdRef(i) => {
                    writer.write(XmlEvent::start_element("xacml3:PolicySetIdReference"))?;
                    writer.write(XmlEvent::characters(i))?;
                    writer.write(XmlEvent::end_element())?;
                }
                XPolicyEntry::Policy(sub_p) => {
                    sub_p.write_xml(writer)?;
                }
                XPolicyEntry::PolicySet(sub_p) => {
                    sub_p.write_xml(writer)?;
                }
            }
        }
        // Write Policies
        //        for rule in &self.rules {
        //            rule.write_xml(writer)?;
        //        }

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
impl TryFrom<&PolicySet> for XPolicySet {
    type Error = ParseError;
    fn try_from(p: &PolicySet) -> Result<Self, Self::Error> {
        if p.condition.is_some() {
            info!("this policy has conditions, converting to one without");
            let pd = p.clone().decondition()?;
            // convert this version of the policyset that does not
            // have conditions at the policyset level.
            return XPolicySet::try_from(&pd);
        }

        // Store policy children
        let mut children = vec![];
        // Ensure we have a finalized name.
        p.finalize_id();
        // ID and description are straightforward copies.
        // the rule combining algorithm needs to be resolved.
        let combining_alg = p
            .ctx
            .lookup_policy_combinator(&p.apply.id, &p.ns)?
            .uri
            .to_string();
        info!("finished lookup of combining alg");
        let target = p
            .target
            .as_ref()
            .map(XTarget::try_from)
            .transpose()? // swap option/result
            .unwrap_or_default();
        info!("finished conversion of target");
        // look over each child policy entry (ref, policy, policyset)
        for pe in &p.policies {
            warn!("looping through child policies...");
            match pe {
                PolicyEntry::Ref(pr) => {
                    // A child policy by reference.
                    // It should not be possible to create a policyset
                    // and policy with the same name, because the
                    // (TODO) registration step should check and make
                    // sure there is no policy/policyset with the same
                    // name.
                    debug!("lookup for policy ref: {pr:?}");
                    // lookup a policyset with this name.
                    if let Ok(ps) = p.ctx.lookup_policyset(&pr.fully_qualified_name(), &p.ns) {
                        // get the ID, and print it out.
                        info!("we found a policyset");
                        children.push(XPolicyEntry::PolicySetIdRef(ps.get_id()));
                    } else if let Ok(ps) = p.ctx.lookup_policy(&pr.fully_qualified_name(), &p.ns) {
                        info!("we found a policy");
                        // if the policy has a condition, it will be
                        // transformed into a policyset, and we need
                        // to reference it appropriately.
                        if ps.condition.is_none() {
                            children.push(XPolicyEntry::PolicyIdRef(ps.get_id()));
                        } else {
                            children.push(XPolicyEntry::PolicySetIdRef(ps.get_id()));
                        }
                    } else {
                        warn!("failed to resolve policy reference in a policyset");
                        return Err(ParseError::UnexpectedRuleError(
                            "Could not find policy/policyset for policy reference".to_owned(),
                        ));
                    }
                    //                    todo!("Policy Refs not handled yet");
                    // lookup a policy, and then a policyset with the name.

                    // This implies that there should be a shared
                    // namespace between policies and policysets,
                    // since there is no way to distinguish them.

                    // these need to be resolved and then the ID returned.
                }
                PolicyEntry::PolicySet(p) => {
                    info!("found a child policyset to convert");
                    children.push(XPolicyEntry::PolicySet(XPolicySet::try_from(p)?));
                }
                PolicyEntry::Policy(p) => {
                    info!("found a child policy to convert");
                    // if this policy can't be converted into an XPolicy (because it is a condition),
                    // we could de-condition it into a PolicySet, and then convert it.
                    children.push(XPolicyEntry::try_from(p)?);
                }
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
        info!("creating an xpolicyset with filename: {filename:?}");
        Ok(XPolicySet {
            id: p.get_id(),
            filename,
            combining_alg,
            description: p.description.clone(),
            target,
            prescriptions,
            children,
        })
    }
}

impl TryFrom<&Policy> for XPolicySet {
    type Error = ParseError;
    fn try_from(p: &Policy) -> Result<Self, Self::Error> {
        if p.condition.is_none() {
            info!("this policy has no condition, cannot convert to XPolicySet");
            return Err(ParseError::PolicyHasCondition);
        }
        let policyset = p.decondition()?;
        XPolicySet::try_from(&policyset)
    }
}
