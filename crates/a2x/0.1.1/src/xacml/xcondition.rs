//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML Conditions

use super::xapply::XApply;
use super::xattr_designator::XAttrDesignator;
use super::xexpression::XExpression;
use super::xfunction::XFunction;
use super::XAttrValue;
use crate::ast::Spanned;
use crate::ast::condition::{CondExpression, CondFunctionCall, Condition, FunctionReference};
use crate::ast::constant::Constant;
use crate::ast::designator::AttributeDesignator;
use crate::ast::function::FunctionOutputArg;
use crate::ast::infix::Infix;
use crate::ast::infix::InfixSignature;
use crate::ast::operator::Operator;
use crate::context::Context;
use crate::errors::{ParseError, SrcError};
use log::debug;
use log::error;
use log::info;
use std::io::Write;
use xml::writer::EventWriter;
use xml::writer::XmlEvent;

/// A `<Condition>` element that contains an expression.
#[derive(Debug, PartialEq, Clone)]
pub struct XCondition {
    pub expr: XExpression,
}

/// Conversion of Alfa Rule to XACML Rule
impl TryFrom<&Condition> for XCondition {
    type Error = ParseError;
    fn try_from(c: &Condition) -> Result<Self, Self::Error> {
        let ctx = c.ctx.upgrade().ok_or(ParseError::ContextMissing)?;
        // Ensure the top-level type is an atomic boolean.
        // if the expression has a symbol error (say, function not found), we just get None instead of the error.
        let t = type_for_expr(&c.cond_expr, &c.ns, &ctx).ok();
        if let Some(FunctionTypeResolved::Atomic(n)) = t {
            // ensure boolean
            if n.uri != crate::ast::typedef::BOOLEAN_URI {
                info!("The condition expression does not resolve to a boolean type");
                info!("Observed type, atomic: {:?}", &n);
                return Err(SrcError::new(
                    "Conditions must evaluate to booleans",
                    &format!("type is {:?}", n.uri),
                    c.span().clone(),
                ));
            }
        } else {
            // If the type is not atomic, this is an error.
            info!("The condition expression does not resolve to an atomic type");
            return Err(SrcError::new(
                "Conditions must evaluate to atomic booleans",
                "non-atomic type",
                c.span().clone(),
            ));
        }
        // The ALFA spec seems to imply literals are not allowed, but
        // the resulting XACML would still be valid, so we don't care.
        Ok(XCondition {
            expr: expr_to_xexpr(&c.cond_expr, &c.ns, &ctx)?,
        })
    }
}

/// Determine the fully-resolved type name for an expression.
fn type_for_expr(
    e: &CondExpression,
    source_ns: &[String],
    ctx: &Context,
) -> Result<FunctionTypeResolved, ParseError> {
    match e {
        CondExpression::Infix(e1, o, e2) => {
            sig_and_type_for_infix(e1, o, e2, source_ns, ctx).map(|(_sig, typeres)| typeres)
        }
        CondExpression::Lit(c) => Ok(FunctionTypeResolved::Atomic(
            resolve_literal_types(c, source_ns, ctx).ok_or(ParseError::AstConvertError)?,
        )),
        CondExpression::Attr(ad) => {
            // lookup the attribute
            let attr = ctx.lookup_attribute(&ad.fully_qualified_name(), source_ns)?;
            info!("found attribute: {attr:?}");
            // lookup the type definition
            let typedef = ctx.lookup_type(&attr.typedef, &[attr.ns.join(".")])?;
            // Attributes are always bags of atomics
            Ok(FunctionTypeResolved::AtomicBag(ResolvedAtomicName {
                uri: typedef.uri.clone(),
            }))
        }
        CondExpression::Fn(fn_call) => {
            info!("request was for {fn_call:?}");
            // simply check the return type of the function.
            // TODO: check (somewhere) that the arguments are compatible.
            let func = ctx.lookup_function(&fn_call.fully_qualified_name(), source_ns)?;
            info!("finished lookup");
            match &func.output_arg {
                FunctionOutputArg::Atomic(a) => {
                    // a is the alfa name of  the output type, which we need to convert to a URI
                    // the symbol is whatever came from the function definition.
                    // the source namespace is the function's namespace.
                    let typedef = ctx.lookup_type(a, &func.ns)?;
                    Ok(FunctionTypeResolved::Atomic(ResolvedAtomicName {
                        uri: typedef.uri.clone(),
                    }))
                }
                // TODO: remaining types
                // This would be helpful for troubleshooting.
                x => {
                    info!("type was {:?}", x);
                    // TODO: produce a type name here, this isn't an error.
                    Err(ParseError::AstConvertError)
                }
            }
        }
        CondExpression::FnRef(fn_ref) => {
            // A function ref is simply a function type, which isn't
            // allowed as the output type of an expression, so we will
            // always return None.
            info!("type resolution requested for {fn_ref:?}");
            Err(ParseError::AstConvertError)
        }
        CondExpression::Empty => {
            info!("Expression was empty, has no type");
            Err(ParseError::AstConvertError)
        }
    }
}

