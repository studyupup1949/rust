//! Explained evaluation results for audit logging and diagnostics.

use crate::decision::Decision;
use crate::rule::RuleType;

/// A rule that matched during explained evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct RuleMatch {
    /// The stable identifier from the matched rule, if set.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub id: Option<String>,
    /// The rule name.
    pub name: String,
    /// Whether this is an allow or deny rule.
    pub rule_type: RuleType,
    /// Whether the rule is temporal.
    pub temporal: bool,
}

/// Result of an explained evaluation.
///
/// Contains the final decision plus the rules that matched and contributed
/// to it. Intended for audit logging and interactive `test-request` commands,
/// not hot-path evaluation.
///
/// # Matching semantics
///
/// - For `Deny`: all matching deny rules are collected (no short-circuit).
/// - For `Allow`: the first matching allow rule is recorded.
/// - For default `Deny` (no rules matched): `matched_rules` is empty.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct ExplainedDecision {
    /// The final decision.
    pub decision: Decision,
    /// Rules that matched the request and contributed to the decision.
    pub matched_rules: Vec<RuleMatch>,
    /// Total number of rules evaluated (for diagnostics).
    pub rules_evaluated: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_match_debug() {
        let rm = RuleMatch {
            id: None,
            name: "test-rule".to_string(),
            rule_type: RuleType::Allow,
            temporal: false,
        };
        let debug = format!("{:?}", rm);
        assert!(debug.contains("test-rule"));
    }

    #[test]
    fn test_explained_decision_debug() {
        let ed = ExplainedDecision {
            decision: Decision::Allow,
            matched_rules: vec![],
            rules_evaluated: 0,
        };
        assert!(ed.decision.is_allowed());
        assert!(ed.matched_rules.is_empty());
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_explained_decision_serde_round_trip() {
        let ed = ExplainedDecision {
            decision: Decision::Allow,
            matched_rules: vec![
                RuleMatch {
                    id: Some("uuid-1".to_string()),
                    name: "allow-read".to_string(),
                    rule_type: RuleType::Allow,
                    temporal: false,
                },
                RuleMatch {
                    id: None,
                    name: "temporal-deny".to_string(),
                    rule_type: RuleType::Deny,
                    temporal: true,
                },
            ],
            rules_evaluated: 5,
        };

        let json = serde_json::to_string(&ed).unwrap();
        let deserialized: ExplainedDecision = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.decision, Decision::Allow);
        assert_eq!(deserialized.matched_rules.len(), 2);
        assert_eq!(deserialized.matched_rules[0].name, "allow-read");
        assert!(!deserialized.matched_rules[0].temporal);
        assert_eq!(deserialized.matched_rules[1].name, "temporal-deny");
        assert!(deserialized.matched_rules[1].temporal);
        assert_eq!(deserialized.rules_evaluated, 5);
    }
}
