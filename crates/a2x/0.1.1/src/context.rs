//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration and mutable state needed during an ALFA to XACML
//! conversion.
use crate::ast::advice::AdviceDef;
use crate::ast::attribute::Attribute;
use crate::ast::category::{standard_categories, Category};
use crate::ast::constant::Constant;
use crate::ast::function::Function;
use crate::ast::import::Import;
use crate::ast::infix::Infix;
use crate::ast::obligation::ObligationDef;
use crate::ast::policy::{Policy, PolicyId};
use crate::ast::policycombinator::{
    protected_policycombinators, standard_policycombinators, PolicyCombinator,
};
use crate::ast::policyset::{PolicyEntry, PolicySet};
use crate::ast::rule::RuleDef;
use crate::ast::rulecombinator::{
    protected_rulecombinators, standard_rulecombinators, RuleCombinator,
};
use crate::ast::std_attributes::standard_attributes;
use crate::ast::std_functions::standard_functions;
use crate::ast::std_infix::standard_infix;
use crate::ast::typedef::{standard_types, TypeDef};
use crate::ast::QualifiedName;
use crate::ast::{AsAlfa, SrcLoc};
use crate::errors::{ParseError, SrcError};
use log::debug;
use log::info;
use std::any::type_name;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::Instant;

pub const SYSTEM_NS: &str = "_A2X";
pub const PROTECTED_NS: &str = "_A2X.PROTECTED";

// A conversion context.  This contains configuration and mutable
// state that the conversion needs to keep track of.

/// Configuration for conversions.
#[derive(Debug, PartialEq)]
pub struct Config {
    /// Default namespace prefix.
    pub base_namespace: Option<String>,
    /// Use built-in functions, attributes, etc.
    pub enable_builtins: bool,
    /// Version string to be included in all policies.
    pub version: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            base_namespace: None,
            enable_builtins: true,
            version: None,
        }
    }
}

impl Config {
    /// Retrieve the base (default) namespace prefix, if one exists.
    #[must_use]
    pub fn get_base_namespace(&self) -> String {
        self.base_namespace
            .as_deref()
            .unwrap_or("https://sr.ht/~gheartsfield/a2x/alfa/ident/")
            .to_owned()
    }
}

/// Store and provide lookup facilities for an ALFA type.
///
/// This embodies the default namespace resolution strategy for all
/// elements that can be referenced from ALFA.
#[derive(Debug, PartialEq, Default)]
pub struct Resolver<T> {
    /// Mapping of fully-qualified names to specific elements.
    pub elements: RefCell<HashMap<String, Rc<T>>>,
}

