//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use common::compile_alfa_src;
use common::xentry_to_str;
use pretty_assertions::assert_eq;
use unwrap::unwrap;
mod common;

/// End-to-End ALFA-to-XACML test
#[test]
#[allow(clippy::too_many_lines)]
fn xacml_e2e_complex() {
    let e2e = compile_alfa_src(
        r#"
// define attributes in a namespace (these are also available in the default
// system namespace, but we want to test attribute definition here.
namespace Attributes {
  attribute subjectId {
    id = "urn:oasis:names:tc:xacml:1.0:subject:subject-id"
    type = string
    category = subjectCat
  }
  attribute resourceId {
    id = "urn:oasis:names:tc:xacml:1.0:resource:resource-id"
    type = string
    category = resourceCat
  }
  attribute currentDate {
    id = "urn:oasis:names:tc:xacml:1.0:environment:current-date"
    type = date
    category = environmentCat
  }
}
namespace foo {
  /* First Policy */
  policyset master = "master-id" {
    target clause Attributes.subjectId == "foo" and Attributes.resourceId == "bar"
    apply firstApplicable
    /* Child Policy */
    policy child = "child-id" {
      target clause Attributes.subjectId == "foo" or Attributes.subjectId == "bar"
             clause Attributes.resourceId == "bar" and Attributes.currentDate == "2025-10-31":date
      apply firstApplicable
      /* Child Rule */
      rule r {
        permit
        condition anyOf(function[stringStartsWith], "foo", Attributes.resourceId)
      }
    }
  }
}
"#,
    );
    let xp = unwrap!(e2e.first(), "there is at least one policyset");
    let xacml = xentry_to_str(xp);
    assert_eq!(
        xacml,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xacml3:PolicySet xmlns:xacml3="urn:oasis:names:tc:xacml:3.0:core:schema:wd-17" PolicySetId="master-id" PolicyCombiningAlgId="urn:oasis:names:tc:xacml:1.0:policy-combining-algorithm:first-applicable" Version="1.0">
  <xacml3:Description>First Policy</xacml3:Description>
  <xacml3:Target>
    <xacml3:AnyOf>
      <xacml3:AllOf>
        <xacml3:Match MatchId="urn:oasis:names:tc:xacml:1.0:function:string-equal">
          <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo</xacml3:AttributeValue>
          <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:subject:subject-id" Category="urn:oasis:names:tc:xacml:1.0:subject-category:access-subject" DataType="http://www.w3.org/2001/XMLSchema#string" MustBePresent="false" />
        </xacml3:Match>
        <xacml3:Match MatchId="urn:oasis:names:tc:xacml:1.0:function:string-equal">
          <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">bar</xacml3:AttributeValue>
          <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" MustBePresent="false" />
        </xacml3:Match>
      </xacml3:AllOf>
    </xacml3:AnyOf>
  </xacml3:Target>
  <xacml3:Policy PolicyId="child-id" RuleCombiningAlgId="urn:oasis:names:tc:xacml:1.0:rule-combining-algorithm:first-applicable" Version="1.0">
    <xacml3:Description>Child Policy</xacml3:Description>
    <xacml3:Target>
      <xacml3:AnyOf>
        <xacml3:AllOf>
          <xacml3:Match MatchId="urn:oasis:names:tc:xacml:1.0:function:string-equal">
            <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo</xacml3:AttributeValue>
            <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:subject:subject-id" Category="urn:oasis:names:tc:xacml:1.0:subject-category:access-subject" DataType="http://www.w3.org/2001/XMLSchema#string" MustBePresent="false" />
          </xacml3:Match>
        </xacml3:AllOf>
        <xacml3:AllOf>
          <xacml3:Match MatchId="urn:oasis:names:tc:xacml:1.0:function:string-equal">
            <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">bar</xacml3:AttributeValue>
            <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:subject:subject-id" Category="urn:oasis:names:tc:xacml:1.0:subject-category:access-subject" DataType="http://www.w3.org/2001/XMLSchema#string" MustBePresent="false" />
          </xacml3:Match>
        </xacml3:AllOf>
      </xacml3:AnyOf>
      <xacml3:AnyOf>
        <xacml3:AllOf>
          <xacml3:Match MatchId="urn:oasis:names:tc:xacml:1.0:function:string-equal">
            <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">bar</xacml3:AttributeValue>
            <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" MustBePresent="false" />
          </xacml3:Match>
          <xacml3:Match MatchId="urn:oasis:names:tc:xacml:1.0:function:date-equal">
            <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#date">2025-10-31</xacml3:AttributeValue>
            <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:environment:current-date" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:environment" DataType="http://www.w3.org/2001/XMLSchema#date" MustBePresent="false" />
          </xacml3:Match>
        </xacml3:AllOf>
      </xacml3:AnyOf>
    </xacml3:Target>
    <xacml3:Rule Effect="Permit" RuleId="https://sr.ht/~gheartsfield/a2x/alfa/ident/foo/master/child/r">
      <xacml3:Description>Child Rule</xacml3:Description>
      <xacml3:Target />
      <xacml3:Condition>
        <xacml3:Apply FunctionId="urn:oasis:names:tc:xacml:3.0:function:any-of">
          <xacml3:Function FunctionId="urn:oasis:names:tc:xacml:3.0:function:string-starts-with" />
          <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo</xacml3:AttributeValue>
          <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" MustBePresent="false" />
        </xacml3:Apply>
      </xacml3:Condition>
    </xacml3:Rule>
  </xacml3:Policy>
</xacml3:PolicySet>"#
    );
}
