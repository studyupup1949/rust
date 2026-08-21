use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakeEntry {
    pub pool_id: Vec<u8>,
    pub stake: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakeSnapshot {
    pub age: u64,
    pub entries: Vec<StakeEntry>,
}

impl StakeSnapshot {
    pub fn validate(&self) -> Result<(), BlockProductionError> {
        let mut seen = HashSet::new();
        for entry in &self.entries {
            if entry.pool_id.is_empty() {
                return Err(BlockProductionError::EmptyStakePoolId);
            }
            if !seen.insert(entry.pool_id.as_slice()) {
                return Err(BlockProductionError::DuplicateStakePoolId);
            }
        }
        Ok(())
    }

    pub fn total_stake(&self) -> u64 {
        self.entries
            .iter()
            .fold(0_u64, |total, entry| total.saturating_add(entry.stake))
    }

    pub fn pool_stake(&self, pool_id: &[u8]) -> Option<u64> {
        self.entries
            .iter()
            .find(|entry| entry.pool_id == pool_id)
            .map(|entry| entry.stake)
    }

    pub fn pool_ratio(&self, pool_id: &[u8]) -> f64 {
        let total = self.total_stake();
        if total == 0 {
            return 0.0;
        }
        self.pool_stake(pool_id).unwrap_or(0) as f64 / total as f64
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockProductionSchedule {
    pub format_version: u16,
    pub age: u64,
    pub pool_id: Vec<u8>,
    pub pool_stake: u64,
    pub total_stake: u64,
    pub age_nonce: Vec<u8>,
    scheduled_slots: Vec<u64>,
}

pub const BLOCK_PRODUCTION_SCHEDULE_FORMAT_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockProductionWindow {
    pub start_slot: u64,
    pub slot_count: u64,
    pub max_scheduled_slots: usize,
}

impl BlockProductionWindow {
    pub fn new(start_slot: u64, slot_count: u64, max_scheduled_slots: usize) -> Self {
        Self {
            start_slot,
            slot_count,
            max_scheduled_slots,
        }
    }
}

impl BlockProductionSchedule {
    pub fn new(
        age: u64,
        pool_id: Vec<u8>,
        pool_stake: u64,
        total_stake: u64,
        age_nonce: Vec<u8>,
    ) -> Self {
        Self {
            format_version: BLOCK_PRODUCTION_SCHEDULE_FORMAT_VERSION,
            age,
            pool_id,
            pool_stake,
            total_stake,
            age_nonce,
            scheduled_slots: Vec::new(),
        }
    }

    pub fn add_scheduled_slot(&mut self, slot: u64) {
        match self.scheduled_slots.binary_search(&slot) {
            Ok(_) => {}
            Err(index) => self.scheduled_slots.insert(index, slot),
        }
    }

    pub fn is_scheduled_slot(&self, slot: u64) -> bool {
        self.scheduled_slots.binary_search(&slot).is_ok()
    }

    pub fn slot_count(&self) -> usize {
        self.scheduled_slots.len()
    }

    pub fn scheduled_slots(&self) -> Vec<u64> {
        self.scheduled_slots.clone()
    }

    pub fn next_scheduled_slot_at_or_after(&self, slot: u64) -> Option<u64> {
        match self.scheduled_slots.binary_search(&slot) {
            Ok(index) => Some(self.scheduled_slots[index]),
            Err(index) => self.scheduled_slots.get(index).copied(),
        }
    }

    pub fn upcoming_scheduled_slots_at_or_after(&self, slot: u64, max_count: usize) -> Vec<u64> {
        if max_count == 0 {
            return Vec::new();
        }
        let start = match self.scheduled_slots.binary_search(&slot) {
            Ok(index) | Err(index) => index,
        };
        self.scheduled_slots
            .iter()
            .skip(start)
            .take(max_count)
            .copied()
            .collect()
    }

    pub fn summary_line(
        &self,
        at_or_after_slot: u64,
        max_upcoming_slots: usize,
    ) -> Result<String, BlockProductionError> {
        self.validate_persisted()?;
        let next_slot = self
            .next_scheduled_slot_at_or_after(at_or_after_slot)
            .map(|slot| slot.to_string())
            .unwrap_or_else(|| "none".to_string());
        let upcoming =
            self.upcoming_scheduled_slots_at_or_after(at_or_after_slot, max_upcoming_slots);
        let upcoming_slots = if upcoming.is_empty() {
            "none".to_string()
        } else {
            upcoming
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        };
        Ok(format!(
            "block_production_schedule age={} pool_id_bytes={} pool_stake={} total_stake={} stake_ratio={:.6} slots={} next_slot={} upcoming_slots={}",
            self.age,
            self.pool_id.len(),
            self.pool_stake,
            self.total_stake,
            self.stake_ratio(),
            self.slot_count(),
            next_slot,
            upcoming_slots,
        ))
    }

    pub fn validate_persisted(&self) -> Result<(), BlockProductionError> {
        if self.format_version != BLOCK_PRODUCTION_SCHEDULE_FORMAT_VERSION {
            return Err(BlockProductionError::BadFormatVersion {
                expected: BLOCK_PRODUCTION_SCHEDULE_FORMAT_VERSION,
                actual: self.format_version,
            });
        }
        if self.pool_id.is_empty() {
            return Err(BlockProductionError::EmptyPoolId);
        }
        if self.age_nonce.is_empty() {
            return Err(BlockProductionError::EmptyAgeNonce);
        }
        if self.pool_stake > self.total_stake {
            return Err(BlockProductionError::PoolStakeExceedsTotal {
                pool_stake: self.pool_stake,
                total_stake: self.total_stake,
            });
        }
        if self
            .scheduled_slots
            .windows(2)
            .any(|slots| slots[0] >= slots[1])
        {
            return Err(BlockProductionError::UnsortedSlots);
        }
        Ok(())
    }

    pub fn stake_ratio(&self) -> f64 {
        if self.total_stake == 0 {
            0.0
        } else {
            self.pool_stake as f64 / self.total_stake as f64
        }
    }

    pub fn evaluate_proof_shape(
        &self,
        proof: &SlotProof,
        threshold: f64,
    ) -> Result<SlotProofDecision, BlockProductionError> {
        if proof.age != self.age {
            return Err(BlockProductionError::WrongAge {
                expected: self.age,
                actual: proof.age,
            });
        }
        if proof.pool_id != self.pool_id {
            return Err(BlockProductionError::WrongPool);
        }
        if !self.is_scheduled_slot(proof.slot) {
            return Err(BlockProductionError::SlotNotScheduled(proof.slot));
        }
        if !threshold.is_finite() || !(0.0..=1.0).contains(&threshold) {
            return Err(BlockProductionError::InvalidThreshold);
        }
        Ok(SlotProofDecision {
            eligible: proof.sample_ratio()? <= threshold,
            sample_ratio: proof.sample_ratio()?,
            threshold,
        })
    }

    pub fn verify_slot_proof_fixture(
        &self,
        proof: &SlotProof,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
    ) -> Result<SlotProofVerification, BlockProductionError> {
        certificate.validate_for_slot(&self.pool_id, proof.slot)?;
        if self.age_nonce.is_empty() {
            return Err(BlockProductionError::EmptyAgeNonce);
        }
        let expected_proof_len = 8 + self.age_nonce.len();
        if proof.proof_bytes.len() < expected_proof_len {
            return Err(BlockProductionError::BadProofSize {
                min: expected_proof_len,
                actual: proof.proof_bytes.len(),
            });
        }
        if &proof.proof_bytes[8..] != self.age_nonce.as_slice() {
            return Err(BlockProductionError::ProofNonceMismatch);
        }
        let threshold = calculator.threshold(self.stake_ratio());
        let decision = self.evaluate_proof_shape(proof, threshold)?;
        Ok(SlotProofVerification {
            decision,
            certificate_valid_from_slot: certificate.valid_from_slot,
            certificate_valid_until_slot: certificate.valid_until_slot,
        })
    }

    pub fn local_slot_proof(
        &self,
        slot: u64,
        sample: u64,
    ) -> Result<SlotProof, BlockProductionError> {
        if self.pool_id.is_empty() {
            return Err(BlockProductionError::EmptyPoolId);
        }
        if self.age_nonce.is_empty() {
            return Err(BlockProductionError::EmptyAgeNonce);
        }
        if !self.is_scheduled_slot(slot) {
            return Err(BlockProductionError::SlotNotScheduled(slot));
        }

        Ok(SlotProof {
            age: self.age,
            slot,
            pool_id: self.pool_id.clone(),
            proof_bytes: [sample.to_be_bytes().as_slice(), self.age_nonce.as_slice()].concat(),
        })
    }

    pub fn local_scheduled_slot_proof(&self, slot: u64) -> Result<SlotProof, BlockProductionError> {
        let sample = local_slot_sample(&self.pool_id, &self.age_nonce, slot);
        self.local_slot_proof(slot, sample)
    }

    pub fn plan_local_window(
        snapshot: &StakeSnapshot,
        pool_id: Vec<u8>,
        age_nonce: Vec<u8>,
        calculator: SlotEligibilityCalculator,
        window: BlockProductionWindow,
    ) -> Result<Self, BlockProductionError> {
        snapshot.validate()?;
        calculator.validate()?;
        if pool_id.is_empty() {
            return Err(BlockProductionError::EmptyPoolId);
        }
        if age_nonce.is_empty() {
            return Err(BlockProductionError::EmptyAgeNonce);
        }
        if window.slot_count == 0
            || window.max_scheduled_slots == 0
            || calculator.slots_per_age == 0
            || window.slot_count > calculator.slots_per_age
        {
            return Err(BlockProductionError::InvalidSlotWindow);
        }

        let pool_stake = snapshot.pool_stake(&pool_id).unwrap_or(0);
        let total_stake = snapshot.total_stake();
        let mut schedule = Self::new(snapshot.age, pool_id, pool_stake, total_stake, age_nonce);
        let threshold = calculator.threshold(schedule.stake_ratio());
        if threshold <= 0.0 {
            return Ok(schedule);
        }

        for offset in 0..window.slot_count {
            let slot = window
                .start_slot
                .checked_add(offset)
                .ok_or(BlockProductionError::InvalidSlotWindow)?;
            let sample_ratio =
                local_slot_sample_ratio(&schedule.pool_id, &schedule.age_nonce, slot);
            if sample_ratio <= threshold {
                schedule.add_scheduled_slot(slot);
                if schedule.slot_count() >= window.max_scheduled_slots {
                    break;
                }
            }
        }
        Ok(schedule)
    }
}

pub const MIN_OPERATIONAL_CERTIFICATE_BYTES: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationalCertificateFixture {
    pub pool_id: Vec<u8>,
    pub producer_key_hash: Vec<u8>,
    pub valid_from_slot: u64,
    pub valid_until_slot: u64,
    pub certificate_bytes: Vec<u8>,
}

impl OperationalCertificateFixture {
    pub fn validate_for_slot(
        &self,
        expected_pool: &[u8],
        slot: u64,
    ) -> Result<(), BlockProductionError> {
        if self.pool_id != expected_pool {
            return Err(BlockProductionError::CertificatePoolMismatch);
        }
        if self.producer_key_hash.is_empty() {
            return Err(BlockProductionError::EmptyProducerKeyHash);
        }
        if self.valid_from_slot > self.valid_until_slot {
            return Err(BlockProductionError::InvalidCertificateWindow);
        }
        if self.certificate_bytes.len() < MIN_OPERATIONAL_CERTIFICATE_BYTES {
            return Err(BlockProductionError::CertificateTooSmall {
                min: MIN_OPERATIONAL_CERTIFICATE_BYTES,
                actual: self.certificate_bytes.len(),
            });
        }
        if slot < self.valid_from_slot || slot > self.valid_until_slot {
            return Err(BlockProductionError::CertificateNotValidForSlot {
                slot,
                valid_from: self.valid_from_slot,
                valid_until: self.valid_until_slot,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotProof {
    pub age: u64,
    pub slot: u64,
    pub pool_id: Vec<u8>,
    pub proof_bytes: Vec<u8>,
}

impl SlotProof {
    pub fn sample_ratio(&self) -> Result<f64, BlockProductionError> {
        let bytes = self
            .proof_bytes
            .get(0..8)
            .ok_or(BlockProductionError::BadProofSize {
                min: 8,
                actual: self.proof_bytes.len(),
            })?;
        let sample = u64::from_be_bytes(bytes.try_into().expect("slice length checked"));
        Ok(sample as f64 / u64::MAX as f64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotProofDecision {
    pub eligible: bool,
    pub sample_ratio: f64,
    pub threshold: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotProofVerification {
    pub decision: SlotProofDecision,
    pub certificate_valid_from_slot: u64,
    pub certificate_valid_until_slot: u64,
}

#[derive(Debug, Clone, Default)]
pub struct BlockProductionScheduleStore {
    schedules: HashMap<(u64, Vec<u8>), BlockProductionSchedule>,
}

impl BlockProductionScheduleStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn save(&mut self, schedule: BlockProductionSchedule) -> Result<(), BlockProductionError> {
        schedule.validate_persisted()?;
        self.schedules
            .insert((schedule.age, schedule.pool_id.clone()), schedule);
        Ok(())
    }

    pub fn load(
        &self,
        age: u64,
        pool_id: &[u8],
    ) -> Result<Option<BlockProductionSchedule>, BlockProductionError> {
        let Some(schedule) = self.schedules.get(&(age, pool_id.to_vec())) else {
            return Ok(None);
        };
        schedule.validate_persisted()?;
        Ok(Some(schedule.clone()))
    }

    pub fn len(&self) -> usize {
        self.schedules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.schedules.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotEligibilityCalculator {
    pub active_slot_coeff: f64,
    pub slots_per_age: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockProductionError {
    BadFormatVersion {
        expected: u16,
        actual: u16,
    },
    UnsortedSlots,
    WrongAge {
        expected: u64,
        actual: u64,
    },
    WrongPool,
    SlotNotScheduled(u64),
    InvalidThreshold,
    InvalidActiveSlotCoefficient,
    InvalidSlotWindow,
    EmptyPoolId,
    EmptyStakePoolId,
    DuplicateStakePoolId,
    PoolStakeExceedsTotal {
        pool_stake: u64,
        total_stake: u64,
    },
    BadProofSize {
        min: usize,
        actual: usize,
    },
    EmptyAgeNonce,
    ProofNonceMismatch,
    CertificatePoolMismatch,
    EmptyProducerKeyHash,
    InvalidCertificateWindow,
    CertificateTooSmall {
        min: usize,
        actual: usize,
    },
    CertificateNotValidForSlot {
        slot: u64,
        valid_from: u64,
        valid_until: u64,
    },
}

impl fmt::Display for BlockProductionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadFormatVersion { expected, actual } => write!(
                f,
                "block production schedule format mismatch: expected {expected}, got {actual}"
            ),
            Self::UnsortedSlots => {
                f.write_str("block production schedule slots must be strictly sorted")
            }
            Self::WrongAge { expected, actual } => {
                write!(
                    f,
                    "slot proof age mismatch: expected={expected} actual={actual}"
                )
            }
            Self::WrongPool => f.write_str("slot proof pool does not match schedule"),
            Self::SlotNotScheduled(slot) => write!(f, "slot {slot} is not scheduled"),
            Self::InvalidThreshold => f.write_str("slot threshold must be a finite ratio"),
            Self::InvalidActiveSlotCoefficient => {
                f.write_str("active slot coefficient must be a finite ratio above zero")
            }
            Self::InvalidSlotWindow => f.write_str("block production slot window is invalid"),
            Self::EmptyPoolId => f.write_str("block production pool id is empty"),
            Self::EmptyStakePoolId => f.write_str("stake snapshot pool id is empty"),
            Self::DuplicateStakePoolId => f.write_str("stake snapshot contains duplicate pool ids"),
            Self::PoolStakeExceedsTotal {
                pool_stake,
                total_stake,
            } => write!(
                f,
                "block production pool stake exceeds total stake: pool_stake={pool_stake} total_stake={total_stake}"
            ),
            Self::BadProofSize { min, actual } => {
                write!(f, "slot proof too small: min={min} actual={actual}")
            }
            Self::EmptyAgeNonce => f.write_str("block production schedule age nonce is empty"),
            Self::ProofNonceMismatch => f.write_str("slot proof nonce does not match schedule"),
            Self::CertificatePoolMismatch => {
                f.write_str("operational certificate pool does not match schedule")
            }
            Self::EmptyProducerKeyHash => {
                f.write_str("operational certificate producer key hash is empty")
            }
            Self::InvalidCertificateWindow => {
                f.write_str("operational certificate validity window is invalid")
            }
            Self::CertificateTooSmall { min, actual } => write!(
                f,
                "operational certificate too small: min={min} actual={actual}"
            ),
            Self::CertificateNotValidForSlot {
                slot,
                valid_from,
                valid_until,
            } => write!(
                f,
                "operational certificate is not valid for slot {slot}: valid_from={valid_from} valid_until={valid_until}"
            ),
        }
    }
}

impl std::error::Error for BlockProductionError {}

impl SlotEligibilityCalculator {
    pub fn new(active_slot_coeff: f64, slots_per_age: u64) -> Self {
        Self {
            active_slot_coeff,
            slots_per_age,
        }
    }

    pub fn validate(&self) -> Result<(), BlockProductionError> {
        if !self.active_slot_coeff.is_finite()
            || self.active_slot_coeff <= 0.0
            || self.active_slot_coeff > 1.0
        {
            return Err(BlockProductionError::InvalidActiveSlotCoefficient);
        }
        if self.slots_per_age == 0 {
            return Err(BlockProductionError::InvalidSlotWindow);
        }
        Ok(())
    }

    pub fn threshold(&self, stake_ratio: f64) -> f64 {
        if !stake_ratio.is_finite() || stake_ratio <= 0.0 {
            return 0.0;
        }
        let f = self.active_slot_coeff;
        if !f.is_finite() || f <= 0.0 || f > 1.0 {
            return 0.0;
        }
        if stake_ratio >= 1.0 {
            return f;
        }
        1.0 - (1.0 - f).powf(stake_ratio)
    }
}

fn local_slot_sample_ratio(pool_id: &[u8], age_nonce: &[u8], slot: u64) -> f64 {
    local_slot_sample(pool_id, age_nonce, slot) as f64 / u64::MAX as f64
}

fn local_slot_sample(pool_id: &[u8], age_nonce: &[u8], slot: u64) -> u64 {
    let mut state = slot ^ 0x9e37_79b9_7f4a_7c15;
    for byte in pool_id.iter().chain(age_nonce) {
        state ^= u64::from(*byte).wrapping_add(0x9e37_79b9_7f4a_7c15);
        state = state.rotate_left(13).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduled_slots_stay_sorted_and_searchable() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9]);
        schedule.add_scheduled_slot(20);
        schedule.add_scheduled_slot(10);
        schedule.add_scheduled_slot(20);
        assert_eq!(schedule.scheduled_slots(), vec![10, 20]);
        assert!(schedule.is_scheduled_slot(10));
        assert!(!schedule.is_scheduled_slot(11));
    }

    #[test]
    fn next_scheduled_slot_lookup_uses_sorted_schedule() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9]);
        assert_eq!(schedule.next_scheduled_slot_at_or_after(10), None);

        schedule.add_scheduled_slot(20);
        schedule.add_scheduled_slot(10);
        schedule.add_scheduled_slot(30);

        assert_eq!(schedule.next_scheduled_slot_at_or_after(5), Some(10));
        assert_eq!(schedule.next_scheduled_slot_at_or_after(10), Some(10));
        assert_eq!(schedule.next_scheduled_slot_at_or_after(11), Some(20));
        assert_eq!(schedule.next_scheduled_slot_at_or_after(30), Some(30));
        assert_eq!(schedule.next_scheduled_slot_at_or_after(31), None);
    }

    #[test]
    fn upcoming_scheduled_slots_are_bounded_and_sorted() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9]);
        schedule.add_scheduled_slot(20);
        schedule.add_scheduled_slot(10);
        schedule.add_scheduled_slot(30);
        schedule.add_scheduled_slot(40);

        assert_eq!(
            schedule.upcoming_scheduled_slots_at_or_after(5, 0),
            Vec::<u64>::new()
        );
        assert_eq!(
            schedule.upcoming_scheduled_slots_at_or_after(5, 2),
            vec![10, 20]
        );
        assert_eq!(
            schedule.upcoming_scheduled_slots_at_or_after(20, 3),
            vec![20, 30, 40]
        );
        assert_eq!(
            schedule.upcoming_scheduled_slots_at_or_after(21, 10),
            vec![30, 40]
        );
        assert_eq!(
            schedule.upcoming_scheduled_slots_at_or_after(41, 10),
            Vec::<u64>::new()
        );
    }

    #[test]
    fn block_production_schedule_renders_summary_line() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1, 2], 10, 100, vec![9]);
        schedule.add_scheduled_slot(20);
        schedule.add_scheduled_slot(10);
        schedule.add_scheduled_slot(30);

        assert_eq!(
            schedule.summary_line(11, 2).unwrap(),
            "block_production_schedule age=3 pool_id_bytes=2 pool_stake=10 total_stake=100 stake_ratio=0.100000 slots=3 next_slot=20 upcoming_slots=20,30"
        );
    }

    #[test]
    fn block_production_schedule_summary_reports_no_upcoming_slots() {
        let schedule = BlockProductionSchedule::new(3, vec![1], 0, 0, vec![9]);

        assert_eq!(
            schedule.summary_line(11, 2).unwrap(),
            "block_production_schedule age=3 pool_id_bytes=1 pool_stake=0 total_stake=0 stake_ratio=0.000000 slots=0 next_slot=none upcoming_slots=none"
        );
    }

    #[test]
    fn threshold_matches_float_formula() {
        let calculator = SlotEligibilityCalculator::new(0.05, 100);
        let threshold = calculator.threshold(0.5);
        assert!((threshold - (1.0_f64 - 0.95_f64.powf(0.5_f64))).abs() < 1e-12);
        assert_eq!(calculator.threshold(0.0), 0.0);
    }

    #[test]
    fn slot_eligibility_calculator_validation_rejects_invalid_shapes() {
        assert_eq!(
            SlotEligibilityCalculator::new(0.0, 10).validate(),
            Err(BlockProductionError::InvalidActiveSlotCoefficient)
        );
        assert_eq!(
            SlotEligibilityCalculator::new(1.1, 10).validate(),
            Err(BlockProductionError::InvalidActiveSlotCoefficient)
        );
        assert_eq!(
            SlotEligibilityCalculator::new(f64::NAN, 10).validate(),
            Err(BlockProductionError::InvalidActiveSlotCoefficient)
        );
        assert_eq!(
            SlotEligibilityCalculator::new(1.0, 0).validate(),
            Err(BlockProductionError::InvalidSlotWindow)
        );
    }

    #[test]
    fn schedule_store_round_trips_valid_schedules() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1, 2], 10, 100, vec![9]);
        schedule.add_scheduled_slot(12);
        let mut store = BlockProductionScheduleStore::new();
        store.save(schedule.clone()).unwrap();
        assert_eq!(store.load(3, &[1, 2]).unwrap(), Some(schedule));
    }

    #[test]
    fn persisted_validation_rejects_bad_format() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9]);
        schedule.format_version = 999;
        assert_eq!(
            schedule.validate_persisted(),
            Err(BlockProductionError::BadFormatVersion {
                expected: BLOCK_PRODUCTION_SCHEDULE_FORMAT_VERSION,
                actual: 999,
            })
        );
    }

    #[test]
    fn persisted_validation_rejects_unsafe_schedule_shape() {
        assert_eq!(
            BlockProductionSchedule::new(3, Vec::new(), 0, 0, vec![9]).validate_persisted(),
            Err(BlockProductionError::EmptyPoolId)
        );
        assert_eq!(
            BlockProductionSchedule::new(3, vec![1], 0, 0, Vec::new()).validate_persisted(),
            Err(BlockProductionError::EmptyAgeNonce)
        );
        assert_eq!(
            BlockProductionSchedule::new(3, vec![1], 101, 100, vec![9]).validate_persisted(),
            Err(BlockProductionError::PoolStakeExceedsTotal {
                pool_stake: 101,
                total_stake: 100,
            })
        );
    }

    #[test]
    fn stake_snapshot_calculates_pool_ratio() {
        let snapshot = StakeSnapshot {
            age: 7,
            entries: vec![
                StakeEntry {
                    pool_id: vec![1],
                    stake: 30,
                },
                StakeEntry {
                    pool_id: vec![2],
                    stake: 70,
                },
            ],
        };
        assert_eq!(snapshot.total_stake(), 100);
        assert_eq!(snapshot.pool_ratio(&[1]), 0.3);
    }

    #[test]
    fn stake_snapshot_validation_rejects_ambiguous_pool_entries() {
        assert_eq!(
            StakeSnapshot {
                age: 7,
                entries: vec![StakeEntry {
                    pool_id: Vec::new(),
                    stake: 1,
                }],
            }
            .validate(),
            Err(BlockProductionError::EmptyStakePoolId)
        );
        assert_eq!(
            StakeSnapshot {
                age: 7,
                entries: vec![
                    StakeEntry {
                        pool_id: vec![1],
                        stake: 1,
                    },
                    StakeEntry {
                        pool_id: vec![1],
                        stake: 2,
                    },
                ],
            }
            .validate(),
            Err(BlockProductionError::DuplicateStakePoolId)
        );
    }

    #[test]
    fn local_window_planner_rejects_ambiguous_snapshot_without_side_effects() {
        let snapshot = StakeSnapshot {
            age: 7,
            entries: vec![
                StakeEntry {
                    pool_id: vec![1],
                    stake: 1,
                },
                StakeEntry {
                    pool_id: vec![1],
                    stake: 2,
                },
            ],
        };

        assert_eq!(
            BlockProductionSchedule::plan_local_window(
                &snapshot,
                vec![1],
                vec![9],
                SlotEligibilityCalculator::new(1.0, 10),
                BlockProductionWindow::new(100, 1, 1),
            ),
            Err(BlockProductionError::DuplicateStakePoolId)
        );
    }

    #[test]
    fn local_window_planner_selects_bounded_slots_from_snapshot() {
        let snapshot = StakeSnapshot {
            age: 7,
            entries: vec![StakeEntry {
                pool_id: vec![1],
                stake: 100,
            }],
        };
        let schedule = BlockProductionSchedule::plan_local_window(
            &snapshot,
            vec![1],
            vec![9, 8],
            SlotEligibilityCalculator::new(1.0, 10),
            BlockProductionWindow::new(100, 10, 3),
        )
        .unwrap();

        assert_eq!(schedule.age, 7);
        assert_eq!(schedule.pool_stake, 100);
        assert_eq!(schedule.total_stake, 100);
        assert_eq!(schedule.scheduled_slots(), vec![100, 101, 102]);
    }

    #[test]
    fn local_window_planner_rejects_unsafe_shapes_without_side_effects() {
        let snapshot = StakeSnapshot {
            age: 7,
            entries: vec![StakeEntry {
                pool_id: vec![1],
                stake: 100,
            }],
        };

        assert_eq!(
            BlockProductionSchedule::plan_local_window(
                &snapshot,
                Vec::new(),
                vec![9],
                SlotEligibilityCalculator::new(1.0, 10),
                BlockProductionWindow::new(100, 1, 1),
            ),
            Err(BlockProductionError::EmptyPoolId)
        );
        assert_eq!(
            BlockProductionSchedule::plan_local_window(
                &snapshot,
                vec![1],
                Vec::new(),
                SlotEligibilityCalculator::new(1.0, 10),
                BlockProductionWindow::new(100, 1, 1),
            ),
            Err(BlockProductionError::EmptyAgeNonce)
        );
        assert_eq!(
            BlockProductionSchedule::plan_local_window(
                &snapshot,
                vec![1],
                vec![9],
                SlotEligibilityCalculator::new(1.0, 10),
                BlockProductionWindow::new(100, 0, 1),
            ),
            Err(BlockProductionError::InvalidSlotWindow)
        );
        assert_eq!(
            BlockProductionSchedule::plan_local_window(
                &snapshot,
                vec![1],
                vec![9],
                SlotEligibilityCalculator::new(0.0, 10),
                BlockProductionWindow::new(100, 1, 1),
            ),
            Err(BlockProductionError::InvalidActiveSlotCoefficient)
        );
    }

    #[test]
    fn local_slot_proof_builds_shape_for_scheduled_slot() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(12);

        let proof = schedule.local_slot_proof(12, 7).unwrap();

        assert_eq!(proof.age, 3);
        assert_eq!(proof.slot, 12);
        assert_eq!(proof.pool_id, vec![1]);
        assert_eq!(
            proof.proof_bytes,
            [7_u64.to_be_bytes().as_slice(), &[9, 8]].concat()
        );
        assert_eq!(proof.sample_ratio().unwrap(), 7_f64 / u64::MAX as f64);
    }

    #[test]
    fn local_slot_proof_rejects_unscheduled_slot_without_side_effects() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(12);

        assert_eq!(
            schedule.local_slot_proof(13, 7),
            Err(BlockProductionError::SlotNotScheduled(13))
        );
    }

    #[test]
    fn local_scheduled_slot_proof_uses_planner_sample() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(12);

        let proof = schedule.local_scheduled_slot_proof(12).unwrap();
        let expected_sample = local_slot_sample(&[1], &[9, 8], 12);

        assert_eq!(proof.age, 3);
        assert_eq!(proof.slot, 12);
        assert_eq!(proof.pool_id, vec![1]);
        assert_eq!(
            proof.proof_bytes,
            [expected_sample.to_be_bytes().as_slice(), &[9, 8]].concat()
        );
        assert_eq!(
            proof.sample_ratio().unwrap(),
            local_slot_sample_ratio(&[1], &[9, 8], 12)
        );
    }

    #[test]
    fn slot_proof_shape_evaluates_against_threshold_locally() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9]);
        schedule.add_scheduled_slot(12);
        let proof = SlotProof {
            age: 3,
            slot: 12,
            pool_id: vec![1],
            proof_bytes: 0_u64.to_be_bytes().to_vec(),
        };
        let decision = schedule.evaluate_proof_shape(&proof, 0.1).unwrap();
        assert!(decision.eligible);
        assert_eq!(decision.sample_ratio, 0.0);
    }

    #[test]
    fn slot_proof_fixture_verifies_certificate_nonce_and_threshold() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(12);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 10,
            valid_until_slot: 20,
            certificate_bytes: vec![3; MIN_OPERATIONAL_CERTIFICATE_BYTES],
        };
        let proof = SlotProof {
            age: 3,
            slot: 12,
            pool_id: vec![1],
            proof_bytes: [0_u64.to_be_bytes().as_slice(), &[9, 8]].concat(),
        };

        let verification = schedule
            .verify_slot_proof_fixture(
                &proof,
                &certificate,
                SlotEligibilityCalculator::new(0.1, 100),
            )
            .unwrap();
        assert!(verification.decision.eligible);
        assert_eq!(verification.decision.sample_ratio, 0.0);
        assert_eq!(verification.certificate_valid_until_slot, 20);
    }

    #[test]
    fn slot_proof_fixture_rejects_wrong_certificate_pool() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(12);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![2],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 10,
            valid_until_slot: 20,
            certificate_bytes: vec![3; MIN_OPERATIONAL_CERTIFICATE_BYTES],
        };
        let proof = SlotProof {
            age: 3,
            slot: 12,
            pool_id: vec![1],
            proof_bytes: [0_u64.to_be_bytes().as_slice(), &[9, 8]].concat(),
        };

        assert_eq!(
            schedule.verify_slot_proof_fixture(
                &proof,
                &certificate,
                SlotEligibilityCalculator::new(0.1, 100)
            ),
            Err(BlockProductionError::CertificatePoolMismatch)
        );
    }

    #[test]
    fn slot_proof_fixture_rejects_nonce_mismatch() {
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(12);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 10,
            valid_until_slot: 20,
            certificate_bytes: vec![3; MIN_OPERATIONAL_CERTIFICATE_BYTES],
        };
        let proof = SlotProof {
            age: 3,
            slot: 12,
            pool_id: vec![1],
            proof_bytes: [0_u64.to_be_bytes().as_slice(), &[1, 2]].concat(),
        };

        assert_eq!(
            schedule.verify_slot_proof_fixture(
                &proof,
                &certificate,
                SlotEligibilityCalculator::new(0.1, 100)
            ),
            Err(BlockProductionError::ProofNonceMismatch)
        );
    }
}
