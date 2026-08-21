//! Keyword-based knowledge entry selector with relevance scoring.
//!
//! Scoring model:
//! - exact tag match: +10
//! - title token match: +5
//! - `use_case` token match: +3
//! - description token match: +1

use super::format::KnowledgeEntry;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SuggestResult {
    pub entry_id: String,
    pub score: f32,
    pub match_reason: String,
}

/// Rank entries by relevance to a query.
#[must_use]
pub fn suggest(entries: &[KnowledgeEntry], query: &str, limit: usize) -> Vec<SuggestResult> {
    let tokens: Vec<String> = tokenize(query);
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<SuggestResult> = entries
        .iter()
        .filter_map(|e| {
            let (score, reason) = score_entry(e, &tokens);
            if score > 0.0 {
                Some(SuggestResult {
                    entry_id: e.id.clone(),
                    score,
                    match_reason: reason,
                })
            } else {
                None
            }
        })
        .collect();

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(limit);
    scored
}

fn tokenize(s: &str) -> Vec<String> {
    s.to_ascii_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() >= 2)
        .map(str::to_string)
        .collect()
}

fn score_entry(entry: &KnowledgeEntry, tokens: &[String]) -> (f32, String) {
    let mut score = 0.0_f32;
    let mut reasons: Vec<String> = Vec::new();

    for tok in tokens {
        // Tag exact match
        if entry.tags.iter().any(|t| t.to_ascii_lowercase() == *tok) {
            score += 10.0;
            reasons.push(format!("tag:{tok}"));
            continue;
        }
        // Title contains token
        if tokenize(&entry.title).contains(tok) {
            score += 5.0;
            reasons.push(format!("title:{tok}"));
            continue;
        }
        // Use case contains token
        if entry.use_cases.iter().any(|uc| tokenize(uc).contains(tok)) {
            score += 3.0;
            reasons.push(format!("use_case:{tok}"));
            continue;
        }
        // Description contains token
        if tokenize(&entry.description).contains(tok) {
            score += 1.0;
            reasons.push(format!("desc:{tok}"));
        }
    }

    (score, reasons.join(", "))
}

#[cfg(test)]
mod tests {
    use super::super::format::Complexity;
    use super::*;
    use crate::types::Host;
    use serde_json::json;

    fn make(id: &str, title: &str, tags: Vec<&str>) -> KnowledgeEntry {
        KnowledgeEntry {
            id: id.to_string(),
            title: title.to_string(),
            description: String::new(),
            category: "test".to_string(),
            tags: tags.into_iter().map(String::from).collect(),
            use_cases: vec![],
            host_targets: vec![Host::Teams],
            complexity: Complexity::Basic,
            card: json!({}),
            notes: String::new(),
        }
    }

    #[test]
    fn tag_match_ranks_highest() {
        let entries = vec![
            make("a", "Generic", vec!["foo"]),
            make("b", "Expense Approval", vec!["expense", "approval"]),
        ];
        let results = suggest(&entries, "expense approval", 5);
        assert_eq!(results[0].entry_id, "b");
        assert!(results[0].score >= 20.0);
    }

    #[test]
    fn title_match_lower_than_tag() {
        let entries = vec![
            make("a", "Expense Report", vec!["finance"]),
            make("b", "Profile", vec!["expense"]),
        ];
        let results = suggest(&entries, "expense", 5);
        assert_eq!(results[0].entry_id, "b");
    }

    #[test]
    fn limit_truncates() {
        let entries: Vec<_> = (0..10)
            .map(|i| make(&format!("e{i}"), "test", vec!["test"]))
            .collect();
        let results = suggest(&entries, "test", 3);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn empty_query_returns_empty() {
        let entries = vec![make("a", "x", vec!["y"])];
        assert!(suggest(&entries, "", 5).is_empty());
    }
}
