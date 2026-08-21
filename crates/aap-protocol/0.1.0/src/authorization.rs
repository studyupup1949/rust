//! AAP Authorization — human-granted permission for an agent to act.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::{KeyPair, signable, verify_signature};
use crate::errors::{AAPError, Result};

/// AAP authorization levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Observe    = 0,
    Suggest    = 1,
    Assisted   = 2,
    Supervised = 3,
    Autonomous = 4,
}

impl Level {
    pub fn name(&self) -> &'static str {
        match self {
            Level::Observe    => "observe",
            Level::Suggest    => "suggest",
            Level::Assisted   => "assisted",
            Level::Supervised => "supervised",
            Level::Autonomous => "autonomous",
        }
    }

    pub fn as_u8(&self) -> u8 { *self as u8 }
}

/// Maximum level allowed for physical nodes.
/// PHYSICAL WORLD RULE: enforced in code. Not configurable.
const PHYSICAL_MAX_LEVEL: Level = Level::Supervised;

/// Authorization token granting an agent permission to act.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorization {
    pub aap_version: String,
    pub agent_id: String,
    pub level: u8,
    pub level_name: String,
    pub scope: Vec<String>,
    pub physical: bool,
    pub granted_by: String,
    pub granted_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    pub session_id: String,
    pub signature: String,
    #[serde(skip)]
    revoked: bool,
}

impl Authorization {
    /// Create and sign a new Authorization.
    /// Enforces the Physical World Rule — returns `Err(PhysicalWorldViolation)` if violated.
    pub fn new(
        agent_id: &str,
        level: Level,
        scope: Vec<String>,
        physical: bool,
        supervisor_kp: &KeyPair,
        supervisor_did: &str,
    ) -> Result<Self> {
        // PHYSICAL WORLD RULE — enforced here. Cannot be bypassed.
        if physical && level > PHYSICAL_MAX_LEVEL {
            return Err(AAPError::PhysicalWorldViolation { agent_id: agent_id.into() });
        }

        let mut auth = Self {
            aap_version: "0.1".into(),
            agent_id: agent_id.into(),
            level: level.as_u8(),
            level_name: level.name().into(),
            scope,
            physical,
            granted_by: supervisor_did.into(),
            granted_at: Utc::now(),
            expires_at: None,
            session_id: Uuid::new_v4().to_string(),
            signature: String::new(),
            revoked: false,
        };

        let v = serde_json::to_value(&auth)?;
        let data = signable(&v)?;
        auth.signature = supervisor_kp.sign(&data);
        Ok(auth)
    }

    pub fn revoke(&mut self) { self.revoked = true; }
    pub fn is_revoked(&self) -> bool { self.revoked }
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|e| Utc::now() > e).unwrap_or(false)
    }
    pub fn is_valid(&self) -> bool { !self.is_revoked() && !self.is_expired() }

    pub fn check(&self) -> Result<()> {
        if self.is_revoked() {
            return Err(AAPError::Revocation { id: self.session_id.clone() });
        }
        if self.is_expired() {
            return Err(AAPError::Revocation {
                id: format!("{} (expired)", self.session_id),
            });
        }
        Ok(())
    }

    pub fn verify(&self, supervisor_public_key_b64: &str) -> Result<()> {
        let v = serde_json::to_value(self)?;
        let data = signable(&v)?;
        verify_signature(supervisor_public_key_b64, &data, &self.signature)
    }
}
