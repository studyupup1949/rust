use crate::memory::store::MemoryStore;
use crate::swarm::types::*;
use chrono::Utc;
use uuid::Uuid;

pub struct PatternEngine {
    store: std::sync::Arc<MemoryStore>,
}

impl PatternEngine {
    pub fn new(store: std::sync::Arc<MemoryStore>) -> Self {
        PatternEngine { store }
    }

    pub async fn find_or_create_pattern(
        &self,
        issue: &Issue,
        solution: &str,
        confidence: f64,
        execution_time_ms: u64,
    ) -> Pattern {
        let existing = self.store.find_similar_pattern(&issue.signature, &issue.domain).await;

        if let Some(mut pattern) = existing {
            pattern.occurrences += 1;
            pattern.last_seen = Utc::now();
            pattern.confidence = (pattern.confidence * 0.7) + (confidence * 0.3);
            pattern.avg_execution_time_ms = (pattern.avg_execution_time_ms
                * (pattern.occurrences as u64 - 1)
                + execution_time_ms)
                / pattern.occurrences as u64;
            pattern.solution_description = solution.to_string();
            self.store.store_pattern(&pattern).await;
            pattern
        } else {
            let pattern = Pattern {
                id: Uuid::new_v4().to_string(),
                name: issue.title.clone(),
                description: issue.description.clone(),
                domain: issue.domain.clone(),
                indicators: vec![issue.signature.clone()],
                solution_description: solution.to_string(),
                confidence,
                occurrences: 1,
                first_seen: Utc::now(),
                last_seen: Utc::now(),
                avg_execution_time_ms: execution_time_ms,
            };
            self.store.store_pattern(&pattern).await;
            pattern
        }
    }

    pub async fn match_issue_to_pattern(&self, issue: &Issue) -> Option<Pattern> {
        self.store.find_similar_pattern(&issue.signature, &issue.domain).await
    }
}