/// Determine the function URI and return type and for an invocation of an infix function.
fn sig_and_type_for_infix(
    arg1: &CondExpression,
    op: &Operator,
    arg2: &CondExpression,
    source_ns: &[String],
    ctx: &Context,
) -> Result<(InfixSignature, FunctionTypeResolved), ParseError> {
    // an operation needs to be converted into a function ID.
    info!("operation is: {op}");
    // lookup operation, using inverse as a fallback.
    // there is no point in looking up the inverse, because the user should be using the correct function.
    let infix = ctx.lookup_infix(&op.qualified_name(), source_ns)?;
    info!("found operation:  {infix:?}");
    // lookup types of the arguments.  This currently only works for constants, but we will expand this.
    // get an expression for the first arg.
    // Determine type of first and second arguments
    let first_arg_type = type_for_expr(arg1, source_ns, ctx)?;
    let second_arg_type = type_for_expr(arg2, source_ns, ctx)?;
    // Lookup a signature for our infix operator which is compatible.
    let mut op_fn_sig = None;
    for s in &infix.signatures {
        // lookup the fully resolved type for each argument.
        info!("checking signature: {s:?}");
        let first_type_uri = &ctx.lookup_type(&s.first_arg, &infix.ns)?.uri;
        let second_type_uri = &ctx.lookup_type(&s.second_arg, &infix.ns)?.uri;
        let output_type_uri = &ctx.lookup_type(&s.output, &infix.ns)?.uri;
        info!("1st:  {first_type_uri:?}");
        info!("2nd:  {second_type_uri:?}");
        info!("output:  {output_type_uri:?}");
        let compat = check_infix_sign_compat(
            &infix,
            &first_arg_type,
            &second_arg_type,
            first_type_uri,
            second_type_uri,
            s,
        );
        if compat {
            info!("found compatible function signature");
            op_fn_sig = Some(s);
            break;
        }
        info!("compatibility: {compat:?}");
    }
    if let Some(sig) = op_fn_sig {
        // are either of the arguments bags?  If not, then both
        let is_arg_1_bag = first_arg_type.is_bag();
        let is_arg_2_bag = second_arg_type.is_bag();
        info!("left arg bag? {is_arg_1_bag:?}; right arg bag? {is_arg_2_bag:?}");
        // determine output type
        let output_type = &ctx.lookup_type(&sig.output, &infix.ns)?;
        info!(
            "setting output type: {:?}, but should be using {:?}",
            sig.output, output_type
        );
        let output_type = FunctionTypeResolved::Atomic(ResolvedAtomicName {
            uri: output_type.uri.clone(),
        });
        // we now know the output type, and the infix signature.
        return Ok((sig.clone(), output_type));
    }
    // TODO: custom error
    Err(ParseError::AstConvertError)
}

/// Convert literal constants into a XACML expression.
fn handle_literal(
    c: &Constant,
    source_ns: &[String],
    ctx: &Context,
) -> Result<XExpression, ParseError> {
    Ok(XExpression::Value(XAttrValue {
        v: ctx.constant_to_typedliteral(c.clone(), source_ns)?,
    }))
}

/// Convert function references into a XACML expression.
///
/// These will be used as arguments to function application.
fn handle_function_reference(
    fr: &FunctionReference,
    source_ns: &[String],
    ctx: &Context,
) -> Result<XExpression, ParseError> {
    let fn_name = ctx.lookup_function(&fr.fully_qualified_name(), source_ns)?;
    Ok(XExpression::Function(XFunction {
        function_uri: fn_name.function_uri.clone(),
    }))
}