impl<T> Resolver<T>
where
    T: QualifiedName,
    T: Debug,
{
    /// Create a new, empty resolver.
    #[must_use]
    pub fn new() -> Self {
        Resolver {
            elements: RefCell::new(HashMap::new()),
        }
    }

    /// Get the (Rust) type name this resolver contains, without any
    /// parent module names.
    ///
    /// This only exists to support log messages.
    fn short_type_name() -> &'static str {
        let full_type_name = type_name::<T>();
        match full_type_name.rfind("::") {
            Some(pos) => &full_type_name[pos + 2..],
            None => full_type_name,
        }
    }

    /// List all element names
    pub fn names(&self) -> Vec<String> {
        self.elements.borrow().keys().cloned().collect()
    }

    /// List all elements (sorted by key)
    pub fn elements(&self) -> Vec<Rc<T>> {
        let map_ref = self.elements.borrow();
        let mut items: Vec<_> = map_ref.iter().collect();
        items.sort_by_key(|(k, _v)| *k);
        items.into_iter().map(|(_k, v)| v.clone()).collect()
    }

    /// Element insertion
    ///
    /// Note, if the element to be registered does not have a
    /// qualified name (`fully_qualified_name` is `None`, then this
    /// still returns `Ok`, but does nothing.  Types that have no
    /// names cannot be looked up, so this seems sensible.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the registered element duplicates an
    /// existing symbol.
    pub fn register(&self, elem: Rc<T>) -> Result<(), ParseError> {
        // determine the fully-qualified name:
        if let Some(n) = elem.fully_qualified_name() {
            // add to map
            let mut m = self.elements.borrow_mut();
            let type_name = Resolver::<T>::short_type_name();
            debug!("registering {type_name}: {n:?}",);
            if let std::collections::hash_map::Entry::Vacant(e) = m.entry(n.clone()) {
                e.insert(elem);
                Ok(())
            } else {
                Err(ParseError::DuplicateSymbol(format!(
                    "Duplicate {type_name} definition: '{n}'"
                )))
            }
        } else {
            debug!("ignoring registration of anonymous element");
            Ok(())
        }
    }

    /// Existence check of fully-qualified name
    pub fn exists_fq(&self, fq_name: &str) -> bool {
        // check if elements contains this:
        let e = self.elements.borrow();
        e.get(fq_name).is_some()
    }

    fn match_one_and_only(matches: &[Rc<T>]) -> Result<Option<Rc<T>>, ParseError> {
        if let Some(m) = matches.first() {
            if matches.len() == 1 {
                return Ok(Some(m.clone()));
            }
            return Err(ParseError::AmbiguousImport(
                format!(
                    "symbol {:?} resolved to multiple locations from import statements",
                    m.fully_qualified_name()
                )
                .to_owned(),
            ));
        }
        Ok(None)
    }

    /// Start from the source namespace, where the reference was made,
    /// and see if the match exists in the same location.
    #[must_use]
    fn lookup_source_namespace(&self, symbol: &str, source_ns: &[String]) -> Option<Rc<T>> {
        // A symbol is the thing in the text, and may consist of dotted components (foo.bar).
        // The source_ns is where it was referenced.
        // imports are the set of import statements in effect where it was referenced.

        // iterate through the 6 namespace resolution rules. End with
        // SymbolNotFound if we couldn't locate anything.

        // Rule #1: child match: start from the source_namespace,
        // append symbol, and check for match. (can only be one)
        {
            let mut candidate = source_ns.join(".");
            candidate.push('.');
            candidate.push_str(symbol);
            debug!("R1: candidate is {candidate}");
            // check if elements contains this:
            let e = self.elements.borrow();
            //debug!("did a get: {:?}", x);
            if let Some(k) = e.get(&candidate) {
                //debug!("R1: Found key {:?}", k);
                // does the debug for function fail?
                return Some(k.clone());
            }
            debug!("R1: did not find match");
        }
        None
    }

    // Lookup a symbol starting at the root of the namespace.
    #[must_use]
    fn lookup_root(&self, symbol: &str) -> Option<Rc<T>> {
        // Rule #2: root match: attempt to match the full symbol.
        debug!("R2: candidate to match is [{symbol}]");
        let e = self.elements.borrow();
        if let Some(k) = e.get(symbol) {
            debug!("R2 Found key");
            return Some(k.clone());
        } else {
            debug!("R2 did not find key [{symbol}]");
        }
        None
    }

    // Lookup a symbol against a list of static imports (the import
    // itself must list the symbol name), assuming the imports are
    // referencing children of the current namespace.
    fn lookup_static_import_child(
        &self,
        symbol: &str,
        source_ns: &[String],
        static_imports: &Vec<Rc<Import>>,
    ) -> Result<Option<Rc<T>>, ParseError> {
        // for static-imports, the candidate must be formed by
        // matching the last element of the import with the
        // totality of the symbol.

        // We have to look through all possible imports, and
        // if there are more than one, we throw an error.
        let mut matches = vec![];
        for i in static_imports {
            // last component must match the candidate symbol in scope
            if let Some(last_component) = i.components.last() {
                if last_component == symbol {
                    debug!("R3: checking static import (relative): {i:?}");
                    let mut candidate = source_ns.join(".");
                    candidate.push('.');
                    let c = i.components.join(".");
                    candidate.push_str(&c);
                    debug!("R3: candidate is {candidate}");
                    // check if elements contains this:
                    let e = self.elements.borrow();
                    if let Some(k) = e.get(&candidate) {
                        debug!("R3: Found value {k:?}");
                        matches.push(k.clone());
                    }
                }
            }
        }
        if let Some(m) = Self::match_one_and_only(&matches)? {
            Ok(Some(m))
        } else {
            Ok(None)
        }
    }

    // Lookup a symbol against a list of static imports (the import
    // itself must list the symbol name), assuming the imports are
    // from the root namespace (fully qualified).
    fn lookup_static_import_qualified(
        &self,
        symbol: &str,
        static_imports: &Vec<Rc<Import>>,
    ) -> Result<Option<Rc<T>>, ParseError> {
        let mut matches = vec![];
        for i in static_imports {
            // last component must match the candidate symbol in scope
            if let Some(last_component) = i.components.last() {
                if last_component == symbol {
                    debug!("R4: checking static import (absolute): {i:?}");
                    let candidate = i.components.join(".");
                    debug!("R4: candidate is {candidate}");
                    // check if elements contains this:
                    let e = self.elements.borrow();
                    if let Some(k) = e.get(&candidate) {
                        debug!("R4: Found key {k:?}");
                        matches.push(k.clone());
                    }
                }
            }
        }
        if let Some(m) = Self::match_one_and_only(&matches)? {
            Ok(Some(m))
        } else {
            Ok(None)
        }
    }

    // Lookup a symbol against a list of wildcard imports, assuming
    // the imports are children of the current namespace.
    fn lookup_wildcard_child(
        &self,
        symbol: &str,
        source_ns: &[String],
        wildcard_imports: &Vec<Rc<Import>>,
    ) -> Result<Option<Rc<T>>, ParseError> {
        let mut matches = vec![];
        for i in wildcard_imports {
            // last component must match the candidate symbol in scope
            debug!("R5: checking wildcard import: {i:?}");
            // start with the current location
            let mut candidate = source_ns.join(".");
            candidate.push('.');
            // add the import statement
            let c = i.components.join(".");
            candidate.push_str(&c);
            // finish with the symbol
            candidate.push('.');
            candidate.push_str(symbol);
            debug!("R5: candidate is {candidate}");
            // check if elements contains this:
            let e = self.elements.borrow();
            if let Some(k) = e.get(&candidate) {
                debug!("R5: Found key {k:?}");
                matches.push(k.clone());
            }
        }
        if let Some(m) = Self::match_one_and_only(&matches)? {
            Ok(Some(m))
        } else {
            Ok(None)
        }
    }

    // Lookup a symbol against a list of wildcard imports, assuming
    // the imports are fully qualified.
    fn lookup_wildcard_qualified(
        &self,
        symbol: &str,
        wildcard_imports: &Vec<Rc<Import>>,
    ) -> Result<Option<Rc<T>>, ParseError> {
        let mut matches = vec![];
        for i in wildcard_imports {
            // last component must match the candidate symbol in scope
            debug!("R6: checking static import: {i:?}");
            // add the import statement, since we are assuming it is at the root
            let mut candidate = i.components.join(".");
            // finish with the symbol
            candidate.push('.');
            candidate.push_str(symbol);
            debug!("R6: candidate is {candidate}");
            // check if elements contains this:
            let e = self.elements.borrow();
            if let Some(k) = e.get(&candidate) {
                debug!("R6: Found key {k:?}");
                matches.push(k.clone());
            }
        }
        if let Some(m) = Self::match_one_and_only(&matches)? {
            Ok(Some(m))
        } else {
            Ok(None)
        }
    }

    /// Lookup a symbol located in a namespace using a set of imports.
    ///
    /// # Arguments
    /// * `symbol` - A name exactly as it appears in the ALFA source
    ///   text, including namespace references.
    /// * `source_ns` - The namespace where the symbol was located.
    /// * `src_loc` - Source of the symbol reference for error reporting.
    /// * `imports` - All import statements in effect where the symbol was used.
    /// # Returns
    ///
    /// Reference-counted element found according to namespace
    /// resolution rules.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the lookup could not find an element or
    /// the result was ambiguous.
    pub fn lookup(
        &self,
        symbol: &str,
        source_ns: &[String],
        src_loc: &SrcLoc,
        imports: Option<&Vec<Rc<Import>>>,
    ) -> Result<Rc<T>, ParseError> {
        info!(
            "doing a lookup of symbol [{}], in namespace [{}]",
            symbol,
            source_ns.join(".")
        );
        let type_name = Resolver::<T>::short_type_name();
        // A symbol is the thing in the text, and may consist of dotted components (foo.bar).
        // The source_ns is where it was referenced.
        // imports are the set of import statements in effect where it was referenced.

        // iterate through the 6 namespace resolution rules. End with
        // SymbolNotFound if we couldn't locate anything.

        // Rule #1: child match: start from the source_namespace,
        // append symbol, and check for match. (can only be one)
        if let Some(m) = self.lookup_source_namespace(symbol, source_ns) {
            return Ok(m);
        }
        // Rule #2: root match: attempt to match the full symbol.
        if let Some(m) = self.lookup_root(symbol) {
            return Ok(m);
        }
        // check for import statements
        if let Some(imports) = imports {
            // find static imports
            let (wildcard_imports, static_imports): (Vec<_>, Vec<_>) =
                imports.iter().map(Rc::clone).partition(|n| n.is_wildcard);
            info!(
                "There are {} wildcard imports and {} static imports",
                wildcard_imports.len(),
                static_imports.len()
            );
            // Rule #3: static-import child match
            // for static-imports, the candidate must be formed by
            // matching the last element of the import with the
            // totality of the symbol.

            // We have to look through all possible imports, and
            // if there are more than one, we throw an error.
            if let Some(m) = self.lookup_static_import_child(symbol, source_ns, &static_imports)? {
                return Ok(m);
            }
            // Rule #4: static-import fully-qualified
            // for static-imports, the candidate must be formed by
            // matching the last element of the import with the
            // totality of the symbol.
            if let Some(m) = self.lookup_static_import_qualified(symbol, &static_imports)? {
                return Ok(m);
            }
            // Rule #5: wildcard-import child match
            if let Some(m) = self.lookup_wildcard_child(symbol, source_ns, &wildcard_imports)? {
                return Ok(m);
            }
            // Rule #6: wildcard-import fully-qualified
            if let Some(m) = self.lookup_wildcard_qualified(symbol, &wildcard_imports)? {
                return Ok(m);
            }
        } else {
            debug!("There were no import statements, so this {type_name} could not be resolved");
        }
        Err(SrcError::new(
            "All referenced symbols must be defined",
            &format!("this {type_name} could not be resolved"),
            src_loc.clone(),
        ))
    }
}

