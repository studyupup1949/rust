//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use crate::{ast::SrcLoc, Rule};
use miette::{Diagnostic, LabeledSpan, NamedSource};
use std::{
    fmt::{self},
    sync::Arc,
};
use thiserror::Error;

/// Error thrown when parsing fails.
#[derive(Debug, Clone, Eq, PartialEq, Error, Diagnostic)]
pub enum ParseError {
    #[error("Parse error: {}", _0)]
    PestParseError(Box<pest::error::Error<Rule>>),
    #[error("Conversion into AST error")]
    AstConvertError,
    #[error("Rule not expected at this location: {}", _0)]
    UnexpectedRuleError(String),
    #[error("Duplicate symbol: {}", _0)]
    DuplicateSymbol(String),
    #[error("Could not resolve symbol: {}, from namespace: {}", _0, _1)]
    SymbolNotFound(String, String),
    #[error("Ambiguous Import")]
    AmbiguousImport(String),
    #[error("Duplicate definition of infix function modifier")]
    DuplicateInfixModifier,
    #[error("Duplicate definition of an effect in a rule")]
    DuplicateRuleEffect,
    #[error("Duplicate definition of a target")]
    DuplicateTarget,
    #[error("Duplicate definition of a condition")]
    DuplicateCondition,
    #[error("Commutative infix functions cannot have inverses")]
    CommutativeWithInverseError,
    #[error("Attempt to use non-commutative infix function with arguments reversed")]
    ReversedInfixArgsNotCommutativeError,
    #[error("Could not find inverse infix definition")]
    InverseInfixNotFound,
    #[error("Infix operator does not allow bags")]
    InfixBagsDisallowed,
    #[error("Infix operators on bags must return booleans")]
    InfixBagsBooleanRequired,
    #[error("Infix operators had no matching signature for argument types")]
    InfixNoMatchingSignature,
    #[error("Context no longer available")]
    ContextMissing,
    #[error("Could not write to XACML file")]
    XacmlWriteIoError,
    #[error("Could not determine policy filename")]
    XacmlMissingFilename,
    #[error("Duplicate URI for Policy/PolicySet: {}", _0)]
    DuplicateURI(String),
    #[error("PolicySet has no condition, cannot convert to condition structure")]
    PolicySetNoCondition,
    #[error("Policy has no condition, cannot convert to XACML PolicySet")]
    PolicyNoCondition,
    #[error("Policy has condition, cannot convert to XACML Policy")]
    PolicyHasCondition,
    #[error("A PolicySet and Policy have the same name in the same policy: {}", _0)]
    DuplicatePolicyEntity(String),
    #[error(transparent)]
    #[diagnostic(transparent)]
    SrcError(#[from] SrcError),
}

#[derive(Error, Debug, Eq, PartialEq, Clone)]
#[allow(unused_assignments)]
pub struct SrcError {
    //#[label("error here")]
    //at: SourceSpan,
    //    #[label]
    labels: Vec<LabeledSpan>,
    msg: String,
    //    #[source_code]
    src: Arc<NamedSource<String>>,
}

impl Diagnostic for SrcError {
    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        Some(Box::new(self.labels.iter().cloned()))
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&*self.src)
    }
}

impl fmt::Display for SrcError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl SrcError {
    pub fn new(msg: &str, label: &str, src_loc: SrcLoc) -> ParseError {
        ParseError::SrcError(SrcError {
            src: src_loc.get_src(),
            labels: vec![LabeledSpan::new_with_span(
                Some(label.to_owned()),
                src_loc.get_span(),
            )],
            msg: msg.to_owned(),
        })
    }
}

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(err: pest::error::Error<Rule>) -> ParseError {
        ParseError::PestParseError(Box::new(err))
    }
}

impl From<Box<pest::error::Error<Rule>>> for ParseError {
    fn from(err: Box<pest::error::Error<Rule>>) -> ParseError {
        ParseError::PestParseError(err)
    }
}
