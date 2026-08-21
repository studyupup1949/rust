//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! Parsing and represention of the Alfa abstract syntax tree.
pub mod advice;
pub mod attribute;
pub mod category;
pub mod condition;
pub mod constant;
pub mod designator;
pub mod function;
pub mod import;
pub mod infix;
pub mod namespace;
pub mod naming;
pub mod obligation;
pub mod operator;
pub mod policy;
pub mod policycombinator;
pub mod policyset;
pub mod prescription;
pub mod rule;
pub mod rulecombinator;
pub mod std_attributes;
pub mod std_functions;
pub mod std_infix;
pub mod target;
pub mod typedef;

use crate::AlfaParseTree;
use crate::Context;
use crate::Rule;
use crate::ast::category::Category;
use crate::ast::constant::{Constant, CustomType};
use crate::ast::function::Function;
use crate::ast::function::FunctionInputArg;
use crate::ast::function::FunctionInputs;
use crate::ast::function::FunctionOutputArg;
use crate::ast::import::Import;
use crate::ast::infix::Infix;
use crate::ast::infix::InfixSignature;
use crate::ast::namespace::Namespace;
use crate::ast::obligation::ObligationDef;
use crate::ast::operator::Operator;
use crate::ast::policy::Policy;
use crate::ast::policy::PolicyId;
use crate::ast::policycombinator::PolicyCombinator;
use crate::ast::policyset::{PolicyEntry, PolicyReference, PolicySet};
use crate::ast::rule::RuleEntry;
use crate::ast::rule::RuleReference;
use crate::ast::rulecombinator::RuleCombinator;
use crate::ast::target::{
    ConjunctiveSeq, DisjunctiveSeq, Match, MatchFunction, MatchOperation, Target,
};
use crate::ast::typedef::TypeDef;
use crate::errors::{ParseError, SrcError};
pub use a2x_derive::Spanned;
use advice::AdviceDef;
use attribute::Attribute;
use condition::{
    CondAtomUnparsed, CondExpressionUnparsed, CondFunctionCallUnparsed, CondItemUnparsed,
    Condition, ConditionUnparsed, FunctionReference,
};
use designator::AttributeDesignator;
use log::{debug, error, info, warn};
use miette::{NamedSource, SourceCode, SourceSpan};
use naming::GenName;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use policyset::PolicyCombiningAlgorithm;
use prescription::{
    AttrAssignmentSource, AttributeAssignment, Prescription, PrescriptionExpr, PrescriptionType,
};
use rule::{Effect, RuleDef};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use unescaper::unescape;

#[cfg(test)]
mod tests;

/// A fully parsed Alfa syntax tree.
#[derive(Debug)]
pub struct AlfaSyntaxTree {
    /// Namespaces which make up the Alfa source file.
    pub namespaces: Vec<Namespace>,
    /// Contextual information used to build and use the tree.
    pub ctx: Rc<Context>,
}

impl AlfaSyntaxTree {
    /// Retrieve all top-level policies in this AST.
    #[must_use]
    pub fn policies(&self) -> Vec<Rc<Policy>> {
        let mut ps = vec![];
        for ns in &self.namespaces {
            ps.append(&mut ns.policies());
        }
        ps
    }
    /// Retrieve all top-level policysets in this AST.
    #[must_use]
    pub fn policysets(&self) -> Vec<Rc<PolicySet>> {
        let mut ps = vec![];
        for ns in &self.namespaces {
            ps.append(&mut ns.policysets());
        }
        ps
    }
}

/// Identify a specific location or range in a source tree.
#[derive(Debug, Clone, PartialEq)]
pub struct SrcLoc {
    src: Arc<NamedSource<String>>,
    span: SourceSpan,
}

impl SrcLoc {
    /// Create a new SrcLoc
    pub fn new(src: NamedSource<String>, span: SourceSpan) -> SrcLoc {
        let mut s = SrcLoc {
            src: std::sync::Arc::new(src),
            span,
        };
        s.trim_trailing_whitespace();
        s
    }
    /// Create a new SrcLoc from a Pair
    pub fn new_from_pair(src: NamedSource<String>, pair: &Pair<'_, Rule>) -> SrcLoc {
        let pair_span = pair.as_span();
        let start = pair_span.start();
        let len = pair_span.end() - start;
        let mut s = SrcLoc {
            src: std::sync::Arc::new(src),
            span: (start, len).into(),
        };
        s.trim_trailing_whitespace();
        s
    }

    /// Retrieve the source text.
    pub fn get_src(&self) -> Arc<NamedSource<String>> {
        self.src.clone()
    }
    /// Retrieve the span information.
    pub fn get_span(&self) -> SourceSpan {
        self.span
    }

    /// Define the span based on a start and end position in the source
    pub fn with_start_end(&self, start_pos: usize, end_pos: usize) -> SrcLoc {
        let mut s = SrcLoc {
            src: self.src.clone(),
            span: (start_pos, end_pos - start_pos).into(),
        };
        s.trim_trailing_whitespace();
        s
    }

    /// Produce a new `SrcLoc` with the span from a Pair.
    pub fn from_pair(&self, pair: &Pair<'_, Rule>) -> SrcLoc {
        let pair_span = pair.as_span();
        self.with_start_end(pair_span.start(), pair_span.end())
    }

    /// Replace the span with a new value.
    pub fn with_new_span(&self, span: SourceSpan) -> SrcLoc {
        let mut s = SrcLoc {
            src: self.src.clone(),
            span,
        };
        s.trim_trailing_whitespace();
        s
    }
    /// Update the span to remove trailing whitespace
    fn trim_trailing_whitespace(&mut self) {
        if let Ok(s) = self.src.read_span(&self.span, 0, 0) {
            let new_len = s.data().trim_ascii_end().len();
            self.span = (self.span.offset(), new_len).into();
        }
    }
}

/// Trait for AST elements that have source location information.
///
/// Types implementing this trait can provide their original source
/// file and location.  This is used for error messages produced
/// later.
pub trait Spanned {
    /// Returns a reference to the source location of this element.
    fn span(&self) -> Option<&SrcLoc>;
}

impl TryFrom<AlfaParseTree<'_>> for AlfaSyntaxTree {
    type Error = ParseError;
    fn try_from(pt: AlfaParseTree) -> Result<Self, Self::Error> {
        // read the parse tree to create a series of potentially nested namespaces.
        let ctx = pt.ctx;
        let mut pairs = pt.pairs;
        // the first item must be an alfa_doc as the top-level.
        let doc = pairs.next().ok_or(ParseError::AstConvertError)?;
        assert!(doc.as_rule() == Rule::alfa_doc);
        let ns_toplevel = doc.into_inner();
        let mut namespaces = vec![];
        for ns in ns_toplevel {
            if ns.as_rule() == Rule::namespace {
                let src_loc = Some(SrcLoc::new_from_pair(pt.src.clone(), &ns));
                debug!("processing a namespace...");
                let n = process_namespace(ns, &vec![], src_loc, ctx.clone())?;
                namespaces.push(n);
            } else {
                // skip comments/whitespace/end-of-input tokens
                debug!("skipping comment/whitespace...rule: {:?}", ns.as_rule());
            }
        }
        Ok(AlfaSyntaxTree { namespaces, ctx })
    }
}

/// Take a comment string and remove the comment markers.
fn comment_cleanup(raw: &str) -> &str {
    if raw.starts_with("//") {
        raw.strip_prefix("//")
            .and_then(|x| x.strip_suffix("\n"))
            .map(str::trim)
            .unwrap()
    } else if raw.starts_with("/*") {
        raw.strip_prefix("/*")
            .and_then(|x| x.strip_suffix("*/"))
            // get rid of extra *'s at the beginning and end.
            .map(|x| x.trim_matches('*'))
            .map(str::trim)
            .unwrap()
    } else {
        warn!("text provided for cleanup did not have comment markers");
        raw
    }
}

