use crate::chain::{ChainPoint, ChainTip};
use crate::events::{Event, EventPayload};
use crate::ledger::blocks::{BlockBundle, BlockDecodeError, BlockEraRule};
use std::fmt;

pub const BOOTSTRAP_BLOCKED_EVENT: &str = "bootstrap.blocked";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapRequest {
    pub subject: String,
    pub location: String,
}

impl BootstrapRequest {
    pub fn new(subject: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            location: location.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BootstrapPolicy {
    pub allow_external: bool,
    pub allow_private_addresses: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapPlan {
    Blocked { reason: String },
    WouldFetch { request: BootstrapRequest },
    LocalFixture { snapshot: BootstrapSnapshot },
}

impl BootstrapPlan {
    pub fn events(&self) -> Vec<Event> {
        match self {
            Self::Blocked { reason } => vec![Event::new(
                BOOTSTRAP_BLOCKED_EVENT,
                EventPayload::Text(format!("reason={reason}")),
            )],
            Self::WouldFetch { .. } | Self::LocalFixture { .. } => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapSnapshot {
    pub subject: String,
    pub network_magic: u32,
    pub tip: ChainTip,
    pub payload_bytes: Vec<u8>,
}

impl BootstrapSnapshot {
    pub fn new(
        subject: impl Into<String>,
        network_magic: u32,
        tip: ChainTip,
        payload_bytes: Vec<u8>,
    ) -> Self {
        Self {
            subject: subject.into(),
            network_magic,
            tip,
            payload_bytes,
        }
    }

    pub fn validate(&self, expected_network_magic: u32) -> Result<(), BootstrapError> {
        if self.subject.trim().is_empty() {
            return Err(BootstrapError::EmptySubject);
        }
        if self.network_magic != expected_network_magic {
            return Err(BootstrapError::NetworkMismatch {
                expected: expected_network_magic,
                actual: self.network_magic,
            });
        }
        if self.payload_bytes.is_empty() {
            return Err(BootstrapError::EmptySnapshotPayload);
        }
        if self.tip.point != ChainPoint::ORIGIN && self.tip.point.hash == [0; 32] {
            return Err(BootstrapError::EmptyTipHash);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalBootstrapReport {
    pub subject: String,
    pub tip: ChainTip,
    pub payload_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalBootstrapPayloadInspection {
    pub subject: String,
    pub tip: ChainTip,
    pub payload_bytes: usize,
    pub item_count: usize,
}

pub fn plan_bootstrap_request(
    request: BootstrapRequest,
    policy: BootstrapPolicy,
) -> Result<BootstrapPlan, BootstrapError> {
    if request.subject.trim().is_empty() {
        return Err(BootstrapError::EmptySubject);
    }
    if !request.location.starts_with("https://") {
        return Ok(BootstrapPlan::Blocked {
            reason: "location must use a secure scheme".to_string(),
        });
    }
    if !policy.allow_external {
        return Ok(BootstrapPlan::Blocked {
            reason: "external bootstrap is disabled".to_string(),
        });
    }
    if !policy.allow_private_addresses && is_private_address(&request.location) {
        return Ok(BootstrapPlan::Blocked {
            reason: "private bootstrap address is disabled".to_string(),
        });
    }
    Ok(BootstrapPlan::WouldFetch { request })
}

pub fn plan_local_bootstrap_snapshot(
    snapshot: BootstrapSnapshot,
    expected_network_magic: u32,
) -> Result<BootstrapPlan, BootstrapError> {
    snapshot.validate(expected_network_magic)?;
    Ok(BootstrapPlan::LocalFixture { snapshot })
}

pub fn apply_local_bootstrap_snapshot(
    snapshot: &BootstrapSnapshot,
    expected_network_magic: u32,
) -> Result<LocalBootstrapReport, BootstrapError> {
    snapshot.validate(expected_network_magic)?;
    Ok(LocalBootstrapReport {
        subject: snapshot.subject.clone(),
        tip: snapshot.tip,
        payload_bytes: snapshot.payload_bytes.len(),
    })
}

pub fn inspect_local_bootstrap_block_payload(
    snapshot: &BootstrapSnapshot,
    expected_network_magic: u32,
    previous: Option<ChainTip>,
    rule: BlockEraRule,
) -> Result<LocalBootstrapPayloadInspection, BootstrapError> {
    snapshot.validate(expected_network_magic)?;
    let bundle =
        BlockBundle::decode_local_fixture_and_validate(&snapshot.payload_bytes, previous, rule)
            .map_err(BootstrapError::LocalBlockPayload)?;
    let actual_tip = bundle.header.tip();
    if actual_tip != snapshot.tip {
        return Err(BootstrapError::SnapshotTipMismatch {
            expected: snapshot.tip,
            actual: actual_tip,
        });
    }
    Ok(LocalBootstrapPayloadInspection {
        subject: snapshot.subject.clone(),
        tip: actual_tip,
        payload_bytes: snapshot.payload_bytes.len(),
        item_count: bundle.items.len(),
    })
}

fn is_private_address(location: &str) -> bool {
    let address = location.trim_start_matches("https://");
    address.starts_with("localhost")
        || address.starts_with("127.")
        || address.starts_with("10.")
        || address.starts_with("192.168.")
        || address.starts_with("172.16.")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapError {
    EmptySubject,
    NetworkMismatch {
        expected: u32,
        actual: u32,
    },
    EmptySnapshotPayload,
    EmptyTipHash,
    LocalBlockPayload(BlockDecodeError),
    SnapshotTipMismatch {
        expected: ChainTip,
        actual: ChainTip,
    },
}

impl fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySubject => f.write_str("bootstrap subject must not be empty"),
            Self::NetworkMismatch { expected, actual } => write!(
                f,
                "bootstrap network mismatch: expected={expected} actual={actual}"
            ),
            Self::EmptySnapshotPayload => {
                f.write_str("bootstrap snapshot payload must not be empty")
            }
            Self::EmptyTipHash => f.write_str("bootstrap snapshot tip hash must not be empty"),
            Self::LocalBlockPayload(err) => {
                write!(f, "bootstrap local block payload invalid: {err}")
            }
            Self::SnapshotTipMismatch { expected, actual } => write!(
                f,
                "bootstrap snapshot tip mismatch: expected bundle={} actual bundle={}",
                expected.bundle_number, actual.bundle_number
            ),
        }
    }
}

impl std::error::Error for BootstrapError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::blocks::{
        local_fixture_body_hash, local_ledger_fixture_catalog, BlockHeader, BlockItem,
        BlockValidationError,
    };

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
    fn bootstrap_plan_blocks_by_default_without_fetching() {
        let plan = plan_bootstrap_request(
            BootstrapRequest::new("pool", "https://example.invalid/bootstrap"),
            BootstrapPolicy::default(),
        )
        .unwrap();
        assert!(matches!(plan, BootstrapPlan::Blocked { .. }));
        let events = plan.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), BOOTSTRAP_BLOCKED_EVENT);
        assert_eq!(
            events[0].payload,
            EventPayload::Text("reason=external bootstrap is disabled".to_string())
        );
    }

    #[test]
    fn bootstrap_plan_blocks_private_addresses() {
        let plan = plan_bootstrap_request(
            BootstrapRequest::new("pool", "https://127.0.0.1/bootstrap"),
            BootstrapPolicy {
                allow_external: true,
                allow_private_addresses: false,
            },
        )
        .unwrap();
        assert_eq!(
            plan,
            BootstrapPlan::Blocked {
                reason: "private bootstrap address is disabled".to_string(),
            }
        );
    }

    #[test]
    fn local_bootstrap_snapshot_plans_without_fetching() {
        let snapshot = BootstrapSnapshot::new("local-chain", 2, tip(10, 10, 7), vec![1, 2, 3]);
        let plan = plan_local_bootstrap_snapshot(snapshot.clone(), 2).unwrap();

        assert_eq!(plan, BootstrapPlan::LocalFixture { snapshot });
    }

    #[test]
    fn local_bootstrap_snapshot_applies_to_report_only() {
        let snapshot = BootstrapSnapshot::new("local-chain", 2, tip(10, 10, 7), vec![1, 2, 3]);
        let report = apply_local_bootstrap_snapshot(&snapshot, 2).unwrap();

        assert_eq!(report.subject, "local-chain");
        assert_eq!(report.tip, tip(10, 10, 7));
        assert_eq!(report.payload_bytes, 3);
    }

    #[test]
    fn local_bootstrap_snapshot_rejects_network_mismatch() {
        let snapshot = BootstrapSnapshot::new("local-chain", 1, tip(10, 10, 7), vec![1]);

        assert_eq!(
            plan_local_bootstrap_snapshot(snapshot, 2),
            Err(BootstrapError::NetworkMismatch {
                expected: 2,
                actual: 1,
            })
        );
    }

    #[test]
    fn local_bootstrap_snapshot_inspects_encoded_block_payload() {
        let previous = tip(9, 9, 4);
        let bundle = fixture_bundle_after(previous, 7);
        let snapshot = BootstrapSnapshot::new(
            "local-chain",
            2,
            bundle.header.tip(),
            bundle.encode_local().unwrap(),
        );
        let rule = local_ledger_fixture_catalog()
            .rule_for_major_version(6)
            .unwrap();

        let inspection =
            inspect_local_bootstrap_block_payload(&snapshot, 2, Some(previous), rule).unwrap();

        assert_eq!(inspection.subject, "local-chain");
        assert_eq!(inspection.tip, tip(10, 10, 7));
        assert_eq!(inspection.item_count, 1);
        assert_eq!(inspection.payload_bytes, snapshot.payload_bytes.len());
    }

    #[test]
    fn local_bootstrap_snapshot_rejects_payload_tip_mismatch() {
        let previous = tip(9, 9, 4);
        let bundle = fixture_bundle_after(previous, 7);
        let snapshot = BootstrapSnapshot::new(
            "local-chain",
            2,
            tip(10, 10, 8),
            bundle.encode_local().unwrap(),
        );
        let rule = local_ledger_fixture_catalog()
            .rule_for_major_version(6)
            .unwrap();

        assert_eq!(
            inspect_local_bootstrap_block_payload(&snapshot, 2, Some(previous), rule),
            Err(BootstrapError::SnapshotTipMismatch {
                expected: tip(10, 10, 8),
                actual: tip(10, 10, 7),
            })
        );
    }

    #[test]
    fn local_bootstrap_snapshot_rejects_invalid_block_payload() {
        let previous = tip(9, 9, 4);
        let mut bundle = fixture_bundle_after(previous, 7);
        bundle.header.body_hash = [8; 32];
        let snapshot = BootstrapSnapshot::new(
            "local-chain",
            2,
            bundle.header.tip(),
            bundle.encode_local().unwrap(),
        );
        let rule = local_ledger_fixture_catalog()
            .rule_for_major_version(6)
            .unwrap();

        assert!(matches!(
            inspect_local_bootstrap_block_payload(&snapshot, 2, Some(previous), rule),
            Err(BootstrapError::LocalBlockPayload(
                BlockDecodeError::Validation(BlockValidationError::BodyHashMismatch { .. })
            ))
        ));
    }
}
