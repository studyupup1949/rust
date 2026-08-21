use std::collections::VecDeque;
use std::fmt;

pub const MEMPOOL_ADD_EVENT: &str = "mempool.add";
pub const MEMPOOL_REMOVE_EVENT: &str = "mempool.remove";
pub const MEMPOOL_SUBMIT_EVENT: &str = "mempool.submit";
pub const MEMPOOL_REJECT_EVENT: &str = "mempool.reject";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolItem {
    pub id: String,
    pub bytes: Vec<u8>,
}

impl MempoolItem {
    pub fn new(id: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            id: id.into(),
            bytes,
        }
    }

    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    pub fn validate(&self, rules: MempoolItemRules) -> Result<(), MempoolError> {
        if self.id.trim().is_empty() {
            return Err(MempoolError::InvalidItemId);
        }
        if !rules.allow_empty && self.bytes.is_empty() {
            return Err(MempoolError::EmptyItem);
        }
        if self.bytes.len() > rules.max_item_bytes {
            return Err(MempoolError::ItemTooLarge {
                max: rules.max_item_bytes,
                actual: self.bytes.len(),
            });
        }
        Ok(())
    }

    pub fn validate_semantics(&self, rules: &MempoolSemanticRules) -> Result<(), MempoolError> {
        if rules.require_ascii_id && !self.id.is_ascii() {
            return Err(MempoolError::InvalidItemId);
        }
        if self.id.len() > rules.max_id_bytes {
            return Err(MempoolError::ItemIdTooLong {
                max: rules.max_id_bytes,
                actual: self.id.len(),
            });
        }
        if self.bytes.len() < rules.min_item_bytes {
            return Err(MempoolError::ItemTooSmall {
                min: rules.min_item_bytes,
                actual: self.bytes.len(),
            });
        }
        if let Some(prefix) = rules.required_prefix {
            if !self.bytes.starts_with(prefix) {
                return Err(MempoolError::ItemPrefixMismatch);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MempoolItemRules {
    pub max_item_bytes: usize,
    pub allow_empty: bool,
}

impl Default for MempoolItemRules {
    fn default() -> Self {
        Self {
            max_item_bytes: 16 * 1024,
            allow_empty: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MempoolSemanticRules {
    pub max_id_bytes: usize,
    pub require_ascii_id: bool,
    pub min_item_bytes: usize,
    pub required_prefix: Option<&'static [u8]>,
}

impl Default for MempoolSemanticRules {
    fn default() -> Self {
        Self {
            max_id_bytes: 128,
            require_ascii_id: true,
            min_item_bytes: 1,
            required_prefix: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mempool {
    capacity_bytes: usize,
    used_bytes: usize,
    items: VecDeque<MempoolItem>,
    lifecycle_events: Vec<MempoolLifecycleEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MempoolWatermarks {
    pub eviction: f64,
    pub rejection: f64,
}

impl Default for MempoolWatermarks {
    fn default() -> Self {
        Self {
            eviction: 0.75,
            rejection: 0.90,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolAdmission {
    pub evicted: Vec<MempoolItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MempoolItemLifecycle {
    Submitted,
    Accepted,
    Evicted,
    Removed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolLifecycleEvent {
    pub name: &'static str,
    pub item_id: String,
    pub item_size: usize,
    pub lifecycle: MempoolItemLifecycle,
}

impl MempoolLifecycleEvent {
    pub fn new(item: &MempoolItem, lifecycle: MempoolItemLifecycle) -> Self {
        Self {
            name: lifecycle.event_name(),
            item_id: item.id.clone(),
            item_size: item.size(),
            lifecycle,
        }
    }
}

impl MempoolItemLifecycle {
    pub fn event_name(self) -> &'static str {
        match self {
            Self::Submitted => MEMPOOL_SUBMIT_EVENT,
            Self::Accepted => MEMPOOL_ADD_EVENT,
            Self::Evicted | Self::Removed => MEMPOOL_REMOVE_EVENT,
            Self::Rejected => MEMPOOL_REJECT_EVENT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolSubmission {
    pub admission: MempoolAdmission,
    pub events: Vec<MempoolLifecycleEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolSubmissionReport {
    pub admission: Option<MempoolAdmission>,
    pub error: Option<MempoolError>,
    pub events: Vec<MempoolLifecycleEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolBlockCandidatePlan {
    pub selected: Vec<MempoolItem>,
    pub skipped: Vec<MempoolBlockCandidateSkip>,
    pub total_bytes: usize,
    pub max_items: usize,
    pub max_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MempoolBlockCandidateSkip {
    pub item_id: String,
    pub item_size: usize,
    pub reason: MempoolBlockCandidateSkipReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MempoolBlockCandidateSkipReason {
    MaxItems,
    MaxBytes,
    Invalid(MempoolError),
}

impl MempoolSubmissionReport {
    pub fn accepted(admission: MempoolAdmission, events: Vec<MempoolLifecycleEvent>) -> Self {
        Self {
            admission: Some(admission),
            error: None,
            events,
        }
    }

    pub fn rejected(error: MempoolError, events: Vec<MempoolLifecycleEvent>) -> Self {
        Self {
            admission: None,
            error: Some(error),
            events,
        }
    }
}

impl Mempool {
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            capacity_bytes,
            used_bytes: 0,
            items: VecDeque::new(),
            lifecycle_events: Vec::new(),
        }
    }

    pub fn add(&mut self, item: MempoolItem) -> Result<(), MempoolError> {
        self.reject_duplicate(&item)?;
        if item.size() > self.capacity_bytes.saturating_sub(self.used_bytes) {
            return Err(MempoolError::CapacityExceeded {
                capacity: self.capacity_bytes,
                used: self.used_bytes,
                candidate: item.size(),
            });
        }
        self.used_bytes += item.size();
        self.record_event(&item, MempoolItemLifecycle::Accepted);
        self.items.push_back(item);
        Ok(())
    }

    pub fn add_with_watermarks(
        &mut self,
        item: MempoolItem,
        watermarks: MempoolWatermarks,
    ) -> Result<MempoolAdmission, MempoolError> {
        watermarks.validate()?;
        self.reject_duplicate(&item)?;
        let item_size = item.size();
        let rejection_bytes = ratio_bytes(self.capacity_bytes, watermarks.rejection);
        if item_size > rejection_bytes
            || self.used_bytes.saturating_add(item_size) > rejection_bytes
        {
            return Err(MempoolError::RejectedByWatermark {
                rejection_bytes,
                used: self.used_bytes,
                candidate: item_size,
            });
        }

        let mut evicted = Vec::new();
        let mut target = ratio_bytes(self.capacity_bytes, watermarks.eviction);
        target = target.max(item_size);
        while self.used_bytes.saturating_add(item_size) > target {
            let Some(oldest) = self.items.pop_front() else {
                break;
            };
            self.used_bytes = self.used_bytes.saturating_sub(oldest.size());
            self.record_event(&oldest, MempoolItemLifecycle::Evicted);
            evicted.push(oldest);
        }

        self.used_bytes += item_size;
        self.record_event(&item, MempoolItemLifecycle::Accepted);
        self.items.push_back(item);
        Ok(MempoolAdmission { evicted })
    }

    pub fn submit_local_report(
        &mut self,
        item: MempoolItem,
        watermarks: MempoolWatermarks,
        rules: MempoolItemRules,
    ) -> MempoolSubmissionReport {
        let submitted = MempoolLifecycleEvent::new(&item, MempoolItemLifecycle::Submitted);
        self.lifecycle_events.push(submitted.clone());
        if let Err(err) = item.validate(rules) {
            let rejected = MempoolLifecycleEvent::new(&item, MempoolItemLifecycle::Rejected);
            self.lifecycle_events.push(rejected.clone());
            return MempoolSubmissionReport::rejected(err, vec![submitted, rejected]);
        }
        let rejected_item = item.clone();
        let accepted = MempoolLifecycleEvent::new(&item, MempoolItemLifecycle::Accepted);
        match self.add_with_watermarks(item, watermarks) {
            Ok(admission) => {
                let mut events = Vec::with_capacity(2 + admission.evicted.len());
                events.push(submitted);
                events.extend(
                    admission.evicted.iter().map(|item| {
                        MempoolLifecycleEvent::new(item, MempoolItemLifecycle::Evicted)
                    }),
                );
                events.push(accepted);
                MempoolSubmissionReport::accepted(admission, events)
            }
            Err(err) => {
                let rejected =
                    MempoolLifecycleEvent::new(&rejected_item, MempoolItemLifecycle::Rejected);
                self.lifecycle_events.push(rejected.clone());
                MempoolSubmissionReport::rejected(err, vec![submitted, rejected])
            }
        }
    }

    pub fn submit_local(
        &mut self,
        item: MempoolItem,
        watermarks: MempoolWatermarks,
        rules: MempoolItemRules,
    ) -> Result<MempoolSubmission, MempoolError> {
        let report = self.submit_local_report(item, watermarks, rules);
        if let Some(error) = report.error {
            return Err(error);
        }
        Ok(MempoolSubmission {
            admission: report.admission.expect("accepted report has admission"),
            events: report.events,
        })
    }

    pub fn submit_local_semantic_report(
        &mut self,
        item: MempoolItem,
        watermarks: MempoolWatermarks,
        item_rules: MempoolItemRules,
        semantic_rules: MempoolSemanticRules,
    ) -> MempoolSubmissionReport {
        let submitted = MempoolLifecycleEvent::new(&item, MempoolItemLifecycle::Submitted);
        self.lifecycle_events.push(submitted.clone());
        if let Err(err) = item
            .validate(item_rules)
            .and_then(|()| item.validate_semantics(&semantic_rules))
        {
            let rejected = MempoolLifecycleEvent::new(&item, MempoolItemLifecycle::Rejected);
            self.lifecycle_events.push(rejected.clone());
            return MempoolSubmissionReport::rejected(err, vec![submitted, rejected]);
        }
        let rejected_item = item.clone();
        let accepted = MempoolLifecycleEvent::new(&item, MempoolItemLifecycle::Accepted);
        match self.add_with_watermarks(item, watermarks) {
            Ok(admission) => {
                let mut events = Vec::with_capacity(2 + admission.evicted.len());
                events.push(submitted);
                events.extend(
                    admission.evicted.iter().map(|item| {
                        MempoolLifecycleEvent::new(item, MempoolItemLifecycle::Evicted)
                    }),
                );
                events.push(accepted);
                MempoolSubmissionReport::accepted(admission, events)
            }
            Err(err) => {
                let rejected =
                    MempoolLifecycleEvent::new(&rejected_item, MempoolItemLifecycle::Rejected);
                self.lifecycle_events.push(rejected.clone());
                MempoolSubmissionReport::rejected(err, vec![submitted, rejected])
            }
        }
    }

    pub fn plan_block_candidate(
        &self,
        max_items: usize,
        max_bytes: usize,
        semantic_rules: Option<MempoolSemanticRules>,
    ) -> MempoolBlockCandidatePlan {
        let mut selected = Vec::new();
        let mut skipped = Vec::new();
        let mut total_bytes = 0_usize;

        for item in &self.items {
            let reason = if selected.len() >= max_items {
                Some(MempoolBlockCandidateSkipReason::MaxItems)
            } else if let Some(rules) = semantic_rules {
                item.validate_semantics(&rules)
                    .err()
                    .map(MempoolBlockCandidateSkipReason::Invalid)
            } else {
                None
            };
            let reason = reason.or_else(|| {
                if total_bytes.saturating_add(item.size()) > max_bytes {
                    Some(MempoolBlockCandidateSkipReason::MaxBytes)
                } else {
                    None
                }
            });

            if let Some(reason) = reason {
                skipped.push(MempoolBlockCandidateSkip {
                    item_id: item.id.clone(),
                    item_size: item.size(),
                    reason,
                });
                continue;
            }

            total_bytes += item.size();
            selected.push(item.clone());
        }

        MempoolBlockCandidatePlan {
            selected,
            skipped,
            total_bytes,
            max_items,
            max_bytes,
        }
    }

    pub fn remove(&mut self, id: &str) -> Option<MempoolItem> {
        let index = self.items.iter().position(|item| item.id == id)?;
        let item = self.items.remove(index)?;
        self.used_bytes = self.used_bytes.saturating_sub(item.size());
        self.record_event(&item, MempoolItemLifecycle::Removed);
        Some(item)
    }

    pub fn remove_with_event(&mut self, id: &str) -> Option<(MempoolItem, MempoolLifecycleEvent)> {
        let item = self.remove(id)?;
        let event = MempoolLifecycleEvent::new(&item, MempoolItemLifecycle::Removed);
        Some((item, event))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn capacity_bytes(&self) -> usize {
        self.capacity_bytes
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    pub fn contains_id(&self, id: &str) -> bool {
        self.items.iter().any(|item| item.id == id)
    }

    pub fn lifecycle_events(&self) -> &[MempoolLifecycleEvent] {
        &self.lifecycle_events
    }

    pub fn clear_lifecycle_events(&mut self) {
        self.lifecycle_events.clear();
    }

    fn reject_duplicate(&self, item: &MempoolItem) -> Result<(), MempoolError> {
        if self.contains_id(&item.id) {
            return Err(MempoolError::DuplicateItem(item.id.clone()));
        }
        Ok(())
    }

    fn record_event(&mut self, item: &MempoolItem, lifecycle: MempoolItemLifecycle) {
        self.lifecycle_events
            .push(MempoolLifecycleEvent::new(item, lifecycle));
    }
}

impl MempoolWatermarks {
    pub fn validate(self) -> Result<(), MempoolError> {
        if !self.eviction.is_finite()
            || !self.rejection.is_finite()
            || self.eviction <= 0.0
            || self.rejection <= 0.0
            || self.eviction > self.rejection
            || self.rejection > 1.0
        {
            return Err(MempoolError::InvalidWatermarks);
        }
        Ok(())
    }
}

fn ratio_bytes(capacity: usize, ratio: f64) -> usize {
    (capacity as f64 * ratio).floor() as usize
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MempoolError {
    InvalidItemId,
    EmptyItem,
    ItemIdTooLong {
        max: usize,
        actual: usize,
    },
    ItemTooSmall {
        min: usize,
        actual: usize,
    },
    ItemPrefixMismatch,
    DuplicateItem(String),
    ItemTooLarge {
        max: usize,
        actual: usize,
    },
    CapacityExceeded {
        capacity: usize,
        used: usize,
        candidate: usize,
    },
    InvalidWatermarks,
    RejectedByWatermark {
        rejection_bytes: usize,
        used: usize,
        candidate: usize,
    },
}

impl fmt::Display for MempoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidItemId => f.write_str("mempool item id is empty"),
            Self::EmptyItem => f.write_str("mempool item bytes are empty"),
            Self::ItemIdTooLong { max, actual } => {
                write!(f, "mempool item id too long: max={max} actual={actual}")
            }
            Self::ItemTooSmall { min, actual } => {
                write!(f, "mempool item too small: min={min} actual={actual}")
            }
            Self::ItemPrefixMismatch => f.write_str("mempool item prefix mismatch"),
            Self::DuplicateItem(id) => write!(f, "duplicate mempool item id {id}"),
            Self::ItemTooLarge { max, actual } => {
                write!(f, "mempool item too large: max={max} actual={actual}")
            }
            Self::CapacityExceeded {
                capacity,
                used,
                candidate,
            } => write!(
                f,
                "mempool capacity exceeded: capacity={capacity} used={used} candidate={candidate}"
            ),
            Self::InvalidWatermarks => f.write_str("invalid mempool watermarks"),
            Self::RejectedByWatermark {
                rejection_bytes,
                used,
                candidate,
            } => write!(
                f,
                "mempool rejected item: rejection_bytes={rejection_bytes} used={used} candidate={candidate}"
            ),
        }
    }
}

impl std::error::Error for MempoolError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mempool_capacity_is_local_and_deterministic() {
        let mut mempool = Mempool::new(3);
        mempool.add(MempoolItem::new("a", vec![1, 2])).unwrap();
        assert!(matches!(
            mempool.add(MempoolItem::new("b", vec![3, 4])),
            Err(MempoolError::CapacityExceeded { .. })
        ));
        assert_eq!(mempool.remove("a").unwrap().id, "a");
        assert_eq!(mempool.used_bytes(), 0);
    }

    #[test]
    fn mempool_item_validation_rejects_empty_ids_and_large_items() {
        let rules = MempoolItemRules {
            max_item_bytes: 2,
            allow_empty: false,
        };
        assert_eq!(
            MempoolItem::new(" ", vec![1]).validate(rules),
            Err(MempoolError::InvalidItemId)
        );
        assert_eq!(
            MempoolItem::new("a", vec![1, 2, 3]).validate(rules),
            Err(MempoolError::ItemTooLarge { max: 2, actual: 3 })
        );
    }

    #[test]
    fn mempool_semantic_rules_validate_id_shape_and_payload_prefix() {
        let rules = MempoolSemanticRules {
            max_id_bytes: 4,
            require_ascii_id: true,
            min_item_bytes: 3,
            required_prefix: Some(b"tx"),
        };

        assert_eq!(
            MempoolItem::new("abcde", b"tx1".to_vec()).validate_semantics(&rules),
            Err(MempoolError::ItemIdTooLong { max: 4, actual: 5 })
        );
        assert_eq!(
            MempoolItem::new("abc", b"xx1".to_vec()).validate_semantics(&rules),
            Err(MempoolError::ItemPrefixMismatch)
        );
        assert_eq!(
            MempoolItem::new("abc", b"tx".to_vec()).validate_semantics(&rules),
            Err(MempoolError::ItemTooSmall { min: 3, actual: 2 })
        );
        assert_eq!(
            MempoolItem::new("abc", b"tx1".to_vec()).validate_semantics(&rules),
            Ok(())
        );
    }

    #[test]
    fn watermarks_evict_oldest_before_accepting() {
        let mut mempool = Mempool::new(10);
        mempool.add(MempoolItem::new("a", vec![0; 3])).unwrap();
        mempool.add(MempoolItem::new("b", vec![0; 3])).unwrap();
        let admission = mempool
            .add_with_watermarks(
                MempoolItem::new("c", vec![0; 3]),
                MempoolWatermarks {
                    eviction: 0.6,
                    rejection: 1.0,
                },
            )
            .unwrap();
        assert_eq!(
            admission
                .evicted
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a"]
        );
        assert_eq!(mempool.used_bytes(), 6);
    }

    #[test]
    fn watermarks_reject_before_mutating() {
        let mut mempool = Mempool::new(10);
        mempool.add(MempoolItem::new("a", vec![0; 4])).unwrap();
        assert!(matches!(
            mempool.add_with_watermarks(
                MempoolItem::new("b", vec![0; 6]),
                MempoolWatermarks {
                    eviction: 0.5,
                    rejection: 0.8,
                },
            ),
            Err(MempoolError::RejectedByWatermark { .. })
        ));
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.used_bytes(), 4);
    }

    #[test]
    fn duplicate_ids_are_rejected_without_mutating() {
        let mut mempool = Mempool::new(10);
        mempool.add(MempoolItem::new("a", vec![1])).unwrap();
        assert_eq!(
            mempool.add(MempoolItem::new("a", vec![2])),
            Err(MempoolError::DuplicateItem("a".to_string()))
        );
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.used_bytes(), 1);
    }

    #[test]
    fn mempool_submit_local_returns_lifecycle_event_plan() {
        let mut mempool = Mempool::new(10);
        let submission = mempool
            .submit_local(
                MempoolItem::new("a", vec![1, 2]),
                MempoolWatermarks::default(),
                MempoolItemRules::default(),
            )
            .unwrap();
        assert_eq!(mempool.len(), 1);
        assert_eq!(submission.events.len(), 2);
        assert_eq!(submission.events[0].name, MEMPOOL_SUBMIT_EVENT);
        assert_eq!(submission.events[1].name, MEMPOOL_ADD_EVENT);

        let (_, event) = mempool.remove_with_event("a").unwrap();
        assert_eq!(event.name, MEMPOOL_REMOVE_EVENT);
        assert_eq!(event.lifecycle, MempoolItemLifecycle::Removed);
        assert_eq!(
            mempool
                .lifecycle_events()
                .iter()
                .map(|event| event.lifecycle)
                .collect::<Vec<_>>(),
            vec![
                MempoolItemLifecycle::Submitted,
                MempoolItemLifecycle::Accepted,
                MempoolItemLifecycle::Removed,
            ]
        );
    }

    #[test]
    fn submit_report_returns_rejected_lifecycle_event() {
        let mut mempool = Mempool::new(10);
        let report = mempool.submit_local_report(
            MempoolItem::new("bad", Vec::new()),
            MempoolWatermarks::default(),
            MempoolItemRules::default(),
        );
        assert_eq!(report.admission, None);
        assert_eq!(report.error, Some(MempoolError::EmptyItem));
        assert_eq!(report.events.len(), 2);
        assert_eq!(report.events[0].lifecycle, MempoolItemLifecycle::Submitted);
        assert_eq!(report.events[1].lifecycle, MempoolItemLifecycle::Rejected);
        assert!(mempool.is_empty());
    }

    #[test]
    fn semantic_submit_report_rejects_without_mutating_mempool() {
        let mut mempool = Mempool::new(10);
        let report = mempool.submit_local_semantic_report(
            MempoolItem::new("tx-1", b"bad".to_vec()),
            MempoolWatermarks::default(),
            MempoolItemRules::default(),
            MempoolSemanticRules {
                required_prefix: Some(b"tx"),
                ..MempoolSemanticRules::default()
            },
        );

        assert_eq!(report.admission, None);
        assert_eq!(report.error, Some(MempoolError::ItemPrefixMismatch));
        assert_eq!(report.events[0].lifecycle, MempoolItemLifecycle::Submitted);
        assert_eq!(report.events[1].lifecycle, MempoolItemLifecycle::Rejected);
        assert!(mempool.is_empty());
    }

    #[test]
    fn lifecycle_log_records_evictions_before_acceptance() {
        let mut mempool = Mempool::new(10);
        mempool.add(MempoolItem::new("a", vec![0; 3])).unwrap();
        mempool.add(MempoolItem::new("b", vec![0; 3])).unwrap();
        mempool.clear_lifecycle_events();
        mempool
            .add_with_watermarks(
                MempoolItem::new("c", vec![0; 3]),
                MempoolWatermarks {
                    eviction: 0.6,
                    rejection: 1.0,
                },
            )
            .unwrap();
        assert_eq!(
            mempool
                .lifecycle_events()
                .iter()
                .map(|event| (event.item_id.as_str(), event.lifecycle))
                .collect::<Vec<_>>(),
            vec![
                ("a", MempoolItemLifecycle::Evicted),
                ("c", MempoolItemLifecycle::Accepted),
            ]
        );
    }

    #[test]
    fn block_candidate_plan_selects_queue_order_under_limits_without_mutating() {
        let mut mempool = Mempool::new(20);
        mempool.add(MempoolItem::new("a", vec![0; 2])).unwrap();
        mempool.add(MempoolItem::new("b", vec![0; 4])).unwrap();
        mempool.add(MempoolItem::new("c", vec![0; 2])).unwrap();

        let plan = mempool.plan_block_candidate(2, 5, None);

        assert_eq!(
            plan.selected
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "c"]
        );
        assert_eq!(plan.total_bytes, 4);
        assert_eq!(plan.skipped.len(), 1);
        assert_eq!(plan.skipped[0].item_id, "b");
        assert_eq!(
            plan.skipped[0].reason,
            MempoolBlockCandidateSkipReason::MaxBytes
        );
        assert_eq!(mempool.len(), 3);
        assert_eq!(mempool.used_bytes(), 8);
    }

    #[test]
    fn block_candidate_plan_reports_max_item_skips() {
        let mut mempool = Mempool::new(20);
        mempool.add(MempoolItem::new("a", vec![0; 2])).unwrap();
        mempool.add(MempoolItem::new("b", vec![0; 2])).unwrap();

        let plan = mempool.plan_block_candidate(1, 20, None);

        assert_eq!(plan.selected.len(), 1);
        assert_eq!(plan.skipped.len(), 1);
        assert_eq!(plan.skipped[0].item_id, "b");
        assert_eq!(
            plan.skipped[0].reason,
            MempoolBlockCandidateSkipReason::MaxItems
        );
    }

    #[test]
    fn block_candidate_plan_can_apply_local_semantic_rules() {
        let mut mempool = Mempool::new(20);
        mempool
            .add(MempoolItem::new("bad", b"no".to_vec()))
            .unwrap();
        mempool
            .add(MempoolItem::new("good", b"tx-local".to_vec()))
            .unwrap();

        let plan = mempool.plan_block_candidate(
            10,
            20,
            Some(MempoolSemanticRules {
                required_prefix: Some(b"tx"),
                ..MempoolSemanticRules::default()
            }),
        );

        assert_eq!(plan.selected.len(), 1);
        assert_eq!(plan.selected[0].id, "good");
        assert_eq!(plan.skipped.len(), 1);
        assert_eq!(plan.skipped[0].item_id, "bad");
        assert_eq!(
            plan.skipped[0].reason,
            MempoolBlockCandidateSkipReason::Invalid(MempoolError::ItemPrefixMismatch)
        );
    }
}
