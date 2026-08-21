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
use crate::errors::ParseError;
use crate::AlfaParseTree;
use crate::Context;
use crate::Rule;
use advice::AdviceDef;
use attribute::Attribute;
use condition::{
    CondAtomUnparsed, CondExpressionUnparsed, CondFunctionCallUnparsed, CondItemUnparsed,
    Condition, ConditionUnparsed, FunctionReference,
};
use designator::AttributeDesignator;
use log::{debug, error, info, warn};
use naming::GenName;
use pest::iterators::Pair;
use pest::iterators::Pairs;
use prescription::{
    AttrAssignmentSource, AttributeAssignment, Prescription, PrescriptionExpr, PrescriptionType,
};
use rule::{Effect, RuleDef};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
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

// AlfaSyntaxTree implement TryFrom<AlfaParseTree> to create itself.
impl TryFrom<AlfaParseTree<'_>> for AlfaSyntaxTree {
    type Error = ParseError;
    fn try_from(pt: AlfaParseTree) -> Result<Self, Self::Error> {
        // read the parse tree to create a series of potentially nested namespaces.
        // Get the pairs
        let ctx = pt.ctx;
        let mut pairs = pt.pairs;
        // the first item must be an alfa_doc as the top-level.
        let doc = pairs.next().ok_or(ParseError::AstConvertError)?;
        if doc.as_rule() == Rule::alfa_doc {
            debug!("Found Alfa Document: <<<{}>>>", doc.as_str());
        }
        let ns_toplevel = doc.into_inner();
        let mut namespaces = vec![];
        for ns in ns_toplevel {
            if ns.as_rule() == Rule::namespace {
                debug!("processing a namespace...({ns})");
                let n = process_namespace(ns.into_inner(), vec![], ctx.clone())?;
                namespaces.push(n);
            } else {
                debug!("skipping comment...");
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

/// Attempt to fully parse `Pairs` into a namespace.
#[allow(clippy::needless_pass_by_value)]
fn process_namespace(
    mut ns_pairs: Pairs<Rule>,
    ns_path: Vec<String>,
    ctx: Rc<Context>,
) -> Result<Namespace, ParseError> {
    let mut ns_path = ns_path.clone();
    debug!("process_namespace pairs:  {ns_pairs:?}");
    // namespace rules are made up of an identifier, and then a list
    // of alfa statements that occur inside the namespace.
    let namespace_ident = ns_pairs.next().ok_or(ParseError::AstConvertError)?;
    // skip comments...
    debug!("next rule is {:?}", namespace_ident.as_rule());
    // TODO: add result return type so that we can assume this succeeds
    if namespace_ident.as_rule() == Rule::ns_identifier {
        debug!("ns_identifier: {namespace_ident:?}");
        // comprised of a list of ns_component
        let mut components = components_to_path(namespace_ident.into_inner());
        ns_path.append(&mut components);
        let mut ns = Namespace::from_components(ns_path, ctx.clone());
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
            // we have to break them open one level deeper.
            let mut inner_stmt = stmt.into_inner();
            // get first Pair, which will identify the type of statement
            if let Some(first_stmt) = inner_stmt.next() {
                let r = first_stmt.as_rule();
                if r == Rule::namespace {
                    let child_ns =
                        process_namespace(first_stmt.into_inner(), ns.path.clone(), ctx.clone())?;
                    ns.add_namespace(child_ns);
                } else if r == Rule::policyset_decl {
                    ns.add_policyset(process_policyset(
                        first_stmt,
                        ns.path.clone(),
                        GenName::default(),
                        last_comment.clone(),
                        true,
                        ctx.clone(),
                    )?)?;
                } else if r == Rule::policy_decl {
                    // provide namespace to policy
                    ns.add_policy(process_policy(
                        first_stmt.into_inner(),
                        ns.path.clone(),
                        GenName::default(),
                        last_comment.clone(),
                        true,
                        ctx.clone(),
                    )?)?;
                } else if r == Rule::rule_combinator_decl {
                    ns.add_rulecombinator(process_rulecombinator(
                        first_stmt.into_inner(),
                        ns.path.clone(),
                    )?)?;
                } else if r == Rule::policy_combinator_decl {
                    ns.add_policycombinator(process_policycombinator(
                        first_stmt.into_inner(),
                        ns.path.clone(),
                    )?)?;
                } else if r == Rule::import_decl {
                    let import_stmt = process_import(first_stmt.into_inner());
                    ns.add_import(import_stmt);
                } else if r == Rule::type_decl {
                    let typedef_stmt = process_typedef(first_stmt.into_inner(), ns.path.clone())?;
                    ns.add_typedef(typedef_stmt)?;
                } else if r == Rule::function_decl {
                    let function_def = process_function(first_stmt, ns.path.clone())?;
                    ns.add_function(function_def)?;
                } else if r == Rule::cat_decl {
                    let category_stmt = process_category(first_stmt.into_inner(), ns.path.clone())?;
                    ns.add_category(category_stmt)?;
                } else if r == Rule::attribute_decl {
                    let attribute_stmt =
                        process_attribute(first_stmt.into_inner(), ns.path.clone())?;
                    ns.add_attribute(attribute_stmt)?;
                } else if r == Rule::infix_decl {
                    let infix = process_infix(first_stmt.into_inner(), ns.path.clone())?;
                    ns.add_infix(infix)?;
                } else if r == Rule::advice_decl {
                    let advice = process_advice(first_stmt.into_inner(), ns.path.clone())?;
                    ns.add_advice(advice)?;
                } else if r == Rule::obligation_decl {
                    let obligation = process_obligation(first_stmt.into_inner(), ns.path.clone())?;
                    ns.add_obligation(obligation)?;
                } else if r == Rule::rule_decl {
                    let rule_item = process_rule(
                        first_stmt.into_inner(),
                        ns.path.clone(),
                        GenName::default(),
                        last_comment.clone(),
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

fn process_function(function_pair: Pair<Rule>, ns: Vec<String>) -> Result<Function, ParseError> {
    assert!(function_pair.as_rule() == Rule::function_decl);
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
        process_function_arguments(tok.into_inner())?
    } else {
        return Err(ParseError::AstConvertError);
    };
    // process output result
    let tok = skip_comments(&mut function_pairs).ok_or(ParseError::AstConvertError)?;
    let output_arg = if tok.as_rule() == Rule::func_out {
        process_function_output(tok.into_inner())?
    } else {
        return Err(ParseError::AstConvertError);
    };
    // build a Function
    Ok(Function {
        id,
        ns,
        function_uri,
        input_args,
        output_arg,
    })
}

fn process_function_arguments(mut arg_pairs: Pairs<Rule>) -> Result<FunctionInputs, ParseError> {
    let mut args = vec![];
    let mut wildcard: bool = false;
    // Also need to return whether this was a wildcard
    if let Some(tok) = skip_comments(&mut arg_pairs) {
        if tok.as_rule() == Rule::wildcard_arg {
            debug!("found a wilcard argument");
            wildcard = true;
        }
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

fn process_function_output(mut arg_pairs: Pairs<Rule>) -> Result<FunctionOutputArg, ParseError> {
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
    mut rule_pairs: Pairs<Rule>,
    ns: Vec<String>,
    policy_ns: GenName,
    description: Option<String>,
    ctx: &Rc<Context>,
) -> Result<RuleDef, ParseError> {
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
    // a rule may contain a target.
    // a rule may contain a condition.
    // a rule may contain many advice declarations or references.
    // a rule may contain many obligation declarations or references.
    while let Some(tok) = skip_comments(&mut rule_pairs) {
        debug!("looking at rule {:?}: {:?}", tok.as_rule(), tok.as_str());
        if tok.as_rule() == Rule::effect_permit {
            if found_effect.is_none() {
                found_effect = Some(rule::Effect::Permit);
            } else {
                return Err(ParseError::DuplicateRuleEffect);
            }
        } else if tok.as_rule() == Rule::effect_deny {
            if found_effect.is_none() {
                found_effect = Some(rule::Effect::Deny);
            } else {
                return Err(ParseError::DuplicateRuleEffect);
            }
        } else if tok.as_rule() == Rule::target_stmt {
            if target.is_none() {
                target = Some(process_target(tok.into_inner(), ns.clone(), ctx)?);
            } else {
                return Err(ParseError::DuplicateTarget);
            }
        } else if tok.as_rule() == Rule::condition_stmt {
            if condition.is_none() {
                condition = Some(process_condition(tok.into_inner(), ns.clone(), ctx)?);
            } else {
                return Err(ParseError::DuplicateCondition);
            }
        } else if tok.as_rule() == Rule::on_effect {
            prescriptions.push(process_prescription(tok, ns.clone(), ctx)?);
        } else {
            let r = tok.as_rule();
            return Err(ParseError::UnexpectedRuleError(format!(
                "found unexpected rule {r:?}"
            )));
        }
    }

    if let Some(effect) = found_effect {
        info!("returning rule def");
        Ok(rule::RuleDef {
            id,
            ns,
            policy_ns,
            description,
            target,
            condition,
            prescriptions,
            effect,
            ctx: Rc::<Context>::downgrade(ctx),
        })
    } else {
        warn!("Error parsing rule");
        Err(ParseError::AstConvertError)
    }
}
// produces single target, which is a collection of disjunctiveseqs.
fn process_target(
    mut target_pairs: Pairs<Rule>,
    ns: Vec<String>,
    ctx: &Rc<Context>,
) -> Result<Target, ParseError> {
    // loop through the target_disjunctions
    // each of the clauses will be ANDed together.
    let mut clauses = vec![];
    while let Some(tok) = skip_comments(&mut target_pairs) {
        assert_eq!(tok.as_rule(), Rule::target_disjunction);
        clauses.push(process_target_clause(tok.into_inner())?);
    }
    Ok(Target {
        clauses,
        ns,
        ctx: Rc::<Context>::downgrade(ctx),
    })
}

/// Take a clause with or'd entries and produce a `DisjunctiveSeq`
fn process_target_clause(mut conj_pairs: Pairs<Rule>) -> Result<DisjunctiveSeq, ParseError> {
    let mut conj_seq = vec![];
    while let Some(tok) = skip_comments(&mut conj_pairs) {
        assert_eq!(tok.as_rule(), Rule::target_conjunction);
        let m = process_target_conjunctions(tok.into_inner())?;
        conj_seq.push(m);
    }
    Ok(DisjunctiveSeq {
        statements: conj_seq,
    })
}
/// Take a list of and'd matches and produce a `ConjuctiveSeq`
fn process_target_conjunctions(mut disj_pairs: Pairs<Rule>) -> Result<ConjunctiveSeq, ParseError> {
    let mut matches = vec![];
    while let Some(tok) = skip_comments(&mut disj_pairs) {
        let m = process_target_match(tok)?;
        matches.push(m);
    }
    Ok(ConjunctiveSeq { matches })
}

fn process_operator(op_ident: &Pair<Rule>) -> Result<Operator, ParseError> {
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
        })
    } else {
        Err(ParseError::AstConvertError)
    }
}

/// Take a child of rule `target_match` and produce a Match struct
fn process_target_match(match_pair: Pair<Rule>) -> Result<Match, ParseError> {
    let r = match_pair.as_rule();
    let mut match_pairs = match_pair.into_inner();
    // this creates one Match, either operator-based or a function call.
    if r == Rule::target_match_rev_op {
        let tok = skip_comments(&mut match_pairs).ok_or(ParseError::AstConvertError)?;
        let attr = process_attribute_designator(tok)?;
        let op_tok = skip_comments(&mut match_pairs).ok_or(ParseError::AstConvertError)?;
        let operator = process_operator(&op_tok)?;
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
        let operator = process_operator(&tok)?;

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
    Err(ParseError::AstConvertError)
}

fn process_condition(
    cond_pairs: Pairs<Rule>,
    ns: Vec<String>,
    ctx: &Rc<Context>,
) -> Result<Condition, ParseError> {
    info!("Parsing a condition");
    // first and only non-comment item in a cond_stmt is a cond_expression.
    let cond_expr = process_condition_expr(cond_pairs, &ns)?;
    let c = ConditionUnparsed {
        cond_expr,
        ns,
        ctx: Rc::<Context>::downgrade(ctx),
    };
    // Pratt-Parsed condition
    Condition::try_from(&c)
}

/// Process a condition expression from Pairs, skipping comments
fn process_condition_expr(
    mut cond_pairs: Pairs<Rule>,
    ns: &[String],
) -> Result<CondExpressionUnparsed, ParseError> {
    let cond_expr = skip_comments(&mut cond_pairs).ok_or(ParseError::AstConvertError)?;
    assert_eq!(cond_expr.as_rule(), Rule::cond_expr);
    process_condition_expr_pair(cond_expr, ns)
}

/// Process a condition expression Rule.
fn process_condition_expr_pair(
    cond_pair: Pair<Rule>,
    ns: &[String],
) -> Result<CondExpressionUnparsed, ParseError> {
    assert_eq!(cond_pair.as_rule(), Rule::cond_expr);
    let mut items: Vec<CondItemUnparsed> = vec![];
    let mut cond_expr = cond_pair.into_inner();
    while let Some(tok) = skip_comments(&mut cond_expr) {
        if tok.as_rule() == Rule::cond_atom {
            let c = process_condition_atom(tok.into_inner(), ns)?;
            items.push(CondItemUnparsed::Atom(c));
        } else if tok.as_rule() == Rule::operator_identifier {
            items.push(CondItemUnparsed::Op(process_operator(&tok)?));
        } else {
            return Err(ParseError::UnexpectedRuleError(format!(
                "Expected an atom or operator, found {:?}",
                tok.as_rule()
            )));
        }
    }
    Ok(CondExpressionUnparsed { items })
}

fn process_condition_atom(
    mut cond_atom: Pairs<Rule>,
    ns: &[String],
) -> Result<CondAtomUnparsed, ParseError> {
    info!("parsing condition atom: {cond_atom:?}");
    // Unparsed Cond Atom
    if let Some(tok) = skip_comments(&mut cond_atom) {
        let r = tok.as_rule();
        if r == Rule::cond_function_call {
            info!("atom > function call");
            let f = process_condition_function(tok.into_inner(), ns)?;
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
            //info!("atom > expression: {:?}", cond_atom.as_str());
            // we need to call it with the cond_atom
            let e = process_condition_expr_pair(tok, ns)?;
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
    mut cond_fn: Pairs<Rule>,
    ns: &[String],
) -> Result<CondFunctionCallUnparsed, ParseError> {
    info!("parsing cond fn: {:?}", cond_fn.as_str());
    //  first item will always be an elem_identifier.
    // second item will be a cond_argument_list;
    // inside of that, everything will be a CondExpr.
    let elem_ident = skip_comments(&mut cond_fn).ok_or(ParseError::AstConvertError)?;
    let identifier: Vec<String> = elem_ident.as_str().split('.').map(String::from).collect();

    let mut arguments = vec![];
    info!("elem_identifier:  {elem_ident:?}");
    if let Some(arglist) = skip_comments(&mut cond_fn) {
        assert_eq!(arglist.as_rule(), Rule::cond_argument_list);
        arguments = process_cond_argument_list(arglist.into_inner(), ns)?;
    }

    Ok(CondFunctionCallUnparsed {
        identifier,
        arguments,
    })
}

fn process_cond_argument_list(
    mut arg_pairs: Pairs<Rule>,
    ns: &[String],
) -> Result<Vec<CondExpressionUnparsed>, ParseError> {
    let mut arguments = vec![];
    while let Some(tok) = skip_comments(&mut arg_pairs) {
        if tok.as_rule() == Rule::cond_expr {
            let x = process_condition_expr_pair(tok, ns)?;
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
            // get string
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

/// Convert a `string_literal` rule into the (unquoted) string.  Works for either double or single-quoted strings.
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

fn process_import(mut import_pairs: Pairs<Rule>) -> Import {
    let mut components = vec![];
    let mut is_wildcard = false;
    while let Some(tok) = skip_comments(&mut import_pairs) {
        // read the import identifier
        if tok.as_rule() == Rule::import_identifier {
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
    }
}

/// Process a single instance of an "on <effect>" statement within a
/// rule/policy/policyset.
fn process_prescription(
    presc_pair: Pair<Rule>,
    ns: Vec<String>,
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
    // populate this prescription:
    Ok(Prescription {
        effect,
        ns,
        expressions,
        ctx: Rc::<Context>::downgrade(ctx),
    })
}

fn process_typedef(mut typedef_pairs: Pairs<Rule>, ns: Vec<String>) -> Result<TypeDef, ParseError> {
    let identifier = skip_comments(&mut typedef_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str();
    let id = identifier.to_string();
    let uri = skip_comments(&mut typedef_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"')
        .to_string();
    Ok(TypeDef { id, uri, ns })
}

fn process_category(
    mut category_pairs: Pairs<Rule>,
    ns: Vec<String>,
) -> Result<Category, ParseError> {
    let identifier = skip_comments(&mut category_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str();
    let id = identifier.to_string();
    let uri = skip_comments(&mut category_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"')
        .to_string();
    Ok(Category { id, uri, ns })
}

fn process_infix(mut infix_pairs: Pairs<Rule>, ns: Vec<String>) -> Result<Infix, ParseError> {
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
            let sig = process_infix_signature(t.into_inner())?;
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
            ns,
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
fn process_infix_signature(mut sig_pairs: Pairs<Rule>) -> Result<InfixSignature, ParseError> {
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
fn process_advice(
    mut typedef_pairs: Pairs<Rule>,
    ns: Vec<String>,
) -> Result<AdviceDef, ParseError> {
    let identifier = skip_comments(&mut typedef_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str();
    let id = identifier.to_string();
    let uri = skip_comments(&mut typedef_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"')
        .to_string();
    Ok(AdviceDef { id, uri, ns })
}

/// Process an obligation declaration
fn process_obligation(
    mut typedef_pairs: Pairs<Rule>,
    ns: Vec<String>,
) -> Result<ObligationDef, ParseError> {
    let identifier = skip_comments(&mut typedef_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str();
    let id = identifier.to_string();
    let uri = skip_comments(&mut typedef_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"')
        .to_string();
    Ok(ObligationDef { id, uri, ns })
}

fn process_attribute(
    mut attr_pairs: Pairs<Rule>,
    ns: Vec<String>,
) -> Result<Attribute, ParseError> {
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
    // we are needing three values,
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
            ns,
        })
    } else {
        // this should not be reachable
        error!("attribute declaration did not include all required values");
        Err(ParseError::AstConvertError)
    }
}

fn process_rulecombinator(
    mut rc_pairs: Pairs<Rule>,
    ns_path: Vec<String>,
) -> Result<RuleCombinator, ParseError> {
    // skip comments
    let identifier = skip_comments(&mut rc_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str();
    let id = identifier.to_string();
    let uri = skip_comments(&mut rc_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"');
    let rc = RuleCombinator {
        id,
        uri: uri.to_string(),
        ns: ns_path,
    };
    Ok(rc)
}

fn process_policycombinator(
    mut pc_pairs: Pairs<Rule>,
    ns_path: Vec<String>,
) -> Result<PolicyCombinator, ParseError> {
    // skip comments
    let identifier = skip_comments(&mut pc_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str();
    let id = identifier.to_string();
    let uri = skip_comments(&mut pc_pairs)
        .ok_or(ParseError::AstConvertError)?
        .as_str()
        .trim_matches('"');
    let pc = PolicyCombinator {
        id,
        uri: uri.to_string(),
        ns: ns_path,
    };
    Ok(pc)
}

fn process_policyset(
    policyset_pair: Pair<Rule>,
    ns_path: Vec<String>,
    mut parent_policy_path: GenName,
    description: Option<String>,
    register: bool,
    ctx: Rc<Context>,
) -> Result<PolicySet, ParseError> {
    info!("found a policyset");
    assert_eq!(policyset_pair.as_rule(), Rule::policyset_decl);
    let mut policyset_pairs = policyset_pair.into_inner();
    let policy_id_rule = skip_comments(&mut policyset_pairs).ok_or(ParseError::AstConvertError)?;
    let policy_id = policy_naming(policy_id_rule, ctx.clone())?;
    // an apply statement is required.
    let mut apply = None;
    // a target is optional
    let mut target: Option<Target> = None;
    // a condition is optional
    let mut condition: Option<Condition> = None;
    // policies and policysets
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
                target = Some(process_target(stmt.into_inner(), ns_path.clone(), &ctx)?);
                info!("target: {target:?}");
            } else if stmt.as_rule() == Rule::condition_stmt {
                if condition.is_none() {
                    condition = Some(process_condition(stmt.into_inner(), ns_path.clone(), &ctx)?);
                } else {
                    return Err(ParseError::DuplicateCondition);
                }
            } else if stmt.as_rule() == Rule::policy_decl {
                // the namespace of a child policy needs to incorporate the name of the parent into the namespace.
                //let mut p_ns = ns_path.clone();
                // TODO: this is wrong.
                //if let Some(pname) = policy_id.get_name() {
                //    p_ns.push(pname);
                //} else {
                //    todo!("define a name for an anonymous parent policy");
                // if the parent was anonymous, nobody else can refer to this child policy.
                // but it will still need to exist in the list of policies.  It is probably fine to generate a random parent ID.
                //}
                let p = process_policy(
                    stmt.into_inner(),
                    ns_path.clone(),
                    parent_policy_path.clone(),
                    last_comment.clone(),
                    do_register,
                    ctx.clone(),
                )?;
                policies.push(PolicyEntry::Policy(p));
            } else if stmt.as_rule() == Rule::policyset_decl {
                info!("PS creation");
                let p = process_policyset(
                    stmt,
                    ns_path.clone(),
                    parent_policy_path.clone(),
                    last_comment.clone(),
                    do_register,
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
                prescriptions.push(process_prescription(stmt, ns_path.clone(), &ctx)?);
            } else {
                todo!("handle {:?}", stmt.as_rule());
            }
        }
    }
    info!("policyset id: {policy_id:?}");
    Ok(PolicySet {
        id: policy_id,
        ns: ns_path,
        policy_ns: parent_policy_path,
        description,
        apply: apply.ok_or(ParseError::MissingApplyStatement)?,
        target,
        condition,
        policies,
        prescriptions,
        ctx,
    })
}

fn process_policy(
    mut policy_pairs: Pairs<Rule>,
    ns_path: Vec<String>,
    mut parent_policy_path: GenName,
    description: Option<String>,
    register: bool,
    ctx: Rc<Context>,
) -> Result<Policy, ParseError> {
    let policy_id_rule = skip_comments(&mut policy_pairs).ok_or(ParseError::AstConvertError)?;
    // an apply statement is required.
    let mut apply = None;
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

    // if this policy is not named, generate one based on the current namespace.
    //    while policy_id == PolicyId::PolicyNoName {
    // generate a name.
    //        let new_name = format!("policy_{}", ctx.get_next_policy_id(&ns_path.join(".")));
    //        info!("we will name this policy: {new_name}");
    //        policy_id = PolicyId::PolicyName(new_name);
    // check if this name is already used.
    // We can't actually finalize this name until we see everything in the parent.
    // we could have a subsequent policy in this namespace that uses the name that we just picked!

    // this means we:
    // don't know what to register this as, until after it is made.
    // don't know what to name the child rules.
    //
    //On the plus side, with anything that has a generated name, that name is really only used for
    // the output, there is actually no reason to "register" these because they /can't be looked up/.

    // We could...
    // leave the name as NoName.
    // hint to the child items that they should not be registered.

    // but we can't hint to child items what their name could be
    // until after we know all the items at least at this
    // namespaces.

    // So there are two fundamental things we need to do.
    // understand if registering an element is possible.  That can be a "register" flag.

    // Consider delaying the use of "get_next_policy_id" to the generation phase.

    // The problem is how to define what the namespace of a item is that is nested under an anonymous item.  The register flag will contaminate anything below it, so even a named Rule will be register=false if any parentp policy/policyset is anonymous.

    // then, when we go to generate, we can pass in/transform the IDs.
    //  }

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
        //let mut inner_stmt = stmt.into_inner();
        //debug!("inner_stmt: {inner_stmt:?}");
        if policy_stmt.as_rule() == Rule::policy_stmt {
            let mut t = policy_stmt.into_inner();
            let stmt = skip_comments(&mut t).ok_or(ParseError::AstConvertError)?;
            // Apply statement
            if stmt.as_rule() == Rule::apply_stmt {
                // get inner
                let mut apply_stmt = stmt.into_inner();
                let apply_ident =
                    skip_comments(&mut apply_stmt).ok_or(ParseError::AstConvertError)?;
                apply = Some(apply_ident.as_str().to_string());
            } else if stmt.as_rule() == Rule::target_stmt {
                // target
                if target.is_none() {
                    target = Some(process_target(stmt.into_inner(), ns_path.clone(), &ctx)?);
                } else {
                    return Err(ParseError::DuplicateCondition);
                }
            } else if stmt.as_rule() == Rule::condition_stmt {
                if condition.is_none() {
                    condition = Some(process_condition(stmt.into_inner(), ns_path.clone(), &ctx)?);
                } else {
                    return Err(ParseError::DuplicateCondition);
                }
            } else if stmt.as_rule() == Rule::rule_decl {
                // a rule is being defined.  since the rule is
                // underneath a policy, we need to append the policy
                // name as part of the rule namespace.
                // the policy MUST have a name though, even a generated one.

                // but we don't have a Policy object yet, which means we don't have something to call gen_id on.

                // this will be a problem for the parent policy as well.

                // instead of generating the name inside the policy, we should use the context to pull out a name.

                // this was obviously put in to make the rule ID pretty, but it messes up actual namespace resolution.
                // TODO: add the policy GenName stuff into the rule, to help create a qualified name/ID.

                debug!("found rule declaration {stmt:?}");
                //let mut ns_rule_path = ns_path.clone();
                //if let Some(n) = policy_id.get_name() {
                //    if do_register {
                //        ns_rule_path.push(n);
                //        info!("extending name of rule namespace to: {ns_rule_path:?}");
                //    }
                //}

                // this used to be the ns_rule_path, but now it is just the ns_path
                let rule_decl = process_rule(
                    stmt.into_inner(),
                    ns_path.clone(),
                    parent_policy_path.clone(),
                    last_comment.clone(),
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
                let rule_ref = RuleReference {
                    id: rule_id,
                    ns: rule_ns,
                };
                rules.push(RuleEntry::Ref(rule_ref));
                info!("finished pushing ruleentry ref");
            } else if stmt.as_rule() == Rule::on_effect {
                info!("adding prescription to policy");
                prescriptions.push(process_prescription(stmt, ns_path.clone(), &ctx)?);
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
        ns: ns_path,
        policy_ns: parent_policy_path,
        description,
        apply: apply.ok_or(ParseError::MissingApplyStatement)?,
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
    /// Filename that contained the source Alfa.
    pub filename: PathBuf,
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
    // TODO: it would be better to just return all distinct serializable elements.

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
