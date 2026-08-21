//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use log::info;
use uuid::Uuid;

use super::condition::Condition;
use super::naming::GenName;
use super::naming::NameSlot;
use super::policy::Policy;
use super::policy::PolicyId;
use super::prescription::Prescription;
use super::rule::Effect;
use super::rule::RuleDef;
use super::rule::RuleEntry;
use super::target::Target;
use super::PrettyPrint;
use super::QualifiedName;
use super::Spanned;
use super::SrcLoc;
use crate::ast::policy::RuleCombiningAlgorithm;
use crate::context::PROTECTED_NS;
use crate::Context;
use crate::ParseError;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
/// An empty `PolicySet`.
#[derive(Debug, PartialEq, Clone, Spanned)]
pub struct PolicySet {
    /// The identifier for the policy
    pub id: PolicyId,
    /// The namespace from general to most specific
    pub ns: Vec<String>,
    /// The policy parents (including this one), after any namespaces.
    pub policy_ns: GenName,
    /// The location of this condition
    pub src_loc: SrcLoc,
    /// Optional description
    pub description: Option<String>,
    /// Optional target
    pub target: Option<Target>,
    /// Optional condition
    pub condition: Option<Condition>,
    /// policy-combining algorithm symbol ("apply" statement)
    pub apply: PolicyCombiningAlgorithm,
    /// policies or policysets within this policyset
    pub policies: Vec<PolicyEntry>,
    /// Obligations and/or advice
    pub prescriptions: Vec<Prescription>,
    /// Context for conversion
    pub ctx: Rc<Context>,
}

impl PolicySet {
    fn get_policy_name_prefix(&self) -> String {
        self.ctx.config.get_base_namespace()
    }

    /// add a policy ns entry at a location
    pub fn insert_ns_entry(&mut self, name: &NameSlot, idx: usize) {
        // insert here
        self.policy_ns.push_name_at_index(name.clone(), idx);
        // for every child, push the same name
        for cp in &mut self.policies {
            match cp {
                PolicyEntry::Policy(inline_policy) => {
                    info!("iterating through policies: {inline_policy}");
                    inline_policy.insert_ns_entry(name.clone(), idx);
                }
                PolicyEntry::PolicySet(inline_policyset) => {
                    info!("iterating through policysets: {inline_policyset}");
                    inline_policyset.insert_ns_entry(name, idx);
                }
                PolicyEntry::Ref(_) => {
                    // do
                    todo!("unexpected type of policy entry")
                }
            }
        }
    }