/// Convert function application into a XACML expression.
fn handle_function_call(
    fc: &CondFunctionCall,
    source_ns: &[String],
    ctx: &Context,
) -> Result<XExpression, ParseError> {
    let func = ctx.lookup_function(&fc.fully_qualified_name(), source_ns)?;
    // Convert each argument to an expression
    let mut args = vec![];
    for a in &fc.arguments {
        let e = expr_to_xexpr(a, source_ns, ctx)?;
        args.push(e);
    }
    // TODO: check that the arguments match the function definition.
    Ok(XExpression::Apply(XApply {
        function_uri: func.function_uri.clone(),
        arguments: args,
        return_type: fn_output_to_resolved(&func.output_arg, &func.ns, ctx)?,
    }))
}

/// Convert a named attribute into a XACML expression.
fn handle_attribute_designator(
    a: &AttributeDesignator,
    source_ns: &[String],
    ctx: &Context,
) -> Result<XExpression, ParseError> {
    info!("attr designator: {a:?}");
    let attr = ctx.lookup_attribute(&a.fully_qualified_name(), source_ns)?;
    let uri = attr.uri.clone();
    let type_uri = ctx.lookup_type(&attr.typedef, &attr.ns)?.uri.clone();
    let category = ctx.lookup_category(&attr.category, &attr.ns)?.uri.clone();
    Ok(XExpression::Attrib(XAttrDesignator {
        uri,
        category,
        type_uri,
        must_be_present: a.mustbepresent,
        issuer: a.issuer.clone(),
    }))
}

/// Convert an infix operator into a XACML expression.
fn handle_infix_expression(
    ce1: &CondExpression,
    o: &Operator,
    ce2: &CondExpression,
    source_ns: &[String],
    ctx: &Context,
) -> Result<XExpression, ParseError> {
    let infix = ctx.lookup_infix(&o.qualified_name(), source_ns)?;
    // Determine type of first and second arguments
    let first_arg_type = type_for_expr(ce1, source_ns, ctx)?;
    let second_arg_type = type_for_expr(ce2, source_ns, ctx)?;
    // Find compatible signature
    let sig = resolve_infix_signature(&infix, &first_arg_type, &second_arg_type, ctx)?;
    // Convert arguments
    let args = vec![
        expr_to_xexpr(ce1, source_ns, ctx)?,
        expr_to_xexpr(ce2, source_ns, ctx)?,
    ];
    // Create the appropriate Apply expression based on bag handling.
    // This either results in simple function application, or
    // generates a new function application with anyOfAny if either
    // arg is a bag.
    create_infix_apply(&sig, args, &first_arg_type, &second_arg_type, &infix, ctx)
}

/// Resolve a compatible infix signature for the given argument types.
///
/// Based on the function argument/output types, find the specific
/// XACML function URI which should be used in place of the infix
/// operator.
fn resolve_infix_signature(
    infix: &Infix,
    first_arg_type: &FunctionTypeResolved,
    second_arg_type: &FunctionTypeResolved,
    ctx: &Context,
) -> Result<InfixSignature, ParseError> {
    for s in &infix.signatures {
        debug!("checking signature: {s:?}");
        let first_type_uri = &ctx.lookup_type(&s.first_arg, &infix.ns)?.uri;
        let second_type_uri = &ctx.lookup_type(&s.second_arg, &infix.ns)?.uri;
        let output_type_uri = &ctx.lookup_type(&s.output, &infix.ns)?.uri;
        debug!("1st:  {first_type_uri:?}");
        debug!("2nd:  {second_type_uri:?}");
        debug!("output:  {output_type_uri:?}");
        // determine if this particular signature is compatible
        let compat = check_infix_sign_compat(
            infix,
            first_arg_type,
            second_arg_type,
            first_type_uri,
            second_type_uri,
            s,
        );
        if compat {
            info!("found compatible function signature");
            return Ok(s.clone());
        }
        info!("compatibility: {compat:?}");
    }
    Err(ParseError::InfixNoMatchingSignature)
}