/// an attribute definition has a URI to identify it, a category URI, and a type URI.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AttributeDefinition {
    pub uri: String,
    pub category: String,
    pub type_uri: String,
}

/// an attribute value is an attribute def + a stringified value.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AttributeValue {
    pub attr: AttributeDefinition,
    pub value: String,
}

/// a literal is just a type URI + string value.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TypedLiteral {
    pub type_uri: String,
    pub value: String,
}

// We intend for the context to be passed around as an Rc<Context>, so
// that it can be shared by as many clients as needed.
/// All shared state needed when parsing an ALFA source file.
#[derive(Debug, PartialEq)]
pub struct Context {
    /// Static configuration
    pub config: Config,
    /// Mapping of namespaces to imports.
    pub imports: RefCell<HashMap<String, Vec<Rc<Import>>>>,
    /// Next available ID for a policy
    next_id: RefCell<usize>,
    /// Mapping of namespaces to the next available policyset ID.
    policyset_id_mapping: RefCell<HashMap<String, usize>>,
    /// Mapping of namespaces to the next available policy ID.
    policy_id_mapping: RefCell<HashMap<String, usize>>,
    /// Mapping of namespaces to the next available rule ID.
    rule_id_mapping: RefCell<HashMap<String, usize>>,
    /// Mapping of fully qualified namespaces to `PolicySet` instances.
    policyset_resolver: Resolver<PolicySet>,
    /// Mapping of fully qualified namespaces to `Policy` instances.
    policy_resolver: Resolver<Policy>,
    /// Mapping of fully qualified namespaces to `Rule` instances.
    rule_resolver: Resolver<RuleDef>,
    /// Mapping of fully qualified namespaces to `RuleCombinator`
    /// instances.
    rulecombinator_resolver: Resolver<RuleCombinator>,
    /// Mapping of fully qualified namespaces to `PolicyCombinator`
    /// instances.
    policycombinator_resolver: Resolver<PolicyCombinator>,
    /// Mapping of fully qualified namespaces to `Function` instances.
    function_resolver: Resolver<Function>,
    /// Mapping of fully qualified namespaces to `Infix` instances.
    infix_resolver: Resolver<Infix>,
    /// Mapping of fully qualified namespaces to `Attribute` instances.
    attribute_resolver: Resolver<Attribute>,
    /// Mapping of fully qualified namespaces to `TypeDef` instances.
    typedef_resolver: Resolver<TypeDef>,
    /// Mapping of fully qualified namespaces to `AdviceDef` instances.
    advice_resolver: Resolver<AdviceDef>,
    /// Mapping of fully qualified namespaces to `ObligationDef` instances.
    obligation_resolver: Resolver<ObligationDef>,
    /// Mapping of fully qualified namespaces to `Category` instances.
    category_resolver: Resolver<Category>,
    /// Set of used URIs for identifying policysets, policies, and rules.
    used_uris: RefCell<HashSet<String>>,
}

