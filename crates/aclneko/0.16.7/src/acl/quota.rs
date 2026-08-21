use serde::{Deserialize, Serialize};
use std::fmt;

const DEFAULT_AUDIT_CAPACITY: usize = 512;

/// AclQuota represents the quota information for the policy.
///
#[derive(Serialize, Deserialize)]
pub struct AclQuota {
    policy_memory: u32,
    audit_memory: u32,
    query_memory: u32,
    audit: Vec<AuditQuota>,
}

impl AclQuota {
    pub fn new() -> AclQuota {
        AclQuota {
            policy_memory: 0,
            audit_memory: 0,
            query_memory: 0,
            audit: Vec::with_capacity(DEFAULT_AUDIT_CAPACITY),
        }
    }
}

impl Clone for AclQuota {
    fn clone(&self) -> AclQuota {
        let mut audit = vec![];
        for a in &self.audit.clone() {
            audit.push(a.clone())
        }
        AclQuota {
            policy_memory: self.policy_memory,
            query_memory: self.query_memory,
            audit_memory: self.audit_memory,
            audit: self.audit.clone(),
        }
    }
}

impl fmt::Debug for AclQuota {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AclQuota")
            .field(&self.policy_memory)
            .field(&self.audit_memory)
            .field(&self.query_memory)
            .field(&self.audit)
            .finish()
    }
}

impl Drop for AclQuota {
    fn drop(&mut self) {
        self.policy_memory = 0;
        self.audit_memory = 0;
        self.query_memory = 0;
        self.audit.clear();
    }
}

/// AuditQuota holds quota information for each audit definition.
///
#[derive(Serialize, Deserialize)]
pub struct AuditQuota {
    allowed: u32,
    denied: u32,
    unmatched: u32,
}

impl Clone for AuditQuota {
    fn clone(&self) -> AuditQuota {
        AuditQuota {
            allowed: self.allowed,
            denied: self.denied,
            unmatched: self.unmatched,
        }
    }
}

impl Drop for AuditQuota {
    fn drop(&mut self) {
        self.allowed = 0;
        self.denied = 0;
        self.unmatched = 0;
    }
}

impl fmt::Debug for AuditQuota {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AuditQuota")
            .field(&self.allowed)
            .field(&self.denied)
            .field(&self.unmatched)
            .finish()
    }
}
