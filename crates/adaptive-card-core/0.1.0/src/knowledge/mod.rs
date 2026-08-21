//! Knowledge base — curated sample library for LLM inspiration.

pub mod format;
pub mod loader;
pub mod selector;

pub use format::{Complexity, KnowledgeEntry, KnowledgeManifest, ManifestEntry};
pub use selector::SuggestResult;

use crate::error::{Error, Result};
use std::path::Path;
use std::sync::LazyLock;

/// In-memory collection of knowledge base entries.
#[derive(Debug, Clone, Default)]
pub struct KnowledgeBase {
    entries: Vec<KnowledgeEntry>,
}

/// Macro to embed all KB entry JSON files at compile time.
/// Each entry is included as a static &str and parsed lazily.
///
/// To add a new entry: add the `include_str!` line below and update
/// `data/knowledge-base/_manifest.json` (manifest is for `from_dir` only;
/// embedded entries are listed here explicitly).
const EMBEDDED_ENTRIES_JSON: &[&str] = &[
    // Domain examples (18 entries)
    include_str!("../../data/knowledge-base/examples/commerce/order-tracker.json"),
    include_str!("../../data/knowledge-base/examples/communication/feedback-survey.json"),
    include_str!("../../data/knowledge-base/examples/data-viz/kpi-dashboard.json"),
    include_str!("../../data/knowledge-base/examples/data-viz/weather-forecast.json"),
    include_str!("../../data/knowledge-base/examples/finance/expense-report.json"),
    include_str!("../../data/knowledge-base/examples/finance/invoice-processing.json"),
    include_str!("../../data/knowledge-base/examples/helpdesk/it-helpdesk.json"),
    include_str!("../../data/knowledge-base/examples/hr/employee-onboarding.json"),
    include_str!("../../data/knowledge-base/examples/hr/leave-management.json"),
    include_str!("../../data/knowledge-base/examples/hr/performance-review.json"),
    include_str!("../../data/knowledge-base/examples/hr/recruitment.json"),
    include_str!("../../data/knowledge-base/examples/hr/timesheet.json"),
    include_str!("../../data/knowledge-base/examples/hr/training-lms.json"),
    include_str!("../../data/knowledge-base/examples/it/asset-management.json"),
    include_str!("../../data/knowledge-base/examples/productivity/meeting-scheduler.json"),
    include_str!("../../data/knowledge-base/examples/reference/element-showcase.json"),
    include_str!("../../data/knowledge-base/examples/travel/digital-travel-agent.json"),
    include_str!("../../data/knowledge-base/examples/workflow/document-approval.json"),
    // Design pattern examples (15 curated entries)
    include_str!("../../data/knowledge-base/examples/patterns/approval-split-layout.json"),
    include_str!("../../data/knowledge-base/examples/patterns/calendar-ooo.json"),
    include_str!("../../data/knowledge-base/examples/patterns/employee-onboarding.json"),
    include_str!("../../data/knowledge-base/examples/patterns/events-timeline.json"),
    include_str!("../../data/knowledge-base/examples/patterns/expense-report-approval.json"),
    include_str!("../../data/knowledge-base/examples/patterns/kpi-counter.json"),
    include_str!("../../data/knowledge-base/examples/patterns/meeting-room-booking.json"),
    include_str!("../../data/knowledge-base/examples/patterns/payslip-viewer.json"),
    include_str!("../../data/knowledge-base/examples/patterns/product-showcase.json"),
    include_str!("../../data/knowledge-base/examples/patterns/pto-balance-request.json"),
    include_str!("../../data/knowledge-base/examples/patterns/safety-alert.json"),
    include_str!("../../data/knowledge-base/examples/patterns/social-news-feed.json"),
    include_str!("../../data/knowledge-base/examples/patterns/stock-price-widget.json"),
    include_str!("../../data/knowledge-base/examples/patterns/warehouse-inventory.json"),
    include_str!("../../data/knowledge-base/examples/patterns/work-anniversary.json"),
    // Add new entries here:
    // include_str!("../../data/knowledge-base/examples/<category>/<name>.json"),
];

impl KnowledgeBase {
    /// Return the embedded, compile-time knowledge base.
    ///
    /// Entries are included via `include_str!` and parsed on first access.
    /// To add a new entry, add the file to `data/knowledge-base/examples/`
    /// and add an `include_str!` line to `EMBEDDED_ENTRIES_JSON` above.
    #[must_use]
    pub fn embedded() -> &'static KnowledgeBase {
        static KB: LazyLock<KnowledgeBase> = LazyLock::new(|| {
            let entries: Vec<KnowledgeEntry> = EMBEDDED_ENTRIES_JSON
                .iter()
                .filter_map(|json_str| serde_json::from_str(json_str).ok())
                .collect();
            KnowledgeBase { entries }
        });
        &KB
    }

    /// Load a knowledge base from a directory on disk.
    pub fn from_dir(dir: &Path) -> Result<Self> {
        let entries = loader::from_dir(dir)?;
        Ok(Self { entries })
    }

    /// Construct from an in-memory list (for tests).
    #[must_use]
    pub fn from_entries(entries: Vec<KnowledgeEntry>) -> Self {
        Self { entries }
    }

    #[must_use]
    pub fn all(&self) -> &[KnowledgeEntry] {
        &self.entries
    }

    #[must_use]
    pub fn by_id(&self, id: &str) -> Option<&KnowledgeEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    #[must_use]
    pub fn by_category(&self, category: &str) -> Vec<&KnowledgeEntry> {
        self.entries
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    /// Get-or-error variant useful in MCP tool handlers.
    pub fn require(&self, id: &str) -> Result<&KnowledgeEntry> {
        self.by_id(id)
            .ok_or(Error::KnowledgeEntryNotFound { id: id.to_string() })
    }

    #[must_use]
    pub fn suggest(&self, query: &str, limit: usize) -> Vec<SuggestResult> {
        selector::suggest(&self.entries, query, limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Host;
    use serde_json::json;

    fn make(id: &str, category: &str) -> KnowledgeEntry {
        KnowledgeEntry {
            id: id.to_string(),
            title: id.to_string(),
            description: String::new(),
            category: category.to_string(),
            tags: vec![],
            use_cases: vec![],
            host_targets: vec![Host::Teams],
            complexity: Complexity::Basic,
            card: json!({ "type": "AdaptiveCard" }),
            notes: String::new(),
        }
    }

    #[test]
    fn embedded_loads_seeded_entries() {
        let kb = KnowledgeBase::embedded();
        assert!(
            !kb.all().is_empty(),
            "embedded KB should have seeded entries"
        );
        // Verify the travel agent entry is present
        assert!(kb.by_id("travel/digital-travel-agent").is_some());
    }

    #[test]
    fn by_id_returns_entry() {
        let kb = KnowledgeBase::from_entries(vec![make("a", "finance"), make("b", "hr")]);
        assert!(kb.by_id("a").is_some());
        assert!(kb.by_id("c").is_none());
    }

    #[test]
    fn by_category_filters() {
        let kb = KnowledgeBase::from_entries(vec![make("a", "finance"), make("b", "hr")]);
        assert_eq!(kb.by_category("finance").len(), 1);
    }

    #[test]
    fn require_errors_on_missing() {
        let kb = KnowledgeBase::default();
        assert!(matches!(
            kb.require("x"),
            Err(Error::KnowledgeEntryNotFound { .. })
        ));
    }
}
