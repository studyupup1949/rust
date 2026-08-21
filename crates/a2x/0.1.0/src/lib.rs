//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later
//! Convert ALFA 1.0 source files to XACML 3.0 XML.
//!
//! ALFA is the "Abbreviated Language for Authorization", which serves
//! as a human-friendly language for writing authorization policies,
//! with the goal of being converted into the XML-based XACML
//! (eXtensible Access Control Markup Language) format.
//!
//! This crate attempts to implement ALFA as specified in the [OASIS
//! Working Draft 01
//! specification](https://www.oasis-open.org/committees/download.php/55228/alfa-for-xacml-v1.0-wd01.doc).
//!
//! As this library evolves, the intent is to consider adding
//! additional features that are planned for ALFA 2.0.  There also may
//! be deviations from the 1.0 spec, where absolutely necessary.
//!
//! The ALFA 1.0 spec is ambiguous in several areas, and I have
//! attempted to follow the observed behavior of other ALFA
//! implementations, notably the [Axiomatics "Visual Studio Code
//! extension for ALFA"
//! plugin](https://marketplace.visualstudio.com/items?itemName=Axiomatics.alfa).

use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;
use xacml::xpolicyentry::XPolicyEntry;
use xacml::XTopPolicy;
pub mod args;
pub mod ast;
pub mod context;
pub mod errors;
pub mod xacml;
use crate::ast::AstCollection;
use crate::ast::AstSource;
use crate::xacml::xpolicyset::XPolicySet;
//use crate::ast::PrettyPrint;
use crate::context::Context;
pub use crate::errors::ParseError;
use crate::xacml::XacmlWriter;
use log::{info, warn};
use std::fs::File;
//use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use xml::writer::EmitterConfig;

/// A pest parser for the ALFA authorization language.
#[derive(Parser)]
#[grammar = "alfa.pest"]
pub struct AlfaDocParser;

/// Parses a single ALFA policy document into a tree of tokens.
///
/// This is the result of simply running the `alfa.pest` PEG grammar,
/// and does not fully parse the policy, but does most of the work to
/// ensure the structure is valid, and associate rules/productions
/// with blocks of text.
///
/// This is only intended to be used as a step within `make_alfa_ast`.
///
/// # Arguments
/// * `policystr` - A string slice containing the ALFA policy document to parse
/// * `ctx` - A reference-counted object that provides additional parsing context
///
/// # Returns
/// Returns a `Result` containing:
/// * `Ok(AlfaParseTree)` - Successfully parsed policy tree with the parsed pairs and context
/// * `Err(ParseError)` - Parse error if the policy string contains invalid syntax
///
/// # Errors
///
/// This function will return a `ParseError` if:
/// * The policy string contains invalid ALFA syntax
/// * The policy structure doesn't conform to the ALFA grammar rules
/// * Required policy elements are missing or malformed
///
pub fn parse_alfadoc(policystr: &str, ctx: Rc<Context>) -> Result<AlfaParseTree<'_>, ParseError> {
    let pairs = AlfaDocParser::parse(Rule::alfa_doc, policystr)?;
    Ok(AlfaParseTree { pairs, ctx })
}

/// Parses a single ALFA policy document into an abstract syntax tree.
///
/// A PEG parser is used to create tokens, and the result is fully
/// parsed into an AST.  The input must be syntactically valid ALFA,
/// but there may still be issues with unresolved references, or type
/// errors.  Only a single document is parsed.
///
/// # Arguments
/// * `policystr` - A string slice containing the ALFA policy document to parse
/// * `ctx` - A reference-counted object that provides additional parsing context
///
/// # Returns
/// Returns a `Result` containing:
/// * `Ok(AlfaSyntaxTree)` - Successfully parsed policy combined with `ctx` object
/// * `Err(ParseError)` - Parse error if the policy string contains invalid syntax
///
/// # Errors
///
/// This function will return a `ParseError` if:
/// * The policy string contains invalid ALFA syntax
/// * The policy structure doesn't conform to the ALFA grammar rules
/// * Required policy elements are missing or malformed
///
pub fn make_alfa_ast(policystr: &str, ctx: Rc<Context>) -> Result<ast::AlfaSyntaxTree, ParseError> {
    let parse_tree = parse_alfadoc(policystr, ctx)?;
    // this would be a good place to weave in the context, but no...
    let alfa_ast = ast::AlfaSyntaxTree::try_from(parse_tree)?;
    Ok(alfa_ast)
}

/// The original path and full contents of an Alfa source file.
#[derive(Debug)]
pub struct AlfaFile {
    /// Filename this alfa source file was read from.
    pub filename: PathBuf,
    /// Original contents of the alfa source file.
    pub contents: String,
}

