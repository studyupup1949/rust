//! AAP Identity — cryptographic identity for an AI agent.
//! Address format: `aap://org/type/name@semver`

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::crypto::{KeyPair, signable, verify_signature};
use crate::errors::{AAPError, Result};

/// An AI agent's cryptographic identity.
/// Signed by a human supervisor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub aap_version: String,
    pub id: String,
    pub public_key: String,
    pub parent: String,
    pub scope: Vec<String>,
    pub issued_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked: bool,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, String>,
    pub signature: String,
}

impl Identity {
    /// Create and sign a new Identity.
    /// `parent_kp` is the human supervisor's keypair — they sign the agent's identity.
    pub fn new(
        id: &str,
        scope: Vec<String>,
        agent_kp: &KeyPair,
        parent_kp: &KeyPair,
        parent_did: &str,
    ) -> Result<Self> {
        // Validate id format
        let id_re = Regex::new(r"^aap://[a-z0-9\-\.]+/[a-z0-9\-]+/[a-z0-9\-\.]+@\d+\.\d+\.\d+$")
            .unwrap();
        if !id_re.is_match(id) {
            return Err(AAPError::Validation {
                field: "id".into(),
                message: format!("invalid format: {id:?} — expected aap://org/type/name@semver"),
            });
        }
        if scope.is_empty() {
            return Err(AAPError::Validation {
                field: "scope".into(),
                message: "must contain at least one item".into(),
            });
        }
        let scope_re = Regex::new(r"^[a-z]+:[a-z0-9_\-\*]+$").unwrap();
        for s in &scope {
            if !scope_re.is_match(s) {
                return Err(AAPError::Validation {
                    field: "scope".into(),
                    message: format!("invalid item {s:?} — expected verb:resource"),
                });
            }
        }

        let mut identity = Self {
            aap_version: "0.1".into(),
            id: id.into(),
            public_key: agent_kp.public_key_b64(),
            parent: parent_did.into(),
            scope,
            issued_at: Utc::now(),
            expires_at: None,
            revoked: false,
            metadata: HashMap::new(),
            signature: String::new(),
        };

        let v = serde_json::to_value(&identity)?;
        let data = signable(&v)?;
        identity.signature = parent_kp.sign(&data);
        Ok(identity)
    }

    /// Check if `action` is within this identity's scope.
    pub fn allows_action(&self, action: &str) -> bool {
        let (verb, resource) = action.split_once(':').unwrap_or((action, ""));
        self.scope.iter().any(|s| {
            let (sv, sr) = s.split_once(':').unwrap_or((s.as_str(), ""));
            sv == verb && (sr == "*" || sr == resource)
        })
    }

    /// Returns true if this identity has passed its expiry.
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|e| Utc::now() > e).unwrap_or(false)
    }

    /// Verify the signature against the parent's public key.
    pub fn verify(&self, parent_public_key_b64: &str) -> Result<()> {
        if self.revoked {
            return Err(AAPError::Revocation { id: self.id.clone() });
        }
        let v = serde_json::to_value(self)?;
        let data = signable(&v)?;
        verify_signature(parent_public_key_b64, &data, &self.signature)
    }
}
