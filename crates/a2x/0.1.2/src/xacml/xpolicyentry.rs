//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML Policy Entries (generic policy/policyset)

use super::xpolicy::XPolicy;
use super::xpolicyset::XPolicySet;
use crate::ast::policy::Policy;
use crate::errors::ParseError;
use log::info;

/// A Policy/Policyset child of a `PolicySet`.  Used to preserve order.
#[derive(Debug, PartialEq)]
pub enum XPolicyEntry {
    PolicyIdRef(String),
    PolicySetIdRef(String),
    PolicySet(XPolicySet),
    Policy(XPolicy),
}

impl XPolicyEntry {
    /// Count the total number of rules contained under this policy
    /// entry.
    pub fn rule_count(&self) -> usize {
        match &self {
            XPolicyEntry::PolicyIdRef(_) => 0,
            XPolicyEntry::PolicySetIdRef(_) => 0,
            XPolicyEntry::PolicySet(p) => p.rule_count(),
            XPolicyEntry::Policy(p) => p.rule_count(),
        }
    }
    /// Count the total number of policies contained under this policy
    /// entry, not including policy references.
    pub fn policy_count(&self) -> usize {
        match &self {
            XPolicyEntry::PolicyIdRef(_) => 0,
            XPolicyEntry::PolicySetIdRef(_) => 0,
            XPolicyEntry::PolicySet(p) => p.policy_count(),
            XPolicyEntry::Policy(_) => 1,
        }
    }
    /// Count the total number of policysets contained under this policy
    /// entry, not including policyset references.
    pub fn policyset_count(&self) -> usize {
        match &self {
            XPolicyEntry::PolicyIdRef(_) => 0,
            XPolicyEntry::PolicySetIdRef(_) => 0,
            XPolicyEntry::PolicySet(p) => p.policyset_count(),
            XPolicyEntry::Policy(_) => 0,
        }
    }
}

/// Convert a policy into either a policyset or policy, depending on
/// whether a condition is present.
impl TryFrom<&Policy> for XPolicyEntry {
    type Error = ParseError;
    fn try_from(p: &Policy) -> Result<Self, Self::Error> {
        if p.condition.is_none() {
            info!("trying to convert a policy with no condition into a XPolicy");
            Ok(XPolicyEntry::Policy(XPolicy::try_from(p)?))
        } else {
            info!("trying to convert a policy with condition into a XPolicySet");
            Ok(XPolicyEntry::PolicySet(XPolicySet::try_from(p)?))
        }
    }
}
