pub use crate::chain::selection::{
    compare_chain_tips, prefer_candidate_tip, TieBreakConfig, TieBreakMode, TipOrdering,
    TipSelectionView,
};
use crate::chain::{ChainPoint, ChainTip};
use crate::events::{Event, EventPayload};
use crate::ledger::blocks::{BlockDecodeError, BlockEraCatalog, BlockEraValidationReport};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

pub const DEFAULT_FIRST_LIGHT_WINDOW_SLOTS: u64 = 6_480;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SyncMode {
    #[default]
    ChainSelection,
    FirstLight,
}

impl std::fmt::Display for SyncMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChainSelection => f.write_str("chain-selection"),
            Self::FirstLight => f.write_str("first-light"),
        }
    }
}

pub fn first_light_window_slots(security_param: u64, active_slots_coeff: f64) -> u64 {
    if security_param == 0 || active_slots_coeff <= 0.0 || !active_slots_coeff.is_finite() {
        return DEFAULT_FIRST_LIGHT_WINDOW_SLOTS;
    }
    let window = (3.0 * security_param as f64 / active_slots_coeff).ceil();
    if window <= 0.0 {
        DEFAULT_FIRST_LIGHT_WINDOW_SLOTS
    } else if window >= u64::MAX as f64 {
        u64::MAX
    } else {
        window as u64
    }
}

pub const SYNC_PEER_TIP_EVENT: &str = "sync.peer_tip";
pub const SYNC_PEER_ROLLBACK_EVENT: &str = "sync.peer_rollback";
pub const SYNC_CHAIN_SWITCH_EVENT: &str = "sync.chain_switch";
pub const SYNC_BLOCK_FETCH_EVENT: &str = "sync.block_fetch";
pub const SYNC_BLOCK_FETCH_BLOCKED_EVENT: &str = "sync.block_fetch.blocked";
pub const SYNC_LOCAL_HEADER_APPLY_EVENT: &str = "sync.local_header_apply";
pub const SYNC_LOCAL_FIXTURE_BLOCK_APPLY_EVENT: &str = "sync.local_fixture_block_apply";
pub const SYNC_LOCAL_ROLLBACK_EVENT: &str = "sync.local_rollback";
pub const SYNC_EXECUTION_FAILED_EVENT: &str = "sync.execution_failed";
pub const DEFAULT_MAX_TRACKED_PEERS: usize = 200;
pub const DEFAULT_STALE_TIP_THRESHOLD: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerTipEvent {
    pub peer_id: String,
    pub tip: ChainTip,
    pub tie_mark: Vec<u8>,
}