/// Attempt to fully parse a `Pair` into a `Namespace`.
///
/// # Arguments
/// * `ns_pair` - The matching token pair for a namespace definition
/// * `ns` - The parent namespace where this namespace was defined
/// * `src_loc` - Source location information
/// * `ctx` - Parsing context
///
#[allow(clippy::needless_pass_by_value)]
fn process_namespace(
    ns_pair: Pair<Rule>,
    ns: &[String],
    src_loc: Option<SrcLoc>,
    ctx: Rc<Context>,
) -> Result<Namespace, ParseError> {
    assert_eq!(ns_pair.as_rule(), Rule::namespace);
    info!("found a namespace");
    let mut ns_pairs = ns_pair.into_inner();
    let mut ns = ns.to_vec();
    debug!("process_namespace pairs:  {ns_pairs:?}");
    // namespace tokens are made up of an identifier, and then a list
    // of alfa statements that occur inside the namespace definition.
    let namespace_ident = ns_pairs.next().ok_or(ParseError::AstConvertError)?;
    if namespace_ident.as_rule() == Rule::ns_identifier {
        debug!("ns_identifier: {namespace_ident:?}");
        // comprised of a list of ns_component
        let mut components = components_to_path(namespace_ident.into_inner());
        ns.append(&mut components);
        let mut ns = Namespace::from_components(&ns, ctx.clone());
        // as we go through the statements, if we find a comment, we
        // hold on to it and pass it to a policy or policy set if
        // necessary.
        let mut last_comment = None;
        for stmt in ns_pairs {
            if stmt.as_rule() == Rule::COMMENT {
                let cleaned_comment = comment_cleanup(stmt.as_str());
                last_comment = Some(cleaned_comment.to_string());
                // no need to break this statement apart further
                continue;
            }
            // these statements are always going to be "alfa_statement" rules.
            // we have to break them open one level deeper to see their specific type.
            let mut inner_stmt = stmt.into_inner();
            // get first Pair, which will identify the type of statement
            if let Some(first_stmt) = inner_stmt.next() {
                let stmt_loc = src_loc.as_ref().map(|s| s.from_pair(&first_stmt));
                let r = first_stmt.as_rule();
                if r == Rule::namespace {
                    let child_ns = process_namespace(first_stmt, &ns.path, stmt_loc, ctx.clone())?;
                    ns.add_namespace(child_ns);
                } else if r == Rule::policyset_decl {
                    ns.add_policyset(process_policyset(
                        first_stmt,
                        last_comment.clone(),
                        &ns.path,
                        GenName::default(),
                        true,
                        stmt_loc,
                        ctx.clone(),
                    )?)?;
                } else if r == Rule::policy_decl {
                    // provide namespace to policy
                    ns.add_policy(process_policy(
                        first_stmt,
                        last_comment.clone(),
                        &ns.path,
                        GenName::default(),
                        true,
                        stmt_loc,
                        ctx.clone(),
                    )?)?;
                } else if r == Rule::rule_combinator_decl {
                    ns.add_rulecombinator(process_rulecombinator(first_stmt, &ns.path, stmt_loc)?)?;
                } else if r == Rule::policy_combinator_decl {
                    ns.add_policycombinator(process_policycombinator(
                        first_stmt, &ns.path, stmt_loc,
                    )?)?;
                } else if r == Rule::import_decl {
                    let import_stmt = process_import(first_stmt, src_loc.clone());
                    ns.add_import(import_stmt);
                } else if r == Rule::type_decl {
                    let typedef_stmt = process_typedef(first_stmt, &ns.path)?;
                    ns.add_typedef(typedef_stmt)?;
                } else if r == Rule::function_decl {
                    let function_def = process_function(first_stmt, &ns.path)?;
                    ns.add_function(function_def)?;
                } else if r == Rule::cat_decl {
                    let category_stmt = process_category(first_stmt, &ns.path)?;
                    ns.add_category(category_stmt)?;
                } else if r == Rule::attribute_decl {
                    let attribute_stmt = process_attribute(first_stmt, &ns.path, src_loc.clone())?;
                    ns.add_attribute(attribute_stmt)?;
                } else if r == Rule::infix_decl {
                    let infix = process_infix(first_stmt, &ns.path)?;
                    ns.add_infix(infix)?;
                } else if r == Rule::advice_decl {
                    let advice = process_advice(first_stmt, &ns.path)?;
                    ns.add_advice(advice)?;
                } else if r == Rule::obligation_decl {
                    let obligation = process_obligation(first_stmt, &ns.path)?;
                    ns.add_obligation(obligation)?;
                } else if r == Rule::rule_decl {
                    let rule_item = process_rule(
                        first_stmt,
                        last_comment.clone(),
                        &ns.path,
                        GenName::default(),
                        stmt_loc,
                        &ctx,
                    )?;
                    ns.add_rule(rule_item)?;
                } else {
                    error!("unexpected rule {r:?}");
                    return Err(ParseError::UnexpectedRuleError(format!(
                        "found unexpected rule: {r:?}"
                    )));
                }
            }
            // clear out comment
            last_comment = None;
        }
        return Ok(ns);
    }
    Err(ParseError::AstConvertError)
}

/// Attempt to fully parse a `Pair` into a `Function`.
///
/// # Arguments
/// * `function_pair` - The matching token pair for a function definition
/// * `ns` - The parent namespace where this function was defined
fn process_function(function_pair: Pair<Rule>, ns: &[String]) -> Result<Function, ParseError> {
    assert!(function_pair.as_rule() == Rule::function_decl);
    info!("found a function");
    let mut function_pairs = function_pair.into_inner();
    // first, get the function name
    let tok = skip_comments(&mut function_pairs).ok_or(ParseError::AstConvertError)?;
    let id = if tok.as_rule() == Rule::function_name {
        tok.as_str().to_string()
    } else {
        return Err(ParseError::AstConvertError);
    };
    // then, get the URI
    let tok = skip_comments(&mut function_pairs).ok_or(ParseError::AstConvertError)?;
    let function_uri = if tok.as_rule() == Rule::string_literal {
        string_from_string_literal(tok)?
    } else {
        return Err(ParseError::AstConvertError);
    };
    // process input arguments
    let tok = skip_comments(&mut function_pairs).ok_or(ParseError::AstConvertError)?;
    let input_args = if tok.as_rule() == Rule::function_args {
        process_function_arguments(tok)?
    } else {
        return Err(ParseError::AstConvertError);
    };
    // process output result
    let tok = skip_comments(&mut function_pairs).ok_or(ParseError::AstConvertError)?;
    let output_arg = if tok.as_rule() == Rule::func_out {
        process_function_output(tok)?
    } else {
        return Err(ParseError::AstConvertError);
    };
    // build a Function
    Ok(Function {
        id,
        ns: ns.to_vec(),
        function_uri,
        input_args,
        output_arg,
    })
}

// TODO: write tests for function args that end in a wildcard
fn process_function_arguments(arg_pair: Pair<Rule>) -> Result<FunctionInputs, ParseError> {
    assert_eq!(arg_pair.as_rule(), Rule::function_args);
    debug!("found function arguments");
    let mut arg_pairs = arg_pair.into_inner();
    let mut args = vec![];
    let mut wildcard: bool = false;
    if let Some(tok) = skip_comments(&mut arg_pairs) {
        // check for leading wildcard
        if tok.as_rule() == Rule::wildcard_arg {
            debug!("found a wilcard argument");
            wildcard = true;
        }
        //
        assert!(tok.as_rule() == Rule::wildcard_arg || tok.as_rule() == Rule::func_arg);
        // expand the argument into the first inner rule of
        // func_identifier, then expand it again into a more specific
        // type.
        let mut func_ident = tok.into_inner();
        if let Some(i) = skip_comments(&mut func_ident) {
            assert!(i.as_rule() == Rule::func_identifier);
            let mut arg = i.into_inner();
            while let Some(a) = skip_comments(&mut arg) {
                // handle all the 6 cases of input args
                let r = a.as_rule();
                match r {
                    Rule::func_bag_ident => {
                        let mut i_pairs = a.into_inner();
                        let i = skip_comments(&mut i_pairs).ok_or(ParseError::AstConvertError)?;
                        args.push(FunctionInputArg::AtomicBag(i.as_str().to_string()));
                    }
                    Rule::func_anyatomic => {
                        args.push(FunctionInputArg::AnyAtomic);
                    }
                    Rule::func_bag_anyatomic => {
                        args.push(FunctionInputArg::AnyAtomicBag);
                    }
                    Rule::func_atomicorbag => {
                        args.push(FunctionInputArg::AnyAtomicOrBag);
                    }
                    Rule::ns_identifier => {
                        args.push(FunctionInputArg::Atomic(a.as_str().to_string()));
                    }
                    Rule::func_fn => {
                        args.push(FunctionInputArg::Function);
                    }
                    _ => {
                        return Err(ParseError::UnexpectedRuleError(format!(
                            "found unexpected function argument rule: {r:?}"
                        )));
                    }
                }
            }
        }
    }
    Ok(FunctionInputs { args, wildcard })
}

fn process_function_output(arg_pair: Pair<Rule>) -> Result<FunctionOutputArg, ParseError> {
    assert_eq!(arg_pair.as_rule(), Rule::func_out);
    let mut arg_pairs = arg_pair.into_inner();
    if let Some(tok) = skip_comments(&mut arg_pairs) {
        let r = tok.as_rule();
        match r {
            Rule::func_bag_ident => {
                let mut i_pairs = tok.into_inner();
                let i = skip_comments(&mut i_pairs).ok_or(ParseError::AstConvertError)?;
                return Ok(FunctionOutputArg::AtomicBag(i.as_str().to_string()));
            }
            Rule::func_anyatomic => {
                return Ok(FunctionOutputArg::AnyAtomic);
            }
            Rule::func_bag_anyatomic => {
                return Ok(FunctionOutputArg::AnyAtomicBag);
            }
            Rule::ns_identifier => {
                return Ok(FunctionOutputArg::Atomic(tok.as_str().to_string()));
            }
            _ => {
                return Err(ParseError::UnexpectedRuleError(format!(
                    "found unexpected function argument rule: {r:?}"
                )));
            }
        }
    }
    Err(ParseError::AstConvertError)
}

