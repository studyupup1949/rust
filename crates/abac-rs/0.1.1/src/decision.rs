//! Access control decision types.

/// Access control decision result.
///
/// Returned by the evaluation pipeline after checking a request against rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}
