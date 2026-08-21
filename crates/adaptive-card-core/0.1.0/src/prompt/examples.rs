//! Build example showcase snippets from knowledge base entries.

use crate::knowledge::KnowledgeBase;
use std::fmt::Write as _;

/// Render the top-N examples matching a query as a system-prompt snippet.
#[must_use]
pub fn showcase(kb: &KnowledgeBase, query: &str, limit: usize) -> String {
    let results = kb.suggest(query, limit);
    if results.is_empty() {
        return String::new();
    }
    let mut out = String::from("\nRELEVANT EXAMPLES:\n");
    for r in results {
        if let Some(entry) = kb.by_id(&r.entry_id) {
            let _ = write!(
                out,
                "\n## {} ({})\n{}\n\n```json\n{}\n```\n",
                entry.title,
                entry.complexity_str(),
                entry.description,
                serde_json::to_string_pretty(&entry.card).unwrap_or_default()
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_kb_produces_empty_string() {
        let kb = KnowledgeBase::default();
        assert!(showcase(&kb, "anything", 3).is_empty());
    }
}