// parse a rule declaration, either in policy or namespace.
fn process_rule(
    rule_pair: Pair<Rule>,
    description: Option<String>,
    ns: &[String],
    parent_policy_path: GenName,
    src_loc: Option<SrcLoc>,
    ctx: &Rc<Context>,
) -> Result<RuleDef, ParseError> {
    assert_eq!(rule_pair.as_rule(), Rule::rule_decl);
    let mut rule_pairs = rule_pair.into_inner();
    debug!("{rule_pairs:?}");
    // rule name/id is optional
    let mut id: Option<String> = None;
    // a rule may have one target
    let mut target: Option<Target> = None;
    // a rule may have one condition
    let mut condition: Option<Condition> = None;
    // a rule must contain an effect.
    let mut found_effect: Option<rule::Effect> = None;
    // rules can have prescriptions (advice/obligations)
    let mut prescriptions: Vec<Prescription> = vec![];
    // Find first non-comment rule, and if it is an identifier, use it
    // as the name.  We can't use skip_comments, because a name might
    // not exist.
    while let Some(n) = rule_pairs.peek() {
        if n.as_rule() == Rule::COMMENT {
            rule_pairs.next();
        } else if n.as_rule() == Rule::identifier {
            // named rule
            id = Some(n.as_str().to_string());
            info!("rule id is: {id:?}");
            // consume the pair
            rule_pairs.next();
            break;
        } else {
            // anonymous rule, don't consume anything, and move on
            break;
        }
    }
    while let Some(tok) = skip_comments(&mut rule_pairs) {
        debug!("looking at rule {:?}: {:?}", tok.as_rule(), tok.as_str());
        let stmt_loc = src_loc.as_ref().map(|s| s.from_pair(&tok));
        if tok.as_rule() == Rule::effect_permit {
            if found_effect.is_none() {
                found_effect = Some(rule::Effect::Permit);
            } else {
                return Err(SrcError::err_opt(
                    "Rule has more than one effect defined",
                    "duplicate effect",
                    src_loc.map(|s| s.from_pair(&tok)).as_ref(),
                ));
            }
        } else if tok.as_rule() == Rule::effect_deny {
            if found_effect.is_none() {
                found_effect = Some(rule::Effect::Deny);
            } else {
                return Err(SrcError::err_opt(
                    "Rule has more than one effect defined",
                    "duplicate effect",
                    src_loc.map(|s| s.from_pair(&tok)).as_ref(),
                ));
            }
        } else if tok.as_rule() == Rule::target_stmt {
            if target.is_none() {
                target = Some(process_target(tok, ns, stmt_loc, ctx)?);
            } else {
                return Err(SrcError::err_opt(
                    "Rule has more than one target defined",
                    "duplicate target",
                    src_loc.map(|s| s.from_pair(&tok)).as_ref(),
                ));
            }
        } else if tok.as_rule() == Rule::condition_stmt {
            if condition.is_none() {
                condition = Some(process_condition(tok, &ns, stmt_loc, ctx)?);
            } else {
                return Err(SrcError::err_opt(
                    "Rule has more than one condition defined",
                    "duplicate condition",
                    src_loc.map(|s| s.from_pair(&tok)).as_ref(),
                ));
            }
        } else if tok.as_rule() == Rule::on_effect {
            prescriptions.push(process_prescription(tok, &ns, ctx)?);
        } else {
            let r = tok.as_rule();
            error!("unexpected parse rule: {r:?}");
            return Err(ParseError::UnexpectedRuleError(format!(
                "found unexpected parse rule {r:?}"
            )));
        }
    }

    if let Some(effect) = found_effect {
        info!("returning rule def");
        Ok(rule::RuleDef {
            id,
            description,
            ns: ns.to_vec(),
            policy_ns: parent_policy_path,
            target,
            condition,
            prescriptions,
            effect,
            src_loc,
            ctx: Rc::<Context>::downgrade(ctx),
        })
    } else {
        warn!("rule was missing effect");
        return Err(SrcError::err_opt(
            "Rule must have an effect defined (permit/deny)",
            "missing effect",
            src_loc.as_ref(),
        ));
    }
}
// produces single target, which is a collection of disjunctiveseqs
fn process_target(
    target_pair: Pair<Rule>,
    ns: &[String],
    src_loc: Option<SrcLoc>,
    ctx: &Rc<Context>,
) -> Result<Target, ParseError> {
    assert_eq!(target_pair.as_rule(), Rule::target_stmt);
    info!("found a target");
    let mut target_pairs = target_pair.into_inner();
    // loop through the target_disjunctions
    // each of the clauses will be ANDed together.
    let mut clauses = vec![];
    while let Some(tok) = skip_comments(&mut target_pairs) {
        assert_eq!(tok.as_rule(), Rule::target_disjunction);
        let loc = src_loc.as_ref().map(|s| s.from_pair(&tok));
        clauses.push(process_target_clause(tok, loc)?);
    }
    Ok(Target {
        clauses,
        ns: ns.to_vec(),
        ctx: Rc::<Context>::downgrade(ctx),
    })
}

/// Take a clause with or'd entries and produce a `DisjunctiveSeq`
fn process_target_clause(
    conj_pair: Pair<Rule>,
    src_loc: Option<SrcLoc>,
) -> Result<DisjunctiveSeq, ParseError> {
    assert_eq!(conj_pair.as_rule(), Rule::target_disjunction);
    let mut conj_pairs = conj_pair.into_inner();
    let mut conj_seq = vec![];
    while let Some(tok) = skip_comments(&mut conj_pairs) {
        assert_eq!(tok.as_rule(), Rule::target_conjunction);
        let loc = src_loc.as_ref().map(|s| s.from_pair(&tok));
        let m = process_target_conjunctions(tok, loc)?;
        conj_seq.push(m);
    }
    Ok(DisjunctiveSeq {
        statements: conj_seq,
    })
}
/// Take a list of and'd matches and produce a `ConjuctiveSeq`
fn process_target_conjunctions(
    disj_pair: Pair<Rule>,
    src_loc: Option<SrcLoc>,
) -> Result<ConjunctiveSeq, ParseError> {
    assert_eq!(disj_pair.as_rule(), Rule::target_conjunction);
    debug!("found target conjunction");
    let mut disj_pairs = disj_pair.into_inner();
    let mut matches = vec![];
    while let Some(tok) = skip_comments(&mut disj_pairs) {
        let m = process_target_match(tok, src_loc.clone())?;
        matches.push(m);
    }
    Ok(ConjunctiveSeq { matches })
}

fn process_operator(
    op_ident: &Pair<Rule>,
    src_loc: Option<SrcLoc>,
) -> Result<Operator, ParseError> {
    info!("processing operator:  {op_ident:?}");
    let full_name: Vec<String> = op_ident
        .as_str()
        .split('.')
        .map(std::string::ToString::to_string)
        .collect();
    if let Some((operator, ns)) = full_name.split_last() {
        Ok(Operator {
            ns: ns.to_vec(),
            operator: operator.to_string(),
            src_loc: src_loc.map(|s| s.from_pair(&op_ident)),
        })
    } else {
        Err(ParseError::AstConvertError)
    }
}

/// Take a child of rule `target_match` and produce a Match struct
fn process_target_match(
    match_pair: Pair<Rule>,
    src_loc: Option<SrcLoc>,
) -> Result<Match, ParseError> {
    let r = match_pair.as_rule();
    // This can handle target match (normal or reverse order)
    let mut match_pairs = match_pair.into_inner();
    // this creates one Match, either operator-based or a function call.
    if r == Rule::target_match_rev_op {
        let tok = skip_comments(&mut match_pairs).ok_or(ParseError::AstConvertError)?;
        let attr = process_attribute_designator(tok)?;
        let op_tok = skip_comments(&mut match_pairs).ok_or(ParseError::AstConvertError)?;
        let operator = process_operator(&op_tok, src_loc.clone())?;
        info!("got operator: {operator:?}");
        // next token will be a designator attribute block or a literal
        let tok = skip_comments(&mut match_pairs).ok_or(ParseError::AstConvertError)?;
        let literal = constant_from_token(tok)?;
        return Ok(Match::MatchOp(MatchOperation {
            attribute: attr.attribute,
            operator,
            literal,
            reversed: true,
            mustbepresent: attr.mustbepresent,
            issuer: attr.issuer,
        }));
    } else if r == Rule::target_match_op {
        // get literal
        let tok = skip_comments(&mut match_pairs).ok_or(ParseError::AstConvertError)?;
        let literal = constant_from_token(tok)?;

        // get operator name
        let tok = skip_comments(&mut match_pairs).ok_or(ParseError::AstConvertError)?;
        let operator = process_operator(&tok, src_loc.clone())?;

        // get the attribute designator
        let tok = skip_comments(&mut match_pairs).ok_or(ParseError::AstConvertError)?;
        let attr = process_attribute_designator(tok)?;
        return Ok(Match::MatchOp(MatchOperation {
            attribute: attr.attribute,
            operator,
            literal,
            reversed: false,
            mustbepresent: attr.mustbepresent,
            issuer: attr.issuer,
        }));
    } else if r == Rule::target_match_func {
        // get the elem_identifier
        let tok = skip_comments(&mut match_pairs).ok_or(ParseError::AstConvertError)?;
        let function_id = tok.as_str().split('.').map(String::from).collect();

        // get literal
        let tok = skip_comments(&mut match_pairs).ok_or(ParseError::AstConvertError)?;
        let literal = constant_from_token(tok)?;

        // get attribute
        let tok = skip_comments(&mut match_pairs).ok_or(ParseError::AstConvertError)?;
        let attr = process_attribute_designator(tok)?;

        return Ok(Match::MatchFunc(MatchFunction {
            function_id,
            literal,
            attribute: attr.attribute,
            issuer: attr.issuer,
            mustbepresent: attr.mustbepresent,
        }));
    }
    error!("encountered an unexpected rule: {:?}", r);
    Err(ParseError::AstConvertError)
}

