//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::condition::Condition;
use super::naming::GenName;
use super::prescription::Prescription;
use super::target::Target;
use super::PrettyPrint;
use super::QualifiedName;
use crate::Context;
use log::warn;
use std::fmt;
use std::rc::Rc;
use std::rc::Weak;
use uuid::Uuid;

/// A rule that can be included in a policy.
#[derive(Debug, PartialEq, Clone)]
pub enum RuleEntry {
    /// Reference to a rule
    Ref(RuleReference),
    Def(Rc<RuleDef>),
}

/// Pretty print namespace and children
impl PrettyPrint for RuleEntry {
    fn pretty_print(&self, indent_level: usize) {
        match self {
            RuleEntry::Ref(r) => r.pretty_print(indent_level),
            RuleEntry::Def(r) => r.pretty_print(indent_level),
        }
    }
}

/// The effect of a rule (or obligation/advice)
#[derive(Debug, PartialEq, Clone)]
pub enum Effect {
    Permit,
    Deny,
}

impl fmt::Display for Effect {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Effect::Permit => write!(f, "Permit"),
            Effect::Deny => write!(f, "Deny"),
        }
    }
}

/// A rule definition
#[derive(Debug, Clone)]
pub struct RuleDef {
    pub id: Option<String>, // rule names are optional when declared
    pub ns: Vec<String>,
    /// The policy parents within a namespace
    pub policy_ns: GenName,
    pub description: Option<String>,
    pub effect: Effect,
    pub target: Option<Target>,
    pub condition: Option<Condition>,
    pub prescriptions: Vec<Prescription>, // on <effect> blocks
    pub ctx: Weak<Context>,
}

impl RuleDef {
    fn get_policy_name_prefix(&self) -> String {
        if let Some(ctx) = self.ctx.upgrade() {
            ctx.config.get_base_namespace()
        } else {
            warn!("could not get base namespace from rule context");
            String::default()
        }
    }
    #[must_use]
    pub fn get_id(&self) -> String {
        // Rules cannot have IDs defined in ALFA, so we always must
        // generate them.

        // Rule IDs do not seem to have a global uniqueness
        // requirement, so we just need to ensure uniqueness per
        // containing policy.

        // For now, we combine the global policy prefix, namespace,
        // parent policies, rule name (if one exists), and an
        // incrementing number (if no name exists).

        // TODO: there could be a rule name collision if the rule name
        // is something like "rule_1", and there is an additional
        // unnamed rule that gets auto-named since the auto-naming
        // does not look at other rule names... Resolving this by
        // using an anchor for the auto-gen'd name, which the manually
        // set rule name cannot contain.

        // TODO: if the parent policy has an ID, we might consider
        // using that instead of generating one with the default
        // prefix!

        if let Some(ppath_res) = self.policy_ns.build_path("/") {
            let mut pn = self.get_policy_name_prefix();
            let ns_str = self.ns.join("/");
            pn.push_str(&ns_str);
            if !ppath_res.is_empty() {
                pn.push('/');
                pn.push_str(&ppath_res);
            }
            match &self.id {
                None => {
                    // generate rule ID suffix if the rule had no name
                    if let Some(ctx) = self.ctx.upgrade() {
                        let next_id = ctx.get_next_rule_id(&self.ns.join("."));
                        pn.push_str(&format!("#rule_{next_id}"));
                    } else {
                        panic!("Context no longer exists, could not generate Rule ID");
                    }
                }
                Some(n) => {
                    // use the rule name if it exists
                    pn.push('/');
                    pn.push_str(n);
                }
            }
            pn
        } else {
            // TODO: Can the policy_ns ever be empty? No? What if
            // this rule was defined outside of a policy and is merely
            // re-used????
            let id = Uuid::now_v7();
            let mut id_str = "urn:uuid:".to_string();
            id_str.push_str(&id.to_string());
            panic!("Trying to name a rule that has no policy_ns set");
            //id_str
        }
    }
}

/// Target equality, ignoring context
impl PartialEq for RuleDef {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.ns == other.ns
            && self.effect == other.effect
            && self.target == other.target
    }
}

/// Reference to a rule defined elsewhere
#[derive(Debug, PartialEq, Clone)]
pub struct RuleReference {
    pub id: String,
    pub ns: Vec<String>,
}

impl QualifiedName for RuleDef {
    fn fully_qualified_name(&self) -> Option<String> {
        if let Some(i) = &self.id.as_ref() {
            let mut qn = self.ns.join(".");
            if !self.ns.is_empty() {
                qn.push('.');
            }
            // push the policy namespaces elements, if they exist.
            if !self.policy_ns.is_empty() {
                if let Some(policy_path) = self.policy_ns.build_path(".") {
                    qn.push_str(&policy_path);
                    qn.push('.');
                }
            }
            qn.push_str(i);
            Some(qn.to_string())
        } else {
            None
        }
    }
}

/// Pretty print namespace and children
impl PrettyPrint for RuleDef {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
        if let Some(t) = &self.target {
            t.pretty_print(indent_level + 1);
        }
        if let Some(c) = &self.condition {
            c.pretty_print(indent_level + 1);
        }
    }
}

impl fmt::Display for RuleDef {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Rule Def: {:?}, effect: {}",
            self.fully_qualified_name(),
            self.effect
        )
    }
}

impl QualifiedName for RuleReference {
    fn fully_qualified_name(&self) -> Option<String> {
        let mut qn = self.ns.join(".");
        if !self.ns.is_empty() {
            qn.push('.');
        }
        qn.push_str(&self.id);
        Some(qn.to_string())
    }
}

/// Pretty print namespace and children
impl PrettyPrint for RuleReference {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

impl fmt::Display for RuleReference {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: deal with Option type.
        write!(f, "Rule Reference: {:?}", self.fully_qualified_name())
    }
}
