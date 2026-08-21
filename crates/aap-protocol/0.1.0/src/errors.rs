//! AAP error types — all errors have codes AAP-001 through AAP-006.

use thiserror::Error;

/// All AAP errors implement this trait.
#[derive(Debug, Error)]
pub enum AAPError {
    /// AAP-001: Schema validation failed.
    #[error("AAP-001: validation error on field '{field}': {message}")]
    Validation { field: String, message: String },

    /// AAP-002: Ed25519 signature verification failed.
    #[error("AAP-002: signature error: {0}")]
    Signature(String),

    /// AAP-003: Physical World Rule — Level 4 forbidden for physical nodes.
    #[error(
        "AAP-003: Physical World Rule: Autonomous (Level 4) is forbidden \
         for physical agent '{agent_id}'. Maximum level is Supervised (Level 3). \
         This rule is not configurable."
    )]
    PhysicalWorldViolation { agent_id: String },

    /// AAP-004: Action is outside the agent's authorized scope.
    #[error("AAP-004: action '{action}' is not in scope for agent '{agent_id}'")]
    Scope { action: String, agent_id: String },

    /// AAP-005: Identity or authorization has been revoked.
    #[error("AAP-005: '{id}' has been revoked")]
    Revocation { id: String },

    /// AAP-006: Audit chain integrity broken.
    #[error("AAP-006: audit chain broken at entry '{entry_id}'")]
    Chain { entry_id: String },

    /// Serialization error (internal).
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, AAPError>;