/// Create an Apply expression for an infix operation, handling bags
/// appropriately.
///
/// Infix operators can apply the `allow_bags` transformation if
/// either argument is a bag.
fn create_infix_apply(
    sig: &InfixSignature,
    args: Vec<XExpression>,
    first_arg_type: &FunctionTypeResolved,
    second_arg_type: &FunctionTypeResolved,
    infix: &Infix,
    ctx: &Context,
) -> Result<XExpression, ParseError> {
    let is_arg_1_bag = first_arg_type.is_bag();
    let is_arg_2_bag = second_arg_type.is_bag();
    // Determine output type
    let output_typedef = &ctx.lookup_type(&sig.output, &infix.ns)?;
    let output_typeres = FunctionTypeResolved::Atomic(ResolvedAtomicName {
        uri: output_typedef.uri.clone(),
    });
    // If neither argument is a bag, apply function directly
    if !is_arg_1_bag && !is_arg_2_bag {
        return Ok(XExpression::Apply(XApply {
            function_uri: sig.uri.clone(),
            arguments: args,
            return_type: output_typeres,
        }));
    }
    // Handle bag arguments
    if infix.allow_bags {
        if output_typedef.uri == crate::ast::typedef::BOOLEAN_URI {
            // Use any-of-any with function as first argument
            let function_arg = XExpression::Function(XFunction {
                function_uri: sig.uri.clone(),
            });
            let mut bag_args = vec![function_arg];
            bag_args.extend_from_slice(&args);

            Ok(XExpression::Apply(XApply {
                function_uri: "urn:oasis:names:tc:xacml:3.0:function:any-of-any".to_owned(),
                arguments: bag_args,
                return_type: output_typeres,
            }))
        } else {
            info!("sigs output was {:?}, expected a boolean", sig.output);
            Err(ParseError::InfixBagsBooleanRequired)
        }
    } else {
        Err(ParseError::InfixBagsDisallowed)
    }
}

/// Convert an AST Expression to a XACML Expression.
fn expr_to_xexpr(
    e: &CondExpression,
    source_ns: &[String],
    ctx: &Context,
) -> Result<XExpression, ParseError> {
    match e {
        CondExpression::Infix(ce1, o, ce2) => handle_infix_expression(ce1, o, ce2, source_ns, ctx),
        CondExpression::Lit(c) => handle_literal(c, source_ns, ctx),
        CondExpression::FnRef(fr) => handle_function_reference(fr, source_ns, ctx),
        CondExpression::Fn(fc) => handle_function_call(fc, source_ns, ctx),
        CondExpression::Attr(a) => handle_attribute_designator(a, source_ns, ctx),
        CondExpression::Empty => Err(ParseError::AstConvertError),
    }
}

