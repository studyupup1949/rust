//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! Advice is used to provide optional guidance to a Policy
//! Enforcement Point.
//!
//! Currently this module handles the Advice declaration which relates
//! an Alfa name to the XACML identifier.
//!
//! The Alfa statement would look like this:
//! ```advice myAdvice = "http://example.com/advice"```
//!
//!
use super::PrettyPrint;
use super::QualifiedName;
use std::fmt;

/// A definition of an `Advice` URI.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct AdviceDef {
    pub id: String,
    pub uri: String,
    /// The namespace from general to most specific.
    pub ns: Vec<String>,
}

impl QualifiedName for AdviceDef {
    fn fully_qualified_name(&self) -> Option<String> {
        let mut qn = self.ns.join(".");
        qn.push('.');
        qn.push_str(&self.id);
        Some(qn.to_string())
    }
}

impl PrettyPrint for AdviceDef {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

impl fmt::Display for AdviceDef {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Advice: \"{}\" => {:?}", self.id, self.uri)
    }
}
