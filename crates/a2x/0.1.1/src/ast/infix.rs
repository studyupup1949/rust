//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::AsAlfa;
use super::PrettyPrint;
use super::QualifiedName;
use std::fmt;

/// Infix operator
#[derive(Debug, PartialEq, Clone)]
pub struct Infix {
    /// The operator symbol
    pub operator: String,
    /// Does this allow bags?
    pub allow_bags: bool,
    /// Is this commutative?  It should not be allowed to construct a
    /// commutative Infix w/ a defined inverse.
    pub commutative: bool,
    /// All the signatures for this operator
    pub signatures: Vec<InfixSignature>,
    /// Fully qualified namespace
    pub ns: Vec<String>,
    /// Optional inverse operator symbol (includes namespace)
    pub inverse: Option<String>,
}

/// Infix signature
#[derive(Debug, PartialEq, Clone)]
pub struct InfixSignature {
    /// The URI of the function
    pub uri: String,
    /// first argument alfa type
    pub first_arg: String,
    /// second argument alfa type
    pub second_arg: String,
    /// output alfa type
    pub output: String,
}

impl AsAlfa for Infix {
    fn to_alfa(&self, indent_level: usize) -> String {
        let indent = "  ".repeat(indent_level);
        // first infix declaration line
        let mut output = format!("{indent}infix ");
        if self.allow_bags {
            output.push_str("allowbags ");
        }
        if self.commutative {
            output.push_str("comm ");
        }
        output.push_str(&format!("({}) = {{\n", self.operator));
        for s in &self.signatures {
            output.push_str(&s.to_alfa(indent_level + 1));
        }
        if let Some(i) = &self.inverse {
            output.push_str(&format!("{indent}}} inv {i}\n"));
        } else {
            output.push_str(&format!("{indent}}}\n"));
        }
        output
    }
}

impl QualifiedName for Infix {
    fn fully_qualified_name(&self) -> Option<String> {
        let mut qn = self.ns.join(".");
        qn.push('.');
        qn.push_str(&self.operator);
        Some(qn.to_string())
    }
}

/// Pretty print function
impl PrettyPrint for Infix {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        print!("{indent}{self}");
        for s in &self.signatures {
            s.pretty_print(indent_level + 1);
        }
    }
}

impl AsAlfa for InfixSignature {
    fn to_alfa(&self, indent_level: usize) -> String {
        let indent = "  ".repeat(indent_level);
        // Ex: "urn:oasis:names:tc:xacml:1.0:function:integer-multiply" : integer integer -> integer
        format!(
            "{}\"{}\" : {} {} -> {}\n",
            indent, self.uri, self.first_arg, self.second_arg, self.output
        )
    }
}

/// Pretty print function
impl PrettyPrint for InfixSignature {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        print!("{indent}{self}");
        println!();
    }
}

impl fmt::Display for Infix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "infix ({})", self.operator)
    }
}

impl fmt::Display for InfixSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "infix-sig ({}) {}, {} => {}",
            self.uri, self.first_arg, self.second_arg, self.output
        )
    }
}
