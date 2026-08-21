//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::condition::Condition;
use super::naming::GenName;
use super::naming::NameSlot;
use super::policyset::PolicySet;
use super::prescription::Prescription;
use super::rule::RuleEntry;
use super::target::Target;
use super::PrettyPrint;
use super::QualifiedName;
use crate::ast::policyset::PolicyEntry;
use crate::ast::rule::Effect;
use crate::ast::rule::RuleDef;
use crate::context::PROTECTED_NS;
use crate::Context;
use crate::ParseError;
use log::info;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use uuid::Uuid;
/// An empty Policy.
#[derive(Debug, PartialEq, Clone)]
pub struct Policy {
    /// The identifier for the policy
    pub id: PolicyId,
    /// The namespace from general to most specific
    pub ns: Vec<String>,
    /// The policy parents within a namespace
    pub policy_ns: GenName,
    /// Optional description
    pub description: Option<String>,
    /// Optional target
    pub target: Option<Target>,
    /// Optional condition
    pub condition: Option<Condition>,
    /// rule-combining algorithm symbol ("apply" statement)
    pub apply: String,
    /// Rules instantiated in the policy
    pub rules: Vec<RuleEntry>,
    /// On-Effect blocks (obligations/advice)
    pub prescriptions: Vec<Prescription>,
    /// Context for conversion
    pub ctx: Rc<Context>,
}

/// Policy identifier
#[derive(Debug, PartialEq, Clone)]
pub enum PolicyId {
    PolicyNoName,
    PolicyName(String),
    PolicyNameAndId(String, String),
}

impl PolicyId {
    /// Get the ALFA symbol for the policy if it exists.
    #[must_use]
    pub fn get_name(&self) -> Option<String> {
        match self {
            PolicyId::PolicyNoName => None,
            PolicyId::PolicyName(s) | PolicyId::PolicyNameAndId(s, _) => Some(s.clone()),
        }
    }
}

impl QualifiedName for Policy {
    fn fully_qualified_name(&self) -> Option<String> {
        match &self.id {
            PolicyId::PolicyName(_) | PolicyId::PolicyNameAndId(_, _) => {
                let mut qn = self.ns.join(".");
                // now we have the qualified name due to namespaces.  We need to add in any additional elements due to the parent policies.
                qn.push('.');
                info!(
                    "the policy_ns for this policy is: {}",
                    &self.policy_ns.build_path(".")?
                );
                qn.push_str(&self.policy_ns.build_path(".")?);
                Some(qn.to_string())
            }
            PolicyId::PolicyNoName => None,
        }
    }
}

/// Pretty print policy
impl PrettyPrint for Policy {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
        // rule-combining algorithm
        // pretty print the target
        if let Some(t) = self.target.as_ref() {
            t.pretty_print(indent_level + 1);
        }
        // pretty print the rule(s)
        for r in &self.rules {
            r.pretty_print(indent_level + 1);
        }
    }
}

impl Policy {
    fn get_policy_name_prefix(&self) -> String {
        self.ctx.config.get_base_namespace()
    }

    /// Suggest a filename.
    #[must_use]
    pub fn get_filename(&self) -> Option<String> {
        // join the namespace and the ID
        let mut name = self.ns.join(".");
        name.push('.');
        let b = self.policy_ns.last_elem()?;
        let gen_name = b.borrow();
        name.push_str(gen_name.as_ref()?);
        name.push_str(".xml");
        info!("suggested filename:  {name}");
        Some(name)
    }

    /// Insert a new policy somewhere in the namespace hierarchy.
    ///
    /// This must happen when we decondition a policyset that contains
    /// this one.
    pub fn insert_ns_entry(&mut self, name: NameSlot, idx: usize) {
        self.policy_ns.push_name_at_index(name, idx);
    }

    /// Generate a permanent ID for this, if necessary.
    /// Every namespace
    ///
    /// # Panics
    ///
    /// This function panics if the policy namespace is empty.  It
    /// should always contain at least this policy.
    pub fn finalize_id(&self) {
        // get the last element of the policy namespace.
        let lastopt = self.policy_ns.last_elem();
        if let Some(last) = lastopt {
            let mut this_id = last.borrow_mut();
            // fix the parent policy name if necessary
            if this_id.is_none() {
                loop {
                    let next_id = self.ctx.get_next_policy_id(&self.ns.join("."));
                    let proposed_name = format!("policy_{next_id}");
                    // check for collision (covers the case of an explicitly named "policy_0", etc.
                    let no_conflict = self.ctx.lookup_policy(&proposed_name, &self.ns).is_err();
                    if no_conflict {
                        info!("no conflict!");
                        this_id.replace(proposed_name);
                        info!("finalizing policy ID to {next_id}");
                        break;
                    }
                    info!("conflict, trying again...");
                }
            }
        } else {
            panic!("policy with empty policy namespace");
        }
    }