impl Default for Context {
    fn default() -> Self {
        Context::new(Config::default())
    }
}

impl Context {
    /// Create a new context, with standard ALFA elements.
    ///
    /// # Panics
    ///
    /// Will panic if the standard elements cannot be added.  This
    /// should never happen, since they are all imported from static
    /// definitions.
    #[must_use]
    pub fn new(cfg: Config) -> Self {
        let mut c = Context {
            config: cfg,
            next_id: RefCell::new(0),
            policy_id_mapping: RefCell::new(HashMap::new()),
            policyset_id_mapping: RefCell::new(HashMap::new()),
            rule_id_mapping: RefCell::new(HashMap::new()),
            policyset_resolver: Resolver::<PolicySet>::new(),
            policy_resolver: Resolver::<Policy>::new(),
            rule_resolver: Resolver::<RuleDef>::new(),
            rulecombinator_resolver: Resolver::<RuleCombinator>::new(),
            policycombinator_resolver: Resolver::<PolicyCombinator>::new(),
            function_resolver: Resolver::<Function>::new(),
            infix_resolver: Resolver::<Infix>::new(),
            attribute_resolver: Resolver::<Attribute>::new(),
            typedef_resolver: Resolver::<TypeDef>::new(),
            advice_resolver: Resolver::<AdviceDef>::new(),
            obligation_resolver: Resolver::<ObligationDef>::new(),
            category_resolver: Resolver::<Category>::new(),
            imports: RefCell::new(HashMap::new()),
            used_uris: RefCell::new(HashSet::new()),
        };
        if c.config.enable_builtins {
            let start = Instant::now();
            c.add_standard_defs()
                .expect("adding standard imports failed");

            let elapsed = start.elapsed().as_micros();
            info!("Adding built-in alfa elements took {elapsed} microseconds");
        }
        // there are a minimal set of ALFA names we must have
        // available to help with some translations (particular the
        // "de-conditioning" of policies and policysets that have
        // conditions).  We call these "protected" because they are
        // always required to be present for this program to
        // successfully transform ALFA to XACML.
        c.add_protected_defs()
            .expect("adding protected imports failed");
        c
    }

    /// Get a new ID for use in identifying alfa elements such as
    /// unnamed policies. (Deprecated?)
    pub fn get_fresh_id(&self) -> usize {
        self.next_id.replace_with(|&mut old| old + 1)
    }

