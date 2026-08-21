//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::PrettyPrint;
use std::fmt;

/// An import statement
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Import {
    /// Components in an import statement, both namespace and element
    /// (policy/rule/etc.)
    pub components: Vec<String>,
    /// Does this import statement end in a wildcard?
    pub is_wildcard: bool,
}

impl PrettyPrint for Import {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

impl fmt::Display for Import {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Import: \"{}\" {}",
            self.components.join("."),
            if self.is_wildcard {
                "*".to_string()
            } else {
                String::default()
            }
        )
    }
}
