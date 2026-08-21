pub mod metrics;
pub mod tracing;

use crate::app::{
    RuntimeLifecyclePlan, RuntimeReloadPreflight, RuntimeShutdownPreflight, StartupPlan,
};
use crate::block_forging::{BlockProducerError, BlockProducerSelfCheckReport};
use crate::events::{Event, EventPayload};
use crate::interfaces::{InterfacePlan, InterfaceRequestPlan, InterfaceRoutePlan};
use crate::ledger::blocks::{BlockDecodeError, BlockEraValidationReport, BlockValidationError};
use crate::network::{
    CardanoHandshakeConformanceReport, CardanoHandshakeErrorProtocolVectorCase,
    CardanoHandshakeErrorProtocolVectorReport, CardanoHandshakeHarnessRun,
    CardanoHandshakeNegotiationReport, CardanoHandshakeRefusalTranscriptReplay,
    CardanoHandshakeResponse, CardanoHandshakeStateMachinePlan,
    CardanoHandshakeTimeoutProtocolVector, CardanoHandshakeTranscriptProtocolVector,
    CardanoHandshakeTranscriptReplay, CardanoMuxFrame, CardanoMuxFrameProtocolVector,
    CardanoMuxFrameStreamSummary, CardanoNtNHandshakeAcceptProtocolVector,
    CardanoNtNHandshakeProtocolVector, CardanoNtNHandshakeRefusalProtocolVector,
    CardanoNtNVersionDataPlan, HandshakeHello, HandshakePlan, LocalHandshakeSketch, NetworkError,
    NetworkOpenReview, NetworkPlan, ParsedCardanoNtNVersionData, TestnetContactLimits,
    TestnetContactPlan, TestnetContactRequest, TestnetHandshakeConformanceMatrix,
    TestnetLiveReadiness, TestnetTcpProbePlan, TestnetTcpProbeRequest,
};
use crate::peers::{PeerConnectionPlan, PeerDiscoveryPlan, PeerLifecyclePlan};
use crate::sync::{
    LocalFixtureBlockApplyReport, LocalHeaderApplyReport, LocalRollbackExecution,
    LocalRollbackRecoveryExecutionReport, SyncExecutionError,
};
use metrics::{MetricsBook, MetricsSnapshot};
use tracing::TraceBook;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocalObservabilityCollector {
    metrics: MetricsBook,
    traces: TraceBook,
}