    /// Get the next ID for a rule at a namespace.
    pub fn get_next_rule_id(&self, ns: &str) -> usize {
        // when a rule is defined within a policy, we need to give it
        // a name, but if it is anonymous, then we need to generate
        // something.  This provides a unique incrementing number that
        // is specific to one namespace.

        // what if there is a situation where there is a namespace,
        // and two anonymous policies with rules under each?.

        let mut rids = self.rule_id_mapping.borrow_mut();
        *rids
            .entry(ns.to_owned())
            .and_modify(|e| *e += 1)
            .or_insert(0)
    }

    /// Get the next ID for a policy at a namespace.
    pub fn get_next_policy_id(&self, ns: &str) -> usize {
        let mut pids = self.policy_id_mapping.borrow_mut();
        *pids
            .entry(ns.to_owned())
            .and_modify(|e| *e += 1)
            .or_insert(0)
    }

    /// Get the next ID for a policyset at a namespace.
    pub fn get_next_policyset_id(&self, ns: &str) -> usize {
        let mut pids = self.policyset_id_mapping.borrow_mut();
        *pids
            .entry(ns.to_owned())
            .and_modify(|e| *e += 1)
            .or_insert(0)
    }

    /// Add minimal required XACML3 definitions.
    ///
    /// These will not be added to the import path, and must always be
    /// used fully qualified.  We only expect these to be used
    /// internally, not be externally provided ALFA policies.
    fn add_protected_defs(&mut self) -> Result<(), ParseError> {
        info!("adding protected/internal imports under namespace {PROTECTED_NS}");
        // currently we only need some policy/rule combinators.
        for p in protected_policycombinators() {
            self.register_policy_combinator(Rc::new(p))?;
        }
        for p in protected_rulecombinators() {
            self.register_rule_combinator(Rc::new(p))?;
        }
        Ok(())
    }

    /// Add default XACML3 types, categories, and algorithms if requested
    fn add_standard_defs(&mut self) -> Result<(), ParseError> {
        info!("adding standard imports under namespace {SYSTEM_NS}");
        // add the NS into import
        self.register_import(
            &[SYSTEM_NS.to_string()],
            Rc::new(Import {
                components: vec![SYSTEM_NS.to_string()],
                is_wildcard: true,
            }),
        );
        // we'll add a bunch of stuff with a SYSTEM_NS prefix
        // types
        for t in standard_types() {
            self.register_type(Rc::new(t))?;
        }
        // rule combinators
        for r in standard_rulecombinators() {
            self.register_rule_combinator(Rc::new(r))?;
        }
        // policy combinators
        for r in standard_policycombinators() {
            self.register_policy_combinator(Rc::new(r))?;
        }
        // categories
        for c in standard_categories() {
            self.register_category(Rc::new(c))?;
        }
        // infix operators
        for o in standard_infix() {
            self.register_infix(Rc::new(o))?;
        }
        // attributes
        for a in standard_attributes() {
            self.register_attribute(Rc::new(a))?;
        }
        // functions
        for f in standard_functions() {
            self.register_function(Rc::new(f))?;
        }
        Ok(())
    }

    /// Serialize builtins
    ///
    /// # Errors
    ///
    /// Returns `Err` if writing to output stream fails.
    pub fn serialize_builtins<W: std::io::Write>(&self, stream: &mut W) -> std::io::Result<()> {
        // write out namespace.
        stream.write_all(format!("namespace {SYSTEM_NS} {{\n").as_bytes())?;

        // Categories
        stream.write_all("\n  /** Categories **/\n\n".as_bytes())?;
        for c in self.category_resolver.elements() {
            stream.write_all(c.to_alfa(1).as_bytes())?;
        }
        // Type Definitions
        stream.write_all("\n  /** Type Definitions **/\n\n".as_bytes())?;
        for n in self.typedef_resolver.elements() {
            stream.write_all(n.to_alfa(1).as_bytes())?;
        }
        // Attributes
        stream.write_all("\n  /** Attributes **/\n\n".as_bytes())?;
        for a in self.attribute_resolver.elements() {
            stream.write_all(a.to_alfa(1).as_bytes())?;
            // add more space between attributes
            stream.write_all("\n".as_bytes())?;
        }
        // Policy Combinators
        stream.write_all("\n  /** Policy Combining Algorithms **/\n\n".as_bytes())?;
        for p in self.policycombinator_resolver.elements() {
            stream.write_all(p.to_alfa(1).as_bytes())?;
        }
        // Rule Combinators
        stream.write_all("\n  /** Rule Combining Algorithms **/\n\n".as_bytes())?;
        for r in self.rulecombinator_resolver.elements() {
            stream.write_all(r.to_alfa(1).as_bytes())?;
        }
        // Functions
        stream.write_all("\n  /** Functions **/\n\n".as_bytes())?;
        for f in self.function_resolver.elements() {
            stream.write_all(f.to_alfa(1).as_bytes())?;
        }
        // Operators
        stream.write_all("\n  /** Infix Operators **/\n\n".as_bytes())?;
        for i in self.infix_resolver.elements() {
            stream.write_all(i.to_alfa(1).as_bytes())?;
            // add more space between operators
            stream.write_all("\n".as_bytes())?;
        }
        // close namespace
        stream.write_all("}\n".as_bytes())?;
        Ok(())
    }

