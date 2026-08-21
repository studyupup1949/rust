//! Pre-flight flow validation.
//!
//! [`ValidationIssue`] is the unit of output from [`FlowEngine::validate`].
//! Each issue describes one structural problem found in a flow definition.
//!
//! [`FlowEngine::validate`]: crate::engine::FlowEngine::validate

use serde::{Deserialize, Serialize};

/// A structural problem found in a flow definition by [`FlowEngine::validate`].
///
/// [`FlowEngine::validate`]: crate::engine::FlowEngine::validate
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// The node ID where the issue was found, or `None` for flow-level issues.
    pub node_id: Option<String>,
    /// Human-readable description of the problem.
    pub message: String,
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.node_id {
            Some(id) => write!(f, "node '{}': {}", id, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_with_node_id() {
        let issue = ValidationIssue {
            node_id: Some("fetch".into()),
            message: "connection refused".into(),
        };
        assert_eq!(format!("{}", issue), "node 'fetch': connection refused");
    }

    #[test]
    fn display_without_node_id() {
        let issue = ValidationIssue {
            node_id: None,
            message: "missing start node".into(),
        };
        assert_eq!(format!("{}", issue), "missing start node");
    }

    #[test]
    fn debug_round_trip() {
        let issue = ValidationIssue {
            node_id: Some("a".into()),
            message: "b".into(),
        };
        let debug = format!("{:?}", issue);
        assert!(debug.contains("ValidationIssue"));
        assert!(debug.contains("a"));
        assert!(debug.contains("b"));
    }

    #[test]
    fn serde_round_trip() {
        let issue = ValidationIssue {
            node_id: Some("n".into()),
            message: "m".into(),
        };
        let json = serde_json::to_string(&issue).unwrap();
        let back: ValidationIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(back.node_id, issue.node_id);
        assert_eq!(back.message, issue.message);
    }

    #[test]
    fn clone_is_equal() {
        let issue = ValidationIssue {
            node_id: Some("x".into()),
            message: "y".into(),
        };
        let cloned = issue.clone();
        assert_eq!(cloned, issue);
    }
}
