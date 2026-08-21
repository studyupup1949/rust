//! Access control decision types.

/// Access control decision result.
///
/// Returned by the evaluation pipeline after checking a request against rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Decision {
    /// Access is allowed
    Allow,

    /// Access is denied (either explicit deny rule or no matching allow rule)
    Deny,
}

impl Decision {
    /// Check if this decision allows access.
    #[inline]
    pub fn is_allowed(&self) -> bool {
        matches!(self, Decision::Allow)
    }

    /// Check if this decision denies access.
    #[inline]
    pub fn is_denied(&self) -> bool {
        matches!(self, Decision::Deny)
    }
}

impl std::fmt::Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decision::Allow => f.write_str("Allow"),
            Decision::Deny => f.write_str("Deny"),
        }
    }
}

impl Default for Decision {
    /// Default decision is `Deny` (secure by default).
    fn default() -> Self {
        Decision::Deny
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_allow() {
        let d = Decision::Allow;
        assert!(d.is_allowed());
        assert!(!d.is_denied());
    }

    #[test]
    fn test_decision_deny() {
        let d = Decision::Deny;
        assert!(!d.is_allowed());
        assert!(d.is_denied());
    }

    #[test]
    fn test_decision_default() {
        let d = Decision::default();
        assert_eq!(d, Decision::Deny);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_decision_serde_round_trip() {
        let allow = Decision::Allow;
        let json = serde_json::to_string(&allow).unwrap();
        let deserialized: Decision = serde_json::from_str(&json).unwrap();
        assert_eq!(allow, deserialized);

        let deny = Decision::Deny;
        let json = serde_json::to_string(&deny).unwrap();
        let deserialized: Decision = serde_json::from_str(&json).unwrap();
        assert_eq!(deny, deserialized);
    }
}