    /// Import Insertion
    pub fn register_import(&self, ns: &[String], rc: Rc<Import>) {
        let n = ns.join(".");
        let mut i = self.imports.borrow_mut();
        // attempt to get a mutable Vec against ns, add rc.
        if let Some(v) = i.get_mut(&n) {
            v.push(rc);
        } else {
            let import_vec = vec![rc];
            i.insert(n.to_string(), import_vec);
        }
    }

    /// Get imports for a namespace, combining with system default if configured
    fn get_imports(&self, source_ns: &[String]) -> Option<Vec<Rc<Import>>> {
        let imports = self.imports.borrow();
        // get imports from the default system namespace
        // TODO, only if configuration is set to use defaults
        let imports_default = imports.get(SYSTEM_NS);
        // get imports at the given namespace location
        if let Some(imports_at_ns) = imports.get(&source_ns.join(".")) {
            let mut i = vec![];
            i.extend(imports_at_ns.iter().cloned());
            if let Some(id) = imports_default {
                i.extend(id.clone());
            }
            Some(i)
        } else {
            imports_default.cloned()
        }
    }

    /// Register rule combinator
    ///
    /// # Errors
    ///
    /// Returns `Err` if a rule combinator with the same name exists.
    pub fn register_rule_combinator(&self, elem: Rc<RuleCombinator>) -> Result<(), ParseError> {
        self.rulecombinator_resolver.register(elem)
    }

    /// Lookup rule combinator
    ///
    /// # Errors
    ///
    /// Returns `Err` if the rule combinator does not exist, or is
    /// ambiguous.
    pub fn lookup_rule_combinator(
        &self,
        symbol: &str,
        source_ns: &[String],
        src_loc: &SrcLoc,
    ) -> Result<Rc<RuleCombinator>, ParseError> {
        info!("looking up rule combinator: symbol: {symbol:?}, source: {source_ns:?}");
        self.rulecombinator_resolver.lookup(
            symbol,
            source_ns,
            src_loc,
            self.get_imports(source_ns).as_ref(),
        )
    }

    /// Register policy combinator
    ///
    /// # Errors
    ///
    /// Returns `Err` if a policy combinator with the same name exists.
    pub fn register_policy_combinator(&self, elem: Rc<PolicyCombinator>) -> Result<(), ParseError> {
        self.policycombinator_resolver.register(elem)
    }

    /// Lookup policy combinator
    ///
    /// # Errors
    ///
    /// Returns `Err` if the policy combinator does not exist, or is
    /// ambiguous.
    pub fn lookup_policy_combinator(
        &self,
        symbol: &str,
        source_ns: &[String],
    ) -> Result<Rc<PolicyCombinator>, ParseError> {
        self.policycombinator_resolver.lookup(
            symbol,
            source_ns,
            &SrcLoc::default(),
            self.get_imports(source_ns).as_ref(),
        )
    }

    /// Register type
    ///
    /// # Errors
    ///
    /// Returns `Err` if a type with the same name exists.
    pub fn register_type(&self, elem: Rc<TypeDef>) -> Result<(), ParseError> {
        self.typedef_resolver.register(elem)
    }

    /// Lookup type
    ///
    /// # Errors
    ///
    /// Returns `Err` if the type does not exist, or is ambiguous.
    pub fn lookup_type(
        &self,
        symbol: &str,
        source_ns: &[String],
    ) -> Result<Rc<TypeDef>, ParseError> {
        self.typedef_resolver.lookup(
            symbol,
            source_ns,
            &SrcLoc::default(),
            self.get_imports(source_ns).as_ref(),
        )
    }

    /// Register function
    ///
    /// # Errors
    ///
    /// Returns `Err` if a function with the same name exists.
    pub fn register_function(&self, elem: Rc<Function>) -> Result<(), ParseError> {
        self.function_resolver.register(elem)
    }

    /// Lookup function
    ///
    /// # Errors
    ///
    /// Returns `Err` if the function does not exist, or is ambiguous.
    pub fn lookup_function(
        &self,
        symbol: &str,
        source_ns: &[String],
    ) -> Result<Rc<Function>, ParseError> {
        self.function_resolver.lookup(
            symbol,
            source_ns,
            &SrcLoc::default(),
            self.get_imports(source_ns).as_ref(),
        )
    }

    /// Register infix function
    ///
    /// # Errors
    ///
    /// Returns `Err` if an infix operator with the same name exists.
    pub fn register_infix(&self, elem: Rc<Infix>) -> Result<(), ParseError> {
        self.infix_resolver.register(elem)?;
        Ok(())
    }

    /// Register advice
    ///
    /// # Errors
    ///
    /// Returns `Err` if an advice definition with the same name exists.
    pub fn register_advice(&self, elem: Rc<AdviceDef>) -> Result<(), ParseError> {
        self.advice_resolver.register(elem)
    }