    /// Suggest a filename.
    #[must_use]
    pub fn get_filename(&self) -> Option<String> {
        info!("building filename.  namespaces: {:?}", self.ns);
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

    /// Generate a permanent ID for this, if necessary.
    ///
    /// # Panics
    ///
    /// This function panics if the policy namespace is empty (it
    /// should always have at least this policyset contained within).
    pub fn finalize_id(&self) {
        // get the last element of the policy namespace.
        let lastopt = self.policy_ns.last_elem();
        if let Some(last) = lastopt {
            let mut this_id = last.borrow_mut();
            if this_id.is_none() {
                loop {
                    let next_id = self.ctx.get_next_policyset_id(&self.ns.join("."));
                    let proposed_name = format!("policyset_{next_id}");
                    // check for collision
                    let no_conflict = self.ctx.lookup_policy(&proposed_name, &self.ns).is_err();
                    if no_conflict {
                        info!("no conflict!");
                        this_id.replace(proposed_name);
                        info!("finalizing policyset ID to {next_id}");
                        break;
                    }
                    info!("conflict, trying again...");
                }
            }
        } else {
            // no policyset should be created without itself being in
            // the namespace.
            panic!("policyset namespace was empty, should have at least itself as a member");
        }
    }

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
    /// Will return `Err` if the policyset does not have a condition.
    pub fn decondition(&self) -> Result<PolicySet, ParseError> {
        if self.condition.is_none() {
            return Err(ParseError::PolicySetNoCondition);
        }
        // The policyset structure we will end up with will be:
        // A top-level policyset (this object) which has the same
        // name/ID as before.  A combining algorithm that is
        // on-permit-apply-second, with no condition.
        // One child policy, and one child policyset under that new top-level policyset.

        // The first child policy is constructed from scratch, and has
        // a single rule, no target, and the condition from the
        // original policy.

        // The second child policyset is exactly the same as this one,
        // *but* has a different ID, and no condition.

        // clone this policy, so we can insert it as a child.
        let mut original = self.clone();
        // replace ID so we can use it in the container
        let orig_id = original.id;
        original.id = PolicyId::PolicyNoName;

        let mut container = PolicySet {
            id: orig_id,
            ns: original.ns.clone(),
            policy_ns: self.policy_ns.clone(),
            src_loc: self.span().clone(),
            description: original.description.take(),
            target: None,
            condition: None,
            apply: PolicyCombiningAlgorithm {
                id: format!("{}.{}", PROTECTED_NS, "onPermitApplySecond"),
                src_loc: self.apply.span().clone(),
            },
            policies: vec![],
            prescriptions: vec![],
            ctx: self.ctx.clone(),
        };

        // Then, we have a child policy that holds the condition.
        // We'll refer to it as the "condition" policy.

        // Finally, we'll have a policyset that is almost identical to
        // what we started with, but with no condition and a modified
        // name.  This is the "original" policyset, and it starts as a
        // clone of self.

        // we can leave the namespace alone, but the policy_ns might
        // need to change.  What is the purpose of the policy_ns?
        // keeping track of the parent policy names for serialization.
        // so the child policies need to have an extra node created,
        // based on the "last" one from the parent.

        // Define policy namespaces
        let mut cond_policy_ns = container.policy_ns.clone();
        cond_policy_ns.push_name(Rc::new(RefCell::new(Some("cond".to_owned()))));
        let cond_policy_id = PolicyId::PolicyName("cond".to_owned());

        let mut orig_policy_ns = container.policy_ns.clone();
        orig_policy_ns.push_name(Rc::new(RefCell::new(Some("orig".to_owned()))));
        // set the policy namespace for the original policyset
        original.policy_ns = orig_policy_ns;
        original.id = PolicyId::PolicyName("orig".to_owned());

        // create a child policy with the condition of this one; but first build the rule
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
        let condpolicy = Policy {
            id: cond_policy_id,
            ns: original.ns.clone(),
            policy_ns: cond_policy_ns,
            src_loc: self.span().clone(),
            description: None,
            target: None,
            condition: None,
            apply: RuleCombiningAlgorithm {
                id: format!("{}.{}", PROTECTED_NS, "permitOverrides"),
                src_loc: self.apply.span().clone(),
            },
            rules: vec![RuleEntry::Def(Rc::new(condrule))],
            prescriptions: vec![],
            ctx: self.ctx.clone(),
        };

        // place the condpolicy into the container first.
        container.policies.push(PolicyEntry::Policy(condpolicy));
        // then, place the original next.

        // we need to recurse through all the children of the original
        // policy and update their policy_ns.  basically, inserting an
        // extra "orig" entry so that if they don't have an ID, we are
        // printing out the new hierarchy appropriately.

        // we need to do this for every policy/policyset definition under this one.

        // This policy is at a given depth;
        let idx = original.policy_ns.policy_level() - 1;
        // the index we need to insert "orig" will be at index depth-1
        let orig_name: NameSlot = Rc::new(RefCell::new(Some("orig".to_owned())));
        let mut policies = original.policies;
        for cp in &mut policies {
            match cp {
                PolicyEntry::Policy(inline_policy) => {
                    info!("iterating through policies: {inline_policy}");
                    inline_policy.insert_ns_entry(orig_name.clone(), idx);
                }
                PolicyEntry::PolicySet(inline_policyset) => {
                    info!("iterating through policysets: {inline_policyset}");
                    inline_policyset.insert_ns_entry(&orig_name, idx);
                }
                PolicyEntry::Ref(_) => {
                    todo!("what happens if we call this with a Ref??");
                    // do we need to alter the name of the reference?
                    // or does the policy namespace not matter?
                }
            }
        }
        // replace policies
        original.policies = policies;
        container.policies.push(PolicyEntry::PolicySet(original));
        Ok(container)
    }
}

impl QualifiedName for PolicySet {
    fn fully_qualified_name(&self) -> Option<String> {
        match &self.id {
            PolicyId::PolicyName(_) | PolicyId::PolicyNameAndId(_, _) => {
                let mut qn = self.ns.join(".");
                //qn.push_str(s);
                // now we have the qualified name due to namespaces.  We need to add in any additional elements due to the parent policies.
                qn.push('.');
                qn.push_str(&self.policy_ns.build_path(".")?);
                Some(qn.to_string())
            }
            PolicyId::PolicyNoName => None,
        }
    }
}

impl fmt::Display for PolicySet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "policy set")
    }
}

/// Pretty print policyset
impl PrettyPrint for PolicySet {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

#[derive(Debug, PartialEq, Clone, Spanned)]
pub struct PolicyCombiningAlgorithm {
    pub id: String,
    pub src_loc: SrcLoc,
}

impl fmt::Display for PolicyCombiningAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// A policy/policyset that can be included in a policyset.
#[derive(Debug, PartialEq, Clone)]
pub enum PolicyEntry {
    /// Reference to a policy/policyset
    Ref(PolicyReference),
    /// An inline definition of a policy
    Policy(Policy),
    /// An inline definition of a policyset
    PolicySet(PolicySet),
}
impl fmt::Display for PolicyEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PolicyEntry::Ref(pr) => write!(f, "PE: policy ref: {pr}"),
            PolicyEntry::Policy(_p) => write!(f, "PE: policy"),
            PolicyEntry::PolicySet(_ps) => write!(f, "PE: policyset"),
        }
    }
}
/// Pretty print policyentry
impl PrettyPrint for PolicyEntry {
    fn pretty_print(&self, indent_level: usize) {
        match self {
            PolicyEntry::Ref(p) => p.pretty_print(indent_level),
            PolicyEntry::Policy(p) => p.pretty_print(indent_level),
            PolicyEntry::PolicySet(p) => p.pretty_print(indent_level),
        }
    }
}

/// Reference to a policy defined elsewhere
#[derive(Debug, PartialEq, Clone)]
pub struct PolicyReference {
    pub id: String,
    pub ns: Vec<String>,
}

impl PolicyReference {
    #[must_use]
    pub fn fully_qualified_name(&self) -> String {
        let mut fqname = self.ns.join(".");
        if fqname.len() > 0 {
            fqname.push('.');
        }
        fqname.push_str(&self.id);
        fqname
    }
}
/// Pretty print policyreference
impl PrettyPrint for PolicyReference {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

impl fmt::Display for PolicyReference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "policy reference")
    }
}
