//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use common::compile_alfa_src;
use common::xentry_to_str;
use pretty_assertions::assert_eq;
use unwrap::unwrap;
mod common;

/// End-to-End ALFA-to-XACML test
#[test]
fn recursive_policyset() {
    // two policies that recursively reference each other.
    let e2e = compile_alfa_src(
        r#"
namespace foo {
  policyset one = "one" {
    apply firstApplicable
    two
  }
  policyset two = "two" {
    apply firstApplicable
    one
  }
}
"#,
    );
    let xp = unwrap!(e2e.first(), "there is at least one policyset");
    let xacml = xentry_to_str(xp);
    assert_eq!(
        xacml,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xacml3:PolicySet xmlns:xacml3="urn:oasis:names:tc:xacml:3.0:core:schema:wd-17" PolicySetId="one" PolicyCombiningAlgId="urn:oasis:names:tc:xacml:1.0:policy-combining-algorithm:first-applicable" Version="1.0">
  <xacml3:Target />
  <xacml3:PolicySetIdReference>two</xacml3:PolicySetIdReference>
</xacml3:PolicySet>"#
    );
}
