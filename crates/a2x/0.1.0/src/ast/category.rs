//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::AsAlfa;
use super::PrettyPrint;
use super::QualifiedName;
use crate::context::SYSTEM_NS;
use std::fmt;

/// A category statement
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Category {
    pub id: String,
    pub uri: String,
    /// The namespace from general to most specific.
    pub ns: Vec<String>,
}

impl AsAlfa for Category {
    fn to_alfa(&self, indent_level: usize) -> String {
        let indent = "  ".repeat(indent_level);
        // Ex: category actionCat = "urn:oasis:...:attribute-category:action"
        format!("{}category {} = \"{}\"\n", indent, self.id, self.uri)
    }
}

impl QualifiedName for Category {
    fn fully_qualified_name(&self) -> Option<String> {
        let mut qn = self.ns.join(".");
        qn.push('.');
        qn.push_str(&self.id);
        Some(qn.to_string())
    }
}

impl PrettyPrint for Category {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

impl fmt::Display for Category {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Category: \"{}\" => {:?}", self.id, self.uri)
    }
}

fn make_std_cat(id: &str, uri: &str) -> Category {
    Category {
        id: id.to_owned(),
        uri: uri.to_owned(),
        ns: vec![SYSTEM_NS.to_owned()],
    }
}

// build all the "standard" categories
#[must_use]
pub fn standard_categories() -> Vec<Category> {
    let mut c = vec![make_std_cat(
        "subjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject-category:access-subject",
    )];
    c.push(make_std_cat(
        "codebaseCat",
        "urn:oasis:names:tc:xacml:1.0:subject-category:codebase",
    ));
    c.push(make_std_cat(
        "intermediarySubjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject-category:intermediary-subject",
    ));
    c.push(make_std_cat(
        "recipientSubjectCat",
        "urn:oasis:names:tc:xacml:1.0:subject-category:recipient-subject",
    ));
    c.push(make_std_cat(
        "requestingMachineCat",
        "urn:oasis:names:tc:xacml:1.0:subject-category:requesting-machine",
    ));
    c.push(make_std_cat(
        "resourceCat",
        "urn:oasis:names:tc:xacml:3.0:attribute-category:resource",
    ));
    c.push(make_std_cat(
        "actionCat",
        "urn:oasis:names:tc:xacml:3.0:attribute-category:action",
    ));
    c.push(make_std_cat(
        "environmentCat",
        "urn:oasis:names:tc:xacml:3.0:attribute-category:environment",
    ));
    c
}
