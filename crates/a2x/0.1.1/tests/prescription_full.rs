//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

use common::{compile_alfa_src, xentry_to_str};
use pretty_assertions::assert_eq;
mod common;
use unwrap::unwrap;

// This handles sections 4.15 (obligations and advice) of
// alfa-for-xacml-v1.0-csd01
//
// Integration tests for obligations and advice on policyset, policy,
// and rules.
//
// This tries all combinations of literal/designator expressions on
// permit/deny obligations/advice on policysets, policies, and rules;
// so there should be 2x2x2x3=24 expressions total.

/// Test prescriptions on all possible entities
#[test]
#[allow(clippy::too_many_lines)]
fn prescription_everything() {
    let x = compile_alfa_src(
        r#"
namespace main {
  attribute resourceId {
    id = "urn:oasis:names:tc:xacml:1.0:resource:resource-id"
    type = string
    category = resourceCat
  }
  attribute actionId {
    id = "urn:oasis:names:tc:xacml:1.0:action:action-id"
    type = string
    category = actionCat
  }
  obligation o1 = "urn:example:oblig1"
  advice a1 = "urn:example:advice1"
  policyset ps = "ps" {
    apply firstApplicable
    policy p = "p" {
      apply firstApplicable
      rule r {
        permit
        on permit {
          obligation o1 {
            actionId = "foo-oblig-permit"
            resourceId = resourceId[mustbepresent issuer="bar-oblig-permit"]
          }
          advice a1 {
            actionId = "foo-advice-permit"
            resourceId = resourceId[mustbepresent issuer="bar-advice-permit"]
          }
        }
        on deny {
          obligation o1 {
            actionId = "foo-oblig-deny"
            resourceId = resourceId[mustbepresent issuer="bar-oblig-deny"]
          }
          advice a1 {
            actionId = "foo-advice-deny"
            resourceId = resourceId[mustbepresent issuer="bar-advice-deny"]
          }
        }
      }
      on permit {
        obligation o1 {
          actionId = "foo-oblig-permit"
          resourceId = resourceId[mustbepresent issuer="bar-oblig-permit"]
        }
        advice a1 {
          actionId = "foo-advice-permit"
          resourceId = resourceId[mustbepresent issuer="bar-advice-permit"]
        }
      }
      on deny {
        obligation o1 {
          actionId = "foo-oblig-deny"
          resourceId = resourceId[mustbepresent issuer="bar-oblig-deny"]
        }
        advice a1 {
          actionId = "foo-advice-deny"
          resourceId = resourceId[mustbepresent issuer="bar-advice-deny"]
        }
      }
    }
    on permit {
      obligation o1 {
        actionId = "foo-oblig-permit"
        resourceId = resourceId[mustbepresent issuer="bar-oblig-permit"]
      }
      advice a1 {
        actionId = "foo-advice-permit"
        resourceId = resourceId[mustbepresent issuer="bar-advice-permit"]
      }
    }
    on deny {
      obligation o1 {
        actionId = "foo-oblig-deny"
        resourceId = resourceId[mustbepresent issuer="bar-oblig-deny"]
      }
      advice a1 {
        actionId = "foo-advice-deny"
        resourceId = resourceId[mustbepresent issuer="bar-advice-deny"]
      }
    }
  }
}"#,
    );
    let xp = unwrap!(x.first(), "at least one policyset");
    let xacml = xentry_to_str(xp);
    assert_eq!(
        xacml,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xacml3:PolicySet xmlns:xacml3="urn:oasis:names:tc:xacml:3.0:core:schema:wd-17" PolicySetId="ps" PolicyCombiningAlgId="urn:oasis:names:tc:xacml:1.0:policy-combining-algorithm:first-applicable" Version="1.0">
  <xacml3:Target />
  <xacml3:Policy PolicyId="p" RuleCombiningAlgId="urn:oasis:names:tc:xacml:1.0:rule-combining-algorithm:first-applicable" Version="1.0">
    <xacml3:Target />
    <xacml3:Rule Effect="Permit" RuleId="https://sr.ht/~gheartsfield/a2x/alfa/ident/main/ps/p/r">
      <xacml3:Target />
      <xacml3:ObligationExpressions>
        <xacml3:ObligationExpression ObligationId="urn:example:oblig1" FulfillOn="Permit">
          <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
            <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-oblig-permit</xacml3:AttributeValue>
          </xacml3:AttributeAssignmentExpression>
          <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
            <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-oblig-permit" MustBePresent="true" />
          </xacml3:AttributeAssignmentExpression>
        </xacml3:ObligationExpression>
        <xacml3:ObligationExpression ObligationId="urn:example:oblig1" FulfillOn="Deny">
          <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
            <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-oblig-deny</xacml3:AttributeValue>
          </xacml3:AttributeAssignmentExpression>
          <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
            <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-oblig-deny" MustBePresent="true" />
          </xacml3:AttributeAssignmentExpression>
        </xacml3:ObligationExpression>
      </xacml3:ObligationExpressions>
      <xacml3:AdviceExpressions>
        <xacml3:AdviceExpression AdviceId="urn:example:advice1" AppliesTo="Permit">
          <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
            <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-advice-permit</xacml3:AttributeValue>
          </xacml3:AttributeAssignmentExpression>
          <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
            <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-advice-permit" MustBePresent="true" />
          </xacml3:AttributeAssignmentExpression>
        </xacml3:AdviceExpression>
        <xacml3:AdviceExpression AdviceId="urn:example:advice1" AppliesTo="Deny">
          <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
            <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-advice-deny</xacml3:AttributeValue>
          </xacml3:AttributeAssignmentExpression>
          <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
            <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-advice-deny" MustBePresent="true" />
          </xacml3:AttributeAssignmentExpression>
        </xacml3:AdviceExpression>
      </xacml3:AdviceExpressions>
    </xacml3:Rule>
    <xacml3:ObligationExpressions>
      <xacml3:ObligationExpression ObligationId="urn:example:oblig1" FulfillOn="Permit">
        <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
          <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-oblig-permit</xacml3:AttributeValue>
        </xacml3:AttributeAssignmentExpression>
        <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
          <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-oblig-permit" MustBePresent="true" />
        </xacml3:AttributeAssignmentExpression>
      </xacml3:ObligationExpression>
      <xacml3:ObligationExpression ObligationId="urn:example:oblig1" FulfillOn="Deny">
        <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
          <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-oblig-deny</xacml3:AttributeValue>
        </xacml3:AttributeAssignmentExpression>
        <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
          <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-oblig-deny" MustBePresent="true" />
        </xacml3:AttributeAssignmentExpression>
      </xacml3:ObligationExpression>
    </xacml3:ObligationExpressions>
    <xacml3:AdviceExpressions>
      <xacml3:AdviceExpression AdviceId="urn:example:advice1" AppliesTo="Permit">
        <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
          <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-advice-permit</xacml3:AttributeValue>
        </xacml3:AttributeAssignmentExpression>
        <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
          <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-advice-permit" MustBePresent="true" />
        </xacml3:AttributeAssignmentExpression>
      </xacml3:AdviceExpression>
      <xacml3:AdviceExpression AdviceId="urn:example:advice1" AppliesTo="Deny">
        <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
          <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-advice-deny</xacml3:AttributeValue>
        </xacml3:AttributeAssignmentExpression>
        <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
          <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-advice-deny" MustBePresent="true" />
        </xacml3:AttributeAssignmentExpression>
      </xacml3:AdviceExpression>
    </xacml3:AdviceExpressions>
  </xacml3:Policy>
  <xacml3:ObligationExpressions>
    <xacml3:ObligationExpression ObligationId="urn:example:oblig1" FulfillOn="Permit">
      <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
        <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-oblig-permit</xacml3:AttributeValue>
      </xacml3:AttributeAssignmentExpression>
      <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
        <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-oblig-permit" MustBePresent="true" />
      </xacml3:AttributeAssignmentExpression>
    </xacml3:ObligationExpression>
    <xacml3:ObligationExpression ObligationId="urn:example:oblig1" FulfillOn="Deny">
      <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
        <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-oblig-deny</xacml3:AttributeValue>
      </xacml3:AttributeAssignmentExpression>
      <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
        <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-oblig-deny" MustBePresent="true" />
      </xacml3:AttributeAssignmentExpression>
    </xacml3:ObligationExpression>
  </xacml3:ObligationExpressions>
  <xacml3:AdviceExpressions>
    <xacml3:AdviceExpression AdviceId="urn:example:advice1" AppliesTo="Permit">
      <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
        <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-advice-permit</xacml3:AttributeValue>
      </xacml3:AttributeAssignmentExpression>
      <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
        <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-advice-permit" MustBePresent="true" />
      </xacml3:AttributeAssignmentExpression>
    </xacml3:AdviceExpression>
    <xacml3:AdviceExpression AdviceId="urn:example:advice1" AppliesTo="Deny">
      <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:action:action-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:action">
        <xacml3:AttributeValue DataType="http://www.w3.org/2001/XMLSchema#string">foo-advice-deny</xacml3:AttributeValue>
      </xacml3:AttributeAssignmentExpression>
      <xacml3:AttributeAssignmentExpression AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource">
        <xacml3:AttributeDesignator AttributeId="urn:oasis:names:tc:xacml:1.0:resource:resource-id" Category="urn:oasis:names:tc:xacml:3.0:attribute-category:resource" DataType="http://www.w3.org/2001/XMLSchema#string" Issuer="bar-advice-deny" MustBePresent="true" />
      </xacml3:AttributeAssignmentExpression>
    </xacml3:AdviceExpression>
  </xacml3:AdviceExpressions>
</xacml3:PolicySet>"#
    );
}