/// The intended output filepath and raw output contents of a
/// XACML/XML file.
#[derive(Debug)]
pub struct XacmlFile {
    /// Proposed file path for saving the XACML file.  This **should**
    /// be a relative path with a filename ending in `.xml`.
    pub filename: PathBuf,
    /// Raw XACML/XML content suitable for saving to a file.
    pub contents: Vec<u8>,
}

/// Write policy files to disk.
///
/// # Errors
///
/// An `Err` is returned if the individual policy file cannot be
/// created or written to.
///
/// # Panics
///
/// A panic occurs if the target directory cannot be created, or if
/// there are any errors serializing XML.
pub fn write_xentry(dir: &Path, p: &XTopPolicy) -> Result<(), ParseError> {
    // ensure directory exists
    std::fs::create_dir_all(dir).expect("could not create directory");
    match p {
        XTopPolicy::PolicySet(xps) => {
            info!("found xpolicyset");
            let output_filename = xps
                .filename
                .as_ref()
                .ok_or(ParseError::XacmlMissingFilename)?;
            let buffer = File::create(dir.join(output_filename))
                .map_err(|_x| ParseError::XacmlWriteIoError)?;
            let mut writer = EmitterConfig::new()
                .perform_indent(true)
                .create_writer(buffer);
            xps.write_xml(&mut writer).expect("unable to write");
        }
        XTopPolicy::Policy(xp) => {
            info!("found xpolicy");
            let output_filename = xp
                .filename
                .as_ref()
                .ok_or(ParseError::XacmlMissingFilename)?;
            let buffer = File::create(dir.join(output_filename))
                .map_err(|_x| ParseError::XacmlWriteIoError)?;
            let mut writer = EmitterConfig::new()
                .perform_indent(true)
                .create_writer(buffer);
            xp.write_xml(&mut writer).expect("unable to write");
        }
    }
    Ok(())
}

// Simpler method for integration tests, take Alfa Sources, and convert to XACML types, but do not serialize.

/// Compile a collection of Alfa source files into XACML 3.0 XML.
///
/// # Arguments
/// * `ctx` - A parsing context, used to generate and lookup symbols
/// * `alfa_sources` - A list of Alfa source files to process
///
/// # Returns
/// * `Ok(Vec<XTopPolicy>)` - Successfully converted top-level policies
/// * `Err(ParseError)` - Parse error information if conversion failed
///
/// # Errors
///
/// Returns `Err` if the conversion fails for syntactical or semantic
/// reasons.
///
/// # Panics
///
///
pub fn alfa_compile(
    ctx: &Rc<Context>,
    alfa_sources: Vec<AlfaFile>,
) -> Result<Vec<XTopPolicy>, ParseError> {
    info!("compiling...");
    let mut ast_collection = AstCollection::new(ctx.clone());
    // alfa ast conversion
    for asource in alfa_sources {
        info!("== {:?} ==", asource.filename);
        match make_alfa_ast(&asource.contents, ctx.clone()) {
            Ok(ast) => {
                info!("Successfully parsed the document.");
                info!("Contained {} top-level namespace(s):", ast.namespaces.len());
                //for ns in &ast.namespaces {
                // print all the top-level namespaces, and
                // recursively all the child members such as
                // policies, etc.
                //ns.pretty_print(0);
                //}
                ast_collection.add_ast(AstSource {
                    filename: asource.filename,
                    ast,
                });
            }
            Err(e) => {
                if let ParseError::PestParseError(pe) = e {
                    // TODO: cleanup
                    // print out the inner parse error
                    warn!("Parsing error at {pe}");
                    return Err(ParseError::PestParseError(pe));
                }
                warn!("Failed to parse document: {e:?}");
                return Err(e);
            }
        }
    }
    info!("Parsed {} alfa sources into ASTs", ast_collection.len());
    info!(
        "This AST Collection has {} policysets",
        ast_collection.policysets().len()
    );
    info!(
        "This AST Collection has {} policies",
        ast_collection.policies().len()
    );

    for p in ast_collection.policysets() {
        info!("policyset: {p}");
    }
    for p in ast_collection.policies() {
        info!("policy: {p}");
    }

    // Conversion of the policies into XPolicies
    // Each top-level policyset becomes an XFile.
    // Conversion of policysets into XPolicySets

    // top policies that will correspond to output files.
    let mut xtoppolicies = vec![];
    for p in ast_collection.policysets() {
        let xp = XPolicySet::try_from(p.as_ref())?;
        let fname = xp
            .filename
            .as_ref()
            .expect("filename not determined for this policy");
        info!("=== Converted PolicySet:  {fname} ===");
        xtoppolicies.push(XTopPolicy::PolicySet(xp));
    }

    // we will return XFiles

    // Each top-level policy becomes an XFile.
    for p in ast_collection.policies() {
        let xpe = XPolicyEntry::try_from(p.as_ref())?;
        match xpe {
            XPolicyEntry::Policy(xp) => {
                info!("=== Converted Policy:  {:?} ===", xp.filename);
                xtoppolicies.push(XTopPolicy::Policy(xp));
            }
            XPolicyEntry::PolicySet(xpset) => {
                info!("=== Converted PolicySet:  {:?} ===", xpset.filename);
                xtoppolicies.push(XTopPolicy::PolicySet(xpset));
            }
            _ => {
                // it should not be possible for the policyEntry
                // conversion to produce anything other than a policy
                // or policyset (refs can't be emitted)
                return Err(ParseError::AstConvertError);
            }
        }
    }

    // Now we have a pile of ASTs, and we need to convert to (starting
    // with) a set of XPolicy objects.

    // Each XPolicy can be serialized into a file.
    // Can we have a "policyset/policy iterator" on top of namespace?

    // Because policysets refer to policysets and policies, and
    // policies refer to rules, we need to give each of the items
    // returned access to the full root collection so that they can
    // ask if a named policy/rule exists.
    Ok(xtoppolicies)
}