/// Process a top-level condition statement.
///
/// This both performs parsing of tokens, as well as using operator
/// precedence rules and pratt-parser to group expressions.
fn process_condition(
    cond_pair: Pair<Rule>,
    ns: &[String],
    src_loc: Option<SrcLoc>,
    ctx: &Rc<Context>,
) -> Result<Condition, ParseError> {
    assert_eq!(cond_pair.as_rule(), Rule::condition_stmt);
    info!("Parsing a condition");
    let src_loc = src_loc.as_ref().map(|s| s.from_pair(&cond_pair));
    // first and only non-comment item in a cond_stmt is a cond_expression.
    let cond_expr = process_condition_expr(cond_pair, ns, src_loc.clone())?;
    // The unparsed (flat) condition tokens.
    let c = ConditionUnparsed {
        cond_expr,
        ns: ns.to_vec(),
        src_loc,
        ctx: Rc::<Context>::downgrade(ctx),
    };
    // Pratt-Parsed condition expression tree
    Condition::try_from(&c)
}

/// Process a condition expression from Pairs, skipping comments
fn process_condition_expr(
    cond_pair: Pair<Rule>,
    ns: &[String],
    src_loc: Option<SrcLoc>,
) -> Result<CondExpressionUnparsed, ParseError> {
    assert_eq!(cond_pair.as_rule(), Rule::condition_stmt);
    let mut cond_pairs = cond_pair.into_inner();
    let cond_expr = skip_comments(&mut cond_pairs).ok_or(ParseError::AstConvertError)?;
    process_condition_expr_pair(cond_expr, ns, src_loc)
}

/// Process a condition expression Rule.
fn process_condition_expr_pair(
    cond_pair: Pair<Rule>,
    ns: &[String],
    src_loc: Option<SrcLoc>,
) -> Result<CondExpressionUnparsed, ParseError> {
    assert_eq!(cond_pair.as_rule(), Rule::cond_expr);
    // since this is a Pair, we can determine the start and end point.
    let mut items: Vec<CondItemUnparsed> = vec![];
    let expr_loc = src_loc.as_ref().map(|s| s.from_pair(&cond_pair));
    let mut cond_expr = cond_pair.into_inner();
    while let Some(tok) = skip_comments(&mut cond_expr) {
        let tok_loc = src_loc.as_ref().map(|s| s.from_pair(&tok));
        if tok.as_rule() == Rule::cond_atom {
            let c = process_condition_atom(tok, ns, tok_loc)?;
            items.push(CondItemUnparsed::Atom(c));
        } else if tok.as_rule() == Rule::operator_identifier {
            items.push(CondItemUnparsed::Op(process_operator(&tok, tok_loc)?));
        } else {
            return Err(ParseError::UnexpectedRuleError(format!(
                "Expected an atom or operator, found {:?}",
                tok.as_rule()
            )));
        }
    }
    Ok(CondExpressionUnparsed {
        src_loc: expr_loc,
        items,
    })
}

fn process_condition_atom(
    cond_atom: Pair<Rule>,
    ns: &[String],
    src_loc: Option<SrcLoc>,
) -> Result<CondAtomUnparsed, ParseError> {
    assert_eq!(cond_atom.as_rule(), Rule::cond_atom);
    info!("parsing condition atom: {cond_atom:?}");
    // Unparsed Cond Atom
    let mut cond_atom = cond_atom.into_inner();
    if let Some(tok) = skip_comments(&mut cond_atom) {
        let r = tok.as_rule();
        let tok_loc = src_loc.as_ref().map(|s| s.from_pair(&tok));
        if r == Rule::cond_function_call {
            info!("atom > function call");
            let f = process_condition_function(tok, ns, tok_loc)?;
            return Ok(CondAtomUnparsed::Fn(f));
        } else if r == Rule::cond_function_ref {
            info!("function reference");
            let mut fr = tok.into_inner();
            if let Some(fr_ident) = skip_comments(&mut fr) {
                let identifier: Vec<String> =
                    fr_ident.as_str().split('.').map(String::from).collect();
                return Ok(CondAtomUnparsed::FnRef(FunctionReference { identifier }));
            }
            panic!("expected a function reference identifier");
        } else if r == Rule::cond_expr {
            info!("the tok is: {tok:?}");
            // we need to call it with the cond_atom
            let e = process_condition_expr_pair(tok, ns, src_loc)?;
            return Ok(CondAtomUnparsed::Expr(e));
        } else if r == Rule::attribute_designator {
            info!("got an attribute designator in condition");
            let attrd = process_attribute_designator(tok)?;
            return Ok(CondAtomUnparsed::Attr(attrd));
        } else if r == Rule::numeric_literal
            || r == Rule::boolean_literal
            || r == Rule::string_literal
            || r == Rule::custom_literal
        {
            info!("literal: {:?}", tok.as_str());
            let con = constant_from_token(tok)?;
            return Ok(CondAtomUnparsed::Lit(con));
        }
    }
    Err(ParseError::AstConvertError)
}

fn process_attribute_designator(attr_pair: Pair<Rule>) -> Result<AttributeDesignator, ParseError> {
    assert_eq!(attr_pair.as_rule(), Rule::attribute_designator);
    // defaults for mustbepresent and issuer
    let mut mustbepresent = false;
    let mut issuer: Option<String> = None;
    let mut attr_pairs = attr_pair.into_inner();
    // get the element identifier, which must be present
    let ident_tok = skip_comments(&mut attr_pairs).ok_or(ParseError::AstConvertError)?;
    if ident_tok.as_rule() == Rule::elem_identifier {
        let attribute: Vec<String> = ident_tok.as_str().split('.').map(String::from).collect();
        // block of options after an attribute (optional)
        let attr_block_opt = skip_comments(&mut attr_pairs);
        if let Some(attr_block) = attr_block_opt {
            let mut attr_block_entries = attr_block.into_inner();
            while let Some(attr_block_entry) = skip_comments(&mut attr_block_entries) {
                info!("looking at entry: {attr_block_entry:?}");
                if attr_block_entry.as_rule() == Rule::mustbepresent {
                    mustbepresent = true;
                } else if attr_block_entry.as_rule() == Rule::issuer {
                    let mut issuer_pairs = attr_block_entry.into_inner();
                    let issuer_str_lit =
                        skip_comments(&mut issuer_pairs).ok_or(ParseError::AstConvertError)?;
                    issuer = Some(string_from_string_literal(issuer_str_lit)?);
                }
            }
        }
        info!("attr designator: {attribute:?}, {mustbepresent:?}, {issuer:?}");
        Ok(AttributeDesignator {
            attribute,
            issuer,
            mustbepresent,
        })
    } else {
        Err(ParseError::AstConvertError)
    }
}

fn process_condition_function(
    cond_fn: Pair<Rule>,
    ns: &[String],
    src_loc: Option<SrcLoc>,
) -> Result<CondFunctionCallUnparsed, ParseError> {
    assert_eq!(cond_fn.as_rule(), Rule::cond_function_call);
    let mut cond_fn = cond_fn.into_inner();
    info!("parsing cond fn: {:?}", cond_fn.as_str());
    // first item will always be an elem_identifier.
    // second item will be a cond_argument_list;
    // inside of that, everything will be a CondExpr.
    let elem_ident = skip_comments(&mut cond_fn).ok_or(ParseError::AstConvertError)?;
    let identifier: Vec<String> = elem_ident.as_str().split('.').map(String::from).collect();
    let mut arguments = vec![];
    info!("elem_identifier:  {elem_ident:?}");
    if let Some(arglist) = skip_comments(&mut cond_fn) {
        let arg_loc = src_loc.as_ref().map(|s| s.from_pair(&arglist));
        arguments = process_cond_argument_list(arglist, ns, arg_loc)?;
    }
    Ok(CondFunctionCallUnparsed {
        identifier,
        arguments,
    })
}

