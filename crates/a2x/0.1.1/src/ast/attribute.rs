//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::AsAlfa;
use super::PrettyPrint;
use super::QualifiedName;
use std::fmt;

/// An Attribute definition
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Attribute {
    // short name
    pub id: String,
    // type of attribute
    pub typedef: String,
    pub category: String,
    pub uri: String,
    /// The namespace from general to most specific.
    pub ns: Vec<String>,
}

impl AsAlfa for Attribute {
    fn to_alfa(&self, indent_level: usize) -> String {
        let indent = "  ".repeat(indent_level);
        let nested_indent = "  ".repeat(indent_level + 1);
        // Ex: function stringEqual = "urn:oasis:...:string-equal"
        //         : string string -> boolean
        let mut output = format!("{indent}attribute {} {{\n", &self.id);
        output.push_str(&format!("{nested_indent}id = {:?}\n", &self.uri));
        output.push_str(&format!("{nested_indent}type = {}\n", &self.typedef));
        output.push_str(&format!("{nested_indent}category = {}\n", &self.category));
        output.push_str(&format!("{indent}}}\n"));
        output
    }
}

impl QualifiedName for Attribute {
    fn fully_qualified_name(&self) -> Option<String> {
        let mut qn = self.ns.join(".");
        qn.push('.');
        qn.push_str(&self.id);
        Some(qn.to_string())
    }
}

impl PrettyPrint for Attribute {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

impl fmt::Display for Attribute {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attribute: \"{}\" => {:?}", self.id, self.uri)
    }
}