/// The raw parse tree and context of an ALFA source file.
#[derive(Debug)]
pub struct AlfaParseTree<'a> {
    /// Result of parsing an ALFA document with the pest grammar.
    pub pairs: Pairs<'a, Rule>,
    ctx: Rc<Context>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_type_oneline_decl() {
        // ensure we can parse a simple one-line type sting
        let input = "type string = \"foo\"";
        let _ = AlfaDocParser::parse(Rule::type_decl, input).unwrap();
    }

    #[test]
    fn test_parse_type_multiline_decl() {
        // ensure we can parse a multi-line type sting
        let input = "type string = \n\"foo\"";
        let _ = AlfaDocParser::parse(Rule::type_decl, input).unwrap();
    }

    #[test]
    fn test_parse_category_oneline_decl() {
        // ensure we can parse a simple one-line category sting
        let input = r#"category subjectCat  = "urn:oasis:names:tc:xacml:1.0:subject-category:access-subject""#;
        let _ = AlfaDocParser::parse(Rule::cat_decl, input).unwrap();
    }

    #[test]
    fn test_parse_category_multiline_decl() {
        // ensure we can parse a multi-line category sting with lots of whitespace
        let input = "category \n   subjectCat  \n= \n\n\t\"urn:oasis:names:tc:xacml:1.0:subject-category:access-subject\"";
        let _ = AlfaDocParser::parse(Rule::cat_decl, input).unwrap();
    }

    #[test]
    fn test_parse_infix_no_mods() {
        let input = r#"infix (<) = {
    "urn:oasis:names:tc:xacml:1.0:function:integer-less-than" : integer integer -> boolean
    "urn:oasis:names:tc:xacml:1.0:function:double-less-than" : double double -> boolean
    }"#;
        let _ = AlfaDocParser::parse(Rule::infix_decl, input).unwrap();
    }
    #[test]
    fn test_parse_infix_allowbags() {
        let input = r#"infix allowbags (<) = {
    "urn:oasis:names:tc:xacml:1.0:function:integer-less-than" : integer integer -> boolean
    "urn:oasis:names:tc:xacml:1.0:function:double-less-than" : double double -> boolean
    }"#;
        let _ = AlfaDocParser::parse(Rule::infix_decl, input).unwrap();
    }
    #[test]
    fn test_parse_infix_multimodifiers() {
        let input = r#"infix allowbags comm (<) = {
    "urn:oasis:names:tc:xacml:1.0:function:integer-less-than" : integer integer -> boolean
    "urn:oasis:names:tc:xacml:1.0:function:double-less-than" : double double -> boolean
    }"#;
        let _ = AlfaDocParser::parse(Rule::infix_decl, input).unwrap();
        // switch order
        let input = r#"infix comm allowbags (<) = {
    "urn:oasis:names:tc:xacml:1.0:function:integer-less-than" : integer integer -> boolean
    "urn:oasis:names:tc:xacml:1.0:function:double-less-than" : double double -> boolean
    }"#;
        let _ = AlfaDocParser::parse(Rule::infix_decl, input).unwrap();
    }

    #[test]
    fn test_numeric_literal() {
        let valid_numbers = vec!["0", "0.0", "9.2", "-1", "-23", "-1.0", ".3", "-.9", "-0"];
        for n in valid_numbers {
            let parsed = AlfaDocParser::parse(Rule::numeric_literal, n).unwrap();
            // verify the we got the full string
            assert_eq!(parsed.as_str(), n);
        }
    }

    #[test]
    fn test_boolean_literal() {
        let parsed = AlfaDocParser::parse(Rule::boolean_literal, "true").unwrap();
        assert_eq!(parsed.as_str(), "true");
        let parsed = AlfaDocParser::parse(Rule::boolean_literal, "false").unwrap();
        assert_eq!(parsed.as_str(), "false");
        let parsed = AlfaDocParser::parse(Rule::boolean_literal, "TRUE");
        assert!(parsed.is_err());
        let parsed = AlfaDocParser::parse(Rule::boolean_literal, "True");
        assert!(parsed.is_err());
    }

    #[test]
    fn test_nested_namespaces() {
        let input = "namespace main.foo { namespace bar { namespace baz.wham {} } }";
        parse_alfadoc(input, Rc::default()).unwrap();
    }

    #[test]
    fn test_no_leading_numeric_namespaces() {
        // leading number in a nested namespace
        let input = "namespace main.foo { namespace 9bar { namespace baz.wham {} } }";
        let input_parsed = parse_alfadoc(input, Rc::default());
        assert!(input_parsed.is_err());
        // leading zero inside second component of a top-level namespace
        let input = "namespace main.0foo { namespace 9bar { namespace baz.wham {} } }";
        let input_parsed = parse_alfadoc(input, Rc::default());
        assert!(input_parsed.is_err());
        // all numeric for top-level namespace
        let input = "namespace 444.foo { namespace 9bar { namespace baz.wham {} } }";
        let input_parsed = parse_alfadoc(input, Rc::default());
        assert!(input_parsed.is_err());
        // leading empty namespace
        let input = "namespace .foo { namespace 9bar { namespace baz.wham {} } }";
        let input_parsed = parse_alfadoc(input, Rc::default());
        assert!(input_parsed.is_err());
        // empty namespace
        let input = "namespace main..foo { namespace 9bar { namespace baz.wham {} } }";
        let input_parsed = parse_alfadoc(input, Rc::default());
        assert!(input_parsed.is_err());
    }

    #[test]
    fn test_parse_alfadoc() {
        let input = "namespace foo { type string = \"foo\"}";
        parse_alfadoc(input, Rc::default()).unwrap();
    }

    /// Parse a document with a single namespace.
    #[test]
    fn test_parse_namespace() {
        let ast = make_alfa_ast("namespace simple {}", Rc::default()).unwrap();
        let namespaces = ast.namespaces;
        // there is a single top-level namespace.
        assert!(namespaces.len() == 1);
        // that namespaces has a name of "simple"
        assert_eq!(
            *namespaces.first().unwrap(),
            ast::namespace::Namespace::new_root("simple".to_string(), Rc::new(Context::default()))
        );
    }
    /// Parse a document with a single multi-component namespace.
    #[test]
    fn test_parse_multicomp_namespace() {
        let ast = make_alfa_ast("namespace foo.bar {}", Rc::default()).unwrap();
        let namespaces = ast.namespaces;
        // there is a single top-level namespace.
        assert!(namespaces.len() == 1);
        // that namespaces has a name of [foo, bar]"
        assert_eq!(
            *namespaces.first().unwrap(),
            ast::namespace::Namespace::from_components(
                vec!["foo".to_string(), "bar".to_string()],
                Rc::new(Context::default())
            )
        );
    }

    /// Parse a document with a nested namespace.
    #[test]
    fn test_parse_nested_namespace() {
        let ast = make_alfa_ast("namespace foo { namespace bar {}}", Rc::default()).unwrap();
        let namespaces = ast.namespaces;
        // there is a single top-level namespace.
        assert!(namespaces.len() == 1);
        // check first top-level namespace
        let toplevel_ns = namespaces.first().unwrap();
        // check that there is a single nested namespace
        let child_ns = &toplevel_ns.namespaces;
        assert_eq!(child_ns.len(), 1);
    }

    /// Parse a document with a multi-part infix operator
    #[test]
    fn test_multipart_infix_op() {
        let _ast = make_alfa_ast(
            r#"
namespace foo {
  infix allowbags (>>) = {
        "urn:oasis:names:tc:xacml:1.0:function:integer-greater-than" : integer integer -> boolean
    }
}
namespace main {
  attribute subjectAge {
    id = "urn:oasis:names:tc:xacml:1.0:subject:subject-age"
    type = integer
    category = subjectCat
  }
  // named rule
  policy main {
    apply firstApplicable
    target clause 40 foo.>> subjectAge
  }
}
"#,
            Rc::default(),
        )
        .unwrap();
    }
}