    /// Build the URI Policy identifier
    #[must_use]
    pub fn get_id(&self) -> String {
        // append the policy name
        if let Some(ppath_res) = self.policy_ns.build_path("/") {
            match &self.id {
                PolicyId::PolicyNameAndId(_, i) => i.clone(),
                PolicyId::PolicyNoName | PolicyId::PolicyName(_) => {
                    // form the namespaces prefix
                    let mut pn = self.get_policy_name_prefix();
                    let ns_str = self.ns.join("/");
                    pn.push_str(&ns_str);
                    pn.push('/');
                    pn.push_str(&ppath_res);
                    pn
                }
            }
        } else {
            // Can the policy_ns ever be empty?
            let id = Uuid::now_v7();
            let mut id_str = "urn:uuid:".to_string();
            id_str.push_str(&id.to_string());
            id_str
        }
    }

    /// Convert this to a new de-conditioned policyset.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the policy does not have a condition.
    pub fn decondition(&self) -> Result<PolicySet, ParseError> {
        info!("deconditioning..");
        if self.condition.is_none() {
            return Err(ParseError::PolicyNoCondition);
        }

        // TODO: use apply values from a protected namespace that is
        // predefined and always present
        // (_A2X_PROTECTED.onPermitApplySecond)

        // clone this policy so we can insert it as a child of a new top-level policyset.
        let mut original = self.clone();
        // replace the ID, which will be used by the new container/parent.
        let orig_id = original.id;
        original.id = PolicyId::PolicyNoName;

        // This container policy will hold a policy that contains a
        // rule with the condition, and then the original policy.
        let mut container = PolicySet {
            id: orig_id,
            ns: original.ns.clone(),
            policy_ns: self.policy_ns.clone(),
            description: original.description.take(),
            target: None,
            condition: None,
            apply: format!("{}.{}", PROTECTED_NS, "onPermitApplySecond"),
            policies: vec![],
            prescriptions: vec![],
            ctx: self.ctx.clone(),
        };

        // Define policy namespaces
        let mut cond_policy_ns = container.policy_ns.clone();
        cond_policy_ns.push_name(Rc::new(RefCell::new(Some("cond".to_owned()))));
        let cond_policy_id = PolicyId::PolicyName("cond".to_owned());

        let mut orig_policy_ns = container.policy_ns.clone();
        orig_policy_ns.push_name(Rc::new(RefCell::new(Some("orig".to_owned()))));
        // set the policy namespace for the original policy
        original.policy_ns = orig_policy_ns;
        original.id = PolicyId::PolicyName("orig".to_owned());

        // Create a rule which has the condition from the original policy.
        let condrule = RuleDef {
            id: None,
            ns: original.ns.clone(), // needs to be a child of the container
            policy_ns: cond_policy_ns.clone(),
            description: None,
            effect: Effect::Permit,
            target: None,
            condition: original.condition.take(),
            prescriptions: vec![],
            ctx: Rc::<Context>::downgrade(&self.ctx),
        };
        // Create a child policy with the rule.
        let condpolicy = Policy {
            id: cond_policy_id,
            ns: original.ns.clone(),
            policy_ns: cond_policy_ns,
            description: None,
            target: None,
            condition: None,
            apply: format!("{}.{}", PROTECTED_NS, "permitOverrides"),
            rules: vec![RuleEntry::Def(Rc::new(condrule))],
            prescriptions: vec![],
            ctx: self.ctx.clone(),
        };

        // place the condpolicy into the container first.
        container.policies.push(PolicyEntry::Policy(condpolicy));
        // then, place the original next.
        container.policies.push(PolicyEntry::Policy(original));
        Ok(container)
    }
}

impl fmt::Display for Policy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.id {
            PolicyId::PolicyNoName => {
                write!(
                    f,
                    "Policy: \"{} ~ anonymous\" (descr: {:?}) (algorithm: {})",
                    self.ns.join("."),
                    self.description,
                    self.apply
                )
            }
            PolicyId::PolicyName(n) | PolicyId::PolicyNameAndId(n, _) => {
                write!(
                    f,
                    "Policy: \"{}.{}\" (descr: {:?}) (algorithm: {})",
                    self.ns.join("."),
                    n,
                    self.description,
                    self.apply
                )
            }
        }
    }
}
