//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use super::AsAlfa;
use super::PrettyPrint;
use super::QualifiedName;
use crate::context::PROTECTED_NS;
use crate::context::SYSTEM_NS;
use std::fmt;

/// A policy combining algorithm
#[derive(Debug, PartialEq, Clone)]
pub struct PolicyCombinator {
    pub id: String,
    pub uri: String, // URI definition of algorithm
    /// The namespace from general to most specific.
    pub ns: Vec<String>,
}

impl AsAlfa for PolicyCombinator {
    fn to_alfa(&self, indent_level: usize) -> String {
        let indent = "  ".repeat(indent_level);
        // Ex: policyCombinator denyOverrides =
        //   "urn:oasis:...:deny-overrides"
        format!(
            "{}policyCombinator {} = \"{}\"\n",
            indent, self.id, self.uri
        )
    }
}

impl QualifiedName for PolicyCombinator {
    fn fully_qualified_name(&self) -> Option<String> {
        let mut qn = self.ns.join(".");
        qn.push('.');
        qn.push_str(&self.id);
        Some(qn.to_string())
    }
}

impl PrettyPrint for PolicyCombinator {
    fn pretty_print(&self, indent_level: usize) {
        let indent = "  ".repeat(indent_level);
        println!("{indent}{self}");
    }
}

impl fmt::Display for PolicyCombinator {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PolicyCombinator: \"{}.{}\" => {:?}",
            self.ns.join("."),
            self.id,
            self.uri
        )
    }
}

fn make_std_comb(id: &str, alg: &str) -> PolicyCombinator {
    PolicyCombinator {
        id: id.to_string(),
        uri: alg.to_string(),
        ns: vec![SYSTEM_NS.to_string()],
    }
}

fn make_protected_comb(id: &str, alg: &str) -> PolicyCombinator {
    PolicyCombinator {
        id: id.to_string(),
        uri: alg.to_string(),
        ns: vec![PROTECTED_NS.to_string()],
    }
}

#[must_use]
pub fn protected_policycombinators() -> Vec<PolicyCombinator> {
    // from XACML spec 10.2.3
    let mut p = vec![make_protected_comb(
        "permitOverrides",
        "urn:oasis:names:tc:xacml:3.0:policy-combining-algorithm:permit-overrides",
    )];
    // on-permit-apply-second:  https://docs.oasis-open.org/xacml/xacml-3.0-combalgs/v1.0/csprd03/xacml-3.0-combalgs-v1.0-csprd03.html
    p.push(make_protected_comb(
        "onPermitApplySecond",
        "urn:oasis:names:tc:xacml:3.0:policy-combining-algorithm:on-permit-apply-second",
    ));
    p
}

#[must_use]
pub fn standard_policycombinators() -> Vec<PolicyCombinator> {
    // from XACML spec 10.2.3
    let mut p = vec![
        (make_std_comb(
            "denyOverrides",
            "urn:oasis:names:tc:xacml:3.0:policy-combining-algorithm:deny-overrides",
        )),
    ];
    p.push(make_std_comb(
        "denyUnlessPermit",
        "urn:oasis:names:tc:xacml:3.0:policy-combining-algorithm:deny-unless-permit",
    ));
    p.push(make_std_comb(
        "firstApplicable",
        "urn:oasis:names:tc:xacml:1.0:policy-combining-algorithm:first-applicable",
    ));
    p.push(make_std_comb(
        "onlyOneApplicable",
        "urn:oasis:names:tc:xacml:1.0:policy-combining-algorithm:only-one-applicable",
    ));
    p.push(make_std_comb(
        "orderedDenyOverrides",
        "urn:oasis:names:tc:xacml:3.0:policy-combining-algorithm:ordered-deny-overrides",
    ));
    p.push(make_std_comb(
        "orderedPermitOverrides",
        "urn:oasis:names:tc:xacml:3.0:policy-combining-algorithm:ordered-permit-overrides",
    ));
    p.push(make_std_comb(
        "permitOverrides",
        "urn:oasis:names:tc:xacml:3.0:policy-combining-algorithm:permit-overrides",
    ));
    p.push(make_std_comb(
        "permitUnlessDeny",
        "urn:oasis:names:tc:xacml:3.0:policy-combining-algorithm:permit-unless-deny",
    ));
    // on-permit-apply-second:  https://docs.oasis-open.org/xacml/xacml-3.0-combalgs/v1.0/csprd03/xacml-3.0-combalgs-v1.0-csprd03.html
    p.push(make_std_comb(
        "onPermitApplySecond",
        "urn:oasis:names:tc:xacml:3.0:policy-combining-algorithm:on-permit-apply-second",
    ));

    p
}
