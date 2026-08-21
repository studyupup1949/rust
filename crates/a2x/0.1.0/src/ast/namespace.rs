//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::advice::AdviceDef;
use super::attribute::Attribute;
use super::category::Category;
use super::function::Function;
use super::import::Import;
use super::infix::Infix;
use super::obligation::ObligationDef;
use super::policy::Policy;
use super::policycombinator::PolicyCombinator;
use super::policyset::PolicySet;
use super::rule::RuleDef;
use super::rulecombinator::RuleCombinator;
use super::typedef::TypeDef;
use super::PrettyPrint;
use crate::context::Context;
use crate::errors::ParseError;
use log::{debug, info};

/// A namespace path that can have holes.
///
/// Holes are needed due to anonymous entities within the path, such
/// as a policy with no ID.
///
/// An alternative would be generating these during the parse tree, but then we would
/// not be able to distinguish what is in the source text vs what we made up.
pub struct NsPath {
    pub path: Vec<Option<String>>,
}

impl NsPath {
    // are there any anonymous entries?
    // turn this into a string
    // use a context to fill in entries?
}

use std::fmt;
use std::rc::Rc;
/// A Namespace node and all children.
///
/// An Alfa namespace is the top-level element for all documents.
///
/// Namespaces are also responsible for populating the context with
/// fully-qualified references to everything added to them.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Namespace {
    /// The namespace declared path elements from general to most
    /// specific.  This is the full path from the root.
    pub path: Vec<String>,
    /// Child namespaces contained within.
    pub namespaces: Vec<Namespace>,
    /// Import statements
    imports: Vec<Rc<Import>>,
    /// Child policysets contained within.
    policysets: Vec<Rc<PolicySet>>,
    /// Child policies contained within.
    policies: Vec<Rc<Policy>>,
    /// Child rules contained within.
    rules: Vec<Rc<RuleDef>>,
    /// Child rule combinators.
    rulecombinators: Vec<Rc<RuleCombinator>>,
    /// Child policy combinators.
    policycombinators: Vec<Rc<PolicyCombinator>>,
    /// Child type definitions.
    types: Vec<Rc<TypeDef>>,
    /// Child category definitions.
    categories: Vec<Rc<Category>>,
    /// Child attribute definitions.
    attributes: Vec<Rc<Attribute>>,
    /// Child function definitions.
    functions: Vec<Rc<Function>>,
    /// Child infix function definitions.
    infix_fns: Vec<Rc<Infix>>,
    /// Advice definitions.
    advice: Vec<Rc<AdviceDef>>,
    /// Obligation definitions.
    obligations: Vec<Rc<ObligationDef>>,
    /// Context for conversion
    pub ctx: Rc<Context>,
}

impl Namespace {
    /// A plain empty namespace with a single root identifier.
    pub fn new_root(name: String, ctx: Rc<Context>) -> Self {
        Namespace {
            path: vec![name],
            namespaces: vec![],
            imports: vec![],
            policysets: vec![],
            policies: vec![],
            rules: vec![],
            rulecombinators: vec![],
            types: vec![],
            categories: vec![],
            attributes: vec![],
            functions: vec![],
            infix_fns: vec![],
            advice: vec![],
            obligations: vec![],
            policycombinators: vec![],
            ctx,
        }
    }
    /// An empty namespace with multiple components of its name.
    pub fn from_components(components: Vec<String>, ctx: Rc<Context>) -> Self {
        Namespace {
            path: components,
            namespaces: vec![],
            imports: vec![],
            policysets: vec![],
            policies: vec![],
            rules: vec![],
            rulecombinators: vec![],
            types: vec![],
            categories: vec![],
            attributes: vec![],
            functions: vec![],
            infix_fns: vec![],
            advice: vec![],
            obligations: vec![],
            policycombinators: vec![],
            ctx,
        }
    }

    /// Add a child namespace
    pub fn add_namespace(&mut self, namespace: Namespace) {
        self.namespaces.push(namespace);
    }

