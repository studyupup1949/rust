//! Audit result types.

use crate::diff::entry::DiffEntry;
use crate::config::definition::AuditEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditStatus {
    Ok,
    Pending,
    Failed,
    Ignored,
    Error,
}

impl std::fmt::Display for AuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AuditStatus::Ok => "OK",
            AuditStatus::Pending => "Pending",
            AuditStatus::Failed => "Failed",
            AuditStatus::Ignored => "Ignored",
            AuditStatus::Error => "Error",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub struct FileAuditResult {
    pub diff: DiffEntry,
    pub entry: Option<AuditEntry>,
    pub status: AuditStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AuditSummary {
    pub total: usize,
    pub ok: usize,
    pub pending: usize,
    pub failed: usize,
    pub ignored: usize,
    pub error: usize,
}

impl AuditSummary {
    pub fn from_results(results: &[FileAuditResult]) -> Self {
        let mut s = AuditSummary::default();
        s.total = results.len();
        for r in results {
            match r.status {
                AuditStatus::Ok => s.ok += 1,
                AuditStatus::Pending => s.pending += 1,
                AuditStatus::Failed => s.failed += 1,
                AuditStatus::Ignored => s.ignored += 1,
                AuditStatus::Error => s.error += 1,
            }
        }
        s
    }

    pub fn is_passing(&self) -> bool {
        self.failed == 0 && self.pending == 0 && self.error == 0
    }
}

#[derive(Debug, Clone)]
pub struct AuditResult {
    pub results: Vec<FileAuditResult>,
    pub summary: AuditSummary,
}

impl AuditResult {
    pub fn new(results: Vec<FileAuditResult>) -> Self {
        let summary = AuditSummary::from_results(&results);
        AuditResult { results, summary }
    }
}
