// ABOUTME: Alert engine for storing, deduplicating, and managing anomaly alerts
// ABOUTME: Handles confidence scoring, dedupe windows, and broadcasting via WebSocket

#![allow(dead_code)]

use crate::error::{Error, Result};
use crate::model::{AnomalyCandidate, AnomalyDetection, AnomalyType};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tokio::sync::broadcast;

/// Alert ready to be stored in database
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Alert {
    pub ts_ms: i64,
    pub hex: String,
    pub anomaly_type: AnomalyType,
    pub subtype: String,
    pub confidence: f64,
    pub details_json: Option<String>,
}

impl Alert {
    #[allow(dead_code)]
    pub fn from_candidate(candidate: AnomalyCandidate) -> Self {
        Self {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            hex: candidate.hex,
            anomaly_type: candidate.anomaly_type,
            subtype: candidate.subtype,
            confidence: candidate.confidence,
            details_json: candidate.details.map(|d| d.to_string()),
        }
    }

    /// Generate dedupe signature for windowing
    pub fn dedupe_signature(&self) -> String {
        format!("{}:{}:{}", self.hex, self.anomaly_type, self.subtype)
    }
}

/// Alert broadcaster for WebSocket clients
#[derive(Debug, Clone)]
pub struct AlertBroadcaster {
    sender: broadcast::Sender<Alert>,
}

impl AlertBroadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    pub fn broadcast(&self, alert: Alert) -> Result<()> {
        self.sender.send(alert).map_err(|_| Error::Internal {
            message: "Failed to broadcast alert".to_string(),
        })?;
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Alert> {
        self.sender.subscribe()
    }
}

/// Alert manager with dedupe tracking
#[derive(Debug)]
pub struct AlertManager {
    pool: SqlitePool,
    broadcaster: AlertBroadcaster,
    recent_alerts: HashMap<String, i64>, // signature -> last_ts_ms
    dedupe_window_ms: i64,
}

