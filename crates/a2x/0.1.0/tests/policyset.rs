//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use common::compile_alfa_src;
use common::xentry_to_str;
use pretty_assertions::assert_eq;
use unwrap::unwrap;
mod common;

// Integration tests for policysets.
//
// This primarily handles sections 4.9 (policies) of
// alfa-for-xacml-v1.0-csd01

// This is mostly minimal policyset contents & naming, not
// conditions/targets/advice/etc.

/// End-to-End ALFA-to-XACML test
#[test]
fn xacml_policyset_simple() {
    let xpolicysets = compile_alfa_src(
        r#"
namespace foo {
  /** First PolicySet **/
  policyset master = "master-id" {
    apply firstApplicable
  }
}
"#,
    );
    let xp = unwrap!(xpolicysets.first(), "there is at least one policyset");
    let xacml = xentry_to_str(xp);
    assert_eq!(
        xacml,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xacml3:PolicySet xmlns:xacml3="urn:oasis:names:tc:xacml:3.0:core:schema:wd-17" PolicySetId="master-id" PolicyCombiningAlgId="urn:oasis:names:tc:xacml:1.0:policy-combining-algorithm:first-applicable" Version="1.0">
  <xacml3:Description>First PolicySet</xacml3:Description>
  <xacml3:Target />
</xacml3:PolicySet>"#
    );
}

/// Policyset reference
#[test]
fn xacml_policyset_ref() {
    let xpolicysets = compile_alfa_src(
        r#"
namespace foo {
  /** Parent PolicySet **/
  policyset parent = "parent" {
    apply firstApplicable
    child
  }
  /** Child PolicySet **/
  policyset child = "child" {
    apply firstApplicable
  }
}
"#,
    );
    let xp = unwrap!(xpolicysets.first(), "there is at least one policyset");
    let xacml = xentry_to_str(xp);
    assert_eq!(
        xacml,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xacml3:PolicySet xmlns:xacml3="urn:oasis:names:tc:xacml:3.0:core:schema:wd-17" PolicySetId="parent" PolicyCombiningAlgId="urn:oasis:names:tc:xacml:1.0:policy-combining-algorithm:first-applicable" Version="1.0">
  <xacml3:Description>Parent PolicySet</xacml3:Description>
  <xacml3:Target />
  <xacml3:PolicySetIdReference>child</xacml3:PolicySetIdReference>
</xacml3:PolicySet>"#
    );
}
