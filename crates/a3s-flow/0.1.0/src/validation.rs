//! Pre-flight flow validation.
//!
//! [`ValidationIssue`] is the unit of output from [`FlowEngine::validate`].
//! Each issue describes one structural problem found in a flow definition.
//!
//! [`FlowEngine::validate`]: crate::engine::FlowEngine::validate

/// A structural problem found in a flow definition by [`FlowEngine::validate`].
///
/// [`FlowEngine::validate`]: crate::engine::FlowEngine::validate
#[derive(Debug, Clone, PartialEq, Eq)]
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