impl AlertManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            broadcaster: AlertBroadcaster::new(),
            recent_alerts: HashMap::new(),
            dedupe_window_ms: 5 * 60 * 1000, // 5 minutes
        }
    }

    pub fn broadcaster(&self) -> &AlertBroadcaster {
        &self.broadcaster
    }

    /// Record an alert with deduplication
    pub async fn record_alert(&mut self, alert: Alert) -> Result<Option<i64>> {
        let now = chrono::Utc::now().timestamp_millis();
        let signature = alert.dedupe_signature();

        // Check if we've seen this alert recently
        if let Some(&last_ts) = self.recent_alerts.get(&signature) {
            if now - last_ts < self.dedupe_window_ms {
                tracing::debug!(
                    "Skipping duplicate alert: {} (last seen {}ms ago)",
                    signature,
                    now - last_ts
                );
                return Ok(None);
            }
        }

        // Clean old entries periodically
        if self.recent_alerts.len() > 1000 {
            self.cleanup_old_signatures(now);
        }

        // Record the alert
        let alert_id = self.insert_alert(&alert).await?;

        // Update dedupe tracking
        self.recent_alerts.insert(signature, now);

        // Broadcast to WebSocket clients
        if let Err(e) = self.broadcaster.broadcast(alert.clone()) {
            tracing::warn!("Failed to broadcast alert: {}", e);
        }

        Ok(Some(alert_id))
    }

    /// Insert alert into database
    async fn insert_alert(&self, alert: &Alert) -> Result<i64> {
        let anomaly_type_str = alert.anomaly_type.to_string();
        let result = sqlx::query(
            r#"
            INSERT INTO anomaly_detections (ts_ms, hex, anomaly_type, confidence, details_json, reviewed)
            VALUES (?, ?, ?, ?, ?, 0)
            "#
        )
        .bind(alert.ts_ms)
        .bind(&alert.hex)
        .bind(anomaly_type_str)
        .bind(alert.confidence)
        .bind(&alert.details_json)
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(result.last_insert_rowid())
    }

    /// Clean up old dedupe signatures
    fn cleanup_old_signatures(&mut self, now: i64) {
        let cutoff = now - self.dedupe_window_ms;
        self.recent_alerts.retain(|_, &mut ts| ts >= cutoff);
        tracing::debug!(
            "Cleaned up old alert signatures, {} remaining",
            self.recent_alerts.len()
        );
    }

    /// Get alerts with filtering
    pub async fn get_alerts(
        &self,
        anomaly_type: Option<AnomalyType>,
        since_ms: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<AnomalyDetection>> {
        let limit = limit.unwrap_or(100).min(1000);
        let since_ms = since_ms.unwrap_or(0);

        // Build dynamic query
        let (_query_str, rows) = if let Some(atype) = anomaly_type {
            let atype_str = atype.to_string();
            let query_str = "SELECT id, ts_ms, hex, anomaly_type, confidence, details_json, reviewed FROM anomaly_detections WHERE ts_ms >= ? AND anomaly_type = ? ORDER BY ts_ms DESC LIMIT ?";
            let rows = sqlx::query(query_str)
                .bind(since_ms)
                .bind(atype_str)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
                .map_err(Error::Database)?;
            (query_str, rows)
        } else {
            let query_str = "SELECT id, ts_ms, hex, anomaly_type, confidence, details_json, reviewed FROM anomaly_detections WHERE ts_ms >= ? ORDER BY ts_ms DESC LIMIT ?";
            let rows = sqlx::query(query_str)
                .bind(since_ms)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
                .map_err(Error::Database)?;
            (query_str, rows)
        };

        // Convert rows to AnomalyDetection structs
        let mut results = Vec::new();
        for row in rows {
            use sqlx::Row;

            let anomaly_type_str: String = row.try_get("anomaly_type").map_err(Error::Database)?;
            let anomaly_type =
                anomaly_type_str
                    .parse::<AnomalyType>()
                    .map_err(|_| Error::Internal {
                        message: format!("Invalid anomaly type in database: {}", anomaly_type_str),
                    })?;

            let reviewed_int: i64 = row.try_get("reviewed").map_err(Error::Database)?;

            results.push(AnomalyDetection {
                id: row.try_get("id").map_err(Error::Database)?,
                ts_ms: row.try_get("ts_ms").map_err(Error::Database)?,
                hex: row.try_get("hex").map_err(Error::Database)?,
                anomaly_type,
                confidence: row.try_get("confidence").map_err(Error::Database)?,
                details_json: row.try_get("details_json").map_err(Error::Database)?,
                reviewed: reviewed_int != 0,
            });
        }

        Ok(results)
    }
}