impl PeerTipEvent {
    pub fn to_event(&self) -> Event {
        Event::new(
            SYNC_PEER_TIP_EVENT,
            EventPayload::Text(format!(
                "peer={} slot={} bundle={} tie_mark_bytes={}",
                self.peer_id,
                self.tip.point.slot,
                self.tip.bundle_number,
                self.tie_mark.len()
            )),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerRollbackEvent {
    pub peer_id: String,
    pub point: ChainPoint,
    pub tip: ChainTip,
}

impl PeerRollbackEvent {
    pub fn to_event(&self) -> Event {
        Event::new(
            SYNC_PEER_ROLLBACK_EVENT,
            EventPayload::Text(format!(
                "peer={} rollback_slot={} tip_slot={} tip_bundle={}",
                self.peer_id, self.point.slot, self.tip.point.slot, self.tip.bundle_number
            )),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainSwitchEvent {
    pub previous_peer_id: Option<String>,
    pub new_peer_id: Option<String>,
    pub old_tip: ChainTip,
    pub new_tip: ChainTip,
    pub mode: SyncMode,
}

impl ChainSwitchEvent {
    pub fn to_event(&self) -> Event {
        Event::new(
            SYNC_CHAIN_SWITCH_EVENT,
            EventPayload::Text(format!(
                "previous_peer={} new_peer={} old_bundle={} new_bundle={} mode={}",
                self.previous_peer_id.as_deref().unwrap_or("none"),
                self.new_peer_id.as_deref().unwrap_or("none"),
                self.old_tip.bundle_number,
                self.new_tip.bundle_number,
                self.mode
            )),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockFetchRequest {
    pub peer_id: String,
    pub from: ChainTip,
    pub to: ChainTip,
    pub max_bundles: u64,
}

impl BlockFetchRequest {
    pub fn to_event(&self) -> Event {
        Event::new(
            SYNC_BLOCK_FETCH_EVENT,
            EventPayload::Text(format!(
                "peer={} from_bundle={} to_bundle={} max_bundles={}",
                self.peer_id, self.from.bundle_number, self.to.bundle_number, self.max_bundles
            )),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockFetchPlan {
    pub request: Option<BlockFetchRequest>,
    pub open_paths: bool,
    pub blocked_reason: Option<String>,
}

impl BlockFetchPlan {
    pub fn between(
        peer_id: impl Into<String>,
        local_tip: ChainTip,
        peer_tip: ChainTip,
        max_bundles: u64,
        allow_paths: bool,
    ) -> Self {
        let request = if peer_tip.bundle_number > local_tip.bundle_number && max_bundles > 0 {
            Some(BlockFetchRequest {
                peer_id: peer_id.into(),
                from: local_tip,
                to: peer_tip,
                max_bundles: max_bundles.min(peer_tip.bundle_number - local_tip.bundle_number),
            })
        } else {
            None
        };
        Self {
            open_paths: allow_paths,
            blocked_reason: if allow_paths || request.is_none() {
                None
            } else {
                Some("block fetch is blocked by safety config".to_string())
            },
            request,
        }
    }

    pub fn events(&self) -> Vec<Event> {
        let mut events = Vec::new();
        if let Some(request) = &self.request {
            events.push(request.to_event());
        }
        if let Some(reason) = &self.blocked_reason {
            events.push(Event::new(
                SYNC_BLOCK_FETCH_BLOCKED_EVENT,
                EventPayload::Text(reason.clone()),
            ));
        }
        events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackRecoveryPlan {
    pub rollback: PeerRollbackEvent,
    pub fetch_after_rollback: Option<BlockFetchRequest>,
    pub open_paths: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalHeaderApplyReport {
    pub from: ChainTip,
    pub to: ChainTip,
    pub applied: usize,
    pub chain_switch: Option<ChainSwitchEvent>,
}

impl LocalHeaderApplyReport {
    pub fn summary_line(&self) -> String {
        format!(
            "local_header_apply from_bundle={} to_bundle={} applied={} chain_switch={}",
            self.from.bundle_number,
            self.to.bundle_number,
            self.applied,
            self.chain_switch.is_some(),
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            SYNC_LOCAL_HEADER_APPLY_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn events(&self) -> Vec<Event> {
        let mut events = vec![self.to_event()];
        if let Some(chain_switch) = &self.chain_switch {
            events.push(chain_switch.to_event());
        }
        events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalFixtureBlockApplyReport {
    pub headers: LocalHeaderApplyReport,
    pub validations: Vec<BlockEraValidationReport>,
}

impl LocalFixtureBlockApplyReport {
    pub fn summary_line(&self) -> String {
        format!(
            "local_fixture_block_apply from_bundle={} to_bundle={} applied={} validation_reports={}",
            self.headers.from.bundle_number,
            self.headers.to.bundle_number,
            self.headers.applied,
            self.validations.len(),
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            SYNC_LOCAL_FIXTURE_BLOCK_APPLY_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn events(&self) -> Vec<Event> {
        let mut events = vec![self.to_event()];
        events.extend(self.headers.events());
        for validation in &self.validations {
            events.extend(validation.events());
        }
        events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRollbackExecution {
    pub from: ChainTip,
    pub to: ChainTip,
    pub removed_headers: usize,
    pub chain_switch: Option<ChainSwitchEvent>,
}

impl LocalRollbackExecution {
    pub fn summary_line(&self) -> String {
        format!(
            "local_rollback from_bundle={} to_bundle={} removed_headers={} chain_switch={}",
            self.from.bundle_number,
            self.to.bundle_number,
            self.removed_headers,
            self.chain_switch.is_some(),
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            SYNC_LOCAL_ROLLBACK_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn events(&self) -> Vec<Event> {
        let mut events = vec![self.to_event()];
        if let Some(chain_switch) = &self.chain_switch {
            events.push(chain_switch.to_event());
        }
        events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRollbackRecoveryExecutionReport {
    pub plan: RollbackRecoveryPlan,
    pub execution: LocalRollbackExecution,
}

impl LocalRollbackRecoveryExecutionReport {
    pub fn events(&self) -> Vec<Event> {
        let mut events = self.plan.events();
        events.extend(self.execution.events());
        events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncExecutionError {
    EmptyHeaderHash,
    HeaderSlotWentBackwards { previous: u64, candidate: u64 },
    HeaderBundleNotNext { expected: u64, actual: u64 },
    FetchFromMismatch { local: ChainTip, planned: ChainTip },
    FetchExceededPlan { max: u64, actual: u64 },
    FetchPastTarget { target: ChainTip, actual: ChainTip },
    LocalBlockDecode(BlockDecodeError),
    RollbackAfterTip { rollback_slot: u64, tip_slot: u64 },
    RollbackPointNotFound(ChainPoint),
}

impl std::fmt::Display for SyncExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyHeaderHash => f.write_str("sync header hash must not be empty"),
            Self::HeaderSlotWentBackwards {
                previous,
                candidate,
            } => write!(
                f,
                "sync header slot went backwards: previous={previous} candidate={candidate}"
            ),
            Self::HeaderBundleNotNext { expected, actual } => write!(
                f,
                "sync header bundle is not contiguous: expected={expected} actual={actual}"
            ),
            Self::FetchFromMismatch { local, planned } => write!(
                f,
                "fetch result starts from wrong tip: local bundle={} planned bundle={}",
                local.bundle_number, planned.bundle_number
            ),
            Self::FetchExceededPlan { max, actual } => {
                write!(f, "fetch result exceeded plan: max={max} actual={actual}")
            }
            Self::FetchPastTarget { target, actual } => write!(
                f,
                "fetch result passed target: target bundle={} actual bundle={}",
                target.bundle_number, actual.bundle_number
            ),
            Self::LocalBlockDecode(err) => write!(f, "local fixture block decode failed: {err}"),
            Self::RollbackAfterTip {
                rollback_slot,
                tip_slot,
            } => write!(
                f,
                "rollback slot {rollback_slot} is after local tip slot {tip_slot}"
            ),
            Self::RollbackPointNotFound(point) => write!(f, "rollback point not found: {point}"),
        }
    }
}

impl SyncExecutionError {
    pub fn summary_line(&self) -> String {
        match self {
            Self::EmptyHeaderHash => "sync_execution_failed reason=empty_header_hash".to_string(),
            Self::HeaderSlotWentBackwards {
                previous,
                candidate,
            } => format!(
                "sync_execution_failed reason=header_slot_went_backwards previous={previous} candidate={candidate}"
            ),
            Self::HeaderBundleNotNext { expected, actual } => format!(
                "sync_execution_failed reason=header_bundle_not_next expected={expected} actual={actual}"
            ),
            Self::FetchFromMismatch { local, planned } => format!(
                "sync_execution_failed reason=fetch_from_mismatch local_bundle={} planned_bundle={}",
                local.bundle_number, planned.bundle_number
            ),
            Self::FetchExceededPlan { max, actual } => format!(
                "sync_execution_failed reason=fetch_exceeded_plan max={max} actual={actual}"
            ),
            Self::FetchPastTarget { target, actual } => format!(
                "sync_execution_failed reason=fetch_past_target target_bundle={} actual_bundle={}",
                target.bundle_number, actual.bundle_number
            ),
            Self::LocalBlockDecode(_) => {
                "sync_execution_failed reason=local_block_decode".to_string()
            }
            Self::RollbackAfterTip {
                rollback_slot,
                tip_slot,
            } => format!(
                "sync_execution_failed reason=rollback_after_tip rollback_slot={rollback_slot} tip_slot={tip_slot}"
            ),
            Self::RollbackPointNotFound(point) => format!(
                "sync_execution_failed reason=rollback_point_not_found slot={}",
                point.slot
            ),
        }
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            SYNC_EXECUTION_FAILED_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn events(&self) -> Vec<Event> {
        let mut events = vec![self.to_event()];
        if let Self::LocalBlockDecode(err) = self {
            events.extend(err.events());
        }
        events
    }
}

impl std::error::Error for SyncExecutionError {}

impl RollbackRecoveryPlan {
    pub fn after_rollback(
        peer_id: impl Into<String>,
        point: ChainPoint,
        rolled_tip: ChainTip,
        target_tip: ChainTip,
        allow_paths: bool,
    ) -> Self {
        let peer_id = peer_id.into();
        let fetch_after_rollback = if target_tip.bundle_number > rolled_tip.bundle_number {
            Some(BlockFetchRequest {
                peer_id: peer_id.clone(),
                from: rolled_tip,
                to: target_tip,
                max_bundles: target_tip.bundle_number - rolled_tip.bundle_number,
            })
        } else {
            None
        };
        Self {
            rollback: PeerRollbackEvent {
                peer_id,
                point,
                tip: rolled_tip,
            },
            fetch_after_rollback,
            open_paths: allow_paths,
        }
    }

    pub fn events(&self) -> Vec<Event> {
        let mut events = vec![self.rollback.to_event()];
        if let Some(request) = &self.fetch_after_rollback {
            events.push(request.to_event());
        }
        events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncPeerUpdatePlan {
    pub peer_tip: PeerTipEvent,
    pub chain_switch: Option<ChainSwitchEvent>,
    pub fetch_plan: BlockFetchPlan,
}

impl SyncPeerUpdatePlan {
    pub fn events(&self) -> Vec<Event> {
        let mut events = vec![self.peer_tip.to_event()];
        if let Some(chain_switch) = &self.chain_switch {
            events.push(chain_switch.to_event());
        }
        events.extend(self.fetch_plan.events());
        events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerChainTip {
    pub peer_id: String,
    pub tip: ChainTip,
    pub observed_tip: ChainTip,
    pub tie_mark: Vec<u8>,
    pub selection_view: TipSelectionView,
    pub last_updated: SystemTime,
    observed_slots: Vec<u64>,
}

impl PeerChainTip {
    pub fn new(peer_id: impl Into<String>, tip: ChainTip, tie_mark: Vec<u8>) -> Self {
        let selection_view = TipSelectionView::from_tip(tip, &tie_mark, TieBreakConfig::unknown());
        Self {
            peer_id: peer_id.into(),
            tip,
            observed_tip: tip,
            tie_mark,
            selection_view,
            last_updated: SystemTime::now(),
            observed_slots: Vec::new(),
        }
    }

    pub fn update_tip(&mut self, tip: ChainTip, tie_mark: Vec<u8>) {
        self.update_tip_with_observed(tip, tip, tie_mark);
    }

    pub fn update_tip_with_observed(
        &mut self,
        tip: ChainTip,
        observed_tip: ChainTip,
        tie_mark: Vec<u8>,
    ) {
        let selection_view =
            TipSelectionView::from_tip(observed_tip, &tie_mark, TieBreakConfig::unknown());
        self.update_tip_with_selection_view(tip, observed_tip, tie_mark, selection_view);
    }

    pub fn update_tip_with_selection_view(
        &mut self,
        tip: ChainTip,
        observed_tip: ChainTip,
        tie_mark: Vec<u8>,
        selection_view: TipSelectionView,
    ) {
        self.tip = tip;
        self.observed_tip = observed_tip;
        self.tie_mark = tie_mark;
        self.selection_view = selection_view;
        self.last_updated = SystemTime::now();
    }

    pub fn apply_rollback(&mut self, point: ChainPoint, tip: ChainTip) {
        self.tip = tip;
        self.observed_tip = tip;
        self.tie_mark.clear();
        self.selection_view = TipSelectionView::from_tip(tip, &[], TieBreakConfig::unknown());
        self.last_updated = SystemTime::now();
        if point.slot == 0 || self.observed_slots.is_empty() {
            self.observed_slots.clear();
            return;
        }
        let keep_until = self
            .observed_slots
            .iter()
            .position(|slot| *slot > point.slot)
            .unwrap_or(self.observed_slots.len());
        self.observed_slots.truncate(keep_until);
    }

    pub fn record_observed_slot(&mut self, slot: u64, window: u64) {
        if slot == 0 {
            self.observed_slots.clear();
            return;
        }
        while self.observed_slots.last().is_some_and(|last| *last > slot) {
            self.observed_slots.pop();
        }
        if self.observed_slots.last().is_none_or(|last| *last < slot) {
            self.observed_slots.push(slot);
        }
        if window == 0 {
            if self.observed_slots.len() > 1 {
                let last = *self.observed_slots.last().expect("non-empty checked");
                self.observed_slots.clear();
                self.observed_slots.push(last);
            }
            return;
        }
        let cutoff = slot.saturating_sub(window.saturating_sub(1)).max(1);
        let prune = self
            .observed_slots
            .iter()
            .position(|observed| *observed >= cutoff)
            .unwrap_or(self.observed_slots.len());
        if prune > 0 {
            self.observed_slots.drain(0..prune);
        }
    }

    pub fn observed_density(&self, window: u64) -> u64 {
        let Some(latest_slot) = self.observed_slots.last().copied() else {
            return 0;
        };
        if window == 0 {
            return self.observed_slots.len() as u64;
        }
        let cutoff = latest_slot.saturating_sub(window.saturating_sub(1)).max(1);
        self.observed_slots
            .iter()
            .filter(|slot| **slot >= cutoff)
            .count() as u64
    }

    pub fn selection_tip(&self) -> ChainTip {
        if self.observed_tip.bundle_number > 0
            || self.observed_tip.point.slot > 0
            || self.observed_tip.point.hash != [0; 32]
        {
            self.observed_tip
        } else {
            self.tip
        }
    }

    pub fn touch(&mut self) {
        self.last_updated = SystemTime::now();
    }

    pub fn is_stale_at(&self, threshold: Duration, now: SystemTime) -> bool {
        now.duration_since(self.last_updated)
            .map(|age| age > threshold)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncConfig {
    pub security_param: u64,
    pub first_light_mode: bool,
    pub first_light_window_slots: u64,
    pub max_tracked_peers: usize,
    pub stale_tip_threshold: Duration,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            security_param: 0,
            first_light_mode: false,
            first_light_window_slots: 0,
            max_tracked_peers: DEFAULT_MAX_TRACKED_PEERS,
            stale_tip_threshold: DEFAULT_STALE_TIP_THRESHOLD,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChainSync {
    config: SyncConfig,
    mode: SyncMode,
    peer_tips: HashMap<String, PeerChainTip>,
    best_peer_id: Option<String>,
    local_tip: ChainTip,
    local_headers: Vec<ChainTip>,
}

impl ChainSync {
    pub fn new(config: SyncConfig) -> Self {
        let mode = if config.first_light_mode {
            SyncMode::FirstLight
        } else {
            SyncMode::ChainSelection
        };
        Self {
            config,
            mode,
            peer_tips: HashMap::new(),
            best_peer_id: None,
            local_tip: ChainTip::ORIGIN,
            local_headers: Vec::new(),
        }
    }

    pub fn mode(&self) -> SyncMode {
        self.mode
    }

    pub fn best_peer_id(&self) -> Option<&str> {
        self.best_peer_id.as_deref()
    }

    pub fn local_tip(&self) -> ChainTip {
        self.local_tip
    }

    pub fn update_local_tip(&mut self, tip: ChainTip) -> Option<ChainSwitchEvent> {
        let old_best = self.best_peer_id.clone();
        let old_tip = self.best_tip();
        self.local_tip = tip;
        self.advance_mode();
        self.evaluate_best_peer();
        self.chain_switch_event_if_changed(old_best, old_tip)
    }

    pub fn apply_local_headers(
        &mut self,
        headers: impl IntoIterator<Item = ChainTip>,
    ) -> Result<LocalHeaderApplyReport, SyncExecutionError> {
        let headers = headers.into_iter().collect::<Vec<_>>();
        let from = self.local_tip;
        let mut cursor = from;
        for header in &headers {
            validate_next_local_header(cursor, *header)?;
            cursor = *header;
        }

        let old_best = self.best_peer_id.clone();
        let old_tip = self.best_tip();
        self.local_headers.extend(headers.iter().copied());
        self.local_tip = cursor;
        self.advance_mode();
        self.evaluate_best_peer();
        Ok(LocalHeaderApplyReport {
            from,
            to: self.local_tip,
            applied: headers.len(),
            chain_switch: self.chain_switch_event_if_changed(old_best, old_tip),
        })
    }

    pub fn apply_block_fetch_result(
        &mut self,
        request: &BlockFetchRequest,
        headers: impl IntoIterator<Item = ChainTip>,
    ) -> Result<LocalHeaderApplyReport, SyncExecutionError> {
        if request.from != self.local_tip {
            return Err(SyncExecutionError::FetchFromMismatch {
                local: self.local_tip,
                planned: request.from,
            });
        }
        let headers = headers.into_iter().collect::<Vec<_>>();
        let actual = headers.len() as u64;
        if actual > request.max_bundles {
            return Err(SyncExecutionError::FetchExceededPlan {
                max: request.max_bundles,
                actual,
            });
        }
        if let Some(last) = headers.last() {
            if last.bundle_number > request.to.bundle_number {
                return Err(SyncExecutionError::FetchPastTarget {
                    target: request.to,
                    actual: *last,
                });
            }
        }
        self.apply_local_headers(headers)
    }

    pub fn apply_block_fetch_fixture_bundles(
        &mut self,
        request: &BlockFetchRequest,
        encoded_bundles: impl IntoIterator<Item = Vec<u8>>,
        catalog: &BlockEraCatalog,
        major_version: u64,
    ) -> Result<LocalHeaderApplyReport, SyncExecutionError> {
        self.apply_block_fetch_fixture_bundles_report(
            request,
            encoded_bundles,
            catalog,
            major_version,
        )
        .map(|report| report.headers)
    }

    pub fn apply_block_fetch_fixture_bundles_report(
        &mut self,
        request: &BlockFetchRequest,
        encoded_bundles: impl IntoIterator<Item = Vec<u8>>,
        catalog: &BlockEraCatalog,
        major_version: u64,
    ) -> Result<LocalFixtureBlockApplyReport, SyncExecutionError> {
        if request.from != self.local_tip {
            return Err(SyncExecutionError::FetchFromMismatch {
                local: self.local_tip,
                planned: request.from,
            });
        }
        let mut cursor = self.local_tip;
        let mut headers = Vec::new();
        let mut validations = Vec::new();
        for encoded in encoded_bundles {
            let (bundle, validation) = catalog
                .decode_local_fixture_bundle_report_for_major_version(
                    &encoded,
                    Some(cursor),
                    major_version,
                )
                .map_err(SyncExecutionError::LocalBlockDecode)?;
            let tip = bundle.header.tip();
            validate_next_local_header(cursor, tip)?;
            headers.push(tip);
            validations.push(validation);
            cursor = tip;
        }
        let headers = self.apply_block_fetch_result(request, headers)?;
        Ok(LocalFixtureBlockApplyReport {
            headers,
            validations,
        })
    }

    pub fn execute_local_rollback(
        &mut self,
        point: ChainPoint,
    ) -> Result<LocalRollbackExecution, SyncExecutionError> {
        if point.slot > self.local_tip.point.slot {
            return Err(SyncExecutionError::RollbackAfterTip {
                rollback_slot: point.slot,
                tip_slot: self.local_tip.point.slot,
            });
        }
        let from = self.local_tip;
        let (to, keep_headers) = if point == self.local_tip.point {
            (self.local_tip, self.local_headers.len())
        } else if point == ChainPoint::ORIGIN {
            (ChainTip::ORIGIN, 0)
        } else {
            let index = self
                .local_headers
                .iter()
                .position(|header| header.point == point)
                .ok_or(SyncExecutionError::RollbackPointNotFound(point))?;
            (self.local_headers[index], index + 1)
        };

        let old_best = self.best_peer_id.clone();
        let old_tip = self.best_tip();
        let removed_headers = self.local_headers.len().saturating_sub(keep_headers);
        self.local_headers.truncate(keep_headers);
        self.local_tip = to;
        self.advance_mode();
        self.evaluate_best_peer();
        Ok(LocalRollbackExecution {
            from,
            to,
            removed_headers,
            chain_switch: self.chain_switch_event_if_changed(old_best, old_tip),
        })
    }

    pub fn execute_rollback_recovery_plan(
        &mut self,
        plan: &RollbackRecoveryPlan,
    ) -> Result<LocalRollbackExecution, SyncExecutionError> {
        self.execute_rollback_recovery_plan_report(plan)
            .map(|report| report.execution)
    }

    pub fn execute_rollback_recovery_plan_report(
        &mut self,
        plan: &RollbackRecoveryPlan,
    ) -> Result<LocalRollbackRecoveryExecutionReport, SyncExecutionError> {
        let execution = self.execute_local_rollback(plan.rollback.point)?;
        Ok(LocalRollbackRecoveryExecutionReport {
            plan: plan.clone(),
            execution,
        })
    }

    pub fn update_peer_tip(
        &mut self,
        peer_id: impl Into<String>,
        tip: ChainTip,
        tie_mark: Vec<u8>,
    ) -> Option<ChainSwitchEvent> {
        let peer_id = peer_id.into();
        let old_best = self.best_peer_id.clone();
        let old_tip = self.best_tip();
        if !self.peer_tips.contains_key(&peer_id) {
            self.evict_if_needed();
            self.peer_tips.insert(
                peer_id.clone(),
                PeerChainTip::new(peer_id.clone(), tip, tie_mark),
            );
        } else if let Some(peer_tip) = self.peer_tips.get_mut(&peer_id) {
            peer_tip.update_tip(tip, tie_mark);
        }
        let window = self.first_light_window();
        if let Some(peer_tip) = self.peer_tips.get_mut(&peer_id) {
            peer_tip.record_observed_slot(tip.point.slot, window);
        }
        self.advance_mode();
        self.evaluate_best_peer();
        self.chain_switch_event_if_changed(old_best, old_tip)
    }

    pub fn update_peer_tip_with_fetch_plan(
        &mut self,
        peer_id: impl Into<String>,
        tip: ChainTip,
        tie_mark: Vec<u8>,
        max_bundles: u64,
        allow_paths: bool,
    ) -> SyncPeerUpdatePlan {
        let peer_id = peer_id.into();
        let peer_tip = PeerTipEvent {
            peer_id: peer_id.clone(),
            tip,
            tie_mark: tie_mark.clone(),
        };
        let fetch_plan = BlockFetchPlan::between(
            peer_id.clone(),
            self.local_tip,
            tip,
            max_bundles,
            allow_paths,
        );
        let chain_switch = self.update_peer_tip(peer_id, tip, tie_mark);
        SyncPeerUpdatePlan {
            peer_tip,
            chain_switch,
            fetch_plan,
        }
    }

    pub fn apply_peer_rollback(
        &mut self,
        peer_id: &str,
        point: ChainPoint,
        tip: ChainTip,
    ) -> Option<ChainSwitchEvent> {
        let old_best = self.best_peer_id.clone();
        let old_tip = self.best_tip();
        if let Some(peer_tip) = self.peer_tips.get_mut(peer_id) {
            peer_tip.apply_rollback(point, tip);
        }
        self.evaluate_best_peer();
        self.chain_switch_event_if_changed(old_best, old_tip)
    }

    pub fn remove_stale_peer_tips_at(&mut self, now: SystemTime) -> usize {
        let threshold = self.config.stale_tip_threshold;
        let old_len = self.peer_tips.len();
        self.peer_tips
            .retain(|_, peer_tip| !peer_tip.is_stale_at(threshold, now));
        if self
            .best_peer_id
            .as_ref()
            .is_some_and(|id| !self.peer_tips.contains_key(id))
        {
            self.best_peer_id = None;
        }
        old_len - self.peer_tips.len()
    }

    fn first_light_window(&self) -> u64 {
        if self.config.first_light_window_slots > 0 {
            self.config.first_light_window_slots
        } else if self.config.security_param > 0 {
            self.config.security_param.saturating_mul(3)
        } else {
            DEFAULT_FIRST_LIGHT_WINDOW_SLOTS
        }
    }

    fn best_known_first_light_slot(&self) -> u64 {
        self.peer_tips
            .values()
            .map(|peer_tip| peer_tip.selection_tip().point.slot)
            .max()
            .unwrap_or(0)
    }

    fn advance_mode(&mut self) -> bool {
        if self.mode != SyncMode::FirstLight {
            return false;
        }
        let best_slot = self.best_known_first_light_slot();
        if best_slot == 0 {
            return false;
        }
        if self
            .local_tip
            .point
            .slot
            .saturating_add(self.first_light_window())
            >= best_slot
        {
            self.mode = SyncMode::ChainSelection;
            true
        } else {
            false
        }
    }

    fn evaluate_best_peer(&mut self) {
        self.best_peer_id = match self.mode {
            SyncMode::FirstLight => self.best_peer_by_density(),
            SyncMode::ChainSelection => self.best_peer_by_chain_selection(),
        };
    }

    fn best_peer_by_density(&self) -> Option<String> {
        let window = self.first_light_window();
        self.peer_tips
            .values()
            .max_by(|a, b| {
                a.observed_density(window)
                    .cmp(&b.observed_density(window))
                    .then_with(|| {
                        a.selection_tip()
                            .point
                            .slot
                            .cmp(&b.selection_tip().point.slot)
                    })
                    .then_with(|| {
                        a.selection_tip()
                            .bundle_number
                            .cmp(&b.selection_tip().bundle_number)
                    })
                    .then_with(|| b.peer_id.cmp(&a.peer_id))
            })
            .map(|peer_tip| peer_tip.peer_id.clone())
    }

    fn best_peer_by_chain_selection(&self) -> Option<String> {
        let mut best: Option<&PeerChainTip> = None;
        for candidate in self.peer_tips.values() {
            let Some(current) = best else {
                best = Some(candidate);
                continue;
            };
            let comparison = compare_chain_tips(
                current.selection_tip(),
                candidate.selection_tip(),
                &current.selection_view,
                &candidate.selection_view,
            );
            if comparison == TipOrdering::RightBetter {
                best = Some(candidate);
            }
        }
        best.map(|peer_tip| peer_tip.peer_id.clone())
    }

    fn evict_if_needed(&mut self) {
        let max = self.config.max_tracked_peers.max(1);
        if self.peer_tips.len() < max {
            return;
        }
        let evict = self
            .peer_tips
            .iter()
            .min_by_key(|(_, peer_tip)| peer_tip.last_updated)
            .map(|(id, _)| id.clone());
        if let Some(id) = evict {
            self.peer_tips.remove(&id);
            if self.best_peer_id.as_deref() == Some(id.as_str()) {
                self.best_peer_id = None;
            }
        }
    }

    fn best_tip(&self) -> ChainTip {
        self.best_peer_id
            .as_ref()
            .and_then(|id| self.peer_tips.get(id))
            .map(PeerChainTip::selection_tip)
            .unwrap_or(self.local_tip)
    }

    fn chain_switch_event_if_changed(
        &self,
        old_best: Option<String>,
        old_tip: ChainTip,
    ) -> Option<ChainSwitchEvent> {
        if old_best == self.best_peer_id {
            return None;
        }
        Some(ChainSwitchEvent {
            previous_peer_id: old_best,
            new_peer_id: self.best_peer_id.clone(),
            old_tip,
            new_tip: self.best_tip(),
            mode: self.mode,
        })
    }
}

fn validate_next_local_header(
    previous: ChainTip,
    candidate: ChainTip,
) -> Result<(), SyncExecutionError> {
    if candidate.point.hash == [0; 32] {
        return Err(SyncExecutionError::EmptyHeaderHash);
    }
    if candidate.point.slot < previous.point.slot {
        return Err(SyncExecutionError::HeaderSlotWentBackwards {
            previous: previous.point.slot,
            candidate: candidate.point.slot,
        });
    }
    let expected = previous.bundle_number.saturating_add(1);
    if candidate.bundle_number != expected {
        return Err(SyncExecutionError::HeaderBundleNotNext {
            expected,
            actual: candidate.bundle_number,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::blocks::{
        local_fixture_body_hash, local_ledger_fixture_catalog, BlockBundle, BlockDecodeError,
        BlockHeader, BlockItem, BlockValidationError,
    };

    #[test]
    fn first_light_window_matches_three_k_over_f() {
        assert_eq!(first_light_window_slots(2_160, 0.05), 129_600);
    }

    #[test]
    fn first_light_window_uses_default_for_invalid_params() {
        assert_eq!(first_light_window_slots(0, 0.05), 6_480);
        assert_eq!(first_light_window_slots(2_160, 0.0), 6_480);
        assert_eq!(first_light_window_slots(2_160, f64::NAN), 6_480);
    }

    fn tip(slot: u64, bundle_number: u64, hash: u8) -> ChainTip {
        ChainTip::new(ChainPoint::new(slot, [hash; 32]), bundle_number)
    }

    fn fixture_bundle_after(previous: ChainTip, hash: u8) -> BlockBundle {
        let items = vec![BlockItem::new(format!("tx-{hash}"), vec![b't', b'x', hash])];
        BlockBundle {
            header: BlockHeader {
                point: ChainPoint::new(previous.point.slot + 1, [hash; 32]),
                bundle_number: previous.bundle_number + 1,
                parent_hash: previous.point.hash,
                body_hash: local_fixture_body_hash(&items),
            },
            items,
        }
    }

    #[test]
    fn peer_chain_tip_tracks_bounded_density() {
        let mut peer_tip = PeerChainTip::new("p", tip(1, 1, 1), vec![]);
        for slot in 1..=10 {
            peer_tip.record_observed_slot(slot, 5);
        }
        assert_eq!(peer_tip.observed_density(5), 5);
        peer_tip.record_observed_slot(7, 5);
        assert_eq!(peer_tip.observed_density(5), 2);
    }

    #[test]
    fn sync_selects_best_chain_peer() {
        let mut sync = ChainSync::new(SyncConfig::default());
        let event = sync.update_peer_tip("a", tip(10, 10, 1), vec![2; 64]);
        assert_eq!(event.unwrap().new_peer_id.as_deref(), Some("a"));
        let event = sync.update_peer_tip("b", tip(11, 11, 2), vec![1; 64]);
        assert_eq!(event.unwrap().new_peer_id.as_deref(), Some("b"));
        assert_eq!(sync.best_peer_id(), Some("b"));
    }

    #[test]
    fn first_light_mode_exits_when_local_tip_is_inside_window() {
        let mut sync = ChainSync::new(SyncConfig {
            first_light_mode: true,
            first_light_window_slots: 10,
            ..SyncConfig::default()
        });
        sync.update_peer_tip("a", tip(100, 50, 1), vec![]);
        assert_eq!(sync.mode(), SyncMode::FirstLight);
        sync.update_local_tip(tip(91, 45, 3));
        assert_eq!(sync.mode(), SyncMode::ChainSelection);
    }

    #[test]
    fn block_fetch_plan_stays_closed_without_path_permission() {
        let plan = BlockFetchPlan::between("peer", tip(10, 10, 1), tip(15, 15, 2), 2, false);
        assert!(!plan.open_paths);
        assert!(plan.blocked_reason.is_some());
        let request = plan.request.unwrap();
        assert_eq!(request.max_bundles, 2);
        assert_eq!(request.from.bundle_number, 10);
        assert_eq!(request.to.bundle_number, 15);
    }

    #[test]
    fn local_fetch_result_applies_contiguous_headers() {
        let mut sync = ChainSync::new(SyncConfig::default());
        sync.apply_local_headers([tip(1, 1, 1)]).unwrap();
        let request = BlockFetchRequest {
            peer_id: "peer".to_string(),
            from: tip(1, 1, 1),
            to: tip(3, 3, 3),
            max_bundles: 2,
        };

        let report = sync
            .apply_block_fetch_result(&request, [tip(2, 2, 2), tip(3, 3, 3)])
            .unwrap();
        assert_eq!(report.from, tip(1, 1, 1));
        assert_eq!(report.to, tip(3, 3, 3));
        assert_eq!(report.applied, 2);
        assert_eq!(
            report.summary_line(),
            "local_header_apply from_bundle=1 to_bundle=3 applied=2 chain_switch=false"
        );
        let events = report.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), SYNC_LOCAL_HEADER_APPLY_EVENT);
        assert_eq!(sync.local_tip(), tip(3, 3, 3));
    }

    #[test]
    fn local_fetch_result_rejects_header_gap_without_mutating_tip() {
        let mut sync = ChainSync::new(SyncConfig::default());
        sync.apply_local_headers([tip(1, 1, 1)]).unwrap();
        let request = BlockFetchRequest {
            peer_id: "peer".to_string(),
            from: tip(1, 1, 1),
            to: tip(3, 3, 3),
            max_bundles: 2,
        };

        assert_eq!(
            sync.apply_block_fetch_result(&request, [tip(3, 3, 3)]),
            Err(SyncExecutionError::HeaderBundleNotNext {
                expected: 2,
                actual: 3,
            })
        );
        assert_eq!(sync.local_tip(), tip(1, 1, 1));
    }

    #[test]
    fn sync_execution_error_emits_local_failure_event() {
        let err = SyncExecutionError::HeaderBundleNotNext {
            expected: 2,
            actual: 3,
        };
        let event = err.to_event();
        let events = err.events();

        assert_eq!(
            err.summary_line(),
            "sync_execution_failed reason=header_bundle_not_next expected=2 actual=3"
        );
        assert_eq!(event.name.as_str(), SYNC_EXECUTION_FAILED_EVENT);
        assert_eq!(event.payload, EventPayload::Text(err.summary_line()));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), SYNC_EXECUTION_FAILED_EVENT);
        assert_eq!(events[0].payload, EventPayload::Text(err.summary_line()));
    }

    #[test]
    fn sync_execution_decode_error_emits_nested_failure_events() {
        let err = SyncExecutionError::LocalBlockDecode(BlockDecodeError::TrailingBytes(2));
        let events = err.events();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].name.as_str(), SYNC_EXECUTION_FAILED_EVENT);
        assert_eq!(
            events[1].name.as_str(),
            crate::ledger::blocks::BLOCK_DECODE_FAILED_EVENT
        );
    }

    #[test]
    fn local_fetch_result_applies_encoded_fixture_bundles() {
        let mut sync = ChainSync::new(SyncConfig::default());
        sync.apply_local_headers([tip(1, 1, 1)]).unwrap();
        let bundle_two = fixture_bundle_after(tip(1, 1, 1), 2);
        let bundle_three = fixture_bundle_after(bundle_two.header.tip(), 3);
        let request = BlockFetchRequest {
            peer_id: "peer".to_string(),
            from: tip(1, 1, 1),
            to: tip(3, 3, 3),
            max_bundles: 2,
        };

        let report = sync
            .apply_block_fetch_fixture_bundles(
                &request,
                vec![
                    bundle_two.encode_local().unwrap(),
                    bundle_three.encode_local().unwrap(),
                ],
                &local_ledger_fixture_catalog(),
                6,
            )
            .unwrap();

        assert_eq!(report.from, tip(1, 1, 1));
        assert_eq!(report.to, tip(3, 3, 3));
        assert_eq!(report.applied, 2);
        assert_eq!(sync.local_tip(), tip(3, 3, 3));
    }

    #[test]
    fn local_fetch_result_reports_encoded_fixture_validation() {
        let mut sync = ChainSync::new(SyncConfig::default());
        sync.apply_local_headers([tip(1, 1, 1)]).unwrap();
        let bundle_two = fixture_bundle_after(tip(1, 1, 1), 2);
        let bundle_three = fixture_bundle_after(bundle_two.header.tip(), 3);
        let request = BlockFetchRequest {
            peer_id: "peer".to_string(),
            from: tip(1, 1, 1),
            to: tip(3, 3, 3),
            max_bundles: 2,
        };

        let report = sync
            .apply_block_fetch_fixture_bundles_report(
                &request,
                vec![
                    bundle_two.encode_local().unwrap(),
                    bundle_three.encode_local().unwrap(),
                ],
                &local_ledger_fixture_catalog(),
                6,
            )
            .unwrap();

        assert_eq!(report.headers.from, tip(1, 1, 1));
        assert_eq!(report.headers.to, tip(3, 3, 3));
        assert_eq!(report.headers.applied, 2);
        assert_eq!(report.validations.len(), 2);
        assert!(report
            .validations
            .iter()
            .all(|validation| validation.era_name == "fixture-current"));
        assert!(report
            .validations
            .iter()
            .all(|validation| validation.fixture_integrity_checked));
        assert_eq!(
            report.summary_line(),
            "local_fixture_block_apply from_bundle=1 to_bundle=3 applied=2 validation_reports=2"
        );
        let events = report.events();
        assert_eq!(events.len(), 4);
        assert_eq!(
            events[0].name.as_str(),
            SYNC_LOCAL_FIXTURE_BLOCK_APPLY_EVENT
        );
        assert_eq!(events[1].name.as_str(), SYNC_LOCAL_HEADER_APPLY_EVENT);
        assert_eq!(
            events[2].name.as_str(),
            crate::ledger::blocks::BLOCK_ERA_VALIDATION_EVENT
        );
        assert_eq!(
            events[3].name.as_str(),
            crate::ledger::blocks::BLOCK_ERA_VALIDATION_EVENT
        );
        assert_eq!(sync.local_tip(), tip(3, 3, 3));
    }

    #[test]
    fn local_fetch_result_rejects_bad_fixture_bundle_without_mutating_tip() {
        let mut sync = ChainSync::new(SyncConfig::default());
        sync.apply_local_headers([tip(1, 1, 1)]).unwrap();
        let mut bundle = fixture_bundle_after(tip(1, 1, 1), 2);
        bundle.header.body_hash = [9; 32];
        let request = BlockFetchRequest {
            peer_id: "peer".to_string(),
            from: tip(1, 1, 1),
            to: tip(2, 2, 2),
            max_bundles: 1,
        };

        let result = sync.apply_block_fetch_fixture_bundles(
            &request,
            vec![bundle.encode_local().unwrap()],
            &local_ledger_fixture_catalog(),
            6,
        );

        assert!(matches!(
            result,
            Err(SyncExecutionError::LocalBlockDecode(
                BlockDecodeError::Validation(BlockValidationError::BodyHashMismatch { .. })
            ))
        ));
        assert_eq!(sync.local_tip(), tip(1, 1, 1));
    }

    #[test]
    fn local_rollback_recovery_truncates_applied_headers() {
        let mut sync = ChainSync::new(SyncConfig::default());
        sync.apply_local_headers([tip(1, 1, 1), tip(2, 2, 2), tip(3, 3, 3)])
            .unwrap();
        let plan = RollbackRecoveryPlan::after_rollback(
            "peer",
            tip(1, 1, 1).point,
            tip(1, 1, 1),
            tip(3, 3, 3),
            false,
        );

        let report = sync.execute_rollback_recovery_plan(&plan).unwrap();
        assert_eq!(report.from, tip(3, 3, 3));
        assert_eq!(report.to, tip(1, 1, 1));
        assert_eq!(report.removed_headers, 2);
        assert_eq!(
            report.summary_line(),
            "local_rollback from_bundle=3 to_bundle=1 removed_headers=2 chain_switch=false"
        );
        let events = report.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), SYNC_LOCAL_ROLLBACK_EVENT);
        assert_eq!(sync.local_tip(), tip(1, 1, 1));
    }

    #[test]
    fn local_rollback_recovery_report_emits_plan_and_execution_events() {
        let mut sync = ChainSync::new(SyncConfig::default());
        sync.apply_local_headers([tip(1, 1, 1), tip(2, 2, 2), tip(3, 3, 3)])
            .unwrap();
        let plan = RollbackRecoveryPlan::after_rollback(
            "peer",
            tip(1, 1, 1).point,
            tip(1, 1, 1),
            tip(3, 3, 3),
            false,
        );

        let report = sync.execute_rollback_recovery_plan_report(&plan).unwrap();
        let names = report
            .events()
            .iter()
            .map(|event| event.name.as_str().to_string())
            .collect::<Vec<_>>();

        assert_eq!(report.execution.removed_headers, 2);
        assert_eq!(
            names,
            vec![
                SYNC_PEER_ROLLBACK_EVENT,
                SYNC_BLOCK_FETCH_EVENT,
                SYNC_LOCAL_ROLLBACK_EVENT,
            ]
        );
        assert_eq!(sync.local_tip(), tip(1, 1, 1));
    }

    #[test]
    fn peer_tip_update_plan_emits_local_sync_events() {
        let mut sync = ChainSync::new(SyncConfig::default());
        let plan = sync.update_peer_tip_with_fetch_plan("peer", tip(5, 5, 5), vec![7; 4], 2, false);
        assert_eq!(plan.peer_tip.peer_id, "peer");
        assert_eq!(
            plan.chain_switch.as_ref().unwrap().new_peer_id.as_deref(),
            Some("peer")
        );
        assert_eq!(plan.fetch_plan.request.as_ref().unwrap().max_bundles, 2);
        let names = plan
            .events()
            .iter()
            .map(|event| event.name.as_str().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                SYNC_PEER_TIP_EVENT,
                SYNC_CHAIN_SWITCH_EVENT,
                SYNC_BLOCK_FETCH_EVENT,
                SYNC_BLOCK_FETCH_BLOCKED_EVENT,
            ]
        );
    }

    #[test]
    fn rollback_recovery_plan_records_rollback_and_followup_fetch() {
        let plan = RollbackRecoveryPlan::after_rollback(
            "peer",
            ChainPoint::new(8, [8; 32]),
            tip(8, 8, 8),
            tip(10, 10, 10),
            false,
        );
        assert_eq!(plan.rollback.point.slot, 8);
        assert_eq!(plan.fetch_after_rollback.unwrap().max_bundles, 2);
        assert!(!plan.open_paths);
    }

    #[test]
    fn rollback_recovery_plan_emits_rollback_and_fetch_events() {
        let plan = RollbackRecoveryPlan::after_rollback(
            "peer",
            ChainPoint::new(8, [8; 32]),
            tip(8, 8, 8),
            tip(10, 10, 10),
            false,
        );
        let names = plan
            .events()
            .iter()
            .map(|event| event.name.as_str().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![SYNC_PEER_ROLLBACK_EVENT, SYNC_BLOCK_FETCH_EVENT]
        );
    }
}
