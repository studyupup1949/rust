//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::PrettyPrint;
use super::QualifiedName;
use std::fmt;

/// An obligation definition statement
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ObligationDef {
    pub id: String,
    pub uri: String,
    /// The namespace from general to most specific.
    pub ns: Vec<String>,
}

impl QualifiedName for ObligationDef {
    fn fully_qualified_name(&self) -> Option<String> {
        let mut qn = self.ns.join(".");
        qn.push('.');
        qn.push_str(&self.id);
        Some(qn.to_string())
    }
}

impl PrettyPrint for ObligationDef {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

impl fmt::Display for ObligationDef {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Obligation: \"{}\" => {:?}", self.id, self.uri)
    }
}