fn process_cond_argument_list(
    arg_pair: Pair<Rule>,
    ns: &[String],
    src_loc: Option<SrcLoc>,
) -> Result<Vec<CondExpressionUnparsed>, ParseError> {
    assert_eq!(arg_pair.as_rule(), Rule::cond_argument_list);
    let mut arg_pairs = arg_pair.into_inner();
    let mut arguments = vec![];
    while let Some(tok) = skip_comments(&mut arg_pairs) {
        if tok.as_rule() == Rule::cond_expr {
            let tok_loc = src_loc.as_ref().map(|s| s.from_pair(&tok));
            let x = process_condition_expr_pair(tok, ns, tok_loc)?;
            arguments.push(x);
        } else {
            return Err(ParseError::UnexpectedRuleError("expected expr".to_string()));
        }
    }
    Ok(arguments)
}

/// Given a constant token (string, number, bool, custom), convert to a Constant struct
fn constant_from_token(tok: Pair<Rule>) -> Result<Constant, ParseError> {
    let lit_type = tok.as_rule();
    match lit_type {
        Rule::numeric_literal => {
            let nstr = tok.as_str().to_string();
            // numeric_literal rule assures us that only ascii digits,
            // dashes, and decimals are allowed.
            if nstr.contains('.') {
                Ok(Constant::Double(nstr))
            } else {
                Ok(Constant::Integer(nstr))
            }
        }
        Rule::boolean_literal => {
            if tok.as_str() == "true" {
                Ok(Constant::Boolean(true))
            } else if tok.as_str() == "false" {
                Ok(Constant::Boolean(false))
            } else {
                error!("got a non-boolean string from the rule 'boolean_literal'");
                Err(ParseError::AstConvertError)
            }
        }
        Rule::string_literal => Ok(Constant::String(
            unescape(&string_from_string_literal(tok)?)
                .expect("valid string")
                .clone(),
        )),
        Rule::custom_literal => {
            // get inner, which should be string_literal, elem_component.
            let mut i = tok.into_inner();
            let value = skip_comments(&mut i).ok_or(ParseError::AstConvertError)?;
            let value = string_from_string_literal(value)?;
            let typename = skip_comments(&mut i).ok_or(ParseError::AstConvertError)?;
            let dt = CustomType {
                name: typename.as_str().to_string(),
            };
            Ok(Constant::Custom(dt, value))
        }
        r => {
            error!("found unexpected literal type: {r:?}");
            Err(ParseError::AstConvertError)
        }
    }
}

/// Convert a `string_literal` rule into the (unquoted) string.  Works
/// for either double or single-quoted strings.
fn string_from_string_literal(tok: Pair<Rule>) -> Result<String, ParseError> {
    let mut quoted_inner = tok.into_inner();
    let quoted_literal = skip_comments(&mut quoted_inner).ok_or(ParseError::AstConvertError)?;
    let mut content_inner = quoted_literal.into_inner();
    let content = skip_comments(&mut content_inner).ok_or(ParseError::AstConvertError)?;
    assert!(
        content.as_rule() == Rule::double_string_content
            || content.as_rule() == Rule::single_string_content
    );
    Ok(content.as_str().to_string())
}

/// Process an Import statement.
fn process_import(import_pair: Pair<Rule>, src_loc: Option<SrcLoc>) -> Import {
    assert_eq!(import_pair.as_rule(), Rule::import_decl);
    let mut import_pairs = import_pair.into_inner();
    let mut components = vec![];
    let mut is_wildcard = false;
    let mut src_loc = src_loc.clone();
    while let Some(tok) = skip_comments(&mut import_pairs) {
        // read the import identifier
        if tok.as_rule() == Rule::import_identifier {
            src_loc = src_loc.map(|s| s.from_pair(&tok));
            let mut tok_inner = tok.into_inner();
            while let Some(itok) = skip_comments(&mut tok_inner) {
                if itok.as_rule() == Rule::ns_component {
                    components.push(itok.as_str().to_string());
                } else if itok.as_rule() == Rule::import_wildcard {
                    is_wildcard = true;
                } else if itok.as_rule() == Rule::infix_import {
                    components.push(itok.as_str().to_string());
                }
            }
        }
    }
    Import {
        components,
        is_wildcard,
        src_loc,
    }
}

/// Process a single instance of an "on <effect>" statement within a
/// rule/policy/policyset.
fn process_prescription(
    presc_pair: Pair<Rule>,
    ns: &[String],
    ctx: &Rc<Context>,
) -> Result<Prescription, ParseError> {
    assert_eq!(presc_pair.as_rule(), Rule::on_effect);
    // a prescription is our generic term for obligations/advice
    // emitted by an effect in a rule/policy/policyset.
    let mut presc_pairs = presc_pair.into_inner();
    // first pair will be the effect
    let effect_pair = skip_comments(&mut presc_pairs).ok_or(ParseError::AstConvertError)?;
    let effect = if effect_pair.as_rule() == Rule::effect_permit {
        Effect::Permit
    } else {
        Effect::Deny
    };
    // here we will store all the expressions.  Each expression will
    // represent one oblig/advice ID and all the assignments within.
    let mut expressions = vec![];
    info!("this prescription effect is: {effect}");
    while let Some(tok) = skip_comments(&mut presc_pairs) {
        if tok.as_rule() == Rule::apply_prescription {
            info!("got prescription: {:?}", tok.as_str());
            let mut ptype_pairs = tok.into_inner();
            // this is going to be an advice or obligation statement.
            let prescription_type_pair =
                skip_comments(&mut ptype_pairs).ok_or(ParseError::AstConvertError)?;
            let ptype = match prescription_type_pair.as_rule() {
                Rule::apply_advice => PrescriptionType::Advice,
                Rule::apply_obligation => PrescriptionType::Obligation,
                _ => return Err(ParseError::AstConvertError), // impossible
            };
            let mut ptype_exprs = prescription_type_pair.into_inner();
            // the remaining rules should be for assignments.
            let prescription_id_pair =
                skip_comments(&mut ptype_exprs).ok_or(ParseError::AstConvertError)?;
            let prescription_id = prescription_id_pair.as_str().to_owned();
            info!("the {ptype} ID is {prescription_id:?}");
            let mut assignments = vec![];
            // each item here is an assignment
            while let Some(prescription_assignment_pair) = skip_comments(&mut ptype_exprs) {
                assert_eq!(
                    prescription_assignment_pair.as_rule(),
                    Rule::prescription_assignment
                );
                let mut passignment = prescription_assignment_pair.into_inner();
                // read the destination ID
                let id_pair = skip_comments(&mut passignment).ok_or(ParseError::AstConvertError)?;
                let id = id_pair.as_str().to_owned();
                // read the source ID
                let source = skip_comments(&mut passignment).ok_or(ParseError::AstConvertError)?;
                let a = if source.as_rule() == Rule::attribute_designator {
                    AttrAssignmentSource::Attribute(process_attribute_designator(source)?)
                } else {
                    // the only other possible rule is a constant/literal value
                    AttrAssignmentSource::Value(constant_from_token(source)?)
                };
                assignments.push(AttributeAssignment {
                    destination_id: id,
                    source: a,
                });
            }
            let prescr_expr = PrescriptionExpr {
                ptype,
                id: prescription_id,
                assignments,
            };
            expressions.push(prescr_expr);
        } else {
            warn!("got something unexpected instead of obligation/advice statement");
            return Err(ParseError::AstConvertError);
        }
    }
    Ok(Prescription {
        effect,
        ns: ns.to_vec(),
        expressions,
        ctx: Rc::<Context>::downgrade(ctx),
    })
}

