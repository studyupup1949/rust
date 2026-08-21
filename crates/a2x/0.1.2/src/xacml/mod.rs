//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML structures for serializing to XML.

pub mod xapply;
pub mod xattr_designator;
pub mod xcondition;
pub mod xexpression;
pub mod xfunction;
pub mod xpolicy;
pub mod xpolicyentry;
pub mod xpolicyset;
pub mod xprescription;
pub mod xrule;
pub mod xtarget;
use std::io::Write;
use xml::writer::EventWriter;
use xpolicy::XPolicy;
use xpolicyset::XPolicySet;
use xprescription::XAttrValue;

// the general philosophy in conversion is to keep the ctx on the AST
// side, and generate X-prefixed structs that contain just the
// information needed to output XACML-as-XML.

// The AST structs will have TryFrom implementations (down to the
// level of anything that has a ctx object) that perform conversion to
// the X-prefixed structs.

// The X-prefixed structs will have "write_xml" functions that can
// serialize themselves using an EventWriter.

/// Produce XML events for XACML serialization.
pub trait XacmlWriter {
    /// Write XML events to serialize this object.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the XML events are semantically invalid or
    /// the writer has an I/O error.
    fn write_xml<W: Write>(&self, writer: &mut EventWriter<W>) -> Result<(), xml::writer::Error>;
}

/// Top level of a XACML file will be either one policy set or one
/// policy.
pub enum XTopPolicy {
    Policy(XPolicy),
    PolicySet(XPolicySet),
}

impl XTopPolicy {
    /// Count the total number of rules defined in this policy/set and
    /// its children.
    pub fn rule_count(&self) -> usize {
        match self {
            XTopPolicy::Policy(xp) => xp.rule_count(),
            XTopPolicy::PolicySet(xps) => xps.rule_count(),
        }
    }
    /// Count the total number of policies defined in this policy/set
    /// and its children.
    pub fn policy_count(&self) -> usize {
        match self {
            XTopPolicy::Policy(_) => 1,
            XTopPolicy::PolicySet(xps) => xps.policy_count(),
        }
    }

    /// Count the total number of policysets defined in this
    /// policy/set and its children.
    pub fn policyset_count(&self) -> usize {
        match self {
            XTopPolicy::Policy(_) => 0,
            XTopPolicy::PolicySet(xps) => {
                // count this policyset and its children
                1 + xps.policyset_count()
            }
        }
    }
}
