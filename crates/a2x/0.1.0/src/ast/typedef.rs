//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::AsAlfa;
use super::PrettyPrint;
use super::QualifiedName;
use crate::context::SYSTEM_NS;
use std::fmt;

// helpful constants
pub const BOOLEAN_URI: &str = "http://www.w3.org/2001/XMLSchema#boolean";
pub const STRING_URI: &str = "http://www.w3.org/2001/XMLSchema#string";
pub const INTEGER_URI: &str = "http://www.w3.org/2001/XMLSchema#integer";
pub const DOUBLE_URI: &str = "http://www.w3.org/2001/XMLSchema#double";

/// A type definition statement
#[derive(Debug, PartialEq, Clone, Default)]
pub struct TypeDef {
    pub id: String,
    pub uri: String,
    /// The namespace from general to most specific.
    pub ns: Vec<String>,
}

impl AsAlfa for TypeDef {
    fn to_alfa(&self, indent_level: usize) -> String {
        let indent = "  ".repeat(indent_level);
        // Ex: type anyURI = "http://www.w3.org/2001/XMLSchema#anyURI"
        format!("{}type {} = \"{}\"\n", indent, self.id, self.uri)
    }
}

impl QualifiedName for TypeDef {
    fn fully_qualified_name(&self) -> Option<String> {
        let mut qn = self.ns.join(".");
        qn.push('.');
        qn.push_str(&self.id);
        Some(qn.to_string())
    }
}

impl PrettyPrint for TypeDef {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

impl fmt::Display for TypeDef {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Type: \"{}\" => {:?}", self.id, self.uri)
    }
}

fn make_std_type(id: &str, uri: &str) -> TypeDef {
    TypeDef {
        id: id.to_string(),
        uri: uri.to_string(),
        ns: vec![SYSTEM_NS.to_string()],
    }
}

// build all the "standard" types
#[must_use]
pub fn standard_types() -> Vec<TypeDef> {
    let mut t = vec![make_std_type("string", STRING_URI)];
    t.push(make_std_type("boolean", BOOLEAN_URI));
    t.push(make_std_type("integer", INTEGER_URI));
    t.push(make_std_type("double", DOUBLE_URI));
    t.push(make_std_type(
        "anyURI",
        "http://www.w3.org/2001/XMLSchema#anyURI",
    ));
    t.push(make_std_type(
        "base64Binary",
        "http://www.w3.org/2001/XMLSchema#base64Binary",
    ));
    t.push(make_std_type(
        "date",
        "http://www.w3.org/2001/XMLSchema#date",
    ));
    t.push(make_std_type(
        "dateTime",
        "http://www.w3.org/2001/XMLSchema#dateTime",
    ));
    t.push(make_std_type(
        "dayTimeDuration",
        "http://www.w3.org/2001/XMLSchema#dayTimeDuration",
    ));
    t.push(make_std_type(
        "hexBinary",
        "http://www.w3.org/2001/XMLSchema#hexBinary",
    ));
    t.push(make_std_type(
        "rfc822Name",
        "urn:oasis:names:tc:xacml:1.0:data-type:rfc822Name",
    ));
    t.push(make_std_type(
        "time",
        "http://www.w3.org/2001/XMLSchema#time",
    ));
    t.push(make_std_type(
        "x500Name",
        "urn:oasis:names:tc:xacml:1.0:data-type:x500Name",
    ));
    t.push(make_std_type(
        "xpath",
        "urn:oasis:names:tc:xacml:3.0:data-type:xpathExpression",
    ));
    t.push(make_std_type(
        "yearMonthDuration",
        "http://www.w3.org/2001/XMLSchema#yearMonthDuration",
    ));
    t.push(make_std_type(
        "dnsName",
        "urn:oasis:names:tc:xacml:2.0:data-type:dnsName",
    ));
    t.push(make_std_type(
        "ipAddress",
        "urn:oasis:names:tc:xacml:2.0:data-type:ipAddress",
    ));

    t
}
