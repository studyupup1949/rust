use crate::events::EventName;

pub mod selection;
use std::fmt;

pub const CHAIN_NEW_HEAD_EVENT: &str = "chain.new_head";
pub const CHAIN_FORK_EVENT: &str = "chain.fork";
pub const BLOCK_OFFERED_EVENT: &str = "chain.block_offered";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChainPoint {
    pub slot: u64,
    pub hash: [u8; 32],
}

impl ChainPoint {
    pub const ORIGIN: Self = Self {
        slot: 0,
        hash: [0; 32],
    };

    pub fn new(slot: u64, hash: [u8; 32]) -> Self {
        Self { slot, hash }
    }
}

impl fmt::Display for ChainPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "slot={} hash={}", self.slot, short_hash(&self.hash))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChainTip {
    pub point: ChainPoint,
    pub bundle_number: u64,
}

impl ChainTip {
    pub const ORIGIN: Self = Self {
        point: ChainPoint::ORIGIN,
        bundle_number: 0,
    };

    pub fn new(point: ChainPoint, bundle_number: u64) -> Self {
        Self {
            point,
            bundle_number,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawBlock {
    pub point: ChainPoint,
    pub bundle_number: u64,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainHeadEvent {
    pub tip: ChainTip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockOfferedEvent {
    pub block: RawBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainRollbackEvent {
    pub from: ChainTip,
    pub to: ChainPoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainForkEvent {
    pub old_tip: ChainTip,
    pub candidate_tip: ChainTip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackJournalEntry {
    pub from: ChainTip,
    pub to: ChainPoint,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RollbackJournal {
    entries: Vec<RollbackJournalEntry>,
}

impl RollbackJournal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(
        &mut self,
        from: ChainTip,
        to: ChainPoint,
        reason: impl Into<String>,
    ) -> &RollbackJournalEntry {
        self.entries.push(RollbackJournalEntry {
            from,
            to,
            reason: reason.into(),
        });
        self.entries.last().expect("entry was just pushed")
    }

    pub fn entries(&self) -> &[RollbackJournalEntry] {
        &self.entries
    }

    pub fn last(&self) -> Option<&RollbackJournalEntry> {
        self.entries.last()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkCheck {
    SamePoint,
    SameSlotDifferentHash,
    DifferentSlot,
}

pub fn check_fork(left: ChainTip, right: ChainTip) -> ForkCheck {
    if left.point == right.point {
        ForkCheck::SamePoint
    } else if left.point.slot == right.point.slot {
        ForkCheck::SameSlotDifferentHash
    } else {
        ForkCheck::DifferentSlot
    }
}

#[derive(Debug, Clone)]
pub struct Chain {
    tip: ChainTip,
}

impl Default for Chain {
    fn default() -> Self {
        Self {
            tip: ChainTip::ORIGIN,
        }
    }
}

impl Chain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tip(&self) -> ChainTip {
        self.tip
    }

    pub fn apply_tip(&mut self, tip: ChainTip) -> Result<ChainHeadEvent, ChainError> {
        if tip.bundle_number < self.tip.bundle_number {
            return Err(ChainError::NonMonotonicBundleNumber {
                current: self.tip.bundle_number,
                candidate: tip.bundle_number,
            });
        }
        self.tip = tip;
        Ok(ChainHeadEvent { tip })
    }

    pub fn rollback_to(&mut self, point: ChainPoint) -> ChainRollbackEvent {
        let from = self.tip;
        self.tip = ChainTip::new(point, self.tip.bundle_number);
        ChainRollbackEvent { from, to: point }
    }

    pub fn new_event_name() -> EventName {
        EventName::new(CHAIN_NEW_HEAD_EVENT)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainError {
    NonMonotonicBundleNumber { current: u64, candidate: u64 },
}

impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonMonotonicBundleNumber { current, candidate } => write!(
                f,
                "candidate bundle number {candidate} is behind current bundle number {current}"
            ),
        }
    }
}

impl std::error::Error for ChainError {}

fn short_hash(hash: &[u8; 32]) -> String {
    let mut out = String::with_capacity(8);
    for byte in &hash[..4] {
        out.push(nibble(byte >> 4));
        out.push(nibble(byte & 0x0f));
    }
    out
}

fn nibble(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + value - 10) as char,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_rejects_backward_block_number_without_paths() {
        let mut chain = Chain::new();
        let point = ChainPoint::new(10, [1; 32]);
        chain.apply_tip(ChainTip::new(point, 10)).unwrap();
        assert_eq!(
            chain.apply_tip(ChainTip::new(ChainPoint::new(11, [2; 32]), 9)),
            Err(ChainError::NonMonotonicBundleNumber {
                current: 10,
                candidate: 9,
            })
        );
    }

    #[test]
    fn rollback_journal_records_local_rollbacks() {
        let from = ChainTip::new(ChainPoint::new(10, [1; 32]), 10);
        let to = ChainPoint::new(8, [2; 32]);
        let mut journal = RollbackJournal::new();
        journal.record(from, to, "test");
        assert_eq!(journal.len(), 1);
        assert_eq!(journal.last().unwrap().to, to);
    }

    #[test]
    fn fork_check_detects_same_slot_hash_mismatch() {
        let left = ChainTip::new(ChainPoint::new(10, [1; 32]), 10);
        let right = ChainTip::new(ChainPoint::new(10, [2; 32]), 10);
        assert_eq!(check_fork(left, right), ForkCheck::SameSlotDifferentHash);
    }
}
