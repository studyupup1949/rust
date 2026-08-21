//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! XACML Targets

use crate::ast::target::Match;
use crate::ast::target::Target;
use crate::context::Context;
use crate::errors::ParseError;
use log::debug;
use log::error;
use log::info;
use std::io::Write;
use xml::writer::EventWriter;
use xml::writer::XmlEvent;

/// `<Target>` elements within an [`XRule`], [`XPolicy`], or [`XPolicySet`].
#[derive(Debug, PartialEq, Default)]
pub struct XTarget {
    pub anyofs: Vec<AnyOf>,
}

/// `<AnyOf>` elements within an [`XTarget`].
// AnyOf elements contain AllOf
#[derive(Debug, PartialEq, Default)]
pub struct AnyOf {
    pub allofs: Vec<AllOf>,
}

/// `<AllOf>` elements within an [`AnyOf`].
#[derive(Debug, PartialEq, Default)]
pub struct AllOf {
    pub matches: Vec<XMatch>,
}

/// `<Match>` elements contained within an [`AllOf`].
#[derive(Debug, PartialEq, Default)]
pub struct XMatch {
    // A match statement requires:
    // match function ID, value, value datatype, designator ID, designator category ID, designator type, and must-be-present.x
    pub matchid: String, // URL of match function
    pub value: String,
    pub value_type: String,
    pub designator_id: String,
    pub designator_category: String,
    pub designator_type: String,
    pub must_be_present: bool,
    pub issuer: Option<String>,
}

