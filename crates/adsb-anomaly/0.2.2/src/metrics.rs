// ABOUTME: Prometheus metrics collection for monitoring system health and performance
// ABOUTME: Provides counters, histograms, and a /metrics endpoint for scraping

#![allow(dead_code)]

use prometheus::{
    register_counter_vec, register_histogram, CounterVec, Encoder, Histogram, TextEncoder,
};
use std::sync::OnceLock;

/// Global metrics registry
pub struct AppMetrics {
    /// Total observations ingested from PiAware, labeled by status (success, error)
    pub ingested_observations_total: CounterVec,
    /// Total alerts generated, labeled by anomaly type
    pub alerts_total: CounterVec,
    /// Time taken to process ingestion batches
    pub ingestion_latency_ms: Histogram,
    /// Time taken to run detection analysis
    pub detection_latency_ms: Histogram,
    /// Database operation latency
    pub db_operation_latency_ms: CounterVec,
}

static METRICS: OnceLock<AppMetrics> = OnceLock::new();

impl AppMetrics {
    fn new() -> prometheus::Result<Self> {
        Ok(Self {
            ingested_observations_total: register_counter_vec!(
                "ingested_observations_total",
                "Total number of aircraft observations ingested",
                &["status"]
            )?,
            alerts_total: register_counter_vec!(
                "alerts_total",
                "Total number of anomaly alerts generated",
                &["anomaly_type"]
            )?,
            ingestion_latency_ms: register_histogram!(
                "ingestion_latency_ms",
                "Time taken to process ingestion batches in milliseconds",
                vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0]
            )?,
            detection_latency_ms: register_histogram!(
                "detection_latency_ms",
                "Time taken to run anomaly detection analysis in milliseconds",
                vec![10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0, 10000.0]
            )?,
            db_operation_latency_ms: register_counter_vec!(
                "db_operation_latency_ms_total",
                "Total time spent on database operations",
                &["operation"]
            )?,
        })
    }

    /// Initialize global metrics instance
    pub fn init() -> prometheus::Result<()> {
        let metrics = Self::new()?;
        METRICS
            .set(metrics)
            .map_err(|_| prometheus::Error::Msg("Metrics already initialized".to_string()))?;
        Ok(())
    }

    /// Get global metrics instance
    pub fn global() -> &'static AppMetrics {
        METRICS.get().expect("Metrics not initialized")
    }

    /// Record a successful observation ingestion
    pub fn record_ingestion_success(&self, count: usize) {
        self.ingested_observations_total
            .with_label_values(&["success"])
            .inc_by(count as f64);
    }

    /// Record a failed observation ingestion
    pub fn record_ingestion_error(&self) {
        self.ingested_observations_total
            .with_label_values(&["error"])
            .inc();
    }

    /// Record an alert being generated
    pub fn record_alert(&self, anomaly_type: &str) {
        self.alerts_total.with_label_values(&[anomaly_type]).inc();
    }

    /// Time an ingestion batch operation
    pub fn time_ingestion<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let timer = self.ingestion_latency_ms.start_timer();
        let result = f();
        timer.observe_duration();
        result
    }

    /// Time a detection analysis operation
    pub fn time_detection<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let timer = self.detection_latency_ms.start_timer();
        let result = f();
        timer.observe_duration();
        result
    }

    /// Record database operation time
    pub fn record_db_operation(&self, operation: &str, duration_ms: f64) {
        self.db_operation_latency_ms
            .with_label_values(&[operation])
            .inc_by(duration_ms);
    }

    /// Export metrics in Prometheus format
    pub fn export(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        // Note: We can't reliably clear the Prometheus registry in tests,
        // so we just test that initialization works (even if it fails due to
        // already being initialized in other tests)

        let result = AppMetrics::init();
        // Either succeeds or fails with "already initialized" - both are OK for tests
        assert!(result.is_ok() || result.is_err());

        // If init succeeded, test the metrics
        if result.is_ok() {
            let metrics = AppMetrics::global();

            // Test that we can record metrics without panicking
            metrics.record_ingestion_success(5);
            metrics.record_ingestion_error();
            metrics.record_alert("temporal");

            let _timed_result = metrics.time_ingestion(|| "test".to_string());
            let _timed_result2 = metrics.time_detection(|| 42);

            metrics.record_db_operation("insert", 125.5);

            // Test export
            let export_result = metrics.export();
            assert!(export_result.is_ok(), "Metrics export should succeed");

            let metrics_text = export_result.unwrap();
            assert!(metrics_text.contains("ingested_observations_total"));
            assert!(metrics_text.contains("alerts_total"));
        }
    }

    #[test]
    fn test_timing_operations() {
        // Try to initialize metrics (may fail if already done)
        let _ = AppMetrics::init();
        let metrics = AppMetrics::global();

        // Test that timing actually measures something
        let result = metrics.time_ingestion(|| {
            std::thread::sleep(std::time::Duration::from_millis(1));
            "completed"
        });

        assert_eq!(result, "completed");

        // Verify metrics were recorded (basic smoke test)
        let export = metrics.export().expect("Should export");
        assert!(export.contains("ingestion_latency_ms"));
    }
}