fn process_typedef(typedef_pair: Pair<Rule>, ns: &[String]) -> Result<TypeDef, ParseError> {
    assert_eq!(typedef_pair.as_rule(), Rule::type_decl);
    let mut typedef_pairs = typedef_pair.into_inner();
    let identifier = skip_comments(&mut typedef_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str();
    let id = identifier.to_string();
    let uri = skip_comments(&mut typedef_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"')
        .to_string();
    Ok(TypeDef {
        id,
        uri,
        ns: ns.to_vec(),
    })
}

fn process_category(category_pair: Pair<Rule>, ns: &[String]) -> Result<Category, ParseError> {
    assert_eq!(category_pair.as_rule(), Rule::cat_decl);
    let mut category_pairs = category_pair.into_inner();
    let identifier = skip_comments(&mut category_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str();
    let id = identifier.to_string();
    let uri = skip_comments(&mut category_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"')
        .to_string();
    Ok(Category {
        id,
        uri,
        ns: ns.to_vec(),
    })
}

fn process_infix(infix_pair: Pair<Rule>, ns: &[String]) -> Result<Infix, ParseError> {
    assert_eq!(infix_pair.as_rule(), Rule::infix_decl);
    let mut infix_pairs = infix_pair.into_inner();
    // attributes can contain id/type/category in any order, the PEG
    // does not confirm that all are present, just that exactly three
    // are.
    let mut commutative: Option<bool> = None;
    let mut allow_bags: Option<bool> = None;
    // operator name
    let mut operator_name: Option<String> = None;
    let mut signatures: Vec<InfixSignature> = vec![];
    let mut inverse: Option<String> = None;
    while let Some(t) = skip_comments(&mut infix_pairs) {
        let rule = t.as_rule();
        // process modifiers ("comm" and "allowbags")
        if rule == Rule::infix_modifier {
            let mut mod_inner_pairs = t.into_inner();
            let mod_inner =
                skip_comments(&mut mod_inner_pairs).ok_or(ParseError::AstConvertError)?;
            let modifier_rule = mod_inner.as_rule();
            if modifier_rule == Rule::comm_modifier {
                if commutative.is_some() {
                    return Err(ParseError::DuplicateInfixModifier);
                }
                commutative = Some(true);
            } else if modifier_rule == Rule::allowbags_modifier {
                if allow_bags.is_some() {
                    return Err(ParseError::DuplicateInfixModifier);
                }
                allow_bags = Some(true);
            }
        } else if rule == Rule::operator_name {
            assert!(operator_name.is_none());
            operator_name = Some(t.as_str().to_string());
        } else if rule == Rule::infix_arg_decl {
            let sig = process_infix_signature(t)?;
            signatures.push(sig);
        } else if rule == Rule::infix_inverse {
            let mut inverse_inner = t.into_inner();
            let inverse_op =
                skip_comments(&mut inverse_inner).ok_or(ParseError::AstConvertError)?;
            inverse = Some(inverse_op.as_str().to_string());
        }
    }
    if inverse.is_some() && commutative.unwrap_or(false) {
        Err(ParseError::CommutativeWithInverseError)
    } else if let Some(operator) = operator_name {
        Ok(Infix {
            ns: ns.to_vec(),
            operator,
            allow_bags: allow_bags.unwrap_or(false),
            commutative: commutative.unwrap_or(false),
            signatures,
            inverse,
        })
    } else {
        // should be unreachable based on Pest grammar
        error!("no operator was defined for this infix definition");
        Err(ParseError::AstConvertError)
    }
}

/// Convert an `infix_inverse` rule into a single signature
fn process_infix_signature(sig_pair: Pair<Rule>) -> Result<InfixSignature, ParseError> {
    assert_eq!(sig_pair.as_rule(), Rule::infix_arg_decl);
    let mut sig_pairs = sig_pair.into_inner();
    // expect exactly 4 items: URI, input 1, input 2, and output type.
    let uri_tok = skip_comments(&mut sig_pairs).ok_or(ParseError::AstConvertError)?;
    let uri = string_literal_to_string(uri_tok)?;
    // first and second argument
    let arg1_tok = skip_comments(&mut sig_pairs).ok_or(ParseError::AstConvertError)?;
    let first_arg = arg1_tok.as_str().to_string();
    let arg2_tok = skip_comments(&mut sig_pairs).ok_or(ParseError::AstConvertError)?;
    let second_arg = arg2_tok.as_str().to_string();
    let out_tok = skip_comments(&mut sig_pairs).ok_or(ParseError::AstConvertError)?;
    let output = out_tok.as_str().to_string();
    Ok(InfixSignature {
        uri,
        first_arg,
        second_arg,
        output,
    })
}

/// Process an advice declaration
fn process_advice(advice_pair: Pair<Rule>, ns: &[String]) -> Result<AdviceDef, ParseError> {
    assert_eq!(advice_pair.as_rule(), Rule::advice_decl);
    let mut advice_pairs = advice_pair.into_inner();
    let identifier = skip_comments(&mut advice_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str();
    let id = identifier.to_string();
    let uri = skip_comments(&mut advice_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"')
        .to_string();
    Ok(AdviceDef {
        id,
        uri,
        ns: ns.to_vec(),
    })
}

/// Process an obligation declaration
fn process_obligation(oblig_pair: Pair<Rule>, ns: &[String]) -> Result<ObligationDef, ParseError> {
    assert_eq!(oblig_pair.as_rule(), Rule::obligation_decl);
    let mut oblig_pairs = oblig_pair.into_inner();
    let identifier = skip_comments(&mut oblig_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str();
    let id = identifier.to_string();
    let uri = skip_comments(&mut oblig_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"')
        .to_string();
    Ok(ObligationDef {
        id,
        uri,
        ns: ns.to_vec(),
    })
}

fn process_attribute(
    attr_pair: Pair<Rule>,
    ns: &[String],
    src_loc: Option<SrcLoc>,
) -> Result<Attribute, ParseError> {
    assert_eq!(attr_pair.as_rule(), Rule::attribute_decl);
    let mut attr_pairs = attr_pair.into_inner();
    // attributes can contain id/type/category in any order, the PEG
    // does not confirm that all are present, just that exactly three
    // are.
    debug!("attr:  {attr_pairs:?}");
    // get the name
    let identifier = skip_comments(&mut attr_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .to_string();
    debug!("attr identifier: {identifier}");
    // do this three times, to get attr_type_assoc,  attr_id_assoc, attr_category_assoc
    let mut type_assoc: Option<String> = None;
    let mut id: Option<String> = None;
    let mut category: Option<String> = None;

    for _ in 0..3 {
        let next = skip_comments(&mut attr_pairs).ok_or(ParseError::AstConvertError)?;
        debug!("sub attr: {next:?}");
        // we only expect to see one of the 3 assignments allowed within an attribute def.
        match next.as_rule() {
            Rule::attr_type_assoc => {
                if type_assoc.is_none() {
                    type_assoc = Some(next.into_inner().as_str().to_string());
                } else {
                    error!("type defined twice in an attribute definition");
                    return Err(ParseError::AstConvertError);
                }
            }
            Rule::attr_id_assoc => {
                if id.is_none() {
                    id = Some(next.into_inner().as_str().trim_matches('"').to_string());
                } else {
                    error!("id defined twice in an attribute definition");
                    return Err(ParseError::AstConvertError);
                }
            }
            Rule::attr_category_assoc => {
                if category.is_none() {
                    category = Some(next.into_inner().as_str().to_string());
                } else {
                    error!("category defined twice in an attribute definition");
                    return Err(ParseError::AstConvertError);
                }
            }
            r => {
                return Err(ParseError::UnexpectedRuleError(format!(
                    "found unexpected rule {r:?}"
                )));
            }
        }
    }
    // if all three assignments were made, we can return an attribute
    if let (Some(typedef), Some(uri), Some(category)) = (type_assoc, id, category) {
        Ok(Attribute {
            typedef,
            id: identifier,
            uri,
            category,
            ns: ns.to_vec(),
            src_loc,
        })
    } else {
        // this should not be reachable
        error!("attribute declaration did not include all required values");
        Err(ParseError::AstConvertError)
    }
}

fn process_rulecombinator(
    rc_pair: Pair<Rule>,
    ns: &[String],
    src_loc: Option<SrcLoc>,
) -> Result<RuleCombinator, ParseError> {
    assert_eq!(rc_pair.as_rule(), Rule::rule_combinator_decl);
    let mut rc_pairs = rc_pair.into_inner();
    // skip comments
    let identifier_pair = skip_comments(&mut rc_pairs).ok_or(ParseError::AstConvertError)?;
    let identifier = identifier_pair.as_str();
    let rc_loc = src_loc.map(|s| s.from_pair(&identifier_pair));
    let id = identifier.to_string();
    let uri = skip_comments(&mut rc_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"');
    let rc = RuleCombinator {
        id,
        uri: uri.to_string(),
        src_loc: rc_loc,
        ns: ns.to_vec(),
    };
    Ok(rc)
}

fn process_policycombinator(
    pc_pair: Pair<Rule>,
    ns: &[String],
    src_loc: Option<SrcLoc>,
) -> Result<PolicyCombinator, ParseError> {
    assert_eq!(pc_pair.as_rule(), Rule::policy_combinator_decl);
    let mut pc_pairs = pc_pair.into_inner();
    // skip comments
    let token = skip_comments(&mut pc_pairs).ok_or(ParseError::AstConvertError)?;
    let identifier = token.as_str();
    // use the identifier as the source location.
    let src_loc = src_loc.map(|s| s.from_pair(&token));
    let id = identifier.to_string();
    let uri = skip_comments(&mut pc_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"');
    let pc = PolicyCombinator {
        id,
        uri: uri.to_string(),
        src_loc,
        ns: ns.to_vec(),
    };
    Ok(pc)
}

fn process_policyset(
    policyset_pair: Pair<Rule>,
    description: Option<String>,
    ns: &[String],
    mut parent_policy_path: GenName,
    register: bool,
    src_loc: Option<SrcLoc>,
    ctx: Rc<Context>,
) -> Result<PolicySet, ParseError> {
    assert_eq!(policyset_pair.as_rule(), Rule::policyset_decl);
    info!("found a policyset");
    let sp = policyset_pair.as_span();
    let start_pos = sp.start();
    let end_pos = sp.end();
    let mut policyset_pairs = policyset_pair.into_inner();
    let policy_id_rule = skip_comments(&mut policyset_pairs).ok_or(ParseError::AstConvertError)?;
    let policy_id = policy_naming(policy_id_rule, ctx.clone())?;
    // an apply statement is required.
    let mut apply = None;
    // a target is optional
    let mut target: Option<Target> = None;
    // a condition is optional
    let mut condition: Option<Condition> = None;
    // policies and policysets, in order
    let mut policies = vec![];
    // prescriptions (obligations/advice)
    let mut prescriptions = vec![];
    // only register this policyset if it has a name, and the parent has a name.
    let do_register = (policy_id != PolicyId::PolicyNoName) && register;

    // compute what child elements should receive for their policy_path:
    match &policy_id {
        PolicyId::PolicyNoName => parent_policy_path.push_name(Rc::new(RefCell::new(None))),
        PolicyId::PolicyName(s) | PolicyId::PolicyNameAndId(s, _) => {
            parent_policy_path.push_name(Rc::new(RefCell::new(Some(s.clone()))));
        }
    }
    // keep track of the last comment, for policy/policyset
    // definitions.
    let mut last_comment = None;

    for policyset_stmt in policyset_pairs {
        debug!(
            "looping through ns_pairs, found a: {:?}",
            policyset_stmt.as_rule()
        );
        if policyset_stmt.as_rule() == Rule::COMMENT {
            debug!("setting comment: {policyset_stmt:?}");
            let raw_comment = policyset_stmt.as_str();
            debug!("setting comment(as_str): {raw_comment:?}");
            let cleaned_comment = comment_cleanup(raw_comment);
            last_comment = Some(cleaned_comment.to_string());
            // no need to break this apart further
            continue;
        }
        if policyset_stmt.as_rule() == Rule::policyset_stmt {
            let mut t = policyset_stmt.into_inner();
            let stmt = skip_comments(&mut t).ok_or(ParseError::AstConvertError)?;
            let stmt_loc = src_loc.as_ref().map(|s| s.from_pair(&stmt));
            // Apply statement
            if stmt.as_rule() == Rule::apply_stmt {
                // get inner
                let mut apply_stmt = stmt.into_inner();
                let apply_ident =
                    skip_comments(&mut apply_stmt).ok_or(ParseError::AstConvertError)?;
                apply = Some(apply_ident.as_str().to_string());
                info!("apply: {apply:?}");
            } else if stmt.as_rule() == Rule::target_stmt {
                // target
                target = Some(process_target(stmt, &ns, stmt_loc, &ctx)?);
                info!("target: {target:?}");
            } else if stmt.as_rule() == Rule::condition_stmt {
                if condition.is_none() {
                    condition = Some(process_condition(stmt, &ns, stmt_loc, &ctx)?);
                } else {
                    return Err(ParseError::DuplicateCondition);
                }
            } else if stmt.as_rule() == Rule::policy_decl {
                let p = process_policy(
                    stmt,
                    last_comment.clone(),
                    &ns,
                    parent_policy_path.clone(),
                    do_register,
                    src_loc.clone(),
                    ctx.clone(),
                )?;
                policies.push(PolicyEntry::Policy(p));
            } else if stmt.as_rule() == Rule::policyset_decl {
                info!("PS creation");
                let p = process_policyset(
                    stmt,
                    last_comment.clone(),
                    &ns,
                    parent_policy_path.clone(),
                    do_register,
                    src_loc.clone(),
                    ctx.clone(),
                )?;
                info!("PS finish");
                policies.push(PolicyEntry::PolicySet(p));
            } else if stmt.as_rule() == Rule::policy_reference {
                let (policy_ns, policy_id) = split_dotted_string(stmt.as_str());
                let policy_ref = PolicyReference {
                    id: policy_id,
                    ns: policy_ns,
                };
                policies.push(PolicyEntry::Ref(policy_ref));
            } else if stmt.as_rule() == Rule::on_effect {
                info!("adding prescription to policyset");
                prescriptions.push(process_prescription(stmt, &ns, &ctx)?);
            } else {
                todo!("handle {:?}", stmt.as_rule());
            }
        }
    }
    info!("policyset id: {policy_id:?}");
    Ok(PolicySet {
        id: policy_id,
        ns: ns.to_vec(),
        policy_ns: parent_policy_path,
        src_loc: src_loc.clone(), // TODO: ensure this covers the full span
        description,
        apply: PolicyCombiningAlgorithm {
            id: apply.ok_or(SrcError::err_opt(
                "PolicySets must have an apply statement",
                "missing an apply statement",
                src_loc
                    .as_ref()
                    .map(|s| s.with_start_end(start_pos, end_pos))
                    .as_ref(),
            ))?,
            src_loc: src_loc.clone(),
        },
        target,
        condition,
        policies,
        prescriptions,
        ctx,
    })
}

fn process_policy(
    policy_pair: Pair<Rule>,
    description: Option<String>,
    ns: &[String],
    mut parent_policy_path: GenName,
    register: bool,
    src_loc: Option<SrcLoc>, // the parent source location
    ctx: Rc<Context>,
) -> Result<Policy, ParseError> {
    assert_eq!(policy_pair.as_rule(), Rule::policy_decl);
    info!("found a policy");
    let mut policy_pairs = policy_pair.into_inner();
    let policy_id_rule = skip_comments(&mut policy_pairs).ok_or(ParseError::AstConvertError)?;
    // an apply statement is required.
    let mut apply = None;
    // the apply will have a source location
    let mut apply_srcloc = src_loc.clone();
    // a target is optional
    let mut target = None;
    // a condition is optional
    let mut condition: Option<Condition> = None;
    // rules within policy
    let mut rules = vec![];
    // policies can have prescriptions (advice/obligations)
    let mut prescriptions: Vec<Prescription> = vec![];
    // keep track of last comment for rule definitions
    let mut last_comment = None;
    // get span info for this policy
    let sp = policy_id_rule.as_span();
    let start_pos = sp.start();
    let mut _end_pos = sp.end();
    let policy_src_loc = src_loc
        .as_ref()
        .map(|s| s.with_start_end(start_pos, _end_pos));
    // turn the policy_naming_rule into a PolicyId
    let policy_id = policy_naming(policy_id_rule, ctx.clone())?;
    // only register this policyset if it has a name, and the parent has a name.
    let do_register = (policy_id != PolicyId::PolicyNoName) && register;

    // compute what child elements should receive for their policy_path:
    match &policy_id {
        PolicyId::PolicyNoName => parent_policy_path.push_name(Rc::new(RefCell::new(None))),
        PolicyId::PolicyName(s) | PolicyId::PolicyNameAndId(s, _) => {
            parent_policy_path.push_name(Rc::new(RefCell::new(Some(s.clone()))));
        }
    }
    // look through each policy statement
    for policy_stmt in policy_pairs {
        debug!(
            "looping through ns_pairs, found a: {:?}",
            policy_stmt.as_rule()
        );
        if policy_stmt.as_rule() == Rule::COMMENT {
            debug!("setting comment: {policy_stmt:?}");
            let raw_comment = policy_stmt.as_str();
            debug!("setting comment(as_str): {raw_comment:?}");
            let cleaned_comment = comment_cleanup(raw_comment);
            last_comment = Some(cleaned_comment.to_string());
            // no need to break this apart further
            continue;
        }
        // these statements are always going to be "alfa_statement" rules.
        // we have to break them open one level deeper.
        if policy_stmt.as_rule() == Rule::policy_stmt {
            // update end position of span based on additional policy statements
            let sp = policy_stmt.as_span();
            _end_pos = sp.end();
            let mut t = policy_stmt.into_inner();
            let stmt = skip_comments(&mut t).ok_or(ParseError::AstConvertError)?;
            let stmt_loc = src_loc.as_ref().map(|s| s.from_pair(&stmt));
            // Apply statement
            if stmt.as_rule() == Rule::apply_stmt {
                apply_srcloc = stmt_loc.clone();
                // get inner
                let mut apply_stmt = stmt.into_inner();
                let apply_ident =
                    skip_comments(&mut apply_stmt).ok_or(ParseError::AstConvertError)?;
                apply = Some(apply_ident.as_str().to_string());
            } else if stmt.as_rule() == Rule::target_stmt {
                // target
                if target.is_none() {
                    let target_src = src_loc.as_ref().map(|s| s.from_pair(&stmt));
                    target = Some(process_target(stmt, &ns, target_src, &ctx)?);
                } else {
                    return Err(ParseError::DuplicateCondition);
                }
            } else if stmt.as_rule() == Rule::condition_stmt {
                if condition.is_none() {
                    condition = Some(process_condition(stmt, &ns, stmt_loc, &ctx)?);
                } else {
                    return Err(ParseError::DuplicateCondition);
                }
            } else if stmt.as_rule() == Rule::rule_decl {
                debug!("found rule declaration {stmt:?}");
                let rule_loc = src_loc.as_ref().map(|s| s.from_pair(&stmt));
                let rule_decl = process_rule(
                    stmt,
                    last_comment.clone(),
                    &ns,
                    parent_policy_path.clone(),
                    rule_loc,
                    &ctx,
                )?;
                // if this rule has a name, we must add it to the context resolver.
                let rule = Rc::new(rule_decl);
                if rule.id.is_some() && do_register {
                    info!("adding rule (child of policy) to context resolver");
                    ctx.register_rule(rule.clone())?;
                }
                rules.push(RuleEntry::Def(rule));
            } else if stmt.as_rule() == Rule::rule_reference {
                // an already defined rule is being referenced.
                debug!("found rule reference {stmt:?}");
                // a rule reference is just a bare (possibly qualified) name.
                let (rule_ns, rule_id) = split_dotted_string(stmt.as_str());
                // determine location
                let new_src_loc = src_loc.as_ref().map(|s| s.from_pair(&stmt));
                let rule_ref = RuleReference {
                    id: rule_id,
                    ns: rule_ns,
                    src_loc: new_src_loc,
                };
                rules.push(RuleEntry::Ref(rule_ref));
                info!("finished pushing ruleentry ref");
            } else if stmt.as_rule() == Rule::on_effect {
                info!("adding prescription to policy");
                prescriptions.push(process_prescription(stmt, &ns, &ctx)?);
            } else {
                info!("found something unexpected: {:?}", stmt.as_rule());
                return Err(ParseError::UnexpectedRuleError(format!(
                    "found rule '{:?}' within policy definition",
                    stmt.as_rule()
                )));
            }
        }
        // clear out comment
        last_comment = None;
    }
    Ok(Policy {
        id: policy_id,
        ns: ns.to_vec(),
        policy_ns: parent_policy_path,
        src_loc: policy_src_loc.clone(),
        description,
        apply: policy::RuleCombiningAlgorithm {
            id: apply.ok_or(SrcError::err_opt(
                "PolicySets must have an apply statement",
                "this policy needs an apply statement",
                policy_src_loc.as_ref(),
            ))?,
            src_loc: apply_srcloc,
        },
        target,
        condition,
        rules,
        prescriptions,
        ctx,
    })
}

/// Return the next non-comment rule in a Pairs, if one exists,
/// without consuming anything but comments.
fn _next_non_comment_rule<'a>(pairs: &'a mut Pairs<Rule>) -> Option<Pair<'a, Rule>> {
    let mut n = pairs.peek()?;
    while n.as_rule() == Rule::COMMENT {
        match pairs.next() {
            Some(next_pair) => n = next_pair,
            None => return None,
        }
    }
    Some(n)
}

/// Return the next non-comment rule in a Pairs, if one exists
fn skip_comments<'a>(pairs: &'a mut Pairs<Rule>) -> Option<Pair<'a, Rule>> {
    // Get the first item, return None if there isn't one
    let mut n = pairs.next()?;
    // Skip comments
    while n.as_rule() == Rule::COMMENT {
        // Try to get the next item, return None if there isn't one
        match pairs.next() {
            Some(next_pair) => n = next_pair,
            None => return None,
        }
    }
    Some(n)
}

// Thanks Claude
fn split_dotted_string(input: &str) -> (Vec<String>, String) {
    if input.is_empty() {
        return (vec![], String::new());
    }
    let parts: Vec<&str> = input.split('.').collect();
    if parts.len() == 1 {
        (vec![], parts[0].to_string())
    } else {
        let initial_parts = parts[..parts.len() - 1]
            .iter()
            .map(ToString::to_string)
            .collect();
        let last_part = parts[parts.len() - 1].to_string();
        (initial_parts, last_part)
    }
}

/// Take the first parse rule of a `policy_decl` and create a `PolicyId`.
fn policy_naming(policy_name_rule: Pair<Rule>, _ctx: Rc<Context>) -> Result<PolicyId, ParseError> {
    let policy_naming_rule = policy_name_rule.as_rule();
    // turn the policy_naming_rule into a PolicyId
    match policy_naming_rule {
        Rule::policy_empty => {
            debug!("this policy has no name");
            Ok(PolicyId::PolicyNoName)
        }
        Rule::policy_with_name => {
            debug!("this policy has a name");
            let mut inner = policy_name_rule.into_inner();
            // first item is identifier
            let ident = skip_comments(&mut inner).ok_or(ParseError::AstConvertError)?;
            if ident.as_rule() == Rule::identifier {
                Ok(PolicyId::PolicyName(ident.as_str().to_string()))
            } else {
                Err(ParseError::UnexpectedRuleError(format!(
                    "Expected a policy identifier for ident, but got {:?}",
                    ident.as_rule()
                )))
            }
        }
        Rule::policy_with_id => {
            debug!("this policy has a name and ID");
            let mut inner = policy_name_rule.into_inner();
            // first item is name, then policy identifier
            let name_ident = { skip_comments(&mut inner).ok_or(ParseError::AstConvertError)? };
            let mut name_str = None;
            if name_ident.as_rule() == Rule::identifier {
                name_str = Some(name_ident.as_str().to_string());
            }
            let id_literal = { skip_comments(&mut inner).ok_or(ParseError::AstConvertError)? };
            if name_str.is_some() && id_literal.as_rule() == Rule::string_literal {
                Ok(PolicyId::PolicyNameAndId(
                    name_str.unwrap(),
                    string_literal_to_string(id_literal)?,
                ))
            } else {
                Err(ParseError::UnexpectedRuleError(
                    "Expected a policy name and identifier".to_string(),
                ))
            }
        }
        x => Err(ParseError::UnexpectedRuleError(format!(
            "Expected a policy naming rule, but got {x:?}"
        ))),
    }
}

/// Convert a `string_literal` rule into the string contents (no quotes)
fn string_literal_to_string(lit: Pair<Rule>) -> Result<String, ParseError> {
    let mut contents_inner = lit.into_inner();
    let contents = contents_inner.next().ok_or(ParseError::AstConvertError)?;
    let r = contents.as_rule();
    if r == Rule::double_string_literal || r == Rule::single_string_literal {
        // get inner content (this removes quotes)
        let mut c = contents.into_inner();
        // first and only item is our string
        let s = c.next().ok_or(ParseError::AstConvertError)?;
        Ok(s.as_str().to_string())
    } else {
        Err(ParseError::AstConvertError)
    }
}

/// Given Pairs that contains `ns_components`, return just the path
/// components
fn components_to_path(components: Pairs<Rule>) -> Vec<String> {
    let mut path = vec![];
    for n in components {
        path.push(n.as_str().to_string());
    }
    path
}

/// Things that can return their fully-qualified namespace, if it
/// exists.
pub trait QualifiedName {
    fn fully_qualified_name(&self) -> Option<String>;
}

pub trait PrettyPrint {
    fn pretty_print(&self, indent_level: usize);
}

/// Items should be indented as requested, and emit a trailing
/// newline.
pub trait AsAlfa {
    fn to_alfa(&self, indent_level: usize) -> String;
}

/// An `AlfaSyntaxTree` and the source path information.
#[derive(Debug)]
pub struct AstSource {
    /// NamedSource with original contents, for error reporting
    pub src: NamedSource<String>,
    /// The parsed syntax tree.
    pub ast: AlfaSyntaxTree,
}

/// A collection of ASTs that we can iterate over portions of, and
/// perform lookups on.
#[allow(dead_code)]
pub struct AstCollection {
    /// All the ASTs in this collection.
    asts: Vec<AstSource>,
    ctx: Rc<Context>,
}

impl AstCollection {
    pub fn new(ctx: Rc<Context>) -> Self {
        Self { asts: vec![], ctx }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.asts.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.asts.is_empty()
    }
    pub fn add_ast(&mut self, ast: AstSource) {
        self.asts.push(ast);
    }

    /// Retrieve all policy sets
    // TODO: does this "ast" policysets listing include the deconditioned ones?
    // Do the old ones get removed?
    #[must_use]
    pub fn policysets(&self) -> Vec<Rc<PolicySet>> {
        let mut p = vec![];
        for a in &self.asts {
            info!("appending policysets...");
            p.append(&mut a.ast.policysets());
        }
        p
    }
    /// Retrieve all policies
    #[must_use]
    pub fn policies(&self) -> Vec<Rc<Policy>> {
        let mut p = vec![];
        for a in &self.asts {
            p.append(&mut a.ast.policies());
        }
        p
    }
}