impl LocalObservabilityCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_event(&mut self, event: &Event) {
        let key = metric_key(event.name.as_str());
        self.metrics.increment(format!("events.{key}.count"), 1);
        let mut fields = vec![("event", event.name.as_str().to_string())];
        match &event.payload {
            EventPayload::None => fields.push(("payload", "none".to_string())),
            EventPayload::Text(text) => {
                fields.push(("payload", "text".to_string()));
                fields.push(("text_len", text.len().to_string()));
            }
            EventPayload::Number(value) => {
                fields.push(("payload", "number".to_string()));
                fields.push(("value", value.to_string()));
                self.metrics.set_gauge(
                    format!("events.{key}.last_number"),
                    (*value).min(i64::MAX as u64) as i64,
                );
            }
            EventPayload::Bytes(bytes) => {
                fields.push(("payload", "bytes".to_string()));
                fields.push(("bytes_len", bytes.len().to_string()));
            }
        }
        self.traces.mark_with_fields(format!("event.{key}"), fields);
    }

    pub fn record_events<'a>(&mut self, events: impl IntoIterator<Item = &'a Event>) -> usize {
        let mut recorded = 0;
        for event in events {
            self.record_event(event);
            recorded += 1;
        }
        recorded
    }

    pub fn record_block_producer_self_check_result(
        &mut self,
        result: Result<&BlockProducerSelfCheckReport, &BlockProducerError>,
    ) -> usize {
        match result {
            Ok(report) => {
                let events = report.events();
                self.record_events(events.iter())
            }
            Err(err) => {
                let events = err.events();
                self.record_events(events.iter())
            }
        }
    }

    pub fn record_block_era_validation_result(
        &mut self,
        result: Result<&BlockEraValidationReport, &BlockValidationError>,
    ) -> usize {
        match result {
            Ok(report) => {
                let events = report.events();
                self.record_events(events.iter())
            }
            Err(err) => {
                let events = err.events();
                self.record_events(events.iter())
            }
        }
    }

    pub fn record_block_decode_validation_result(
        &mut self,
        result: Result<&BlockEraValidationReport, &BlockDecodeError>,
    ) -> usize {
        match result {
            Ok(report) => {
                let events = report.events();
                self.record_events(events.iter())
            }
            Err(err) => {
                let events = err.events();
                self.record_events(events.iter())
            }
        }
    }

    pub fn record_sync_fixture_apply_result(
        &mut self,
        result: Result<&LocalFixtureBlockApplyReport, &SyncExecutionError>,
    ) -> usize {
        match result {
            Ok(report) => {
                let events = report.events();
                self.record_events(events.iter())
            }
            Err(err) => {
                let events = err.events();
                self.record_events(events.iter())
            }
        }
    }

    pub fn record_sync_header_apply_result(
        &mut self,
        result: Result<&LocalHeaderApplyReport, &SyncExecutionError>,
    ) -> usize {
        match result {
            Ok(report) => {
                let events = report.events();
                self.record_events(events.iter())
            }
            Err(err) => {
                let events = err.events();
                self.record_events(events.iter())
            }
        }
    }

    pub fn record_sync_rollback_result(
        &mut self,
        result: Result<&LocalRollbackExecution, &SyncExecutionError>,
    ) -> usize {
        match result {
            Ok(report) => {
                let events = report.events();
                self.record_events(events.iter())
            }
            Err(err) => {
                let events = err.events();
                self.record_events(events.iter())
            }
        }
    }

    pub fn record_sync_rollback_recovery_result(
        &mut self,
        result: Result<&LocalRollbackRecoveryExecutionReport, &SyncExecutionError>,
    ) -> usize {
        match result {
            Ok(report) => {
                let events = report.events();
                self.record_events(events.iter())
            }
            Err(err) => {
                let events = err.events();
                self.record_events(events.iter())
            }
        }
    }

    pub fn record_peer_discovery_plan(&mut self, plan: &PeerDiscoveryPlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_startup_plan(&mut self, plan: &StartupPlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_runtime_lifecycle_plan(&mut self, plan: &RuntimeLifecyclePlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_runtime_reload_preflight(&mut self, preflight: &RuntimeReloadPreflight) -> usize {
        let events = preflight.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_runtime_shutdown_preflight(
        &mut self,
        preflight: &RuntimeShutdownPreflight,
    ) -> usize {
        let events = preflight.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_peer_lifecycle_plan(&mut self, plan: &PeerLifecyclePlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_peer_connection_plan(&mut self, plan: &PeerConnectionPlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_interface_plan(&mut self, plan: &InterfacePlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_interface_route_plan(&mut self, plan: &InterfaceRoutePlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_interface_request_plan(&mut self, plan: &InterfaceRequestPlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_network_open_review(&mut self, review: &NetworkOpenReview) -> usize {
        let events = review.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_testnet_contact_plan(&mut self, plan: &TestnetContactPlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_testnet_contact_request(&mut self, request: &TestnetContactRequest) -> usize {
        let events = request.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_testnet_contact_limits(&mut self, limits: &TestnetContactLimits) -> usize {
        let events = limits.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_testnet_tcp_probe_plan(&mut self, plan: &TestnetTcpProbePlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_testnet_tcp_probe_request(&mut self, request: &TestnetTcpProbeRequest) -> usize {
        let events = request.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_testnet_live_readiness(&mut self, readiness: &TestnetLiveReadiness) -> usize {
        let events = readiness.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_conformance_report(
        &mut self,
        report: &CardanoHandshakeConformanceReport,
    ) -> usize {
        let events = report.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_conformance_matrix(
        &mut self,
        matrix: &TestnetHandshakeConformanceMatrix,
    ) -> usize {
        let events = matrix.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_error_protocol_vector_report(
        &mut self,
        report: &CardanoHandshakeErrorProtocolVectorReport,
    ) -> usize {
        let events = report.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_error_protocol_vector_case(
        &mut self,
        case: &CardanoHandshakeErrorProtocolVectorCase,
    ) -> usize {
        let events = case.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_negotiation_report(
        &mut self,
        report: &CardanoHandshakeNegotiationReport,
    ) -> usize {
        let events = report.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_harness_run(&mut self, run: &CardanoHandshakeHarnessRun) -> usize {
        let events = run.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_transcript_protocol_vector(
        &mut self,
        transcript: &CardanoHandshakeTranscriptProtocolVector,
    ) -> usize {
        let events = transcript.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_transcript_replay(
        &mut self,
        replay: &CardanoHandshakeTranscriptReplay,
    ) -> usize {
        let events = replay.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_refusal_transcript_replay(
        &mut self,
        replay: &CardanoHandshakeRefusalTranscriptReplay,
    ) -> usize {
        let events = replay.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_timeout_protocol_vector(
        &mut self,
        protocol_vector: &CardanoHandshakeTimeoutProtocolVector,
    ) -> usize {
        let events = protocol_vector.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_version_data_plan(
        &mut self,
        plan: &CardanoNtNVersionDataPlan,
    ) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_proposal_protocol_vector(
        &mut self,
        protocol_vector: &CardanoNtNHandshakeProtocolVector,
    ) -> usize {
        let events = protocol_vector.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_accept_protocol_vector(
        &mut self,
        protocol_vector: &CardanoNtNHandshakeAcceptProtocolVector,
    ) -> usize {
        let events = protocol_vector.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_refusal_protocol_vector(
        &mut self,
        protocol_vector: &CardanoNtNHandshakeRefusalProtocolVector,
    ) -> usize {
        let events = protocol_vector.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_mux_frame_protocol_vector(
        &mut self,
        protocol_vector: &CardanoMuxFrameProtocolVector,
    ) -> usize {
        let events = protocol_vector.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_mux_frame(&mut self, frame: &CardanoMuxFrame) -> usize {
        let events = frame.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_mux_frame_stream_summary(
        &mut self,
        summary: &CardanoMuxFrameStreamSummary,
    ) -> usize {
        let events = summary.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_response(&mut self, response: &CardanoHandshakeResponse) -> usize {
        let events = response.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_version_data(
        &mut self,
        version_data: &ParsedCardanoNtNVersionData,
    ) -> usize {
        let events = version_data.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_network_error(&mut self, error: &NetworkError) -> usize {
        let events = error.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_network_plan(&mut self, plan: &NetworkPlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_plan(&mut self, plan: &HandshakePlan) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_hello(&mut self, hello: &HandshakeHello) -> usize {
        let events = hello.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_local_handshake_sketch(&mut self, sketch: &LocalHandshakeSketch) -> usize {
        let events = sketch.event_batch();
        self.record_events(events.iter())
    }

    pub fn record_handshake_state_machine_plan(
        &mut self,
        plan: &CardanoHandshakeStateMachinePlan,
    ) -> usize {
        let events = plan.event_batch();
        self.record_events(events.iter())
    }

    pub fn metrics_snapshot(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    pub fn trace_book(&self) -> &TraceBook {
        &self.traces
    }
}

fn metric_key(name: &str) -> String {
    let mut key = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            key.push(ch);
        } else {
            key.push('_');
        }
    }
    if key.is_empty() {
        "unknown".to_string()
    } else {
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_forging::{
        BlockProducerError, BlockProducerSelfCheckReport, BlockProductionWitness,
        BLOCK_PRODUCER_SELF_CHECK_EVENT,
    };
    use crate::block_production::{SlotProofDecision, SlotProofVerification};
    use crate::config::{DataMode, InterfaceConfig};
    use crate::interfaces::{
        InterfacePlan, INTERFACE_BLOCKED_EVENT, INTERFACE_PLAN_EVENT, INTERFACE_REQUEST_PLAN_EVENT,
        INTERFACE_ROUTE_PLAN_EVENT,
    };
    use crate::ledger::blocks::{
        BlockDecodeError, BlockEraValidationReport, BlockValidationError,
        BLOCK_ERA_VALIDATION_EVENT,
    };
    use crate::network::{
        cardano_handshake_negotiation_report, cardano_mux_frame_protocol_vector,
        cardano_ntn_handshake_accept_protocol_vector, cardano_ntn_handshake_conformance_report,
        cardano_ntn_handshake_error_protocol_vector_report, cardano_ntn_handshake_protocol_vector,
        cardano_ntn_handshake_refusal_transcript_replay,
        cardano_ntn_handshake_transcript_protocol_vector, cardano_ntn_handshake_transcript_replay,
        cardano_ntn_handshake_version_mismatch_refusal_protocol_vector,
        cardano_ntn_version_data_plan, local_handshake_sketch, parse_cardano_handshake_response,
        plan_bounded_testnet_contact, plan_testnet_tcp_probe, run_cardano_ntn_handshake_harness,
        testnet_live_readiness, CardanoNtNDiffusionMode, ConnectionRole, HandshakeHello,
        HandshakePlan, NetworkPlan, TestnetContactLimits, TestnetContactRequest,
        TestnetTcpProbeRequest, CARDANO_NTN_REFUSAL_SUPPORTED_VERSIONS,
        CARDANO_NTN_SUPPORTED_VERSIONS, NETWORK_ERROR_EVENT,
        NETWORK_HANDSHAKE_ACCEPT_PROTOCOL_VECTOR_EVENT, NETWORK_HANDSHAKE_CONFORMANCE_EVENT,
        NETWORK_HANDSHAKE_CONFORMANCE_MATRIX_EVENT, NETWORK_HANDSHAKE_ERROR_PROTOCOL_VECTORS_EVENT,
        NETWORK_HANDSHAKE_ERROR_PROTOCOL_VECTOR_CASE_EVENT, NETWORK_HANDSHAKE_HARNESS_EVENT,
        NETWORK_HANDSHAKE_HELLO_EVENT, NETWORK_HANDSHAKE_NEGOTIATION_EVENT,
        NETWORK_HANDSHAKE_PLAN_EVENT, NETWORK_HANDSHAKE_PROPOSAL_PROTOCOL_VECTOR_EVENT,
        NETWORK_HANDSHAKE_REFUSAL_PROTOCOL_VECTOR_EVENT,
        NETWORK_HANDSHAKE_REFUSAL_TRANSCRIPT_REPLAY_EVENT, NETWORK_HANDSHAKE_RESPONSE_EVENT,
        NETWORK_HANDSHAKE_SKETCH_EVENT, NETWORK_HANDSHAKE_STATE_MACHINE_EVENT,
        NETWORK_HANDSHAKE_TIMEOUT_PROTOCOL_VECTOR_EVENT,
        NETWORK_HANDSHAKE_TRANSCRIPT_PROTOCOL_VECTOR_EVENT,
        NETWORK_HANDSHAKE_TRANSCRIPT_REPLAY_EVENT, NETWORK_HANDSHAKE_VERSION,
        NETWORK_HANDSHAKE_VERSION_DATA_EVENT, NETWORK_HANDSHAKE_VERSION_DATA_PLAN_EVENT,
        NETWORK_MUX_FRAME_EVENT, NETWORK_MUX_FRAME_PROTOCOL_VECTOR_EVENT,
        NETWORK_MUX_FRAME_STREAM_EVENT, NETWORK_OPEN_BLOCKED_EVENT, NETWORK_OPEN_REVIEW_EVENT,
        NETWORK_PLAN_EVENT, NETWORK_TESTNET_CONTACT_LIMITS_EVENT,
        NETWORK_TESTNET_CONTACT_PLAN_EVENT, NETWORK_TESTNET_CONTACT_REQUEST_EVENT,
        NETWORK_TESTNET_LIVE_READINESS_EVENT, NETWORK_TESTNET_TCP_PROBE_PLAN_EVENT,
        NETWORK_TESTNET_TCP_PROBE_REQUEST_EVENT,
    };
    use crate::peers::{
        PeerConnectionPlan, PeerDiscoveryEntry, PeerDiscoveryPlan, PeerDiscoverySource,
        PeerLifecyclePlan, PeerState,
    };
    use crate::sync::RollbackRecoveryPlan;
    use crate::topology::PeerAddress;

    #[test]
    fn local_observability_collector_counts_events_and_marks_traces() {
        let mut collector = LocalObservabilityCollector::new();
        collector.record_event(&Event::new("sync.block_fetch", EventPayload::Number(2)));
        collector.record_event(&Event::new(
            "mempool.add",
            EventPayload::Text("accepted".to_string()),
        ));

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.sync_block_fetch.count"], 1);
        assert_eq!(snapshot.counters["events.mempool_add.count"], 1);
        assert_eq!(snapshot.gauges["events.sync_block_fetch.last_number"], 2);
        assert_eq!(collector.trace_book().marks().len(), 2);
        assert_eq!(
            collector.trace_book().render_lines(),
            vec![
                "event.sync_block_fetch event=sync.block_fetch payload=number value=2",
                "event.mempool_add event=mempool.add payload=text text_len=8",
            ]
        );
    }

    #[test]
    fn local_observability_collector_records_byte_payload_sizes() {
        let mut collector = LocalObservabilityCollector::new();
        collector.record_event(&Event::new(
            "chain.block-offered",
            EventPayload::Bytes(vec![1, 2]),
        ));

        assert_eq!(
            collector.trace_book().render_lines(),
            vec!["event.chain_block_offered event=chain.block-offered payload=bytes bytes_len=2"]
        );
    }

    #[test]
    fn local_observability_collector_records_block_self_check_events() {
        let mut collector = LocalObservabilityCollector::new();
        let event = Event::new(
            BLOCK_PRODUCER_SELF_CHECK_EVENT,
            EventPayload::Text("block_self_check bundle_number=1".to_string()),
        );

        collector.record_event(&event);

        assert_eq!(
            collector.metrics_snapshot().counters["events.block_producer_self_check.count"],
            1
        );
        assert_eq!(
            collector.trace_book().render_lines(),
            vec!["event.block_producer_self_check event=block_producer.self_check payload=text text_len=32"]
        );
    }

    #[test]
    fn local_observability_collector_records_block_self_check_failure_events() {
        let mut collector = LocalObservabilityCollector::new();
        let event = BlockProducerError::SlotNotEligible.to_event();

        collector.record_event(&event);

        assert_eq!(
            collector.metrics_snapshot().counters["events.block_producer_self_check_failed.count"],
            1
        );
        assert_eq!(
            collector.trace_book().render_lines(),
            vec!["event.block_producer_self_check_failed event=block_producer.self_check_failed payload=text text_len=48"]
        );
    }

    #[test]
    fn local_observability_collector_records_block_self_check_event_batches() {
        let success = Event::new(
            BLOCK_PRODUCER_SELF_CHECK_EVENT,
            EventPayload::Text("block_self_check bundle_number=1".to_string()),
        );
        let failure = BlockProducerError::SlotNotEligible;
        let mut events = vec![success];
        events.extend(failure.events());
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_events(events.iter()), 2);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.block_producer_self_check.count"],
            1
        );
        assert_eq!(
            snapshot.counters["events.block_producer_self_check_failed.count"],
            1
        );
        assert_eq!(collector.trace_book().render_lines().len(), 2);
    }

    #[test]
    fn local_observability_collector_records_block_self_check_results() {
        let report = BlockProducerSelfCheckReport {
            verification: SlotProofVerification {
                decision: SlotProofDecision {
                    eligible: true,
                    sample_ratio: 0.25,
                    threshold: 0.5,
                },
                certificate_valid_from_slot: 3,
                certificate_valid_until_slot: 9,
            },
            witness: BlockProductionWitness {
                bundle_number: 7,
                slot: 8,
                body_hash: [0; 32],
                proof_bytes: vec![1, 2, 3, 4],
            },
        };
        let err = BlockProducerError::WitnessMismatch;
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(
            collector.record_block_producer_self_check_result(Ok(&report)),
            1
        );
        assert_eq!(
            collector.record_block_producer_self_check_result(Err(&err)),
            1
        );

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.block_producer_self_check.count"],
            1
        );
        assert_eq!(
            snapshot.counters["events.block_producer_self_check_failed.count"],
            1
        );
        assert_eq!(collector.trace_book().render_lines().len(), 2);
    }

    #[test]
    fn local_observability_collector_records_block_era_validation_events() {
        let report = BlockEraValidationReport {
            major_version: 6,
            era_name: "fixture-current",
            age_id: 2,
            item_count: 2,
            max_items: 4_096,
            max_item_bytes: 128 * 1024,
            parent_required: true,
            fixture_integrity_checked: true,
        };
        let mut collector = LocalObservabilityCollector::new();

        collector.record_event(&report.to_event());

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.ledger_block_era_validation.count"],
            1
        );
        assert_eq!(
            collector.trace_book().render_lines(),
            vec![format!(
                "event.ledger_block_era_validation event={} payload=text text_len={}",
                BLOCK_ERA_VALIDATION_EVENT,
                report.summary_line().len()
            )]
        );
    }

    #[test]
    fn local_observability_collector_records_block_era_validation_failure_events() {
        let err = BlockValidationError::UnknownMajorVersion(10);
        let mut collector = LocalObservabilityCollector::new();

        collector.record_event(&err.to_event());

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.ledger_block_era_validation_failed.count"],
            1
        );
        assert_eq!(
            collector.trace_book().render_lines(),
            vec![format!(
                "event.ledger_block_era_validation_failed event=ledger.block_era_validation_failed payload=text text_len={}",
                err.summary_line().len()
            )]
        );
    }

    #[test]
    fn local_observability_collector_records_block_decode_failure_events() {
        let err = BlockDecodeError::TrailingBytes(2);
        let mut collector = LocalObservabilityCollector::new();

        collector.record_event(&err.to_event());

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.ledger_block_decode_failed.count"],
            1
        );
        assert_eq!(
            collector.trace_book().render_lines(),
            vec![format!(
                "event.ledger_block_decode_failed event=ledger.block_decode_failed payload=text text_len={}",
                err.summary_line().len()
            )]
        );
    }

    #[test]
    fn local_observability_collector_records_block_decode_validation_results() {
        let report = BlockEraValidationReport {
            major_version: 6,
            era_name: "fixture-current",
            age_id: 2,
            item_count: 2,
            max_items: 4_096,
            max_item_bytes: 128 * 1024,
            parent_required: true,
            fixture_integrity_checked: true,
        };
        let err = BlockDecodeError::InvalidMagic;
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(
            collector.record_block_decode_validation_result(Ok(&report)),
            1
        );
        assert_eq!(
            collector.record_block_decode_validation_result(Err(&err)),
            1
        );

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.ledger_block_era_validation.count"],
            1
        );
        assert_eq!(
            snapshot.counters["events.ledger_block_decode_failed.count"],
            1
        );
        assert_eq!(collector.trace_book().render_lines().len(), 2);
    }

    #[test]
    fn local_observability_collector_records_block_era_validation_results() {
        let report = BlockEraValidationReport {
            major_version: 6,
            era_name: "fixture-current",
            age_id: 2,
            item_count: 2,
            max_items: 4_096,
            max_item_bytes: 128 * 1024,
            parent_required: true,
            fixture_integrity_checked: true,
        };
        let err = BlockValidationError::UnknownMajorVersion(10);
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_block_era_validation_result(Ok(&report)), 1);
        assert_eq!(collector.record_block_era_validation_result(Err(&err)), 1);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.ledger_block_era_validation.count"],
            1
        );
        assert_eq!(
            snapshot.counters["events.ledger_block_era_validation_failed.count"],
            1
        );
        assert_eq!(collector.trace_book().render_lines().len(), 2);
    }

    #[test]
    fn local_observability_collector_records_sync_fixture_apply_events() {
        let validation = BlockEraValidationReport {
            major_version: 6,
            era_name: "fixture-current",
            age_id: 2,
            item_count: 2,
            max_items: 4_096,
            max_item_bytes: 128 * 1024,
            parent_required: true,
            fixture_integrity_checked: true,
        };
        let report = LocalFixtureBlockApplyReport {
            headers: LocalHeaderApplyReport {
                from: crate::chain::ChainTip::ORIGIN,
                to: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(1, [1; 32]), 1),
                applied: 1,
                chain_switch: None,
            },
            validations: vec![validation],
        };
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_events(report.events().iter()), 3);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.sync_local_fixture_block_apply.count"],
            1
        );
        assert_eq!(snapshot.counters["events.sync_local_header_apply.count"], 1);
        assert_eq!(
            snapshot.counters["events.ledger_block_era_validation.count"],
            1
        );
        assert_eq!(collector.trace_book().render_lines().len(), 3);
    }

    #[test]
    fn local_observability_collector_records_sync_header_apply_events() {
        let report = LocalHeaderApplyReport {
            from: crate::chain::ChainTip::ORIGIN,
            to: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(1, [1; 32]), 1),
            applied: 1,
            chain_switch: None,
        };
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_events(report.events().iter()), 1);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.sync_local_header_apply.count"], 1);
        assert_eq!(collector.trace_book().render_lines().len(), 1);
    }

    #[test]
    fn local_observability_collector_records_sync_header_apply_results() {
        let report = LocalHeaderApplyReport {
            from: crate::chain::ChainTip::ORIGIN,
            to: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(1, [1; 32]), 1),
            applied: 1,
            chain_switch: None,
        };
        let err = SyncExecutionError::FetchExceededPlan { max: 1, actual: 2 };
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_sync_header_apply_result(Ok(&report)), 1);
        assert_eq!(collector.record_sync_header_apply_result(Err(&err)), 1);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.sync_local_header_apply.count"], 1);
        assert_eq!(snapshot.counters["events.sync_execution_failed.count"], 1);
        assert_eq!(collector.trace_book().render_lines().len(), 2);
    }

    #[test]
    fn local_observability_collector_records_sync_rollback_events() {
        let report = LocalRollbackExecution {
            from: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(3, [3; 32]), 3),
            to: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(1, [1; 32]), 1),
            removed_headers: 2,
            chain_switch: None,
        };
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_events(report.events().iter()), 1);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.sync_local_rollback.count"], 1);
        assert_eq!(collector.trace_book().render_lines().len(), 1);
    }

    #[test]
    fn local_observability_collector_records_sync_rollback_results() {
        let report = LocalRollbackExecution {
            from: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(3, [3; 32]), 3),
            to: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(1, [1; 32]), 1),
            removed_headers: 2,
            chain_switch: None,
        };
        let err = SyncExecutionError::RollbackAfterTip {
            rollback_slot: 4,
            tip_slot: 3,
        };
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_sync_rollback_result(Ok(&report)), 1);
        assert_eq!(collector.record_sync_rollback_result(Err(&err)), 1);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.sync_local_rollback.count"], 1);
        assert_eq!(snapshot.counters["events.sync_execution_failed.count"], 1);
        assert_eq!(collector.trace_book().render_lines().len(), 2);
    }

    #[test]
    fn local_observability_collector_records_sync_rollback_recovery_events() {
        let plan = RollbackRecoveryPlan::after_rollback(
            "peer",
            crate::chain::ChainPoint::new(1, [1; 32]),
            crate::chain::ChainTip::new(crate::chain::ChainPoint::new(1, [1; 32]), 1),
            crate::chain::ChainTip::new(crate::chain::ChainPoint::new(3, [3; 32]), 3),
            false,
        );
        let report = LocalRollbackRecoveryExecutionReport {
            plan,
            execution: LocalRollbackExecution {
                from: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(3, [3; 32]), 3),
                to: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(1, [1; 32]), 1),
                removed_headers: 2,
                chain_switch: None,
            },
        };
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_events(report.events().iter()), 3);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.sync_peer_rollback.count"], 1);
        assert_eq!(snapshot.counters["events.sync_block_fetch.count"], 1);
        assert_eq!(snapshot.counters["events.sync_local_rollback.count"], 1);
        assert_eq!(collector.trace_book().render_lines().len(), 3);
    }

    #[test]
    fn local_observability_collector_records_sync_rollback_recovery_results() {
        let plan = RollbackRecoveryPlan::after_rollback(
            "peer",
            crate::chain::ChainPoint::new(1, [1; 32]),
            crate::chain::ChainTip::new(crate::chain::ChainPoint::new(1, [1; 32]), 1),
            crate::chain::ChainTip::new(crate::chain::ChainPoint::new(3, [3; 32]), 3),
            false,
        );
        let report = LocalRollbackRecoveryExecutionReport {
            plan,
            execution: LocalRollbackExecution {
                from: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(3, [3; 32]), 3),
                to: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(1, [1; 32]), 1),
                removed_headers: 2,
                chain_switch: None,
            },
        };
        let err = SyncExecutionError::RollbackAfterTip {
            rollback_slot: 4,
            tip_slot: 3,
        };
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(
            collector.record_sync_rollback_recovery_result(Ok(&report)),
            3
        );
        assert_eq!(collector.record_sync_rollback_recovery_result(Err(&err)), 1);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.sync_peer_rollback.count"], 1);
        assert_eq!(snapshot.counters["events.sync_block_fetch.count"], 1);
        assert_eq!(snapshot.counters["events.sync_local_rollback.count"], 1);
        assert_eq!(snapshot.counters["events.sync_execution_failed.count"], 1);
        assert_eq!(collector.trace_book().render_lines().len(), 4);
    }

    #[test]
    fn local_observability_collector_records_peer_discovery_event_batches() {
        let plan = PeerDiscoveryPlan {
            entries: vec![
                PeerDiscoveryEntry {
                    peer: PeerAddress::new("local.test", 3001),
                    source: PeerDiscoverySource::LocalRoot,
                    advertise: true,
                    trustable: true,
                },
                PeerDiscoveryEntry {
                    peer: PeerAddress::new("seed.test", 3001),
                    source: PeerDiscoverySource::Seed,
                    advertise: false,
                    trustable: false,
                },
            ],
        };
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_peer_discovery_plan(&plan), 3);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.peers_discovery_plan.count"], 1);
        assert_eq!(snapshot.counters["events.peers_discover.count"], 2);
        assert_eq!(collector.trace_book().render_lines().len(), 3);
    }

    #[test]
    fn local_observability_collector_records_runtime_lifecycle_event_batches() {
        let lifecycle = crate::Node::new(crate::NodeConfig::default())
            .unwrap()
            .startup_plan()
            .lifecycle_plan();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_runtime_lifecycle_plan(&lifecycle), 1);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.runtime_lifecycle_plan.count"], 1);
        assert_eq!(collector.trace_book().render_lines().len(), 1);
    }

    #[test]
    fn local_observability_collector_records_startup_plan_event_batches() {
        let plan = crate::Node::new(crate::NodeConfig::default())
            .unwrap()
            .startup_plan();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_startup_plan(&plan), 2);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.startup_plan.count"], 1);
        assert_eq!(snapshot.counters["events.startup_categories.count"], 1);
        assert_eq!(collector.trace_book().render_lines().len(), 2);
    }

    #[test]
    fn local_observability_collector_records_runtime_reload_preflight_event_batches() {
        let preflight = crate::Node::new(crate::NodeConfig::default())
            .unwrap()
            .startup_plan()
            .lifecycle_plan()
            .reload_preflight(&["config", "unknown"]);
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_runtime_reload_preflight(&preflight), 1);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.runtime_reload_preflight.count"],
            1
        );
        assert_eq!(collector.trace_book().render_lines().len(), 1);
    }

    #[test]
    fn local_observability_collector_records_runtime_shutdown_preflight_event_batches() {
        let preflight = crate::Node::new(crate::NodeConfig::default())
            .unwrap()
            .startup_plan()
            .lifecycle_plan()
            .shutdown_preflight();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_runtime_shutdown_preflight(&preflight), 1);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.runtime_shutdown_preflight.count"],
            1
        );
        assert_eq!(collector.trace_book().render_lines().len(), 1);
    }

    #[test]
    fn local_observability_collector_records_peer_lifecycle_event_batches() {
        let plan = PeerLifecyclePlan {
            added: vec![PeerAddress::new("added.test", 3001)],
            promoted: vec![(PeerAddress::new("promoted.test", 3001), PeerState::Hot)],
            pruned: vec![PeerAddress::new("pruned.test", 3001)],
        };
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_peer_lifecycle_plan(&plan), 4);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.peers_lifecycle_plan.count"], 1);
        assert_eq!(snapshot.counters["events.peers_churn.count"], 1);
        assert_eq!(snapshot.counters["events.peers_promote.count"], 1);
        assert_eq!(snapshot.counters["events.peers_prune.count"], 1);
        assert_eq!(collector.trace_book().render_lines().len(), 4);
    }

    #[test]
    fn local_observability_collector_records_peer_connection_event_batches() {
        let plan = PeerConnectionPlan {
            warm: vec![PeerAddress::new("warm.test", 3001)],
            hot: vec![PeerAddress::new("hot.test", 3001)],
            open_paths: false,
            blocked_reason: Some("peer paths are blocked by safety config".to_string()),
        };
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_peer_connection_plan(&plan), 2);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.peers_connection_plan.count"], 1);
        assert_eq!(
            snapshot.counters["events.peers_connection_blocked.count"],
            1
        );
        assert_eq!(collector.trace_book().render_lines().len(), 2);
    }

    #[test]
    fn local_observability_collector_records_sync_execution_failure_events() {
        let err = SyncExecutionError::FetchExceededPlan { max: 1, actual: 2 };
        let mut collector = LocalObservabilityCollector::new();

        collector.record_event(&err.to_event());

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.sync_execution_failed.count"], 1);
        assert_eq!(
            collector.trace_book().render_lines(),
            vec![format!(
                "event.sync_execution_failed event=sync.execution_failed payload=text text_len={}",
                err.summary_line().len()
            )]
        );
    }

    #[test]
    fn local_observability_collector_records_sync_decode_failure_event_batches() {
        let err = SyncExecutionError::LocalBlockDecode(BlockDecodeError::TrailingBytes(2));
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_events(err.events().iter()), 2);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.sync_execution_failed.count"], 1);
        assert_eq!(
            snapshot.counters["events.ledger_block_decode_failed.count"],
            1
        );
        assert_eq!(collector.trace_book().render_lines().len(), 2);
    }

    #[test]
    fn local_observability_collector_records_sync_fixture_apply_results() {
        let validation = BlockEraValidationReport {
            major_version: 6,
            era_name: "fixture-current",
            age_id: 2,
            item_count: 2,
            max_items: 4_096,
            max_item_bytes: 128 * 1024,
            parent_required: true,
            fixture_integrity_checked: true,
        };
        let report = LocalFixtureBlockApplyReport {
            headers: LocalHeaderApplyReport {
                from: crate::chain::ChainTip::ORIGIN,
                to: crate::chain::ChainTip::new(crate::chain::ChainPoint::new(1, [1; 32]), 1),
                applied: 1,
                chain_switch: None,
            },
            validations: vec![validation],
        };
        let err = SyncExecutionError::FetchExceededPlan { max: 1, actual: 2 };
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_sync_fixture_apply_result(Ok(&report)), 3);
        assert_eq!(collector.record_sync_fixture_apply_result(Err(&err)), 1);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.sync_local_fixture_block_apply.count"],
            1
        );
        assert_eq!(snapshot.counters["events.sync_local_header_apply.count"], 1);
        assert_eq!(
            snapshot.counters["events.ledger_block_era_validation.count"],
            1
        );
        assert_eq!(snapshot.counters["events.sync_execution_failed.count"], 1);
        assert_eq!(collector.trace_book().render_lines().len(), 4);
    }

    #[test]
    fn local_observability_collector_records_blocked_plan_event_batches() {
        let plan = InterfacePlan::from_config(
            &InterfaceConfig {
                transaction_port: 10,
                state_port: 11,
                batch_port: 0,
                archive_port: 0,
                archive_base_url: None,
            },
            DataMode::Core,
        );
        let events = plan.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_interface_plan(&plan), 2);
        assert_eq!(events[0].name.as_str(), INTERFACE_PLAN_EVENT);
        assert_eq!(events[1].name.as_str(), INTERFACE_BLOCKED_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.interfaces_blocked.count"],
            1
        );
        assert_eq!(
            collector.metrics_snapshot().counters["events.interfaces_plan.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[1].contains("event=interfaces.blocked"));
        assert!(collector.trace_book().render_lines()[1].contains("payload=text"));
    }

    #[test]
    fn local_observability_collector_records_interface_route_event_batches() {
        let plan = InterfacePlan::route_plan(
            &InterfaceConfig {
                transaction_port: 10,
                state_port: 0,
                batch_port: 0,
                archive_port: 20,
                archive_base_url: None,
            },
            DataMode::Core,
        );
        let events = plan.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_interface_route_plan(&plan), 1);
        assert_eq!(events[0].name.as_str(), INTERFACE_ROUTE_PLAN_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.interfaces_route_plan.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0].contains("event=interfaces.route_plan"));
    }

    #[test]
    fn local_observability_collector_records_interface_request_event_batches() {
        let plan = InterfacePlan::request_plan(&InterfaceConfig {
            transaction_port: 10,
            state_port: 0,
            batch_port: 0,
            archive_port: 20,
            archive_base_url: None,
        });
        let events = plan.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_interface_request_plan(&plan), 1);
        assert_eq!(events[0].name.as_str(), INTERFACE_REQUEST_PLAN_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.interfaces_request_plan.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0].contains("event=interfaces.request_plan"));
    }

    #[test]
    fn local_observability_collector_records_handshake_hello_event_batches() {
        let hello = HandshakeHello::new(2, ConnectionRole::Outer, PeerAddress::new("local", 3001));
        let events = hello.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_handshake_hello(&hello), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_HANDSHAKE_HELLO_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_handshake_hello.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0].contains("event=network.handshake_hello"));
    }

    #[test]
    fn local_observability_collector_records_local_handshake_sketch_event_batches() {
        let sketch = local_handshake_sketch(
            crate::config::network_profile("preview").unwrap(),
            &[NETWORK_HANDSHAKE_VERSION],
        )
        .unwrap();
        let events = sketch.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_local_handshake_sketch(&sketch), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_HANDSHAKE_SKETCH_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_handshake_sketch.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0].contains("event=network.handshake_sketch"));
    }

    #[test]
    fn local_observability_collector_records_handshake_plan_event_batches() {
        let plan = HandshakePlan {
            local: HandshakeHello::new(2, ConnectionRole::Outer, PeerAddress::new("local", 3001)),
            remote: HandshakeHello::new(2, ConnectionRole::Inner, PeerAddress::new("remote", 3002)),
            share_peer: false,
            intersect_tip: false,
        };
        let events = plan.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_handshake_plan(&plan), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_HANDSHAKE_PLAN_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_handshake_plan.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0].contains("event=network.handshake_plan"));
    }

    #[test]
    fn local_observability_collector_records_network_plan_event_batches() {
        let plan = NetworkPlan::from_config(&crate::config::NodeConfig::default());
        let events = plan.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_network_plan(&plan), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_PLAN_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_plan.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0].contains("event=network.plan"));
    }

    #[test]
    fn local_observability_collector_records_network_open_review_event_batches() {
        let plan = NetworkPlan::from_config(&crate::config::NodeConfig::default());
        let review = plan.open_review();
        let events = review.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_network_open_review(&review), 2);
        assert_eq!(events[0].name.as_str(), NETWORK_OPEN_REVIEW_EVENT);
        assert_eq!(events[1].name.as_str(), NETWORK_OPEN_BLOCKED_EVENT);

        let snapshot = collector.metrics_snapshot();
        assert_eq!(snapshot.counters["events.network_open_review.count"], 1);
        assert_eq!(snapshot.counters["events.network_open_blocked.count"], 1);
        assert_eq!(collector.trace_book().render_lines().len(), 2);
    }

    #[test]
    fn local_observability_collector_records_testnet_contact_limits_event_batches() {
        let limits = TestnetContactLimits::smoke_test();
        let events = limits.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_testnet_contact_limits(&limits), 1);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_TESTNET_CONTACT_LIMITS_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_testnet_contact_limits.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.testnet_contact_limits"));
    }

    #[test]
    fn local_observability_collector_records_testnet_contact_request_event_batches() {
        let request = TestnetContactRequest::new("testnet", 2, 100, 1024);
        let events = request.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_testnet_contact_request(&request), 1);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_TESTNET_CONTACT_REQUEST_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_testnet_contact_request.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.testnet_contact_request"));
    }

    #[test]
    fn local_observability_collector_records_testnet_contact_plan_event_batches() {
        let plan = plan_bounded_testnet_contact(
            TestnetContactRequest::new("testnet", 2, 100, 1024),
            TestnetContactLimits::smoke_test(),
        )
        .unwrap();
        let events = plan.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_testnet_contact_plan(&plan), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_TESTNET_CONTACT_PLAN_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_testnet_contact_plan.count"],
            1
        );
        assert!(
            collector.trace_book().render_lines()[0].contains("event=network.testnet_contact_plan")
        );
    }

    #[test]
    fn local_observability_collector_records_testnet_tcp_probe_request_event_batches() {
        let request = TestnetTcpProbeRequest::new("preview", "8.8.8.8:3001", true, 2);
        let events = request.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_testnet_tcp_probe_request(&request), 1);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_TESTNET_TCP_PROBE_REQUEST_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_testnet_tcp_probe_request.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.testnet_tcp_probe_request"));
    }

    #[test]
    fn local_observability_collector_records_testnet_tcp_probe_plan_event_batches() {
        let plan = plan_testnet_tcp_probe(
            TestnetTcpProbeRequest::new("preview", "8.8.8.8:3001", true, 2),
            TestnetContactLimits::smoke_test(),
        )
        .unwrap();
        let events = plan.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_testnet_tcp_probe_plan(&plan), 1);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_TESTNET_TCP_PROBE_PLAN_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_testnet_tcp_probe_plan.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.testnet_tcp_probe_plan"));
    }

    #[test]
    fn local_observability_collector_records_testnet_live_readiness_event_batches() {
        let readiness = testnet_live_readiness();
        let events = readiness.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_testnet_live_readiness(&readiness), 1);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_TESTNET_LIVE_READINESS_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_testnet_live_readiness.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.testnet_live_readiness"));
    }

    #[test]
    fn local_observability_collector_records_handshake_conformance_event_batches() {
        let report = cardano_ntn_handshake_conformance_report(
            crate::config::network_profile("preprod").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();
        let events = report.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_handshake_conformance_report(&report), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_HANDSHAKE_CONFORMANCE_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_handshake_conformance.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_conformance"));
    }

    #[test]
    fn local_observability_collector_records_handshake_response_event_batches() {
        let response =
            parse_cardano_handshake_response(&[0x83, 0x01, 0x0a, 0x82, 0x02, 0xf4]).unwrap();
        let events = response.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_handshake_response(&response), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_HANDSHAKE_RESPONSE_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_handshake_response.count"],
            1
        );
        assert!(
            collector.trace_book().render_lines()[0].contains("event=network.handshake_response")
        );
    }

    #[test]
    fn local_observability_collector_records_handshake_version_data_event_batches() {
        let response =
            parse_cardano_handshake_response(&[0x83, 0x01, 0x0a, 0x82, 0x02, 0xf4]).unwrap();
        let version_data = match response.kind {
            crate::network::CardanoHandshakeResponseKind::AcceptVersion {
                version_data, ..
            } => version_data,
            other => panic!("expected accept-version response, got {other:?}"),
        };
        let events = version_data.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_handshake_version_data(&version_data), 1);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_VERSION_DATA_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_handshake_version_data.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_version_data"));
    }

    #[test]
    fn local_observability_collector_records_handshake_negotiation_event_batches() {
        let response =
            parse_cardano_handshake_response(&[0x83, 0x01, 0x0a, 0x82, 0x02, 0xf4]).unwrap();
        let report = cardano_handshake_negotiation_report(
            crate::config::network_profile("preview").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
            &response,
        )
        .unwrap();
        let events = report.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_handshake_negotiation_report(&report), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_HANDSHAKE_NEGOTIATION_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_handshake_negotiation.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_negotiation"));
    }

    #[test]
    fn local_observability_collector_records_handshake_harness_event_batches() {
        let profile = crate::config::network_profile("preview").unwrap();
        let accept = cardano_ntn_handshake_accept_protocol_vector(profile, 10).unwrap();
        let response_frame =
            cardano_mux_frame_protocol_vector(accept.protocol_id, &accept.encoded, true, 0)
                .unwrap();
        let run = run_cardano_ntn_handshake_harness(
            profile,
            &CARDANO_NTN_SUPPORTED_VERSIONS,
            &response_frame.encoded,
        )
        .unwrap();
        let events = run.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_handshake_harness_run(&run), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_HANDSHAKE_HARNESS_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_handshake_harness.count"],
            1
        );
        assert!(
            collector.trace_book().render_lines()[0].contains("event=network.handshake_harness")
        );
    }

    #[test]
    fn local_observability_collector_records_handshake_transcript_protocol_vector_event_batches() {
        let transcript = cardano_ntn_handshake_transcript_protocol_vector(
            crate::config::network_profile("preprod").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();
        let events = transcript.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(
            collector.record_handshake_transcript_protocol_vector(&transcript),
            1
        );
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_TRANSCRIPT_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters
                ["events.network_handshake_transcript_protocol_vector.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_transcript_protocol_vector"));
    }

    #[test]
    fn local_observability_collector_records_handshake_transcript_replay_event_batches() {
        let replay = cardano_ntn_handshake_transcript_replay(
            crate::config::network_profile("preprod").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();
        let events = replay.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_handshake_transcript_replay(&replay), 1);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_TRANSCRIPT_REPLAY_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters
                ["events.network_handshake_transcript_replay.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_transcript_replay"));
    }

    #[test]
    fn local_observability_collector_records_handshake_refusal_transcript_replay_event_batches() {
        let replay = cardano_ntn_handshake_refusal_transcript_replay(
            crate::config::network_profile("preview").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
            &CARDANO_NTN_REFUSAL_SUPPORTED_VERSIONS,
        )
        .unwrap();
        let events = replay.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(
            collector.record_handshake_refusal_transcript_replay(&replay),
            1
        );
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_REFUSAL_TRANSCRIPT_REPLAY_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters
                ["events.network_handshake_refusal_transcript_replay.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_refusal_transcript_replay"));
    }

    #[test]
    fn local_observability_collector_records_handshake_timeout_protocol_vector_event_batches() {
        let protocol_vector = crate::network::cardano_ntn_handshake_timeout_protocol_vector(
            crate::network::CardanoHandshakeState::Confirm,
            crate::network::CARDANO_NTN_HANDSHAKE_CONFIRM_TIMEOUT_SECS,
        )
        .unwrap();
        let events = protocol_vector.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(
            collector.record_handshake_timeout_protocol_vector(&protocol_vector),
            1
        );
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_TIMEOUT_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters
                ["events.network_handshake_timeout_protocol_vector.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_timeout_protocol_vector"));
    }

    #[test]
    fn local_observability_collector_records_handshake_version_data_plan_event_batches() {
        let plan = cardano_ntn_version_data_plan(
            crate::config::network_profile("preview").unwrap(),
            13,
            CardanoNtNDiffusionMode::InitiatorAndResponder,
            true,
            false,
        )
        .unwrap();
        let events = plan.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_handshake_version_data_plan(&plan), 1);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_VERSION_DATA_PLAN_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters
                ["events.network_handshake_version_data_plan.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_version_data_plan"));
    }

    #[test]
    fn local_observability_collector_records_handshake_proposal_protocol_vector_event_batches() {
        let protocol_vector = cardano_ntn_handshake_protocol_vector(
            crate::config::network_profile("preview").unwrap(),
            &[7, 8, 9, 10],
        )
        .unwrap();
        let events = protocol_vector.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(
            collector.record_handshake_proposal_protocol_vector(&protocol_vector),
            1
        );
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_PROPOSAL_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters
                ["events.network_handshake_proposal_protocol_vector.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_proposal_protocol_vector"));
    }

    #[test]
    fn local_observability_collector_records_handshake_accept_protocol_vector_event_batches() {
        let protocol_vector = cardano_ntn_handshake_accept_protocol_vector(
            crate::config::network_profile("preview").unwrap(),
            10,
        )
        .unwrap();
        let events = protocol_vector.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(
            collector.record_handshake_accept_protocol_vector(&protocol_vector),
            1
        );
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_ACCEPT_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters
                ["events.network_handshake_accept_protocol_vector.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_accept_protocol_vector"));
    }

    #[test]
    fn local_observability_collector_records_handshake_refusal_protocol_vector_event_batches() {
        let protocol_vector = cardano_ntn_handshake_version_mismatch_refusal_protocol_vector(
            &CARDANO_NTN_REFUSAL_SUPPORTED_VERSIONS,
        )
        .unwrap();
        let events = protocol_vector.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(
            collector.record_handshake_refusal_protocol_vector(&protocol_vector),
            1
        );
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_REFUSAL_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters
                ["events.network_handshake_refusal_protocol_vector.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_refusal_protocol_vector"));
    }

    #[test]
    fn local_observability_collector_records_mux_frame_protocol_vector_event_batches() {
        let handshake = cardano_ntn_handshake_protocol_vector(
            crate::config::network_profile("preprod").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();
        let frame =
            cardano_mux_frame_protocol_vector(handshake.protocol_id, &handshake.encoded, false, 0)
                .unwrap();
        let events = frame.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_mux_frame_protocol_vector(&frame), 1);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_MUX_FRAME_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_mux_frame_protocol_vector.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.mux_frame_protocol_vector"));
    }

    #[test]
    fn local_observability_collector_records_mux_frame_event_batches() {
        let handshake = cardano_ntn_handshake_protocol_vector(
            crate::config::network_profile("preprod").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();
        let protocol_vector =
            cardano_mux_frame_protocol_vector(handshake.protocol_id, &handshake.encoded, false, 0)
                .unwrap();
        let frame = crate::network::parse_cardano_mux_frame(&protocol_vector.encoded).unwrap();
        let events = frame.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_mux_frame(&frame), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_MUX_FRAME_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_mux_frame.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0].contains("event=network.mux_frame"));
    }

    #[test]
    fn local_observability_collector_records_mux_frame_stream_summary_event_batches() {
        let transcript = crate::network::cardano_ntn_handshake_transcript_protocol_vector(
            crate::config::network_profile("preprod").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();
        let mut stream = Vec::new();
        stream.extend_from_slice(&transcript.request_frame.encoded);
        stream.extend_from_slice(&transcript.response_frame.encoded);
        let frames = crate::network::parse_cardano_mux_frame_stream(&stream, 2).unwrap();
        let summary = crate::network::cardano_mux_frame_stream_summary(&frames).unwrap();
        let events = summary.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_mux_frame_stream_summary(&summary), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_MUX_FRAME_STREAM_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_mux_frame_stream.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0].contains("event=network.mux_frame_stream"));
    }

    #[test]
    fn local_observability_collector_records_network_error_event_batches() {
        let error = crate::network::NetworkError::UnknownNetwork("local-only".to_string());
        let events = error.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_network_error(&error), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_ERROR_EVENT);
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_error.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0].contains("event=network.error"));
    }

    #[test]
    fn local_observability_collector_records_handshake_conformance_matrix_event_batches() {
        let matrix =
            crate::network::testnet_handshake_conformance_matrix(&CARDANO_NTN_SUPPORTED_VERSIONS)
                .unwrap();
        let events = matrix.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_handshake_conformance_matrix(&matrix), 3);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_CONFORMANCE_MATRIX_EVENT
        );

        let snapshot = collector.metrics_snapshot();
        assert_eq!(
            snapshot.counters["events.network_handshake_conformance_matrix.count"],
            1
        );
        assert_eq!(
            snapshot.counters["events.network_handshake_conformance.count"],
            2
        );
        assert_eq!(collector.trace_book().render_lines().len(), 3);
    }

    #[test]
    fn local_observability_collector_records_handshake_error_protocol_vectors_event_batches() {
        let report = cardano_ntn_handshake_error_protocol_vector_report(
            crate::config::network_profile("preview").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();
        let events = report.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(
            collector.record_handshake_error_protocol_vector_report(&report),
            1
        );
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_ERROR_PROTOCOL_VECTORS_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters
                ["events.network_handshake_error_protocol_vectors.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_error_protocol_vectors"));
    }

    #[test]
    fn local_observability_collector_records_handshake_error_protocol_vector_case_event_batches() {
        let report = cardano_ntn_handshake_error_protocol_vector_report(
            crate::config::network_profile("preview").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();
        let case = &report.cases[0];
        let events = case.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(
            collector.record_handshake_error_protocol_vector_case(case),
            1
        );
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_ERROR_PROTOCOL_VECTOR_CASE_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters
                ["events.network_handshake_error_protocol_vector_case.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_error_protocol_vector_case"));
    }

    #[test]
    fn local_observability_collector_records_handshake_state_machine_event_batches() {
        let plan = crate::network::cardano_ntn_handshake_state_machine_plan();
        let events = plan.event_batch();
        let mut collector = LocalObservabilityCollector::new();

        assert_eq!(collector.record_handshake_state_machine_plan(&plan), 1);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_HANDSHAKE_STATE_MACHINE_EVENT
        );
        assert_eq!(
            collector.metrics_snapshot().counters["events.network_handshake_state_machine.count"],
            1
        );
        assert!(collector.trace_book().render_lines()[0]
            .contains("event=network.handshake_state_machine"));
    }
}
