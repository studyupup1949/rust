use crate::memory::store::MemoryStore;
use crate::swarm::types::*;
use chrono::Utc;
use uuid::Uuid;

pub struct PredictionEngine {
    store: std::sync::Arc<MemoryStore>,
}

impl PredictionEngine {
    pub fn new(store: std::sync::Arc<MemoryStore>) -> Self {
        PredictionEngine { store }
    }

    pub async fn generate_predictions(&self, agent_name: &str) -> Vec<Prediction> {
        let patterns = self.store.get_patterns(Some(agent_name)).await;
        let mut predictions = Vec::new();

        for pattern in patterns {
            if pattern.confidence >= 0.7 && pattern.occurrences >= 3 {
                let prediction = Prediction {
                    id: Uuid::new_v4().to_string(),
                    agent_name: agent_name.to_string(),
                    predicted_issue: format!("Likely recurrence of: {}", pattern.name),
                    description: format!(
                        "Pattern '{}' has occurred {} times with {}% confidence. Similar conditions detected.",
                        pattern.name, pattern.occurrences, (pattern.confidence * 100.0) as u32
                    ),
                    confidence: pattern.confidence * 0.9,
                    time_until_expected: format!("{} hours", (24.0 * (1.0 - pattern.confidence)) as u32 + 1),
                    suggested_action: pattern.solution_description.clone(),
                    based_on_pattern: Some(pattern.id.clone()),
                    created_at: Utc::now(),
                    status: PredictionStatus::Active,
                };

                self.store.store_prediction(&prediction).await;
                predictions.push(prediction);
            }
        }

        predictions
    }

    pub async fn check_trend_prediction(
        &self,
        agent_name: &str,
        metric_name: &str,
        current_value: f64,
        threshold: f64,
        rate_of_change: f64,
    ) -> Option<Prediction> {
        // Predict when value will breach threshold (don't predict if already breached or not rising)
        if current_value >= threshold || rate_of_change <= 0.0 {
            return None;
        }

        let time_to_threshold = ((threshold - current_value) / rate_of_change).abs() as u64;
        if time_to_threshold > 72 {
            return None;
        }

        let prediction = Prediction {
            id: Uuid::new_v4().to_string(),
            agent_name: agent_name.to_string(),
            predicted_issue: format!("{} will exceed threshold", metric_name),
            description: format!(
                "{} is at {:.1} (threshold: {:.1}) trending at {:.1}/hour. Expected to breach threshold in ~{} hours.",
                metric_name, current_value, threshold, rate_of_change, time_to_threshold
            ),
            confidence: 0.75_f64.min(1.0 - (time_to_threshold as f64 / 72.0)),
            time_until_expected: format!("~{} hours", time_to_threshold),
            suggested_action: format!("Investigate {} trend before it reaches threshold", metric_name),
            based_on_pattern: None,
            created_at: Utc::now(),
            status: PredictionStatus::Active,
        };

        self.store.store_prediction(&prediction).await;
        Some(prediction)
    }
}