impl XCondition {
    /// Write an XML (XACML) representation of an `XCondition` to a
    /// stream.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the writer fails.
    pub fn write_xml<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), xml::writer::Error> {
        // Top-level Condition Element
        writer.write(XmlEvent::start_element("xacml3:Condition"))?;
        // Expression Element
        self.expr.write_xml(writer)?;
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

/// A type URI that has been resolved from an ALFA name.
#[derive(Debug, PartialEq, Clone)]
pub struct ResolvedAtomicName {
    uri: String, // XACML type URI
}

/// A function argument type that has been resolved to a URI and
/// (optionally) an ALFA fully qualified name.
///
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionTypeResolved {
    /// A reference to a specific atomic type (`string`, `inetAddress`, etc.)
    Atomic(ResolvedAtomicName),
    /// A reference to a bag of a specific atomic type (`bag[bool]`)
    AtomicBag(ResolvedAtomicName),
    /// A reference to a bag of any atomic type (`bag[anyAtomic]`)
    AnyAtomicBag,
    /// A placeholder for any atomic type (`anyAtomic`)
    AnyAtomic,
}

impl FunctionTypeResolved {
    /// Check if the type is a bag
    #[must_use]
    pub fn is_bag(&self) -> bool {
        match self {
            FunctionTypeResolved::Atomic(_) | FunctionTypeResolved::AnyAtomic => false,
            FunctionTypeResolved::AtomicBag(_) | FunctionTypeResolved::AnyAtomicBag => true,
        }
    }
    #[must_use]
    pub fn is_atomic(&self) -> bool {
        !self.is_bag()
    }
}

/// Resolve an ALFA function output type into the XACML type URI.
///
/// ALFA functions are defined with outputs that can include ALFA type
/// definitions, for example: `function foo = "urn:..." : string
/// string -> boolean` has an output type of `Atomic("boolean")`.
/// This function will resolve that to the XACML URI for a boolean.
///
/// # Arguments
/// * `fn_out` - The output argument type for a function
/// * `source_ns` - The namespace where the function was defined
/// * `ctx` - The parsing context
///
/// # Returns
/// * `Ok(FunctionTypeResolved)` - The function's return type resolved
///    to a XACML URI
/// * `Err(ParseError)` - Any errors resolving the attribute or its type
///
fn fn_output_to_resolved(
    fn_out: &FunctionOutputArg,
    source_ns: &[String],
    ctx: &Context,
) -> Result<FunctionTypeResolved, ParseError> {
    // Note: look up the type from the function's namespace.

    Ok(match fn_out {
        FunctionOutputArg::Atomic(a) => {
            info!("checking for atomic: {a:?}");
            let t = ctx.lookup_type(a, source_ns)?;
            FunctionTypeResolved::Atomic(ResolvedAtomicName { uri: t.uri.clone() })
        }
        FunctionOutputArg::AtomicBag(a) => {
            info!("checking for bag of {a:?}");
            let t = ctx.lookup_type(a, source_ns)?;
            FunctionTypeResolved::AtomicBag(ResolvedAtomicName { uri: t.uri.clone() })
        }
        FunctionOutputArg::AnyAtomicBag => FunctionTypeResolved::AnyAtomicBag,
        FunctionOutputArg::AnyAtomic => FunctionTypeResolved::AnyAtomic,
    })
}

/// Given a constant that appears in some namespace, fully resolve the name.
fn resolve_literal_types(
    c: &Constant,
    source_ns: &[String],
    ctx: &Context,
) -> Option<ResolvedAtomicName> {
    match c {
        Constant::String(_) => Some(ResolvedAtomicName {
            uri: crate::ast::typedef::STRING_URI.to_owned(),
        }),
        Constant::Integer(_) => Some(ResolvedAtomicName {
            uri: crate::ast::typedef::INTEGER_URI.to_owned(),
        }),
        Constant::Double(_) => Some(ResolvedAtomicName {
            uri: crate::ast::typedef::DOUBLE_URI.to_owned(),
        }),
        Constant::Boolean(_) => Some(ResolvedAtomicName {
            uri: crate::ast::typedef::BOOLEAN_URI.to_owned(),
        }),
        Constant::Custom(ct, _) => {
            // lookup the type
            let t = ctx.lookup_type(&ct.name, source_ns).ok()?;
            // construct the fully-qualified name
            info!("type is {t:?}");
            todo!("resolve custom types");
            //None
        }
        Constant::Undefined => None,
    }
}

/// Given an infix operator, arguments, and a proposed signature,
/// determine if the signature matches.
fn check_infix_sign_compat(
    infix: &Infix,
    first_arg_type: &FunctionTypeResolved,
    second_arg_type: &FunctionTypeResolved,
    sig_first_type: &str,
    sig_second_type: &str,
    signature: &InfixSignature,
) -> bool {
    debug!("performing a compatibility check for {:?}", infix.operator);
    debug!("types for arguments to the function were {first_arg_type:?} and {second_arg_type:?}");
    debug!("we are checking signature {signature:?}");
    // if the first argument is a bag, and the second a scalar, AND the function is allowbags, then this works.
    // Check first argument.
    match first_arg_type {
        FunctionTypeResolved::Atomic(n) => {
            info!("atomic: {:?}", n.uri);
            if n.uri != sig_first_type {
                info!("function type:  (SHOULD BE URI) {:?}", n.uri);
                info!("sig first type: {sig_first_type:?}");
                info!("function type and first type sig do not match");
                return false;
            }
        }
        FunctionTypeResolved::AtomicBag(b) => {
            info!("atomic bag: {:?}", b.uri);
        }
        _ => {
            error!(
                "It should not be possible to resolve an infix argument into a non-specific type (AnyAtomicBag/AnyAtomic)."
            );
            panic!("AnyAtomicBag/AnyAtomic are not valid arguments to an infix operator.");
        }
    }
    // Check second argument.
    if let FunctionTypeResolved::Atomic(n) = second_arg_type {
        info!("atomic: {:?}", n.uri);
        if n.uri != sig_second_type {
            return false;
        }
    }
    true
}
