use crate::block_production::{
    BlockProductionError, BlockProductionSchedule, OperationalCertificateFixture,
    SlotEligibilityCalculator, SlotProof, SlotProofVerification,
};
use crate::chain::ChainTip;
use crate::config::BlockProducerConfig;
use crate::events::{Event, EventPayload};
use crate::ledger::blocks::{BlockBundle, BlockValidationError};
use std::fmt;

pub const BLOCK_PRODUCER_SELF_CHECK_EVENT: &str = "block_producer.self_check";
pub const BLOCK_PRODUCER_SELF_CHECK_FAILED_EVENT: &str = "block_producer.self_check_failed";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockProducerMaterial {
    pub tree_key_label: String,
    pub producer_key_label: String,
    pub certificate_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockProducerReadiness {
    Disabled,
    Missing(Vec<&'static str>),
    Ready(BlockProducerMaterial),
}

impl BlockProducerReadiness {
    pub fn from_config(config: &BlockProducerConfig) -> Self {
        if !config.enabled {
            return Self::Disabled;
        }
        let mut missing = Vec::new();
        if config.tree_key.is_none() {
            missing.push("tree_key");
        }
        if config.producer_key.is_none() {
            missing.push("producer_key");
        }
        if config.operational_certificate.is_none() {
            missing.push("certificate");
        }
        if !missing.is_empty() {
            return Self::Missing(missing);
        }
        Self::Ready(BlockProducerMaterial {
            tree_key_label: config.tree_key.as_ref().unwrap().display().to_string(),
            producer_key_label: config.producer_key.as_ref().unwrap().display().to_string(),
            certificate_label: config
                .operational_certificate
                .as_ref()
                .unwrap()
                .display()
                .to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockProducerSelfCheck {
    pub max_item_bytes: usize,
    pub require_material: bool,
}

#[allow(clippy::too_many_arguments)]
impl BlockProducerSelfCheck {
    pub fn check(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
    ) -> Result<(), BlockProducerError> {
        if self.require_material && !matches!(readiness, BlockProducerReadiness::Ready(_)) {
            return Err(BlockProducerError::NotReady);
        }
        bundle.validate(None, self.max_item_bytes)?;
        Ok(())
    }

    pub fn check_local_fixture_integrity(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        previous: Option<ChainTip>,
    ) -> Result<(), BlockProducerError> {
        if self.require_material && !matches!(readiness, BlockProducerReadiness::Ready(_)) {
            return Err(BlockProducerError::NotReady);
        }
        bundle.validate(previous, self.max_item_bytes)?;
        bundle.validate_local_fixture_integrity(previous)?;
        Ok(())
    }

    pub fn check_local_fixture_with_witness(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        previous: Option<ChainTip>,
        witness: &BlockProductionWitness,
        min_proof_bytes: usize,
    ) -> Result<(), BlockProducerError> {
        self.check_local_fixture_integrity(bundle, readiness, previous)?;
        witness.validate_for(bundle, min_proof_bytes)
    }

    pub fn check_with_witness(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        witness: &BlockProductionWitness,
        min_proof_bytes: usize,
    ) -> Result<(), BlockProducerError> {
        self.check(bundle, readiness)?;
        witness.validate_for(bundle, min_proof_bytes)
    }

    pub fn check_slot_proof(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        schedule: &BlockProductionSchedule,
        proof: &SlotProof,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<SlotProofVerification, BlockProducerError> {
        self.check_slot_proof_report(
            bundle,
            readiness,
            schedule,
            proof,
            certificate,
            calculator,
            min_proof_bytes,
        )
        .map(|report| report.verification)
    }

    pub fn check_slot_proof_report(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        schedule: &BlockProductionSchedule,
        proof: &SlotProof,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<BlockProducerSelfCheckReport, BlockProducerError> {
        let verification = schedule.verify_slot_proof_fixture(proof, certificate, calculator)?;
        if !verification.decision.eligible {
            return Err(BlockProducerError::SlotNotEligible);
        }
        let witness = BlockProductionWitness::from_slot_proof(bundle, proof)?;
        self.check_with_witness(bundle, readiness, &witness, min_proof_bytes)?;
        Ok(BlockProducerSelfCheckReport {
            verification,
            witness,
        })
    }

    pub fn check_local_fixture_slot_proof_report(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        previous: Option<ChainTip>,
        schedule: &BlockProductionSchedule,
        proof: &SlotProof,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<BlockProducerSelfCheckReport, BlockProducerError> {
        let verification = schedule.verify_slot_proof_fixture(proof, certificate, calculator)?;
        if !verification.decision.eligible {
            return Err(BlockProducerError::SlotNotEligible);
        }
        let witness = BlockProductionWitness::from_slot_proof(bundle, proof)?;
        self.check_local_fixture_with_witness(
            bundle,
            readiness,
            previous,
            &witness,
            min_proof_bytes,
        )?;
        Ok(BlockProducerSelfCheckReport {
            verification,
            witness,
        })
    }

    pub fn check_local_fixture_slot_proof(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        previous: Option<ChainTip>,
        schedule: &BlockProductionSchedule,
        proof: &SlotProof,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<SlotProofVerification, BlockProducerError> {
        self.check_local_fixture_slot_proof_report(
            bundle,
            readiness,
            previous,
            schedule,
            proof,
            certificate,
            calculator,
            min_proof_bytes,
        )
        .map(|report| report.verification)
    }

    pub fn check_scheduled_slot(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        schedule: &BlockProductionSchedule,
        slot: u64,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<SlotProofVerification, BlockProducerError> {
        self.check_scheduled_slot_report(
            bundle,
            readiness,
            schedule,
            slot,
            certificate,
            calculator,
            min_proof_bytes,
        )
        .map(|report| report.verification)
    }

    pub fn check_local_fixture_scheduled_slot_report(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        previous: Option<ChainTip>,
        schedule: &BlockProductionSchedule,
        slot: u64,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<BlockProducerSelfCheckReport, BlockProducerError> {
        let proof = schedule.local_scheduled_slot_proof(slot)?;
        self.check_local_fixture_slot_proof_report(
            bundle,
            readiness,
            previous,
            schedule,
            &proof,
            certificate,
            calculator,
            min_proof_bytes,
        )
    }

    pub fn check_local_fixture_scheduled_slot(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        previous: Option<ChainTip>,
        schedule: &BlockProductionSchedule,
        slot: u64,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<SlotProofVerification, BlockProducerError> {
        self.check_local_fixture_scheduled_slot_report(
            bundle,
            readiness,
            previous,
            schedule,
            slot,
            certificate,
            calculator,
            min_proof_bytes,
        )
        .map(|report| report.verification)
    }

    pub fn check_scheduled_slot_report(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        schedule: &BlockProductionSchedule,
        slot: u64,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<BlockProducerSelfCheckReport, BlockProducerError> {
        let proof = schedule.local_scheduled_slot_proof(slot)?;
        self.check_slot_proof_report(
            bundle,
            readiness,
            schedule,
            &proof,
            certificate,
            calculator,
            min_proof_bytes,
        )
    }

    pub fn check_next_scheduled_slot_report(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        schedule: &BlockProductionSchedule,
        at_or_after_slot: u64,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<BlockProducerSelfCheckReport, BlockProducerError> {
        let slot = schedule
            .next_scheduled_slot_at_or_after(at_or_after_slot)
            .ok_or(BlockProducerError::NoScheduledSlot { at_or_after_slot })?;
        self.check_scheduled_slot_report(
            bundle,
            readiness,
            schedule,
            slot,
            certificate,
            calculator,
            min_proof_bytes,
        )
    }

    pub fn check_local_fixture_next_scheduled_slot_report(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        previous: Option<ChainTip>,
        schedule: &BlockProductionSchedule,
        at_or_after_slot: u64,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<BlockProducerSelfCheckReport, BlockProducerError> {
        let slot = schedule
            .next_scheduled_slot_at_or_after(at_or_after_slot)
            .ok_or(BlockProducerError::NoScheduledSlot { at_or_after_slot })?;
        self.check_local_fixture_scheduled_slot_report(
            bundle,
            readiness,
            previous,
            schedule,
            slot,
            certificate,
            calculator,
            min_proof_bytes,
        )
    }

    pub fn check_local_fixture_next_scheduled_slot(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        previous: Option<ChainTip>,
        schedule: &BlockProductionSchedule,
        at_or_after_slot: u64,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<SlotProofVerification, BlockProducerError> {
        self.check_local_fixture_next_scheduled_slot_report(
            bundle,
            readiness,
            previous,
            schedule,
            at_or_after_slot,
            certificate,
            calculator,
            min_proof_bytes,
        )
        .map(|report| report.verification)
    }

    pub fn check_next_scheduled_slot(
        &self,
        bundle: &BlockBundle,
        readiness: &BlockProducerReadiness,
        schedule: &BlockProductionSchedule,
        at_or_after_slot: u64,
        certificate: &OperationalCertificateFixture,
        calculator: SlotEligibilityCalculator,
        min_proof_bytes: usize,
    ) -> Result<SlotProofVerification, BlockProducerError> {
        self.check_next_scheduled_slot_report(
            bundle,
            readiness,
            schedule,
            at_or_after_slot,
            certificate,
            calculator,
            min_proof_bytes,
        )
        .map(|report| report.verification)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockProducerSelfCheckReport {
    pub verification: SlotProofVerification,
    pub witness: BlockProductionWitness,
}

impl BlockProducerSelfCheckReport {
    pub fn summary_line(&self) -> String {
        format!(
            "block_self_check bundle_number={} slot={} proof_bytes={} eligible={} sample_ratio={:.6} threshold={:.6} cert_valid_from={} cert_valid_until={}",
            self.witness.bundle_number,
            self.witness.slot,
            self.witness.proof_bytes.len(),
            self.verification.decision.eligible,
            self.verification.decision.sample_ratio,
            self.verification.decision.threshold,
            self.verification.certificate_valid_from_slot,
            self.verification.certificate_valid_until_slot,
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            BLOCK_PRODUCER_SELF_CHECK_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn events(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockProductionWitness {
    pub bundle_number: u64,
    pub slot: u64,
    pub body_hash: [u8; 32],
    pub proof_bytes: Vec<u8>,
}

impl BlockProductionWitness {
    pub fn from_bundle(bundle: &BlockBundle, proof_bytes: Vec<u8>) -> Self {
        Self {
            bundle_number: bundle.header.bundle_number,
            slot: bundle.header.point.slot,
            body_hash: bundle.header.body_hash,
            proof_bytes,
        }
    }

    pub fn from_slot_proof(
        bundle: &BlockBundle,
        proof: &SlotProof,
    ) -> Result<Self, BlockProducerError> {
        if proof.slot != bundle.header.point.slot {
            return Err(BlockProducerError::WitnessMismatch);
        }
        Ok(Self::from_bundle(bundle, proof.proof_bytes.clone()))
    }

    pub fn validate_for(
        &self,
        bundle: &BlockBundle,
        min_proof_bytes: usize,
    ) -> Result<(), BlockProducerError> {
        if self.bundle_number != bundle.header.bundle_number
            || self.slot != bundle.header.point.slot
            || self.body_hash != bundle.header.body_hash
        {
            return Err(BlockProducerError::WitnessMismatch);
        }
        if self.proof_bytes.len() < min_proof_bytes {
            return Err(BlockProducerError::ProofTooSmall {
                min: min_proof_bytes,
                actual: self.proof_bytes.len(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockProducerError {
    NotReady,
    Block(BlockValidationError),
    Production(BlockProductionError),
    SlotNotEligible,
    NoScheduledSlot { at_or_after_slot: u64 },
    WitnessMismatch,
    ProofTooSmall { min: usize, actual: usize },
}

impl fmt::Display for BlockProducerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotReady => f.write_str("block producer material is not ready"),
            Self::Block(err) => write!(f, "block self-check failed: {err}"),
            Self::Production(err) => write!(f, "block production proof check failed: {err}"),
            Self::SlotNotEligible => f.write_str("slot proof is not eligible for production"),
            Self::NoScheduledSlot { at_or_after_slot } => write!(
                f,
                "no scheduled block production slot at or after {at_or_after_slot}"
            ),
            Self::WitnessMismatch => f.write_str("block production witness does not match bundle"),
            Self::ProofTooSmall { min, actual } => {
                write!(
                    f,
                    "block production proof bytes too small: min={min} actual={actual}"
                )
            }
        }
    }
}

impl BlockProducerError {
    pub fn summary_line(&self) -> String {
        match self {
            Self::NotReady => "block_self_check_failed reason=not_ready".to_string(),
            Self::Block(_) => "block_self_check_failed reason=block_validation".to_string(),
            Self::Production(_) => "block_self_check_failed reason=block_production".to_string(),
            Self::SlotNotEligible => {
                "block_self_check_failed reason=slot_not_eligible".to_string()
            }
            Self::NoScheduledSlot { at_or_after_slot } => format!(
                "block_self_check_failed reason=no_scheduled_slot at_or_after_slot={at_or_after_slot}"
            ),
            Self::WitnessMismatch => {
                "block_self_check_failed reason=witness_mismatch".to_string()
            }
            Self::ProofTooSmall { min, actual } => {
                format!("block_self_check_failed reason=proof_too_small min={min} actual={actual}")
            }
        }
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            BLOCK_PRODUCER_SELF_CHECK_FAILED_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn events(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl std::error::Error for BlockProducerError {}

impl From<BlockValidationError> for BlockProducerError {
    fn from(value: BlockValidationError) -> Self {
        Self::Block(value)
    }
}

impl From<BlockProductionError> for BlockProducerError {
    fn from(value: BlockProductionError) -> Self {
        Self::Production(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{ChainPoint, ChainTip};
    use crate::ledger::blocks::{local_fixture_body_hash, BlockHeader, BlockItem};
    use std::path::PathBuf;

    fn bundle() -> BlockBundle {
        BlockBundle {
            header: BlockHeader {
                point: ChainPoint::new(1, [1; 32]),
                bundle_number: 1,
                parent_hash: [2; 32],
                body_hash: [3; 32],
            },
            items: vec![BlockItem::new("item", vec![1])],
        }
    }

    fn fixture_bundle(previous: Option<ChainTip>) -> BlockBundle {
        let items = vec![BlockItem::new("item", vec![1])];
        let body_hash = local_fixture_body_hash(&items);
        let slot = previous.map_or(1, |tip| tip.point.slot + 1);
        let bundle_number = previous.map_or(1, |tip| tip.bundle_number + 1);
        let parent_hash = previous.map_or([2; 32], |tip| tip.point.hash);
        BlockBundle {
            header: BlockHeader {
                point: ChainPoint::new(slot, [4; 32]),
                bundle_number,
                parent_hash,
                body_hash,
            },
            items,
        }
    }

    #[test]
    fn block_producer_readiness_reports_missing_material_without_reading_files() {
        let config = BlockProducerConfig {
            enabled: true,
            tree_key: Some(PathBuf::from("tree.key")),
            producer_key: None,
            operational_certificate: None,
        };
        assert_eq!(
            BlockProducerReadiness::from_config(&config),
            BlockProducerReadiness::Missing(vec!["producer_key", "certificate"])
        );
    }

    #[test]
    fn block_producer_self_check_uses_local_block_validation() {
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };
        assert_eq!(
            check.check(&bundle(), &BlockProducerReadiness::Disabled),
            Ok(())
        );
    }

    #[test]
    fn block_producer_self_check_validates_local_fixture_integrity() {
        let previous = ChainTip::new(ChainPoint::new(1, [9; 32]), 1);
        let bundle = fixture_bundle(Some(previous));
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_local_fixture_integrity(
                &bundle,
                &BlockProducerReadiness::Disabled,
                Some(previous),
            ),
            Ok(())
        );
    }

    #[test]
    fn block_producer_self_check_rejects_bad_local_fixture_body_hash() {
        let mut bundle = fixture_bundle(None);
        let expected = bundle.header.body_hash;
        bundle.header.body_hash = [8; 32];
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_local_fixture_integrity(&bundle, &BlockProducerReadiness::Disabled, None),
            Err(BlockProducerError::Block(
                BlockValidationError::BodyHashMismatch {
                    expected,
                    actual: [8; 32],
                }
            ))
        );
    }

    #[test]
    fn block_producer_self_check_validates_local_fixture_with_witness() {
        let previous = ChainTip::new(ChainPoint::new(1, [9; 32]), 1);
        let bundle = fixture_bundle(Some(previous));
        let witness = BlockProductionWitness::from_bundle(&bundle, vec![1, 2, 3, 4]);
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_local_fixture_with_witness(
                &bundle,
                &BlockProducerReadiness::Disabled,
                Some(previous),
                &witness,
                4,
            ),
            Ok(())
        );
    }

    #[test]
    fn block_producer_self_check_rejects_local_fixture_witness_mismatch() {
        let bundle = fixture_bundle(None);
        let mut witness = BlockProductionWitness::from_bundle(&bundle, vec![1, 2, 3, 4]);
        witness.body_hash = [7; 32];
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_local_fixture_with_witness(
                &bundle,
                &BlockProducerReadiness::Disabled,
                None,
                &witness,
                4,
            ),
            Err(BlockProducerError::WitnessMismatch)
        );
    }

    #[test]
    fn block_production_witness_matches_bundle_without_crypto() {
        let bundle = bundle();
        let witness = BlockProductionWitness::from_bundle(&bundle, vec![1, 2, 3]);
        assert_eq!(witness.validate_for(&bundle, 3), Ok(()));
        assert_eq!(
            witness.validate_for(&bundle, 4),
            Err(BlockProducerError::ProofTooSmall { min: 4, actual: 3 })
        );
    }

    #[test]
    fn block_producer_self_check_validates_witness_locally() {
        let bundle = bundle();
        let witness = BlockProductionWitness::from_bundle(&bundle, vec![1, 2, 3, 4]);
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_with_witness(&bundle, &BlockProducerReadiness::Disabled, &witness, 4),
            Ok(())
        );
        assert_eq!(
            check.check_with_witness(&bundle, &BlockProducerReadiness::Disabled, &witness, 5),
            Err(BlockProducerError::ProofTooSmall { min: 5, actual: 4 })
        );
    }

    #[test]
    fn block_production_witness_can_be_built_from_local_slot_proof() {
        let bundle = bundle();
        let proof = SlotProof {
            age: 1,
            slot: bundle.header.point.slot,
            pool_id: vec![1],
            proof_bytes: vec![1, 2, 3, 4],
        };

        let witness = BlockProductionWitness::from_slot_proof(&bundle, &proof).unwrap();

        assert_eq!(witness.slot, bundle.header.point.slot);
        assert_eq!(witness.proof_bytes, proof.proof_bytes);

        let wrong_slot_proof = SlotProof { slot: 99, ..proof };
        assert_eq!(
            BlockProductionWitness::from_slot_proof(&bundle, &wrong_slot_proof),
            Err(BlockProducerError::WitnessMismatch)
        );
    }

    #[test]
    fn block_producer_self_check_validates_local_slot_proof() {
        let bundle = bundle();
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 100,
            certificate_bytes: vec![3; 8],
        };
        let proof = SlotProof {
            age: 3,
            slot: bundle.header.point.slot,
            pool_id: vec![1],
            proof_bytes: [0_u64.to_be_bytes().as_slice(), &[9, 8]].concat(),
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        let verification = check
            .check_slot_proof(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                &proof,
                &certificate,
                SlotEligibilityCalculator::new(0.1, 100),
                10,
            )
            .unwrap();

        assert!(verification.decision.eligible);
        assert_eq!(verification.decision.sample_ratio, 0.0);
    }

    #[test]
    fn block_producer_self_check_report_returns_witness() {
        let bundle = bundle();
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 100,
            certificate_bytes: vec![3; 8],
        };
        let proof = SlotProof {
            age: 3,
            slot: bundle.header.point.slot,
            pool_id: vec![1],
            proof_bytes: [0_u64.to_be_bytes().as_slice(), &[9, 8]].concat(),
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        let report = check
            .check_slot_proof_report(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                &proof,
                &certificate,
                SlotEligibilityCalculator::new(0.1, 100),
                10,
            )
            .unwrap();

        assert!(report.verification.decision.eligible);
        assert_eq!(report.witness.bundle_number, bundle.header.bundle_number);
        assert_eq!(report.witness.slot, bundle.header.point.slot);
        assert_eq!(report.witness.proof_bytes, proof.proof_bytes);
    }

    #[test]
    fn block_producer_self_check_report_renders_summary_line() {
        let bundle = bundle();
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 100,
            certificate_bytes: vec![3; 8],
        };
        let proof = SlotProof {
            age: 3,
            slot: bundle.header.point.slot,
            pool_id: vec![1],
            proof_bytes: [0_u64.to_be_bytes().as_slice(), &[9, 8]].concat(),
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        let report = check
            .check_slot_proof_report(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                &proof,
                &certificate,
                SlotEligibilityCalculator::new(0.1, 100),
                10,
            )
            .unwrap();

        assert_eq!(
            report.summary_line(),
            "block_self_check bundle_number=1 slot=1 proof_bytes=10 eligible=true sample_ratio=0.000000 threshold=0.010481 cert_valid_from=0 cert_valid_until=100"
        );
    }

    #[test]
    fn block_producer_self_check_report_emits_local_event() {
        let bundle = bundle();
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 100,
            certificate_bytes: vec![3; 8],
        };
        let proof = SlotProof {
            age: 3,
            slot: bundle.header.point.slot,
            pool_id: vec![1],
            proof_bytes: [0_u64.to_be_bytes().as_slice(), &[9, 8]].concat(),
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        let report = check
            .check_slot_proof_report(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                &proof,
                &certificate,
                SlotEligibilityCalculator::new(0.1, 100),
                10,
            )
            .unwrap();
        let event = report.to_event();
        let events = report.events();

        assert_eq!(event.name.as_str(), BLOCK_PRODUCER_SELF_CHECK_EVENT);
        assert_eq!(event.payload, EventPayload::Text(report.summary_line()));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), BLOCK_PRODUCER_SELF_CHECK_EVENT);
        assert_eq!(events[0].payload, EventPayload::Text(report.summary_line()));
    }

    #[test]
    fn block_producer_self_check_error_emits_local_event() {
        let err = BlockProducerError::ProofTooSmall { min: 4, actual: 1 };
        let event = err.to_event();
        let events = err.events();

        assert_eq!(
            err.summary_line(),
            "block_self_check_failed reason=proof_too_small min=4 actual=1"
        );
        assert_eq!(event.name.as_str(), BLOCK_PRODUCER_SELF_CHECK_FAILED_EVENT);
        assert_eq!(
            event.payload,
            EventPayload::Text(
                "block_self_check_failed reason=proof_too_small min=4 actual=1".to_string()
            )
        );
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].name.as_str(),
            BLOCK_PRODUCER_SELF_CHECK_FAILED_EVENT
        );
        assert_eq!(events[0].payload, event.payload);
    }

    #[test]
    fn block_producer_self_check_validates_local_fixture_slot_proof() {
        let previous = ChainTip::new(ChainPoint::new(1, [9; 32]), 1);
        let bundle = fixture_bundle(Some(previous));
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let proof = SlotProof {
            age: 3,
            slot: bundle.header.point.slot,
            pool_id: vec![1],
            proof_bytes: [0_u64.to_be_bytes().as_slice(), &[9, 8]].concat(),
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        let report = check
            .check_local_fixture_slot_proof_report(
                &bundle,
                &BlockProducerReadiness::Disabled,
                Some(previous),
                &schedule,
                &proof,
                &certificate,
                SlotEligibilityCalculator::new(0.1, 100),
                10,
            )
            .unwrap();

        assert!(report.verification.decision.eligible);
        assert_eq!(report.witness.slot, bundle.header.point.slot);
    }

    #[test]
    fn block_producer_self_check_rejects_bad_fixture_slot_proof_bundle() {
        let mut bundle = fixture_bundle(None);
        let expected = bundle.header.body_hash;
        bundle.header.body_hash = [8; 32];
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let proof = SlotProof {
            age: 3,
            slot: bundle.header.point.slot,
            pool_id: vec![1],
            proof_bytes: [0_u64.to_be_bytes().as_slice(), &[9, 8]].concat(),
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_local_fixture_slot_proof(
                &bundle,
                &BlockProducerReadiness::Disabled,
                None,
                &schedule,
                &proof,
                &certificate,
                SlotEligibilityCalculator::new(0.1, 100),
                10,
            ),
            Err(BlockProducerError::Block(
                BlockValidationError::BodyHashMismatch {
                    expected,
                    actual: [8; 32],
                }
            ))
        );
    }

    #[test]
    fn block_producer_self_check_validates_local_fixture_scheduled_slot() {
        let previous = ChainTip::new(ChainPoint::new(1, [9; 32]), 1);
        let bundle = fixture_bundle(Some(previous));
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        let report = check
            .check_local_fixture_scheduled_slot_report(
                &bundle,
                &BlockProducerReadiness::Disabled,
                Some(previous),
                &schedule,
                bundle.header.point.slot,
                &certificate,
                SlotEligibilityCalculator::new(1.0, 100),
                10,
            )
            .unwrap();

        assert!(report.verification.decision.eligible);
        assert_eq!(report.witness.slot, bundle.header.point.slot);
    }

    #[test]
    fn block_producer_self_check_rejects_unscheduled_local_fixture_slot() {
        let bundle = fixture_bundle(None);
        let schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_local_fixture_scheduled_slot(
                &bundle,
                &BlockProducerReadiness::Disabled,
                None,
                &schedule,
                bundle.header.point.slot,
                &certificate,
                SlotEligibilityCalculator::new(1.0, 100),
                10,
            ),
            Err(BlockProducerError::Production(
                BlockProductionError::SlotNotScheduled(bundle.header.point.slot)
            ))
        );
    }

    #[test]
    fn block_producer_self_check_validates_local_fixture_next_slot() {
        let previous = ChainTip::new(ChainPoint::new(1, [9; 32]), 1);
        let bundle = fixture_bundle(Some(previous));
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        schedule.add_scheduled_slot(5);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        let report = check
            .check_local_fixture_next_scheduled_slot_report(
                &bundle,
                &BlockProducerReadiness::Disabled,
                Some(previous),
                &schedule,
                bundle.header.point.slot,
                &certificate,
                SlotEligibilityCalculator::new(1.0, 100),
                10,
            )
            .unwrap();

        assert!(report.verification.decision.eligible);
        assert_eq!(report.witness.slot, bundle.header.point.slot);
    }

    #[test]
    fn block_producer_self_check_reports_missing_local_fixture_next_slot() {
        let bundle = fixture_bundle(None);
        let schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_local_fixture_next_scheduled_slot(
                &bundle,
                &BlockProducerReadiness::Disabled,
                None,
                &schedule,
                0,
                &certificate,
                SlotEligibilityCalculator::new(1.0, 100),
                10,
            ),
            Err(BlockProducerError::NoScheduledSlot {
                at_or_after_slot: 0
            })
        );
    }

    #[test]
    fn block_producer_self_check_rejects_unverified_slot_proof() {
        let bundle = bundle();
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 100,
            certificate_bytes: vec![3; 8],
        };
        let proof = SlotProof {
            age: 3,
            slot: 99,
            pool_id: vec![1],
            proof_bytes: [0_u64.to_be_bytes().as_slice(), &[9, 8]].concat(),
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_slot_proof(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                &proof,
                &certificate,
                SlotEligibilityCalculator::new(0.1, 100),
                10,
            ),
            Err(BlockProducerError::Production(
                BlockProductionError::SlotNotScheduled(99)
            ))
        );
    }

    #[test]
    fn block_producer_self_check_rejects_ineligible_slot_proof() {
        let bundle = bundle();
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let proof = SlotProof {
            age: 3,
            slot: bundle.header.point.slot,
            pool_id: vec![1],
            proof_bytes: [u64::MAX.to_be_bytes().as_slice(), &[9, 8]].concat(),
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_slot_proof(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                &proof,
                &certificate,
                SlotEligibilityCalculator::new(0.1, 100),
                10,
            ),
            Err(BlockProducerError::SlotNotEligible)
        );
    }

    #[test]
    fn block_producer_self_check_builds_proof_for_scheduled_slot() {
        let bundle = bundle();
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        let verification = check
            .check_scheduled_slot(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                bundle.header.point.slot,
                &certificate,
                SlotEligibilityCalculator::new(1.0, 100),
                10,
            )
            .unwrap();

        assert!(verification.decision.eligible);
    }

    #[test]
    fn block_producer_scheduled_slot_report_returns_witness() {
        let bundle = bundle();
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        let report = check
            .check_scheduled_slot_report(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                bundle.header.point.slot,
                &certificate,
                SlotEligibilityCalculator::new(1.0, 100),
                10,
            )
            .unwrap();

        assert!(report.verification.decision.eligible);
        assert_eq!(report.witness.slot, bundle.header.point.slot);
        assert_eq!(report.witness.body_hash, bundle.header.body_hash);
    }

    #[test]
    fn block_producer_self_check_rejects_unscheduled_slot_before_proof_bytes() {
        let bundle = bundle();
        let schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_scheduled_slot(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                bundle.header.point.slot,
                &certificate,
                SlotEligibilityCalculator::new(1.0, 100),
                10,
            ),
            Err(BlockProducerError::Production(
                BlockProductionError::SlotNotScheduled(bundle.header.point.slot)
            ))
        );
    }

    #[test]
    fn block_producer_self_check_uses_next_scheduled_slot() {
        let bundle = bundle();
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        schedule.add_scheduled_slot(5);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        let report = check
            .check_next_scheduled_slot_report(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                0,
                &certificate,
                SlotEligibilityCalculator::new(1.0, 100),
                10,
            )
            .unwrap();

        assert_eq!(report.witness.slot, bundle.header.point.slot);
    }

    #[test]
    fn block_producer_self_check_can_return_next_slot_verification_only() {
        let bundle = bundle();
        let mut schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        schedule.add_scheduled_slot(bundle.header.point.slot);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        let verification = check
            .check_next_scheduled_slot(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                0,
                &certificate,
                SlotEligibilityCalculator::new(1.0, 100),
                10,
            )
            .unwrap();

        assert!(verification.decision.eligible);
        assert_eq!(verification.certificate_valid_until_slot, 10);
    }

    #[test]
    fn block_producer_self_check_reports_missing_next_slot() {
        let bundle = bundle();
        let schedule = BlockProductionSchedule::new(3, vec![1], 10, 100, vec![9, 8]);
        let certificate = OperationalCertificateFixture {
            pool_id: vec![1],
            producer_key_hash: vec![7; 32],
            valid_from_slot: 0,
            valid_until_slot: 10,
            certificate_bytes: vec![3; 8],
        };
        let check = BlockProducerSelfCheck {
            max_item_bytes: 10,
            require_material: false,
        };

        assert_eq!(
            check.check_next_scheduled_slot_report(
                &bundle,
                &BlockProducerReadiness::Disabled,
                &schedule,
                0,
                &certificate,
                SlotEligibilityCalculator::new(1.0, 100),
                10,
            ),
            Err(BlockProducerError::NoScheduledSlot {
                at_or_after_slot: 0
            })
        );
    }
}
