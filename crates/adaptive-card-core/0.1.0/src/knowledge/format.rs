//! Knowledge base entry format definitions.

use crate::types::Host;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    Basic,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub use_cases: Vec<String>,
    pub host_targets: Vec<Host>,
    pub complexity: Complexity,
    pub card: Value,
    #[serde(default)]
    pub notes: String,
}

impl KnowledgeEntry {
    /// Human-readable complexity label for prompt injection.
    #[must_use]
    pub fn complexity_str(&self) -> &'static str {
        match self.complexity {
            Complexity::Basic => "basic",
            Complexity::Intermediate => "intermediate",
            Complexity::Advanced => "advanced",
        }
    }
}

/// Manifest file format at `data/knowledge-base/_manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeManifest {
    pub version: String,
    pub entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub id: String,
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn entry_roundtrip() {
        let entry = KnowledgeEntry {
            id: "finance/expense".to_string(),
            title: "Expense".to_string(),
            description: "x".to_string(),
            category: "finance".to_string(),
            tags: vec!["expense".to_string()],
            use_cases: vec![],
            host_targets: vec![Host::Teams],
            complexity: Complexity::Advanced,
            card: json!({ "type": "AdaptiveCard" }),
            notes: String::new(),
        };
        let json = serde_json::to_value(&entry).unwrap();
        let back: KnowledgeEntry = serde_json::from_value(json).unwrap();
        assert_eq!(back.id, entry.id);
    }
}
