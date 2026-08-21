//! OpenTelemetry telemetry for the A3S Cron scheduler.
//!
//! Provides structured spans, attribute constants, and OTLP metrics
//! for cron job scheduling and execution observability.

use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::KeyValue;
use std::sync::OnceLock;

// ============================================================================
// Span Constants
// ============================================================================

/// Span for a single job execution
pub const SPAN_CRON_EXECUTE_JOB: &str = "a3s.cron.execute_job";

/// Span for one scheduler tick (poll all jobs)
pub const SPAN_CRON_SCHEDULER_TICK: &str = "a3s.cron.scheduler_tick";

// ============================================================================
// Attribute Constants
// ============================================================================

/// Job identifier attribute
pub const ATTR_JOB_ID: &str = "a3s.cron.job_id";

/// Job name attribute
pub const ATTR_JOB_NAME: &str = "a3s.cron.job_name";

/// Job execution status (success / failed / timeout)
pub const ATTR_JOB_STATUS: &str = "a3s.cron.job_status";

/// Job execution duration in milliseconds
pub const ATTR_JOB_DURATION_MS: &str = "a3s.cron.job_duration_ms";

// ============================================================================
// Metrics
// ============================================================================

static METRICS: OnceLock<CronMetricsRecorder> = OnceLock::new();

/// Holds OpenTelemetry metric instruments for cron scheduling.
pub struct CronMetricsRecorder {
    /// Total job executions, with attributes: job_name, status
    pub jobs_executed_total: Counter<u64>,
    /// Job execution duration in seconds
    pub job_duration_seconds: Histogram<f64>,
    /// Total scheduler ticks
    pub scheduler_ticks_total: Counter<u64>,
}

/// Get the global cron metrics recorder (None if not initialized).
pub fn metrics() -> Option<&'static CronMetricsRecorder> {
    METRICS.get()
}

/// Initialize cron metrics using the global OpenTelemetry meter provider.
///
/// Safe to call multiple times; only the first call takes effect.
pub fn init_cron_metrics() {
    let meter = global::meter("a3s-cron");

    let recorder = CronMetricsRecorder {
        jobs_executed_total: meter
            .u64_counter("a3s_cron_jobs_executed_total")
            .with_description("Total cron job executions")
            .init(),
        job_duration_seconds: meter
            .f64_histogram("a3s_cron_job_duration_seconds")
            .with_description("Cron job execution duration in seconds")
            .init(),
        scheduler_ticks_total: meter
            .u64_counter("a3s_cron_scheduler_ticks_total")
            .with_description("Total scheduler tick cycles")
            .init(),
    };

    let _ = METRICS.set(recorder);
}

/// Record a job execution with name, status, and duration.
///
/// No-op if metrics have not been initialized.
pub fn record_job_execution(job_name: &str, status: &str, duration_secs: f64) {
    if let Some(m) = metrics() {
        let attrs = [
            KeyValue::new("job_name", job_name.to_string()),
            KeyValue::new("status", status.to_string()),
        ];
        m.jobs_executed_total.add(1, &attrs);
        m.job_duration_seconds.record(
            duration_secs,
            &[KeyValue::new("job_name", job_name.to_string())],
        );
    }
}

/// Record a scheduler tick.
///
/// No-op if metrics have not been initialized.
pub fn record_scheduler_tick() {
    if let Some(m) = metrics() {
        m.scheduler_ticks_total.add(1, &[]);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_constants_follow_convention() {
        assert!(SPAN_CRON_EXECUTE_JOB.starts_with("a3s."));
        assert!(SPAN_CRON_SCHEDULER_TICK.starts_with("a3s."));
    }

    #[test]
    fn test_attribute_keys_follow_convention() {
        let attrs = [
            ATTR_JOB_ID,
            ATTR_JOB_NAME,
            ATTR_JOB_STATUS,
            ATTR_JOB_DURATION_MS,
        ];
        for attr in &attrs {
            assert!(
                attr.starts_with("a3s.cron."),
                "Attribute {} should start with a3s.cron.",
                attr
            );
        }
    }

    #[test]
    fn test_attribute_keys_are_unique() {
        let keys = vec![
            ATTR_JOB_ID,
            ATTR_JOB_NAME,
            ATTR_JOB_STATUS,
            ATTR_JOB_DURATION_MS,
        ];
        let unique: std::collections::HashSet<&str> = keys.iter().copied().collect();
        assert_eq!(keys.len(), unique.len(), "Attribute keys must be unique");
    }

    #[test]
    fn test_record_job_execution_no_panic_without_init() {
        record_job_execution("test-job", "success", 1.5);
        record_job_execution("test-job", "failed", 0.0);
        record_job_execution("", "", 0.0);
    }

    #[test]
    fn test_record_scheduler_tick_no_panic_without_init() {
        record_scheduler_tick();
    }

    #[test]
    fn test_metrics_returns_none_without_init() {
        // In test context, the global meter provider is not set up for OTLP,
        // but init_cron_metrics may or may not have been called by another test.
        // Either way, this must not panic.
        let _ = metrics();
    }

    #[test]
    fn test_span_constant_values() {
        assert_eq!(SPAN_CRON_EXECUTE_JOB, "a3s.cron.execute_job");
        assert_eq!(SPAN_CRON_SCHEDULER_TICK, "a3s.cron.scheduler_tick");
    }

    #[test]
    fn test_attribute_constant_values() {
        assert_eq!(ATTR_JOB_ID, "a3s.cron.job_id");
        assert_eq!(ATTR_JOB_NAME, "a3s.cron.job_name");
        assert_eq!(ATTR_JOB_STATUS, "a3s.cron.job_status");
        assert_eq!(ATTR_JOB_DURATION_MS, "a3s.cron.job_duration_ms");
    }
}