    /// Add an import statement
    pub fn add_import(&mut self, import: Import) {
        let import = Rc::new(import);
        self.imports.push(import.clone());
        // Add reference into context
        self.ctx.register_import(&self.path, import);
    }

    /// Add a type definition
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_typedef(&mut self, typedef: TypeDef) -> Result<(), ParseError> {
        let t = Rc::new(typedef);
        self.types.push(t.clone());
        self.ctx.register_type(t)?;
        Ok(())
    }

    /// Add a category definition
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_category(&mut self, category: Category) -> Result<(), ParseError> {
        let cat = Rc::new(category);
        self.categories.push(cat.clone());
        self.ctx.register_category(cat)?;
        Ok(())
    }

    /// Add an attribute definition
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_attribute(&mut self, attribute: Attribute) -> Result<(), ParseError> {
        info!("from namespace, adding attribute {attribute:?}");
        let attr = Rc::new(attribute);
        self.attributes.push(attr.clone());
        self.ctx.register_attribute(attr)
    }

    /// Add a child policyset
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_policyset(&mut self, policyset: PolicySet) -> Result<(), ParseError> {
        info!("adding policyset: {:?}", policyset.id);
        let policyset = Rc::new(policyset);
        // check if there is a policy with the same name
        //info!("policies: {:?}", self.policies());
        //for p in &self.policies() {
        //    info!("checking this policyset against policy: {:?}", p);
        //}
        info!("about to push policysets");
        self.policysets.push(policyset.clone());
        // TODO: what happens if we have a policy with the same name?
        let x = self.ctx.register_policyset(policyset);
        info!("registered policyset");
        x
    }

    /// Add a child policy
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_policy(&mut self, policy: Policy) -> Result<(), ParseError> {
        info!("adding policy: {:?}", policy.id);
        info!("policysets: {}", &self.policysets().len());
        for p in &self.policysets() {
            info!("checking this policy against policyset: {:?}", p.id);
        }
        info!("finished checking policysets");
        let policy = Rc::new(policy);
        self.policies.push(policy.clone());
        self.ctx.register_policy(policy)
    }

    /// Add a child rule
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_rule(&mut self, rule: RuleDef) -> Result<(), ParseError> {
        info!("adding rule: {:?}", rule.id);
        let rule = Rc::new(rule);
        self.rules.push(rule.clone());
        self.ctx.register_rule(rule)
    }

    /// Add a child rulecombinator
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_rulecombinator(&mut self, rc: RuleCombinator) -> Result<(), ParseError> {
        // wrap the combinator in a reference count
        let rcrc = Rc::new(rc);
        // Add reference to the combinators directly inside this namespace
        self.rulecombinators.push(rcrc.clone());
        // Add reference to the fully-qualified map in the context
        self.ctx.register_rule_combinator(rcrc)?;
        Ok(())
    }

    /// Add a child policycombinator
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_policycombinator(&mut self, pc: PolicyCombinator) -> Result<(), ParseError> {
        // wrap the combinator in a reference count
        let pcrc = Rc::new(pc);
        // Add reference to the combinators directly inside this namespace
        self.policycombinators.push(pcrc.clone());
        // Add reference to the fully-qualified map in the context
        self.ctx.register_policy_combinator(pcrc)?;
        Ok(())
    }

    /// Add a child function
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_function(&mut self, f: Function) -> Result<(), ParseError> {
        // wrap the function in a reference count
        let frc = Rc::new(f);
        // Add reference to the combinators directly inside this namespace
        self.functions.push(frc.clone());
        // Add reference to the fully-qualified map in the context
        self.ctx.register_function(frc)?;
        Ok(())
    }

    /// Add a child infix function
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_infix(&mut self, f: Infix) -> Result<(), ParseError> {
        // wrap the function in a reference count
        let irc = Rc::new(f);
        // Add reference to the combinators directly inside this namespace
        self.infix_fns.push(irc.clone());
        // Add reference to the fully-qualified map in the context
        self.ctx.register_infix(irc)?;
        Ok(())
    }

