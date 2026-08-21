//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use crate::Rule;
use thiserror::Error;

/// Error thrown when parsing fails.
#[derive(Debug, Clone, Eq, PartialEq, Error)]
pub enum ParseError {
    #[error("Pest parsing error: {}", _0)]
    PestParseError(Box<pest::error::Error<Rule>>),
    #[error("Conversion into AST error")]
    AstConvertError,
    #[error("Rule not expected at this location: {}", _0)]
    UnexpectedRuleError(String),
    #[error("Duplicate symbol: {}", _0)]
    DuplicateSymbol(String),
    #[error("Symbol not found during resolution")]
    SymbolNotFound(String),
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
    #[error("Conditions must return booleans")]
    ConditionBooleanRequired,
    #[error("Infix operators had no matching signature for argument types")]
    InfixNoMatchingSignature,
    #[error("Context no longer available")]
    ContextMissing,
    #[error("Missing apply statement")]
    MissingApplyStatement,
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
