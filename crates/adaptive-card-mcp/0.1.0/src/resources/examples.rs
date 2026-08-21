//! `ac://examples[/{id}]` — knowledge base summaries and full entries.

use adaptive_card_core::KnowledgeBase;
use serde_json::{Value, json};

#[must_use]
pub fn all_body(kb: &KnowledgeBase) -> Value {
    json!(
        kb.all()
            .iter()
            .map(|e| json!({
                "id": e.id,
                "title": e.title,
                "category": e.category,
                "complexity": format!("{:?}", e.complexity).to_lowercase(),
            }))
            .collect::<Vec<_>>()
    )
}

#[must_use]
pub fn one_body(kb: &KnowledgeBase, id: &str) -> Option<Value> {
    kb.by_id(id).and_then(|e| serde_json::to_value(e).ok())
}
