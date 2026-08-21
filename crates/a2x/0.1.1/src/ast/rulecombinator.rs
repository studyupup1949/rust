//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::AsAlfa;
use super::PrettyPrint;
use super::QualifiedName;
use crate::context::PROTECTED_NS;
use crate::context::SYSTEM_NS;
use std::fmt;

/// A rule combining algorithm
#[derive(Debug, PartialEq, Clone)]
pub struct RuleCombinator {
    pub id: String,
    pub uri: String, // URI definition of algorithm
    /// The namespace from general to most specific.
    pub ns: Vec<String>,
}

impl AsAlfa for RuleCombinator {
    fn to_alfa(&self, indent_level: usize) -> String {
        let indent = "  ".repeat(indent_level);
        // Ex: ruleCombinator denyOverrides =
        //   "urn:oasis:...:deny-overrides"
        format!("{}ruleCombinator {} = \"{}\"\n", indent, self.id, self.uri)
    }
}

impl QualifiedName for RuleCombinator {
    fn fully_qualified_name(&self) -> Option<String> {
        let mut qn = self.ns.join(".");
        qn.push('.');
        qn.push_str(&self.id);
        Some(qn.to_string())
    }
}

impl PrettyPrint for RuleCombinator {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

impl fmt::Display for RuleCombinator {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RuleCombinator: \"{}.{}\" => {:?}",
            self.ns.join("."),
            self.id,
            self.uri
        )
    }
}

fn make_std_comb(id: &str, alg: &str) -> RuleCombinator {
    RuleCombinator {
        id: id.to_string(),
        uri: alg.to_string(),
        ns: vec![SYSTEM_NS.to_string()],
    }
}

fn make_protected_comb(id: &str, alg: &str) -> RuleCombinator {
    RuleCombinator {
        id: id.to_string(),
        uri: alg.to_string(),
        ns: vec![PROTECTED_NS.to_string()],
    }
}

#[must_use]
pub fn protected_rulecombinators() -> Vec<RuleCombinator> {
    // from XACML spec 10.2.3
    vec![make_protected_comb(
        "permitOverrides",
        "urn:oasis:names:tc:xacml:3.0:rule-combining-algorithm:permit-overrides",
    )]
}

#[must_use]
pub fn standard_rulecombinators() -> Vec<RuleCombinator> {
    // from XACML spec 10.2.3
    let mut r = vec![make_std_comb(
        "denyOverrides",
        "urn:oasis:names:tc:xacml:3.0:rule-combining-algorithm:deny-overrides",
    )];
    r.push(make_std_comb(
        "denyUnlessPermit",
        "urn:oasis:names:tc:xacml:3.0:rule-combining-algorithm:deny-unless-permit",
    ));
    r.push(make_std_comb(
        "firstApplicable",
        "urn:oasis:names:tc:xacml:1.0:rule-combining-algorithm:first-applicable",
    ));
    r.push(make_std_comb(
        "orderedDenyOverrides",
        "urn:oasis:names:tc:xacml:3.0:rule-combining-algorithm:ordered-deny-overrides",
    ));
    r.push(make_std_comb(
        "orderedPermitOverrides",
        "urn:oasis:names:tc:xacml:3.0:rule-combining-algorithm:ordered-permit-overrides",
    ));
    r.push(make_std_comb(
        "permitUnlessDeny",
        "urn:oasis:names:tc:xacml:3.0:rule-combining-algorithm:permit-unless-deny",
    ));
    r.push(make_std_comb(
        "permitOverrides",
        "urn:oasis:names:tc:xacml:3.0:rule-combining-algorithm:permit-overrides",
    ));
    r
}