    /// Add advice declaration
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_advice(&mut self, a: AdviceDef) -> Result<(), ParseError> {
        let arc = Rc::new(a);
        self.advice.push(arc.clone());
        self.ctx.register_advice(arc)?;
        Ok(())
    }

    /// Add obligation declaration
    /// # Errors
    ///
    /// Will return `Err` if there is a duplicate symbol.
    pub fn add_obligation(&mut self, o: ObligationDef) -> Result<(), ParseError> {
        let orc = Rc::new(o);
        self.obligations.push(orc.clone());
        self.ctx.register_obligation(orc)?;
        Ok(())
    }

    /// Get policies
    #[must_use]
    pub fn policies(&self) -> Vec<Rc<Policy>> {
        //todo!("fix stack overflow");
        // collect all policies at this level.
        let mut ps = self.policies.clone();
        // collect all policies for child namespaces
        for n in &self.namespaces {
            info!("...");
            let mut child_policies = n.policies();
            ps.append(&mut child_policies);
        }
        // TODO: this is iterating through namespaces, do we need to look through policysets too?
        // should we replace this with the context resolver elements() function?
        ps
    }

    /// Get policysets
    #[must_use]
    pub fn policysets(&self) -> Vec<Rc<PolicySet>> {
        debug!("getting list of all policysets at this namespace level");
        // collect all policies at this level.
        self.policysets.clone()
    }

    #[must_use]
    pub fn is_root(&self) -> bool {
        self.path.len() == 1
    }

    #[must_use]
    pub fn dotted_name(&self) -> String {
        self.path.join(".")
    }
}

/// Pretty print namespace and children
impl PrettyPrint for Namespace {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{}Namespace: {:?}", indent, self.dotted_name());
        // Show imports
        if !self.imports.is_empty() {
            for child_i in &self.imports {
                child_i.pretty_print(indent_level + 1);
            }
        }
        // Show policy combinators
        if !self.policycombinators.is_empty() {
            for child_pc in &self.policycombinators {
                child_pc.pretty_print(indent_level + 1);
            }
        }
        // Show rule combinators
        if !self.rulecombinators.is_empty() {
            for child_rc in &self.rulecombinators {
                child_rc.pretty_print(indent_level + 1);
            }
        }
        // Show type definitions
        if !self.types.is_empty() {
            for child_type in &self.types {
                child_type.pretty_print(indent_level + 1);
            }
        }
        // Show category definitions
        if !self.categories.is_empty() {
            for child_cat in &self.categories {
                child_cat.pretty_print(indent_level + 1);
            }
        }
        // Show attribute definitions
        if !self.attributes.is_empty() {
            for child_attr in &self.attributes {
                child_attr.pretty_print(indent_level + 1);
            }
        }
        // Show advice definitions
        if !self.advice.is_empty() {
            for child_advice in &self.advice {
                child_advice.pretty_print(indent_level + 1);
            }
        }
        // Show obligation definitions
        if !self.obligations.is_empty() {
            for child_obligation in &self.obligations {
                child_obligation.pretty_print(indent_level + 1);
            }
        }
        // Show functions
        if !self.functions.is_empty() {
            for child_fn in &self.functions {
                child_fn.pretty_print(indent_level + 1);
            }
        }
        // Show functions
        if !self.infix_fns.is_empty() {
            for child_infix in &self.infix_fns {
                child_infix.pretty_print(indent_level + 1);
            }
        }
        // Show policies
        if !self.policies.is_empty() {
            for child_pol in &self.policies {
                child_pol.pretty_print(indent_level + 1);
            }
        }
        // show all child namespaces
        if !self.namespaces.is_empty() {
            for child in &self.namespaces {
                child.pretty_print(indent_level + 1);
            }
        }
    }
}

impl fmt::Display for Namespace {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(
            f,
            "Namespace('{}', {} inner-ns)",
            self.dotted_name(),
            self.namespaces.len()
        )
    }
}