impl XMatch {
    /// Write an XML (XACML) representation of an `XMatch` to a
    /// stream.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the writer fails.
    pub fn write_xml<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), xml::writer::Error> {
        // will look something like;
        // <xacml3:Match MatchId="urn:simple">
        //   <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">
        //   12</xacml3:AttributeValue>
        //   <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" MustBePresent="false"/>
        // </xacml3:Match>
        writer.write(XmlEvent::start_element("xacml3:Match").attr("MatchId", &self.matchid))?;
        // Write Attribute Value
        writer.write(
            XmlEvent::start_element("xacml3:AttributeValue").attr("DataType", &self.value_type),
        )?;
        writer.write(XmlEvent::characters(&self.value))?;
        writer.write(XmlEvent::end_element())?;
        // Write Attribute Designator
        let mbp = &self.must_be_present.to_string();
        let mut attrdesig = XmlEvent::start_element("xacml3:AttributeDesignator")
            .attr("AttributeId", &self.designator_id)
            .attr("Category", &self.designator_category)
            .attr("DataType", &self.designator_type)
            .attr("MustBePresent", mbp);
        // if issuer exists, add it
        if let Some(i) = &self.issuer {
            attrdesig = attrdesig.attr("Issuer", i);
        }
        writer.write(attrdesig)?;
        writer.write(XmlEvent::end_element())?;
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

impl XTarget {
    /// Write an XML (XACML) representation of an `XTarget` to a
    /// stream.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the writer fails.
    pub fn write_xml<W: Write>(
        &self,
        writer: &mut EventWriter<W>,
    ) -> Result<(), xml::writer::Error> {
        // Add (empty) Target
        writer.write(XmlEvent::start_element("xacml3:Target"))?;

        // Write AnyOfs
        for a in &self.anyofs {
            writer.write(XmlEvent::start_element("xacml3:AnyOf"))?;
            // Write AllOfs
            for b in &a.allofs {
                writer.write(XmlEvent::start_element("xacml3:AllOf"))?;
                // write Matches
                for m in &b.matches {
                    m.write_xml(writer)?;
                }
                writer.write(XmlEvent::end_element())?;
            }
            writer.write(XmlEvent::end_element())?;
        }
        writer.write(XmlEvent::end_element())?;
        Ok(())
    }
}

/// Convert a Match AST to `XMatch`
///
/// # Arguments
/// * `m` - A parsed Match statement from a target.
/// * `source_ns` - The namespace where this match occurred in ALFA source.
///
/// # Returns
/// * `Ok(XMatch)` - Reference-counted XACML representation of a Match.
/// * `Err(ParseError)` - Parse error if the symbols in the Match.
///    could not be resolved
fn match_to_xmatch(m: &Match, source_ns: &[String], ctx: &Context) -> Result<XMatch, ParseError> {
    // the ns is where this match is located.
    // we need to find the function, which means looking up
    match m {
        Match::MatchFunc(mf) => {
            // get the local name of the function.
            let fn_fq = mf.function_id.join(".");
            // lookup the function name, given the target namespace,
            // and all the active imports in this namespace.
            let f = ctx.lookup_function(&fn_fq, source_ns)?;
            let tl = ctx.constant_to_typedliteral(mf.literal.clone(), source_ns)?;
            info!("typed literal: {tl:?}");
            // Next up is getting the designator information from the function.
            let attr = ctx.lookup_attribute(&mf.attribute.join("."), source_ns)?;
            info!("found attribute: {attr:?}");
            // lookup the attribute category, using the attribute namespace.
            let cat = ctx.lookup_category(&attr.category, &attr.ns)?;
            // lookup the attribute type, using the attribute namespace
            let attr_type = ctx.lookup_type(&attr.typedef, &attr.ns)?;
            // TODO: validate that the output argument of the function is Atomic(String), and that the String can be type-looked-up to a boolean.
            Ok(XMatch {
                matchid: f.function_uri.to_string(),
                value: tl.value,
                value_type: tl.type_uri,
                designator_id: attr.uri.clone(),
                designator_category: cat.uri.clone(),
                designator_type: attr_type.uri.clone(),
                must_be_present: mf.mustbepresent,
                issuer: mf.issuer.clone(),
            })
        }
        Match::MatchOp(mo) => {
            debug!("match op: {mo:?}");
            // get the type of the literal
            let tl = ctx.constant_to_typedliteral(mo.literal.clone(), source_ns)?;
            debug!("typed literal: {tl:?}");
            // lookup an operator.
            debug!("operator is: {:?}", mo.operator);
            // this will be the infix operator we use.
            let infix;

            // find the infix operator referenced in the text:
            let source_infix = ctx.lookup_infix(&mo.operator.qualified_name(), source_ns)?;

            // if the matchop is normal (literal, op, attr-designator), then we can proceed easily.
            if mo.reversed {
                // if the operator is commutative, than the order being reversed is no problem.
                if source_infix.commutative {
                    infix = source_infix;
                } else if source_infix.inverse.is_some() {
                    infix = ctx.lookup_infix_inverse(&mo.operator.qualified_name(), source_ns)?;
                    // now, we have to rely on an inverse being defined
                } else {
                    // not commutative, not inverse, but order is reversed.  Nothing we can do!
                    debug!("MatchOperation order is reversed, op is not commutative, no inverse");
                    return Err(ParseError::ReversedInfixArgsNotCommutativeError);
                }
            } else {
                // order was normal, so nothing to do.
                infix = source_infix;
            }
            // what is the type of our attribute?
            info!("ATTR LOOKUP against source_ns: {source_ns:?}");
            let attr = ctx.lookup_attribute(&mo.attribute.join("."), source_ns)?;
            debug!("the attribute is: {attr:?}");
            let attr_type = ctx.lookup_type(&attr.typedef, &attr.ns)?.uri.clone();
            debug!("the attribute type is: {attr_type:?}");
            // get the category
            let cat = ctx.lookup_category(&attr.category, &attr.ns)?;
            // a boolean is one of those types that really can't be
            // meaningfully redefined by the user, so this can be
            // hardcoded.

            // TODO: we actually should be able to look this up.
            let bool_type = crate::ast::typedef::BOOLEAN_URI;
            // TODO: detect/error when multiple signatures match.  For
            // now, take the first one.

            // now, we can look through the signatures available.
            for s in &infix.signatures {
                debug!("checking signature: {s:?}");
                let first_type = ctx.lookup_type(&s.first_arg, &infix.ns)?;
                let second_type = ctx.lookup_type(&s.second_arg, &infix.ns)?;
                let output_type = ctx.lookup_type(&s.output, &infix.ns)?;
                if first_type.uri == tl.type_uri
                    && second_type.uri == attr_type
                    && output_type.uri == bool_type
                {
                    return Ok(XMatch {
                        matchid: s.uri.to_string(),
                        value: tl.value,
                        value_type: tl.type_uri,
                        designator_id: attr.uri.clone(),
                        designator_category: cat.uri.clone(),
                        designator_type: attr_type,
                        must_be_present: mo.mustbepresent,
                        issuer: mo.issuer.clone(),
                    });
                }
            }
            error!("did not find matching signature in {infix:?}");
            Err(ParseError::AstConvertError)
        }
    }
}

/// Conversion of Alfa Target to XACML Target
impl TryFrom<&Target> for XTarget {
    type Error = ParseError;
    fn try_from(t: &Target) -> Result<Self, Self::Error> {
        let ctx = t.ctx.upgrade().ok_or(ParseError::ContextMissing)?;
        // find out how many child
        let mut anyofs = vec![];
        for c in &t.clauses {
            // build the AnyOf
            let mut allofs = vec![];
            for a in &c.statements {
                let mut matches = vec![];
                for m in &a.matches {
                    debug!("looking up information about a match: {m:?}");
                    debug!("Attempting conversion with match_to_xmatch");
                    matches.push(match_to_xmatch(m, &t.ns, &ctx)?);
                }
                allofs.push(AllOf { matches });
            }
            anyofs.push(AnyOf { allofs });
        }
        Ok(XTarget { anyofs })
    }
}
