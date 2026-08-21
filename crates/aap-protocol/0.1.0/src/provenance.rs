//! AAP Provenance — immutable origin record for agent-produced artifacts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::{KeyPair, sha256_of, signable, verify_signature};
use crate::errors::Result;

/// Provenance records the immutable origin of an agent-produced artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub aap_version: String,
    pub artifact_id: String,
    pub agent_id: String,
    pub action: String,
    pub input_hash: String,
    pub output_hash: String,
    pub authorization_id: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_artifact_id: Option<String>,
    pub signature: String,
}

impl Provenance {
    /// Create and sign provenance for a produced artifact.
    pub fn new(
        agent_id: &str,
        action: &str,
        input_data: &[u8],
        output_data: &[u8],
        authorization_id: &str,
        agent_kp: &KeyPair,
    ) -> Result<Self> {
        let mut prov = Self {
            aap_version: "0.1".into(),
            artifact_id: Uuid::new_v4().to_string(),
            agent_id: agent_id.into(),
            action: action.into(),
            input_hash: sha256_of(input_data),
            output_hash: sha256_of(output_data),
            authorization_id: authorization_id.into(),
            timestamp: Utc::now(),
            target: None,
            parent_artifact_id: None,
            signature: String::new(),
        };
        let v = serde_json::to_value(&prov)?;
        let data = signable(&v)?;
        prov.signature = agent_kp.sign(&data);
        Ok(prov)
    }

    /// Verify the signature against the agent's public key.
    pub fn verify(&self, agent_public_key_b64: &str) -> Result<()> {
        let v = serde_json::to_value(self)?;
        let data = signable(&v)?;
        verify_signature(agent_public_key_b64, &data, &self.signature)
    }
}