    /// Lookup advice
    ///
    /// # Errors
    ///
    /// Returns `Err` if the advice does not exist, or is ambiguous.
    pub fn lookup_advice(
        &self,
        symbol: &str,
        source_ns: &[String],
    ) -> Result<Rc<AdviceDef>, ParseError> {
        self.advice_resolver.lookup(
            symbol,
            source_ns,
            &SrcLoc::default(),
            self.get_imports(source_ns).as_ref(),
        )
    }

    /// Register obligation
    ///
    /// # Errors
    ///
    /// Returns `Err` if an obligation with the same name exists.
    pub fn register_obligation(&self, elem: Rc<ObligationDef>) -> Result<(), ParseError> {
        self.obligation_resolver.register(elem)
    }

    /// Lookup obligation
    ///
    /// # Errors
    ///
    /// Returns `Err` if the obligation does not exist, or is ambiguous.
    pub fn lookup_obligation(
        &self,
        symbol: &str,
        source_ns: &[String],
    ) -> Result<Rc<ObligationDef>, ParseError> {
        self.obligation_resolver.lookup(
            symbol,
            source_ns,
            &SrcLoc::default(),
            self.get_imports(source_ns).as_ref(),
        )
    }

    /// Lookup infix function
    ///
    /// # Errors
    ///
    /// Returns `Err` if the infix operator does not exist, or is
    /// ambiguous.
    pub fn lookup_infix(
        &self,
        symbol: &str,
        source_ns: &[String],
    ) -> Result<Rc<Infix>, ParseError> {
        self.infix_resolver.lookup(
            symbol,
            source_ns,
            &SrcLoc::default(),
            self.get_imports(source_ns).as_ref(),
        )
    }

    /// Lookup infix function by its inverse
    ///
    /// # Errors
    ///
    /// Returns `Err` if the inverse infix operator does not exist, or
    /// is ambiguous.
    pub fn lookup_infix_inverse(
        &self,
        symbol: &str,
        source_ns: &[String],
    ) -> Result<Rc<Infix>, ParseError> {
        // this could be more efficient if we let the caller provide
        // the original infix, skipping the first infix_resolver
        // lookup.

        // first we lookup the regular symbol.
        let i = self.infix_resolver.lookup(
            symbol,
            source_ns,
            &SrcLoc::default(),
            self.get_imports(source_ns).as_ref(),
        )?;
        // Then, we get the inverse operator symbol, looking up from
        // the same location as where the original was defined.
        if let Some(inv_sym) = &i.inverse {
            Ok(self.infix_resolver.lookup(
                inv_sym,
                source_ns,
                &SrcLoc::default(),
                self.get_imports(source_ns).as_ref(),
            )?)
        } else {
            // there was no inverse
            Err(ParseError::InverseInfixNotFound)
        }
    }

    /// Register policyset
    ///
    /// # Errors
    ///
    /// Returns `Err` if a policyset with the same name exists.
    pub fn register_policyset(&self, elem: Rc<PolicySet>) -> Result<(), ParseError> {
        // we should ensure that there is no *policy* with the exact
        // same name.  this would make the resolution of
        // policies/policysets by reference under a policyset be
        // ambiguous.

        // register all the child policies.
        for p in &elem.policies {
            info!("need to register {p}");
            match &p {
                PolicyEntry::Ref(_pe) => { // do nothing
                }
                PolicyEntry::Policy(pe) => {
                    debug!("registering policy from policyset");
                    self.register_policy(Rc::new(pe.clone()))?;
                }
                PolicyEntry::PolicySet(pe) => {
                    self.register_policyset(Rc::new(pe.clone()))?;
                }
            }
        }
        // TODO: prevent collisions from generated and assigned names.

        // if the policyset was assigned an ID, record that for
        // collision detection.
        if let PolicyId::PolicyNameAndId(_, i) = &elem.id {
            let mut uu = self.used_uris.borrow_mut();
            let unique = uu.insert(i.clone());
            if !unique {
                return Err(ParseError::DuplicateURI(i.clone()));
            }
        };
        // check if a policy exists in the same namespace with the
        // same name.
        if let Some(fq) = &elem.fully_qualified_name() {
            if self.policy_resolver.exists_fq(fq) {
                return Err(ParseError::DuplicatePolicyEntity(fq.clone()));
            }
        }

        self.policyset_resolver.register(elem)
    }

    /// Lookup policyset
    ///
    /// # Errors
    ///
    /// Returns `Err` if a policyset cannot be located.
    pub fn lookup_policyset(
        &self,
        symbol: &str,
        source_ns: &[String],
    ) -> Result<Rc<PolicySet>, ParseError> {
        let p = self.policyset_resolver.lookup(
            symbol,
            source_ns,
            &SrcLoc::default(),
            self.get_imports(source_ns).as_ref(),
        );
        p
    }

