//! AAP AuditChain — tamper-evident append-only log of all agent actions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use uuid::Uuid;

use crate::crypto::{hash_entry, KeyPair, signable};
use crate::errors::{AAPError, Result};

/// The result of an agent action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditResult {
    Success,
    Failure,
    Blocked,
    Revoked,
}

/// A single tamper-evident entry in the audit chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub aap_version: String,
    pub entry_id: String,
    pub prev_hash: String,
    pub agent_id: String,
    pub action: String,
    pub result: AuditResult,
    pub timestamp: DateTime<Utc>,
    pub provenance_id: String,
    pub authorization_level: u8,
    pub physical: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_detail: Option<String>,
    pub signature: String,
}

/// Tamper-evident append-only audit log.
pub struct AuditChain {
    entries: Vec<AuditEntry>,
    storage_path: Option<String>,
}

impl AuditChain {
    /// Create a new in-memory AuditChain.
    pub fn new() -> Self {
        Self { entries: Vec::new(), storage_path: None }
    }

    /// Create an AuditChain backed by a JSONL file.
    /// Loads existing entries if the file exists.
    pub fn with_storage(path: impl AsRef<Path>) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let mut entries = Vec::new();

        if path.as_ref().exists() {
            let f = File::open(&path)
                .map_err(|e| AAPError::Signature(format!("cannot open audit file: {e}")))?;
            for line in BufReader::new(f).lines() {
                let line = line.map_err(|e| AAPError::Signature(e.to_string()))?;
                if !line.trim().is_empty() {
                    let entry: AuditEntry = serde_json::from_str(&line)?;
                    entries.push(entry);
                }
            }
        }

        Ok(Self { entries, storage_path: Some(path_str) })
    }

    /// Append a new signed entry to the chain.
    pub fn append(
        &mut self,
        agent_id: &str,
        action: &str,
        result: AuditResult,
        provenance_id: &str,
        agent_kp: &KeyPair,
        authorization_level: u8,
        physical: bool,
    ) -> Result<&AuditEntry> {
        let prev_hash = self.last_hash();

        let mut entry = AuditEntry {
            aap_version: "0.1".into(),
            entry_id: Uuid::new_v4().to_string(),
            prev_hash,
            agent_id: agent_id.into(),
            action: action.into(),
            result,
            timestamp: Utc::now(),
            provenance_id: provenance_id.into(),
            authorization_level,
            physical,
            result_detail: None,
            signature: String::new(),
        };

        let v = serde_json::to_value(&entry)?;
        let data = signable(&v)?;
        entry.signature = agent_kp.sign(&data);

        if let Some(ref path) = self.storage_path {
            let mut f = OpenOptions::new()
                .create(true).append(true).open(path)
                .map_err(|e| AAPError::Signature(format!("cannot write audit: {e}")))?;
            let line = serde_json::to_string(&entry)?;
            writeln!(f, "{}", line)
                .map_err(|e| AAPError::Signature(e.to_string()))?;
        }

        self.entries.push(entry);
        Ok(self.entries.last().unwrap())
    }

    /// Verify the hash chain integrity.
    /// Returns `(valid, entries_checked, broken_at_entry_id)`.
    pub fn verify(&self) -> (bool, usize, Option<String>) {
        let mut prev_hash = "genesis".to_string();

        for (i, entry) in self.entries.iter().enumerate() {
            if entry.prev_hash != prev_hash {
                return (false, i, Some(entry.entry_id.clone()));
            }
            let v = serde_json::to_value(entry).unwrap_or_default();
            prev_hash = hash_entry(&v);
        }
        (true, self.entries.len(), None)
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn entries(&self) -> &[AuditEntry] { &self.entries }

    fn last_hash(&self) -> String {
        match self.entries.last() {
            None => "genesis".to_string(),
            Some(e) => {
                let v = serde_json::to_value(e).unwrap_or_default();
                hash_entry(&v)
            }
        }
    }
}

impl Default for AuditChain {
    fn default() -> Self { Self::new() }
}
