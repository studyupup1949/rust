//! A single audit-run history record.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// One entry in `~/.aaai/history.jsonl`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecord {
    /// ISO-8601 UTC timestamp.
    pub run_at: DateTime<Utc>,
    pub before: String,
    pub after: String,
    pub definition: Option<String>,
    pub result: String,   // "PASSED" | "FAILED"
    pub total:   usize,
    pub ok:      usize,
    pub pending: usize,
    pub failed:  usize,
    pub error:   usize,
}

impl HistoryRecord {
    pub fn new(
        before: &std::path::Path,
        after: &std::path::Path,
        definition: Option<&std::path::Path>,
        summary: &crate::audit::result::AuditSummary,
    ) -> Self {
        Self {
            run_at:     Utc::now(),
            before:     before.display().to_string(),
            after:      after.display().to_string(),
            definition: definition.map(|p| p.display().to_string()),
            result:     if summary.is_passing() { "PASSED".into() } else { "FAILED".into() },
            total:      summary.total,
            ok:         summary.ok,
            pending:    summary.pending,
            failed:     summary.failed,
            error:      summary.error,
        }
    }
}