/// Calculate confidence score for different anomaly types
pub fn calculate_confidence(
    anomaly_type: AnomalyType,
    subtype: &str,
    _details: Option<&serde_json::Value>,
) -> f64 {
    match (anomaly_type, subtype) {
        (AnomalyType::Temporal, "rapid_transmission") => 0.9,
        (AnomalyType::Temporal, "burst_after_silence") => 0.8,
        (AnomalyType::Signal, "signal_outlier") => 0.7,
        (AnomalyType::Identity, "suspicious_callsign") => 0.9,
        (AnomalyType::Identity, "invalid_hex") => 0.95,
        (AnomalyType::Identity, "hex_duplicate") => 0.85,
        (AnomalyType::Behavioral, "physics_violation") => 0.95,
        (AnomalyType::Behavioral, "position_jump") => 0.8,
        _ => 0.5, // Default confidence for unknown subtypes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_alert_dedupe_signature() {
        let alert = Alert {
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            anomaly_type: AnomalyType::Temporal,
            subtype: "rapid_transmission".to_string(),
            confidence: 0.9,
            details_json: None,
        };

        let signature = alert.dedupe_signature();
        assert_eq!(signature, "ABC123:temporal:rapid_transmission");
    }

    #[tokio::test]
    async fn test_alert_from_candidate() {
        let candidate = AnomalyCandidate {
            hex: "ABC123".to_string(),
            anomaly_type: AnomalyType::Temporal,
            subtype: "rapid_transmission".to_string(),
            confidence: 0.9,
            details: Some(json!({"rate": 15.5})),
            trigger_observation: None,
        };

        let alert = Alert::from_candidate(candidate);
        assert_eq!(alert.hex, "ABC123");
        assert_eq!(alert.anomaly_type, AnomalyType::Temporal);
        assert_eq!(alert.subtype, "rapid_transmission");
        assert_eq!(alert.confidence, 0.9);
        assert!(alert.details_json.is_some());
        assert!(alert.ts_ms > 0);
    }

    #[tokio::test]
    async fn test_alert_manager_dedupe() {
        let temp_file = NamedTempFile::new().unwrap();
        let pool = store::connect_and_migrate(temp_file.path().to_str().unwrap(), true)
            .await
            .unwrap();

        let mut manager = AlertManager::new(pool);

        let alert1 = Alert {
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            anomaly_type: AnomalyType::Temporal,
            subtype: "rapid_transmission".to_string(),
            confidence: 0.9,
            details_json: None,
        };

        let alert2 = alert1.clone(); // Same alert

        // First alert should be recorded
        let id1 = manager.record_alert(alert1).await.unwrap();
        assert!(id1.is_some());

        // Second identical alert should be deduped
        let id2 = manager.record_alert(alert2).await.unwrap();
        assert!(id2.is_none());
    }

    #[tokio::test]
    async fn test_alert_manager_get_alerts() {
        let temp_file = NamedTempFile::new().unwrap();
        let pool = store::connect_and_migrate(temp_file.path().to_str().unwrap(), true)
            .await
            .unwrap();

        let mut manager = AlertManager::new(pool);

        // Insert test alerts
        let alert1 = Alert {
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            anomaly_type: AnomalyType::Temporal,
            subtype: "rapid_transmission".to_string(),
            confidence: 0.9,
            details_json: Some(json!({"rate": 15.5}).to_string()),
        };

        let alert2 = Alert {
            ts_ms: 1641024001000,
            hex: "DEF456".to_string(),
            anomaly_type: AnomalyType::Signal,
            subtype: "signal_outlier".to_string(),
            confidence: 0.7,
            details_json: None,
        };

        manager.record_alert(alert1).await.unwrap();
        manager.record_alert(alert2).await.unwrap();

        // Get all alerts
        let all_alerts = manager.get_alerts(None, None, None).await.unwrap();
        assert_eq!(all_alerts.len(), 2);

        // Filter by type
        let temporal_alerts = manager
            .get_alerts(Some(AnomalyType::Temporal), None, None)
            .await
            .unwrap();
        assert_eq!(temporal_alerts.len(), 1);
        assert_eq!(temporal_alerts[0].hex, "ABC123");

        // Filter by time
        let recent_alerts = manager
            .get_alerts(None, Some(1641024000500), None)
            .await
            .unwrap();
        assert_eq!(recent_alerts.len(), 1);
        assert_eq!(recent_alerts[0].hex, "DEF456");
    }

    #[tokio::test]
    async fn test_confidence_calculation() {
        assert_eq!(
            calculate_confidence(AnomalyType::Temporal, "rapid_transmission", None),
            0.9
        );
        assert_eq!(
            calculate_confidence(AnomalyType::Identity, "invalid_hex", None),
            0.95
        );
        assert_eq!(
            calculate_confidence(AnomalyType::Behavioral, "unknown_subtype", None),
            0.5
        );
    }

    #[tokio::test]
    async fn test_broadcaster() {
        let broadcaster = AlertBroadcaster::new();
        let mut receiver = broadcaster.subscribe();

        let alert = Alert {
            ts_ms: 1641024000000,
            hex: "ABC123".to_string(),
            anomaly_type: AnomalyType::Temporal,
            subtype: "rapid_transmission".to_string(),
            confidence: 0.9,
            details_json: None,
        };

        broadcaster.broadcast(alert.clone()).unwrap();

        let received = receiver.recv().await.unwrap();
        assert_eq!(received, alert);
    }
}