    /// Register policy
    ///
    /// # Errors
    ///
    /// Returns `Err` if a policy with the same name exists.
    pub fn register_policy(&self, elem: Rc<Policy>) -> Result<(), ParseError> {
        // we should ensure that there is no *policyset* with the exact
        // same name.  this would make the resolution of
        // policies/policysets by reference under a policyset be
        // ambiguous.

        // TODO: prevent collisions from generated and assigned names.

        // if the policyset was assigned an ID, record that for
        // collision detection.
        if let PolicyId::PolicyNameAndId(_, i) = &elem.id {
            let mut uu = self.used_uris.borrow_mut();
            let unique = uu.insert(i.clone());
            if !unique {
                return Err(ParseError::DuplicateURI(i.clone()));
            }
        };
        // check if a policyset exists in the same namespace with the
        // same name.
        if let Some(fq) = &elem.fully_qualified_name() {
            if self.policyset_resolver.exists_fq(fq) {
                return Err(ParseError::DuplicatePolicyEntity(fq.clone()));
            }
        }
        self.policy_resolver.register(elem)
    }

    /// Lookup policy
    ///
    /// # Errors
    ///
    /// Returns `Err` if a policy cannot be located.
    pub fn lookup_policy(
        &self,
        symbol: &str,
        source_ns: &[String],
    ) -> Result<Rc<Policy>, ParseError> {
        info!(
            "looking up policy with imports: {:?}",
            self.get_imports(source_ns)
        );
        let p = self.policy_resolver.lookup(
            symbol,
            source_ns,
            &SrcLoc::default(),
            self.get_imports(source_ns).as_ref(),
        );
        info!("finished policy resolver");
        p
    }

    /// Register rule
    ///
    /// # Errors
    ///
    /// Returns `Err` if a rule with the same name exists.
    pub fn register_rule(&self, elem: Rc<RuleDef>) -> Result<(), ParseError> {
        self.rule_resolver.register(elem)
    }

    /// Lookup rule
    ///
    /// # Errors
    ///
    /// Returns `Err` if the rule does not exist, or is ambiguous.
    pub fn lookup_rule(
        &self,
        symbol: &str,
        source_ns: &[String],
        src_loc: &SrcLoc,
    ) -> Result<Rc<RuleDef>, ParseError> {
        self.rule_resolver.lookup(
            symbol,
            source_ns,
            src_loc,
            self.get_imports(source_ns).as_ref(),
        )
    }

    /// Register attribute
    ///
    /// # Errors
    ///
    /// Returns `Err` if an attribute with the same name exists.
    pub fn register_attribute(&self, elem: Rc<Attribute>) -> Result<(), ParseError> {
        self.attribute_resolver.register(elem)
    }

    /// Lookup attribute
    ///
    /// # Errors
    ///
    /// Returns `Err` if the attribute does not exist, or is
    /// ambiguous.
    pub fn lookup_attribute(
        &self,
        symbol: &str,
        source_ns: &[String],
    ) -> Result<Rc<Attribute>, ParseError> {
        self.attribute_resolver.lookup(
            symbol,
            source_ns,
            &SrcLoc::default(),
            self.get_imports(source_ns).as_ref(),
        )
    }

    /// Register category
    ///
    /// # Errors
    ///
    /// Returns `Err` if a category with the same name exists.
    pub fn register_category(&self, elem: Rc<Category>) -> Result<(), ParseError> {
        self.category_resolver.register(elem)
    }

    /// Lookup category
    ///
    /// # Errors
    ///
    /// Returns `Err` if the category does not exist, or is
    /// ambiguous.
    pub fn lookup_category(
        &self,
        symbol: &str,
        source_ns: &[String],
    ) -> Result<Rc<Category>, ParseError> {
        self.category_resolver.lookup(
            symbol,
            source_ns,
            &SrcLoc::default(),
            self.get_imports(source_ns).as_ref(),
        )
    }

    /// Convert a constant to a typed literal.  This may involve
    /// looking up a type short name.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the lookup of a typedef fails.
    pub fn constant_to_typedliteral(
        &self,
        c: Constant,
        source_ns: &[String],
    ) -> Result<TypedLiteral, ParseError> {
        match c {
            Constant::String(s) => Ok(TypedLiteral {
                type_uri: crate::ast::typedef::STRING_URI.to_string(),
                value: s,
            }),
            Constant::Integer(s) => Ok(TypedLiteral {
                type_uri: crate::ast::typedef::INTEGER_URI.to_string(),
                value: s,
            }),
            Constant::Double(s) => Ok(TypedLiteral {
                type_uri: crate::ast::typedef::DOUBLE_URI.to_string(),
                value: s,
            }),
            Constant::Boolean(b) => Ok(TypedLiteral {
                type_uri: crate::ast::typedef::BOOLEAN_URI.to_string(),
                value: b.to_string(),
            }),
            Constant::Custom(ct, s) => {
                // we can only convert a custom-typed constant when we
                // know where it came from, so we can use imports appropriately.
                let t = self.typedef_resolver.lookup(
                    &ct.name,
                    source_ns,
                    &SrcLoc::default(),
                    self.get_imports(source_ns).as_ref(),
                )?;
                Ok(TypedLiteral {
                    type_uri: t.uri.to_string(),
                    value: s,
                })
            }
            Constant::Undefined => Err(ParseError::AstConvertError),
        }
    }
}
