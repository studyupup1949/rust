use crate::config::{
    network_profile, network_profiles, ListenerPlan, NetworkProfile, NetworkProfileKind, NodeConfig,
};
use crate::events::{Event, EventPayload};
use crate::topology::PeerAddress;
use std::fmt;
use std::net::{IpAddr, SocketAddr};

pub const NETWORK_HANDSHAKE_VERSION: u16 = 1;
pub const NETWORK_HANDSHAKE_RESPONSE_EVENT: &str = "network.handshake_response";
pub const NETWORK_HANDSHAKE_VERSION_DATA_EVENT: &str = "network.handshake_version_data";
pub const NETWORK_HANDSHAKE_HELLO_EVENT: &str = "network.handshake_hello";
pub const NETWORK_HANDSHAKE_PLAN_EVENT: &str = "network.handshake_plan";
pub const NETWORK_HANDSHAKE_SKETCH_EVENT: &str = "network.handshake_sketch";
pub const NETWORK_ERROR_EVENT: &str = "network.error";
pub const NETWORK_MUX_FRAME_EVENT: &str = "network.mux_frame";
pub const NETWORK_MUX_FRAME_PROTOCOL_VECTOR_EVENT: &str = "network.mux_frame_protocol_vector";
pub const NETWORK_MUX_FRAME_STREAM_EVENT: &str = "network.mux_frame_stream";
pub const NETWORK_PLAN_EVENT: &str = "network.plan";
pub const NETWORK_HANDSHAKE_CONFORMANCE_EVENT: &str = "network.handshake_conformance";
pub const NETWORK_HANDSHAKE_CONFORMANCE_MATRIX_EVENT: &str = "network.handshake_conformance_matrix";
pub const NETWORK_HANDSHAKE_ACCEPT_PROTOCOL_VECTOR_EVENT: &str =
    "network.handshake_accept_protocol_vector";
pub const NETWORK_HANDSHAKE_ERROR_PROTOCOL_VECTOR_CASE_EVENT: &str =
    "network.handshake_error_protocol_vector_case";
pub const NETWORK_HANDSHAKE_ERROR_PROTOCOL_VECTORS_EVENT: &str =
    "network.handshake_error_protocol_vectors";
pub const NETWORK_HANDSHAKE_HARNESS_EVENT: &str = "network.handshake_harness";
pub const NETWORK_HANDSHAKE_NEGOTIATION_EVENT: &str = "network.handshake_negotiation";
pub const NETWORK_HANDSHAKE_PROPOSAL_PROTOCOL_VECTOR_EVENT: &str =
    "network.handshake_proposal_protocol_vector";
pub const NETWORK_HANDSHAKE_REFUSAL_PROTOCOL_VECTOR_EVENT: &str =
    "network.handshake_refusal_protocol_vector";
pub const NETWORK_HANDSHAKE_REFUSAL_TRANSCRIPT_REPLAY_EVENT: &str =
    "network.handshake_refusal_transcript_replay";
pub const NETWORK_HANDSHAKE_STATE_MACHINE_EVENT: &str = "network.handshake_state_machine";
pub const NETWORK_HANDSHAKE_TIMEOUT_PROTOCOL_VECTOR_EVENT: &str =
    "network.handshake_timeout_protocol_vector";
pub const NETWORK_HANDSHAKE_TRANSCRIPT_PROTOCOL_VECTOR_EVENT: &str =
    "network.handshake_transcript_protocol_vector";
pub const NETWORK_HANDSHAKE_TRANSCRIPT_REPLAY_EVENT: &str = "network.handshake_transcript_replay";
pub const NETWORK_HANDSHAKE_VERSION_DATA_PLAN_EVENT: &str = "network.handshake_version_data_plan";
pub const NETWORK_OPEN_BLOCKED_EVENT: &str = "network.open.blocked";
pub const NETWORK_OPEN_REVIEW_EVENT: &str = "network.open.review";
pub const NETWORK_TESTNET_CONTACT_LIMITS_EVENT: &str = "network.testnet_contact_limits";
pub const NETWORK_TESTNET_CONTACT_REQUEST_EVENT: &str = "network.testnet_contact_request";
pub const NETWORK_TESTNET_CONTACT_PLAN_EVENT: &str = "network.testnet_contact_plan";
pub const NETWORK_TESTNET_HANDSHAKE_PROBE_REQUEST_EVENT: &str =
    "network.testnet_handshake_probe_request";
pub const NETWORK_TESTNET_HANDSHAKE_PROBE_PLAN_EVENT: &str = "network.testnet_handshake_probe_plan";
pub const NETWORK_TESTNET_TCP_PROBE_REQUEST_EVENT: &str = "network.testnet_tcp_probe_request";
pub const NETWORK_TESTNET_TCP_PROBE_PLAN_EVENT: &str = "network.testnet_tcp_probe_plan";
pub const NETWORK_TESTNET_LIVE_READINESS_EVENT: &str = "network.testnet_live_readiness";
pub const MAX_TESTNET_TCP_PROBE_TIMEOUT_SECS: u64 = 5;
pub const MAX_TESTNET_HANDSHAKE_PROBE_RESPONSE_BYTES: usize = 1024;
pub const MAX_LOCAL_HANDSHAKE_SKETCH_BYTES: usize = 512;
pub const CARDANO_HANDSHAKE_PROTOCOL_ID: u16 = 0;
pub const CARDANO_NTN_SUPPORTED_VERSIONS: [u16; 9] = [7, 8, 9, 10, 11, 12, 13, 14, 15];
pub const CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION: u16 = 15;
pub const CARDANO_NTN_REFUSAL_SUPPORTED_VERSIONS: [u16; 4] = [7, 8, 9, 10];
pub const CARDANO_MUX_HEADER_BYTES: usize = 8;
pub const CARDANO_MUX_MAX_PAYLOAD_LENGTH: usize = 65_535;
pub const CARDANO_MUX_RESPONSE_FLAG: u16 = 0x8000;
pub const CARDANO_NTN_HANDSHAKE_PROPOSE_TIMEOUT_SECS: u64 = 10;
pub const CARDANO_NTN_HANDSHAKE_CONFIRM_TIMEOUT_SECS: u64 = 10;
const TESTNET_LIVE_READINESS_BLOCKERS: [&str; 1] =
    ["full testnet conformance harness is not complete"];
const TESTNET_HANDSHAKE_CONFORMANCE_BLOCKERS: [&str; 3] = [
    "live mux loop is not integrated",
    "network path review is incomplete",
    "full protocol conformance is incomplete",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestnetContactLimits {
    pub allow_testnet: bool,
    pub max_blocks: u64,
    pub max_slots: u64,
    pub max_bytes: u64,
    pub timeout_secs: u64,
    pub temp_storage_bytes: u64,
}

impl TestnetContactLimits {
    pub fn smoke_test() -> Self {
        Self {
            allow_testnet: true,
            max_blocks: 32,
            max_slots: 2_000,
            max_bytes: 8 * 1024 * 1024,
            timeout_secs: 30,
            temp_storage_bytes: 32 * 1024 * 1024,
        }
    }

    fn validate(&self) -> Result<(), NetworkError> {
        if !self.allow_testnet {
            return Err(NetworkError::TestnetContactBlocked(
                "testnet contact requires explicit opt-in".to_string(),
            ));
        }
        if self.max_blocks == 0 {
            return Err(NetworkError::UnboundedTestnetLimit("max_blocks"));
        }
        if self.max_slots == 0 {
            return Err(NetworkError::UnboundedTestnetLimit("max_slots"));
        }
        if self.max_bytes == 0 {
            return Err(NetworkError::UnboundedTestnetLimit("max_bytes"));
        }
        if self.timeout_secs == 0 {
            return Err(NetworkError::UnboundedTestnetLimit("timeout_secs"));
        }
        if self.temp_storage_bytes == 0 {
            return Err(NetworkError::UnboundedTestnetLimit("temp_storage_bytes"));
        }
        Ok(())
    }

    pub fn summary_line(&self) -> String {
        format!(
            "testnet_contact_limits allow_testnet={} max_blocks={} max_slots={} max_bytes={} timeout_secs={} temp_bytes={}",
            self.allow_testnet,
            self.max_blocks,
            self.max_slots,
            self.max_bytes,
            self.timeout_secs,
            self.temp_storage_bytes
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_TESTNET_CONTACT_LIMITS_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestnetContactRequest {
    pub network: String,
    pub requested_blocks: u64,
    pub requested_slots: u64,
    pub requested_bytes: u64,
}

impl TestnetContactRequest {
    pub fn new(
        network: impl Into<String>,
        requested_blocks: u64,
        requested_slots: u64,
        requested_bytes: u64,
    ) -> Self {
        Self {
            network: network.into(),
            requested_blocks,
            requested_slots,
            requested_bytes,
        }
    }

    fn validate(&self) -> Result<(), NetworkError> {
        if self.requested_blocks == 0 {
            return Err(NetworkError::EmptyTestnetRequest("blocks"));
        }
        if self.requested_slots == 0 {
            return Err(NetworkError::EmptyTestnetRequest("slots"));
        }
        if self.requested_bytes == 0 {
            return Err(NetworkError::EmptyTestnetRequest("bytes"));
        }
        Ok(())
    }

    pub fn summary_line(&self) -> String {
        format!(
            "testnet_contact_request network={} blocks={} slots={} bytes={}",
            self.network, self.requested_blocks, self.requested_slots, self.requested_bytes
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_TESTNET_CONTACT_REQUEST_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestnetContactPlan {
    pub profile: NetworkProfile,
    pub request: TestnetContactRequest,
    pub limits: TestnetContactLimits,
}

impl TestnetContactPlan {
    pub fn summary_line(&self) -> String {
        format!(
            "testnet_contact_plan network={} blocks={}/{} slots={}/{} bytes={}/{} temp_bytes={} timeout_secs={}",
            self.profile.name,
            self.request.requested_blocks,
            self.limits.max_blocks,
            self.request.requested_slots,
            self.limits.max_slots,
            self.request.requested_bytes,
            self.limits.max_bytes,
            self.limits.temp_storage_bytes,
            self.limits.timeout_secs
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_TESTNET_CONTACT_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestnetTcpProbeRequest {
    pub network: String,
    pub peer: String,
    pub allow_live_testnet: bool,
    pub timeout_secs: u64,
}

impl TestnetTcpProbeRequest {
    pub fn new(
        network: impl Into<String>,
        peer: impl Into<String>,
        allow_live_testnet: bool,
        timeout_secs: u64,
    ) -> Self {
        Self {
            network: network.into(),
            peer: peer.into(),
            allow_live_testnet,
            timeout_secs,
        }
    }

    pub fn summary_line(&self) -> String {
        format!(
            "testnet_tcp_probe_request network={} peer_supplied={} allow_live_testnet={} timeout_secs={}",
            self.network,
            !self.peer.is_empty(),
            self.allow_live_testnet,
            self.timeout_secs
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_TESTNET_TCP_PROBE_REQUEST_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestnetTcpProbePlan {
    pub profile: NetworkProfile,
    pub peer: SocketAddr,
    pub timeout_secs: u64,
}

impl TestnetTcpProbePlan {
    pub fn summary_line(&self) -> String {
        format!(
            "testnet_tcp_probe_plan network={} peer_port={} timeout_secs={} tcp_only=true retries=0 protocol_bytes=false",
            self.profile.name,
            self.peer.port(),
            self.timeout_secs
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_TESTNET_TCP_PROBE_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestnetHandshakeProbeRequest {
    pub network: String,
    pub peer: String,
    pub allow_live_testnet: bool,
    pub timeout_secs: u64,
}

impl TestnetHandshakeProbeRequest {
    pub fn new(
        network: impl Into<String>,
        peer: impl Into<String>,
        allow_live_testnet: bool,
        timeout_secs: u64,
    ) -> Self {
        Self {
            network: network.into(),
            peer: peer.into(),
            allow_live_testnet,
            timeout_secs,
        }
    }

    pub fn summary_line(&self) -> String {
        format!(
            "testnet_handshake_probe_request network={} peer_supplied={} allow_live_testnet={} timeout_secs={}",
            self.network,
            !self.peer.is_empty(),
            self.allow_live_testnet,
            self.timeout_secs
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_TESTNET_HANDSHAKE_PROBE_REQUEST_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestnetHandshakeProbePlan {
    pub profile: NetworkProfile,
    pub peer: SocketAddr,
    pub timeout_secs: u64,
    pub proposed_versions: Vec<u16>,
    pub request_frame: CardanoMuxFrameProtocolVector,
    pub max_response_bytes: usize,
}

impl TestnetHandshakeProbePlan {
    pub fn summary_line(&self) -> String {
        format!(
            "testnet_handshake_probe_plan network={} peer_port={} versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} timeout_secs={} dials=1 retries=0 write_frames=1 read_frames=1 max_response_bytes={} mainnet_allowed=false",
            self.profile.name,
            self.peer.port(),
            self.proposed_versions.len(),
            cardano_ntn_min_version(&self.proposed_versions),
            cardano_ntn_max_version(&self.proposed_versions),
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            cardano_ntn_leios_overlay_capable(&self.proposed_versions),
            self.timeout_secs,
            self.max_response_bytes
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_TESTNET_HANDSHAKE_PROBE_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestnetLiveReadiness {
    pub tcp_probe_available: bool,
    pub handshake_sketch_available: bool,
    pub offline_conformance_complete: bool,
    pub public_testnet_profiles: usize,
    pub live_command_available: bool,
    pub path_review_complete: bool,
    pub conformance_complete: bool,
    pub live_contact_allowed: bool,
    pub blockers: Vec<&'static str>,
}

impl TestnetLiveReadiness {
    pub fn action_items(&self) -> Vec<&'static str> {
        testnet_live_readiness_action_items(&self.blockers)
    }

    pub fn summary_line(&self) -> String {
        format!(
            "testnet_live_readiness tcp_probe={} handshake_sketch={} offline_conformance={} public_profiles={} live_contact={} blockers={}",
            self.tcp_probe_available,
            self.handshake_sketch_available,
            self.offline_conformance_complete,
            self.public_testnet_profiles,
            self.live_contact_allowed,
            self.blockers.len()
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_TESTNET_LIVE_READINESS_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

fn testnet_live_readiness_action_items(blockers: &[&'static str]) -> Vec<&'static str> {
    blockers
        .iter()
        .filter_map(|blocker| match *blocker {
            "live testnet protocol command is not implemented" => {
                Some("implement reviewed live testnet protocol command")
            }
            "network path opening requires explicit review" => {
                Some("complete network path opening review")
            }
            "full testnet conformance harness is not complete" => {
                Some("complete full testnet conformance harness")
            }
            _ => None,
        })
        .collect()
}

pub fn testnet_live_readiness() -> TestnetLiveReadiness {
    let (offline_conformance_complete, public_testnet_profiles) =
        match testnet_handshake_conformance_matrix(&CARDANO_NTN_SUPPORTED_VERSIONS) {
            Ok(matrix) => (matrix.offline_complete, matrix.public_testnet_profiles),
            Err(_) => (false, 0),
        };
    TestnetLiveReadiness {
        tcp_probe_available: true,
        handshake_sketch_available: true,
        offline_conformance_complete,
        public_testnet_profiles,
        live_command_available: true,
        path_review_complete: true,
        conformance_complete: false,
        live_contact_allowed: true,
        blockers: TESTNET_LIVE_READINESS_BLOCKERS.to_vec(),
    }
}

pub fn plan_testnet_tcp_probe(
    request: TestnetTcpProbeRequest,
    limits: TestnetContactLimits,
) -> Result<TestnetTcpProbePlan, NetworkError> {
    let (profile, peer, timeout_secs) = plan_testnet_probe_peer(
        &request.network,
        &request.peer,
        request.allow_live_testnet,
        request.timeout_secs,
        limits,
        "TCP probe",
    )?;

    Ok(TestnetTcpProbePlan {
        profile,
        peer,
        timeout_secs,
    })
}

pub fn plan_testnet_handshake_probe(
    request: TestnetHandshakeProbeRequest,
    limits: TestnetContactLimits,
    proposed_versions: &[u16],
) -> Result<TestnetHandshakeProbePlan, NetworkError> {
    let (profile, peer, timeout_secs) = plan_testnet_probe_peer(
        &request.network,
        &request.peer,
        request.allow_live_testnet,
        request.timeout_secs,
        limits,
        "handshake probe",
    )?;
    let conformance = cardano_ntn_handshake_conformance_report(profile, proposed_versions)?;
    if !conformance.offline_complete {
        return Err(NetworkError::ProtocolRequiresReview(vec![
            "offline testnet handshake conformance is incomplete".to_string(),
        ]));
    }
    let proposal = cardano_ntn_handshake_protocol_vector(profile, proposed_versions)?;
    let request_frame =
        cardano_mux_frame_protocol_vector(proposal.protocol_id, &proposal.encoded, false, 0)?;

    Ok(TestnetHandshakeProbePlan {
        profile,
        peer,
        timeout_secs,
        proposed_versions: proposed_versions.to_vec(),
        request_frame,
        max_response_bytes: MAX_TESTNET_HANDSHAKE_PROBE_RESPONSE_BYTES,
    })
}

fn plan_testnet_probe_peer(
    network: &str,
    peer: &str,
    allow_live_testnet: bool,
    timeout_secs: u64,
    limits: TestnetContactLimits,
    label: &'static str,
) -> Result<(NetworkProfile, SocketAddr, u64), NetworkError> {
    limits.validate()?;
    if !allow_live_testnet {
        return Err(NetworkError::TestnetContactBlocked(format!(
            "live testnet {label} requires --allow-live-testnet"
        )));
    }
    if timeout_secs == 0 {
        return Err(NetworkError::UnboundedTestnetLimit("probe_timeout_secs"));
    }
    if timeout_secs > limits.timeout_secs {
        return Err(NetworkError::TestnetLimitExceeded {
            limit: "timeout_secs",
            requested: timeout_secs,
            max: limits.timeout_secs,
        });
    }
    if timeout_secs > MAX_TESTNET_TCP_PROBE_TIMEOUT_SECS {
        return Err(NetworkError::TestnetLimitExceeded {
            limit: "probe_timeout_secs",
            requested: timeout_secs,
            max: MAX_TESTNET_TCP_PROBE_TIMEOUT_SECS,
        });
    }

    let profile = network_profile(network)
        .ok_or_else(|| NetworkError::UnknownNetwork(network.to_string()))?;
    if profile.name == "mainnet" {
        return Err(NetworkError::TestnetContactBlocked(
            "mainnet contact is blocked".to_string(),
        ));
    }
    if profile.kind != NetworkProfileKind::Public {
        return Err(NetworkError::TestnetContactBlocked(
            "live contact is limited to public testnet profiles".to_string(),
        ));
    }

    let peer = peer.parse::<SocketAddr>().map_err(|_| {
        NetworkError::TestnetContactBlocked(format!(
            "testnet {label} requires literal ip:port peer"
        ))
    })?;
    if peer.port() != profile.default_node_port {
        return Err(NetworkError::TestnetContactBlocked(format!(
            "testnet {label} port must match profile default {}",
            profile.default_node_port
        )));
    }
    if !is_public_probe_ip(peer.ip()) {
        return Err(NetworkError::TestnetContactBlocked(format!(
            "testnet {label} peer must be a public IP address"
        )));
    }

    Ok((profile, peer, timeout_secs))
}

fn is_public_probe_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(addr) => {
            !(addr.is_unspecified()
                || addr.is_loopback()
                || addr.is_private()
                || addr.is_link_local()
                || addr.is_broadcast()
                || addr.is_multicast())
        }
        IpAddr::V6(addr) => {
            let first = addr.segments()[0];
            let unique_local = (first & 0xfe00) == 0xfc00;
            let link_local = (first & 0xffc0) == 0xfe80;
            !(addr.is_unspecified()
                || addr.is_loopback()
                || addr.is_multicast()
                || unique_local
                || link_local)
        }
    }
}

pub fn plan_bounded_testnet_contact(
    request: TestnetContactRequest,
    limits: TestnetContactLimits,
) -> Result<TestnetContactPlan, NetworkError> {
    limits.validate()?;
    let profile = network_profile(&request.network)
        .ok_or_else(|| NetworkError::UnknownNetwork(request.network.clone()))?;
    if profile.name == "mainnet" {
        return Err(NetworkError::TestnetContactBlocked(
            "mainnet contact is blocked".to_string(),
        ));
    }
    if profile.kind != NetworkProfileKind::Public {
        return Err(NetworkError::TestnetContactBlocked(
            "live contact is limited to public testnet profiles".to_string(),
        ));
    }
    request.validate()?;
    if request.requested_blocks > limits.max_blocks {
        return Err(NetworkError::TestnetLimitExceeded {
            limit: "blocks",
            requested: request.requested_blocks,
            max: limits.max_blocks,
        });
    }
    if request.requested_slots > limits.max_slots {
        return Err(NetworkError::TestnetLimitExceeded {
            limit: "slots",
            requested: request.requested_slots,
            max: limits.max_slots,
        });
    }
    if request.requested_bytes > limits.max_bytes {
        return Err(NetworkError::TestnetLimitExceeded {
            limit: "bytes",
            requested: request.requested_bytes,
            max: limits.max_bytes,
        });
    }
    if request.requested_bytes > limits.temp_storage_bytes {
        return Err(NetworkError::TestnetLimitExceeded {
            limit: "temp_bytes",
            requested: request.requested_bytes,
            max: limits.temp_storage_bytes,
        });
    }
    Ok(TestnetContactPlan {
        profile,
        request,
        limits,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkPlan {
    pub network_magic: u32,
    pub listeners: Vec<ListenerPlan>,
    pub sharing: bool,
    pub intersect_tip: bool,
    pub paths_enabled: bool,
}

impl NetworkPlan {
    pub fn from_config(config: &NodeConfig) -> Self {
        Self {
            network_magic: config.network_magic,
            listeners: config.listener_plan(),
            sharing: false,
            intersect_tip: false,
            paths_enabled: config.safety.allow_paths,
        }
    }

    pub fn assert_safe_to_open(&self) -> Result<(), NetworkError> {
        let review = self.open_review();
        if !self.paths_enabled {
            return Err(NetworkError::PathsClosed);
        }
        if review.blocked() {
            return Err(NetworkError::ProtocolRequiresReview(review.blockers));
        }
        Ok(())
    }

    pub fn open_review(&self) -> NetworkOpenReview {
        let mut blockers = Vec::new();
        if !self.paths_enabled {
            blockers.push("path opening is disabled by safety config".to_string());
        }
        blockers.push("network protocol path opening requires explicit review".to_string());
        NetworkOpenReview {
            paths_enabled: self.paths_enabled,
            listeners: self.listeners.clone(),
            blockers,
        }
    }

    pub fn summary_line(&self) -> String {
        format!(
            "network_plan magic={} listeners={} sharing={} intersect_tip={} paths_enabled={}",
            self.network_magic,
            self.listeners.len(),
            self.sharing,
            self.intersect_tip,
            self.paths_enabled
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(NETWORK_PLAN_EVENT, EventPayload::Text(self.summary_line()))
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkOpenReview {
    pub paths_enabled: bool,
    pub listeners: Vec<ListenerPlan>,
    pub blockers: Vec<String>,
}

impl NetworkOpenReview {
    pub fn blocked(&self) -> bool {
        !self.blockers.is_empty()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "network_open_review paths_enabled={} listeners={} blockers={} blocked={}",
            self.paths_enabled,
            self.listeners.len(),
            self.blockers.len(),
            self.blocked()
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_OPEN_REVIEW_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        let mut events = vec![self.to_event()];
        events.extend(self.events());
        events
    }

    pub fn events(&self) -> Vec<Event> {
        if !self.blocked() {
            return Vec::new();
        }
        vec![Event::new(
            NETWORK_OPEN_BLOCKED_EVENT,
            EventPayload::Text(format!(
                "paths_enabled={} listeners={} blockers={}",
                self.paths_enabled,
                self.listeners.len(),
                self.blockers.len()
            )),
        )]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionRole {
    Outer,
    Inner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakeHello {
    pub network_magic: u32,
    pub version: u16,
    pub role: ConnectionRole,
    pub peer: PeerAddress,
}

impl HandshakeHello {
    pub fn new(network_magic: u32, role: ConnectionRole, peer: PeerAddress) -> Self {
        Self {
            network_magic,
            version: NETWORK_HANDSHAKE_VERSION,
            role,
            peer,
        }
    }

    pub fn summary_line(&self) -> String {
        let role = match self.role {
            ConnectionRole::Outer => "outer",
            ConnectionRole::Inner => "inner",
        };
        format!(
            "handshake_hello magic={} version={} role={}",
            self.network_magic, self.version, role
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_HELLO_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakePlan {
    pub local: HandshakeHello,
    pub remote: HandshakeHello,
    pub share_peer: bool,
    pub intersect_tip: bool,
}

impl HandshakePlan {
    pub fn validate(&self) -> Result<(), NetworkError> {
        if self.local.network_magic != self.remote.network_magic {
            return Err(NetworkError::NetworkMismatch {
                local: self.local.network_magic,
                remote: self.remote.network_magic,
            });
        }
        if self.local.version != NETWORK_HANDSHAKE_VERSION
            || self.remote.version != NETWORK_HANDSHAKE_VERSION
        {
            return Err(NetworkError::UnsupportedVersion {
                local: self.local.version,
                remote: self.remote.version,
            });
        }
        Ok(())
    }

    pub fn summary_line(&self) -> String {
        let local_role = match self.local.role {
            ConnectionRole::Outer => "outer",
            ConnectionRole::Inner => "inner",
        };
        let remote_role = match self.remote.role {
            ConnectionRole::Outer => "outer",
            ConnectionRole::Inner => "inner",
        };
        format!(
            "handshake_plan local_magic={} remote_magic={} local_version={} remote_version={} local_role={} remote_role={} share_peer={} intersect_tip={}",
            self.local.network_magic,
            self.remote.network_magic,
            self.local.version,
            self.remote.version,
            local_role,
            remote_role,
            self.share_peer,
            self.intersect_tip
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalHandshakeSketch {
    pub profile: NetworkProfile,
    pub versions: Vec<u16>,
    pub encoded: Vec<u8>,
    pub production_compatible: bool,
}

impl LocalHandshakeSketch {
    pub fn summary_line(&self) -> String {
        format!(
            "handshake_sketch network={} versions={} encoded_bytes={} production_compatible={}",
            self.profile.name,
            self.versions.len(),
            self.encoded.len(),
            self.production_compatible
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_SKETCH_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardanoNtNDiffusionMode {
    InitiatorOnly,
    InitiatorAndResponder,
}

impl CardanoNtNDiffusionMode {
    fn wire_bool(self) -> bool {
        match self {
            Self::InitiatorOnly => true,
            Self::InitiatorAndResponder => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardanoNtNVersionDataShape {
    NtN7To10,
    NtN11To12,
    NtN13AndUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardanoNtNVersionDataPlan {
    pub version: u16,
    pub network_magic: u32,
    pub diffusion_mode: CardanoNtNDiffusionMode,
    pub peer_sharing_mode: u8,
    pub query: bool,
    pub shape: CardanoNtNVersionDataShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoNtNHandshakeProtocolVector {
    pub profile: NetworkProfile,
    pub versions: Vec<u16>,
    pub encoded: Vec<u8>,
    pub protocol_id: u16,
    pub message_type: u8,
    pub production_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoNtNHandshakeAcceptProtocolVector {
    pub profile: NetworkProfile,
    pub version: u16,
    pub encoded: Vec<u8>,
    pub protocol_id: u16,
    pub message_type: u8,
    pub production_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoNtNHandshakeRefusalProtocolVector {
    pub supported_versions: Vec<u16>,
    pub encoded: Vec<u8>,
    pub protocol_id: u16,
    pub message_type: u8,
    pub production_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoMuxFrameProtocolVector {
    pub protocol_id: u16,
    pub wire_protocol_id: u16,
    pub timestamp: u32,
    pub payload_length: u16,
    pub is_response: bool,
    pub encoded: Vec<u8>,
    pub production_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoMuxFrame {
    pub protocol_id: u16,
    pub wire_protocol_id: u16,
    pub timestamp: u32,
    pub payload_length: u16,
    pub is_response: bool,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoMuxFrameStreamSummary {
    pub frame_count: usize,
    pub request_frames: usize,
    pub response_frames: usize,
    pub total_payload_bytes: usize,
    pub total_frame_bytes: usize,
    pub protocol_count: usize,
    pub production_ready: bool,
    pub live_integrated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCardanoNtNVersionData {
    pub version: u16,
    pub network_magic: u32,
    pub diffusion_mode: CardanoNtNDiffusionMode,
    pub peer_sharing_mode: u8,
    pub peer_sharing: bool,
    pub query: bool,
    pub shape: CardanoNtNVersionDataShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardanoHandshakeResponseKind {
    AcceptVersion {
        version: u16,
        version_data: ParsedCardanoNtNVersionData,
    },
    RefuseVersionMismatch {
        supported_versions: Vec<u16>,
    },
    RefuseDecodeError {
        version: u16,
        message: String,
    },
    RefuseRefused {
        version: u16,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeResponse {
    pub kind: CardanoHandshakeResponseKind,
    pub production_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardanoHandshakeRefusalReason {
    VersionMismatch,
    DecodeError,
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardanoHandshakeNegotiationOutcome {
    Accepted {
        version: u16,
        network_magic: u32,
        diffusion_mode: CardanoNtNDiffusionMode,
        peer_sharing: bool,
        query: bool,
    },
    Refused {
        reason: CardanoHandshakeRefusalReason,
        version: Option<u16>,
        supported_versions: Vec<u16>,
        message: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeNegotiationReport {
    pub profile: NetworkProfile,
    pub proposed_versions: Vec<u16>,
    pub outcome: CardanoHandshakeNegotiationOutcome,
    pub production_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardanoHandshakeAgency {
    Client,
    Server,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardanoHandshakeState {
    Propose,
    Confirm,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardanoHandshakeMessageType {
    ProposeVersions,
    AcceptVersion,
    Refuse,
    QueryReply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardanoHandshakeTransition {
    pub message: CardanoHandshakeMessageType,
    pub next_state: CardanoHandshakeState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeStateEntry {
    pub state: CardanoHandshakeState,
    pub agency: CardanoHandshakeAgency,
    pub timeout_secs: Option<u64>,
    pub transitions: Vec<CardanoHandshakeTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeStateMachinePlan {
    pub entries: Vec<CardanoHandshakeStateEntry>,
    pub production_ready: bool,
    pub live_integrated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeHarnessRun {
    pub profile: NetworkProfile,
    pub proposed_versions: Vec<u16>,
    pub states: Vec<CardanoHandshakeState>,
    pub messages: Vec<CardanoHandshakeMessageType>,
    pub proposal_frame: CardanoMuxFrameProtocolVector,
    pub response_frame: CardanoMuxFrame,
    pub response: CardanoHandshakeResponse,
    pub negotiation: CardanoHandshakeNegotiationReport,
    pub production_ready: bool,
    pub live_integrated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeTranscriptProtocolVector {
    pub profile: NetworkProfile,
    pub proposed_versions: Vec<u16>,
    pub accepted_version: u16,
    pub request_frame: CardanoMuxFrameProtocolVector,
    pub response_frame: CardanoMuxFrameProtocolVector,
    pub harness: CardanoHandshakeHarnessRun,
    pub frame_count: usize,
    pub total_bytes: usize,
    pub production_ready: bool,
    pub live_integrated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeTranscriptReplay {
    pub profile: NetworkProfile,
    pub proposed_versions: Vec<u16>,
    pub frames: Vec<CardanoMuxFrame>,
    pub request_frames: usize,
    pub response_frames: usize,
    pub total_bytes: usize,
    pub accepted_version: u16,
    pub final_state: CardanoHandshakeState,
    pub production_ready: bool,
    pub live_integrated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeRefusalTranscriptReplay {
    pub profile: NetworkProfile,
    pub proposed_versions: Vec<u16>,
    pub frames: Vec<CardanoMuxFrame>,
    pub request_frames: usize,
    pub response_frames: usize,
    pub total_bytes: usize,
    pub final_state: CardanoHandshakeState,
    pub refusal_reason: CardanoHandshakeRefusalReason,
    pub supported_versions: Vec<u16>,
    pub production_ready: bool,
    pub live_integrated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeTimeoutProtocolVector {
    pub state: CardanoHandshakeState,
    pub agency: CardanoHandshakeAgency,
    pub timeout_secs: u64,
    pub elapsed_secs: u64,
    pub timed_out: bool,
    pub production_ready: bool,
    pub live_integrated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardanoHandshakeErrorProtocolVectorKind {
    WrongProtocolId,
    NonResponseFrame,
    MalformedCbor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeErrorProtocolVectorCase {
    pub kind: CardanoHandshakeErrorProtocolVectorKind,
    pub expected_error: NetworkError,
    pub observed_error: NetworkError,
    pub matched: bool,
}

impl CardanoHandshakeErrorProtocolVectorCase {
    pub fn summary_line(&self) -> String {
        let kind = match self.kind {
            CardanoHandshakeErrorProtocolVectorKind::WrongProtocolId => "wrong_protocol_id",
            CardanoHandshakeErrorProtocolVectorKind::NonResponseFrame => "non_response_frame",
            CardanoHandshakeErrorProtocolVectorKind::MalformedCbor => "malformed_cbor",
        };
        format!(
            "handshake_error_protocol_vector_case kind={} matched={}",
            kind, self.matched
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_ERROR_PROTOCOL_VECTOR_CASE_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeErrorProtocolVectorReport {
    pub profile: NetworkProfile,
    pub proposed_versions: Vec<u16>,
    pub cases: Vec<CardanoHandshakeErrorProtocolVectorCase>,
    pub production_ready: bool,
    pub live_integrated: bool,
}

impl CardanoHandshakeErrorProtocolVectorReport {
    pub fn matched_cases(&self) -> usize {
        self.cases.iter().filter(|case| case.matched).count()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "handshake_error_protocol_vectors network={} versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} cases={} matched={} live_integrated={}",
            self.profile.name,
            self.proposed_versions.len(),
            cardano_ntn_min_version(&self.proposed_versions),
            cardano_ntn_max_version(&self.proposed_versions),
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            cardano_ntn_leios_overlay_capable(&self.proposed_versions),
            self.cases.len(),
            self.matched_cases(),
            self.live_integrated
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_ERROR_PROTOCOL_VECTORS_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoHandshakeConformanceReport {
    pub profile: NetworkProfile,
    pub proposed_versions: Vec<u16>,
    pub offline_checks: usize,
    pub passed_checks: usize,
    pub offline_complete: bool,
    pub live_ready: bool,
    pub blockers: Vec<&'static str>,
    pub production_ready: bool,
    pub live_integrated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestnetHandshakeConformanceMatrix {
    pub reports: Vec<CardanoHandshakeConformanceReport>,
    pub public_testnet_profiles: usize,
    pub passed_profiles: usize,
    pub offline_complete: bool,
    pub live_ready: bool,
    pub blockers: Vec<&'static str>,
    pub production_ready: bool,
    pub live_integrated: bool,
}

impl CardanoHandshakeNegotiationReport {
    pub fn summary_line(&self) -> String {
        let (
            outcome,
            accepted_version,
            leios_overlay_negotiated,
            refused_supported_versions,
            refused_supported_min_version,
            refused_supported_max_version,
            refused_supported_leios_overlay_capable,
        ) = match &self.outcome {
            CardanoHandshakeNegotiationOutcome::Accepted { version, .. } => (
                "accepted",
                version.to_string(),
                cardano_ntn_version_leios_overlay_capable(*version).to_string(),
                0,
                0,
                0,
                "none".to_string(),
            ),
            CardanoHandshakeNegotiationOutcome::Refused {
                supported_versions, ..
            } => (
                "refused",
                "none".to_string(),
                "none".to_string(),
                supported_versions.len(),
                cardano_ntn_min_version(supported_versions),
                cardano_ntn_max_version(supported_versions),
                cardano_ntn_leios_overlay_capable(supported_versions).to_string(),
            ),
        };
        format!(
            "handshake_negotiation network={} versions={} outcome={} accepted_version={} leios_overlay_min_version={} leios_overlay_negotiated={} refused_supported_versions={} refused_supported_min_version={} refused_supported_max_version={} refused_supported_leios_overlay_capable={} production_ready={}",
            self.profile.name,
            self.proposed_versions.len(),
            outcome,
            accepted_version,
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            leios_overlay_negotiated,
            refused_supported_versions,
            refused_supported_min_version,
            refused_supported_max_version,
            refused_supported_leios_overlay_capable,
            self.production_ready
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_NEGOTIATION_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoHandshakeHarnessRun {
    pub fn summary_line(&self) -> String {
        let final_state = self
            .states
            .last()
            .map(|state| match state {
                CardanoHandshakeState::Propose => "propose",
                CardanoHandshakeState::Confirm => "confirm",
                CardanoHandshakeState::Done => "done",
            })
            .unwrap_or("unknown");
        let (outcome, leios_overlay_negotiated) = match self.negotiation.outcome {
            CardanoHandshakeNegotiationOutcome::Accepted { version, .. } => (
                "accepted",
                cardano_ntn_version_leios_overlay_capable(version).to_string(),
            ),
            CardanoHandshakeNegotiationOutcome::Refused { .. } => ("refused", "none".to_string()),
        };
        format!(
            "handshake_harness network={} versions={} states={} messages={} final_state={} outcome={} leios_overlay_min_version={} leios_overlay_negotiated={} production_ready={} live_integrated={}",
            self.profile.name,
            self.proposed_versions.len(),
            self.states.len(),
            self.messages.len(),
            final_state,
            outcome,
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            leios_overlay_negotiated,
            self.production_ready,
            self.live_integrated
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_HARNESS_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoHandshakeTranscriptProtocolVector {
    pub fn summary_line(&self) -> String {
        format!(
            "handshake_transcript_protocol_vector network={} versions={} frames={} total_bytes={} accepted_version={} leios_overlay_min_version={} leios_overlay_negotiated={} production_ready={} live_integrated={}",
            self.profile.name,
            self.proposed_versions.len(),
            self.frame_count,
            self.total_bytes,
            self.accepted_version,
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            cardano_ntn_version_leios_overlay_capable(self.accepted_version),
            self.production_ready,
            self.live_integrated
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_TRANSCRIPT_PROTOCOL_VECTOR_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoHandshakeTranscriptReplay {
    pub fn summary_line(&self) -> String {
        let final_state = match self.final_state {
            CardanoHandshakeState::Propose => "propose",
            CardanoHandshakeState::Confirm => "confirm",
            CardanoHandshakeState::Done => "done",
        };
        format!(
            "handshake_transcript_replay network={} versions={} frames={} request_frames={} response_frames={} total_bytes={} accepted_version={} leios_overlay_min_version={} leios_overlay_negotiated={} final_state={} production_ready={} live_integrated={}",
            self.profile.name,
            self.proposed_versions.len(),
            self.frames.len(),
            self.request_frames,
            self.response_frames,
            self.total_bytes,
            self.accepted_version,
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            cardano_ntn_version_leios_overlay_capable(self.accepted_version),
            final_state,
            self.production_ready,
            self.live_integrated
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_TRANSCRIPT_REPLAY_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoHandshakeRefusalTranscriptReplay {
    pub fn summary_line(&self) -> String {
        let final_state = match self.final_state {
            CardanoHandshakeState::Propose => "propose",
            CardanoHandshakeState::Confirm => "confirm",
            CardanoHandshakeState::Done => "done",
        };
        let refusal_reason = match self.refusal_reason {
            CardanoHandshakeRefusalReason::VersionMismatch => "version_mismatch",
            CardanoHandshakeRefusalReason::DecodeError => "decode_error",
            CardanoHandshakeRefusalReason::Refused => "refused",
        };
        format!(
            "handshake_refusal_transcript_replay network={} versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} frames={} request_frames={} response_frames={} total_bytes={} final_state={} refusal_reason={} supported_versions={} supported_min_version={} supported_max_version={} supported_leios_overlay_capable={} production_ready={} live_integrated={}",
            self.profile.name,
            self.proposed_versions.len(),
            cardano_ntn_min_version(&self.proposed_versions),
            cardano_ntn_max_version(&self.proposed_versions),
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            cardano_ntn_leios_overlay_capable(&self.proposed_versions),
            self.frames.len(),
            self.request_frames,
            self.response_frames,
            self.total_bytes,
            final_state,
            refusal_reason,
            self.supported_versions.len(),
            cardano_ntn_min_version(&self.supported_versions),
            cardano_ntn_max_version(&self.supported_versions),
            cardano_ntn_leios_overlay_capable(&self.supported_versions),
            self.production_ready,
            self.live_integrated
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_REFUSAL_TRANSCRIPT_REPLAY_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoHandshakeTimeoutProtocolVector {
    pub fn summary_line(&self) -> String {
        let state = match self.state {
            CardanoHandshakeState::Propose => "propose",
            CardanoHandshakeState::Confirm => "confirm",
            CardanoHandshakeState::Done => "done",
        };
        let agency = match self.agency {
            CardanoHandshakeAgency::Client => "client",
            CardanoHandshakeAgency::Server => "server",
            CardanoHandshakeAgency::None => "none",
        };
        format!(
            "handshake_timeout_protocol_vector state={} agency={} timeout_secs={} elapsed_secs={} timed_out={} production_ready={} live_integrated={}",
            state,
            agency,
            self.timeout_secs,
            self.elapsed_secs,
            self.timed_out,
            self.production_ready,
            self.live_integrated
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_TIMEOUT_PROTOCOL_VECTOR_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoNtNVersionDataPlan {
    pub fn summary_line(&self) -> String {
        let shape = match self.shape {
            CardanoNtNVersionDataShape::NtN7To10 => "ntn7_to_10",
            CardanoNtNVersionDataShape::NtN11To12 => "ntn11_to_12",
            CardanoNtNVersionDataShape::NtN13AndUp => "ntn13_and_up",
        };
        let diffusion = match self.diffusion_mode {
            CardanoNtNDiffusionMode::InitiatorOnly => "initiator_only",
            CardanoNtNDiffusionMode::InitiatorAndResponder => "initiator_and_responder",
        };
        format!(
            "handshake_version_data_plan version={} network_magic={} shape={} diffusion={} peer_sharing_mode={} query={}",
            self.version,
            self.network_magic,
            shape,
            diffusion,
            self.peer_sharing_mode,
            self.query
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_VERSION_DATA_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoNtNHandshakeProtocolVector {
    pub fn summary_line(&self) -> String {
        format!(
            "handshake_proposal_protocol_vector network={} versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} encoded_bytes={} protocol_id={} message_type={} production_ready={}",
            self.profile.name,
            self.versions.len(),
            cardano_ntn_min_version(&self.versions),
            cardano_ntn_max_version(&self.versions),
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            cardano_ntn_leios_overlay_capable(&self.versions),
            self.encoded.len(),
            self.protocol_id,
            self.message_type,
            self.production_ready
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_PROPOSAL_PROTOCOL_VECTOR_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoNtNHandshakeAcceptProtocolVector {
    pub fn summary_line(&self) -> String {
        format!(
            "handshake_accept_protocol_vector network={} version={} encoded_bytes={} protocol_id={} message_type={} production_ready={}",
            self.profile.name,
            self.version,
            self.encoded.len(),
            self.protocol_id,
            self.message_type,
            self.production_ready
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_ACCEPT_PROTOCOL_VECTOR_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoNtNHandshakeRefusalProtocolVector {
    pub fn summary_line(&self) -> String {
        format!(
            "handshake_refusal_protocol_vector supported_versions={} supported_min_version={} supported_max_version={} leios_overlay_min_version={} supported_leios_overlay_capable={} encoded_bytes={} protocol_id={} message_type={} production_ready={}",
            self.supported_versions.len(),
            cardano_ntn_min_version(&self.supported_versions),
            cardano_ntn_max_version(&self.supported_versions),
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            cardano_ntn_leios_overlay_capable(&self.supported_versions),
            self.encoded.len(),
            self.protocol_id,
            self.message_type,
            self.production_ready
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_REFUSAL_PROTOCOL_VECTOR_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoMuxFrameProtocolVector {
    pub fn summary_line(&self) -> String {
        format!(
            "mux_frame_protocol_vector protocol_id={} wire_protocol_id={} payload_bytes={} encoded_bytes={} response={} production_ready={}",
            self.protocol_id,
            self.wire_protocol_id,
            self.payload_length,
            self.encoded.len(),
            self.is_response,
            self.production_ready
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_MUX_FRAME_PROTOCOL_VECTOR_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoMuxFrame {
    pub fn summary_line(&self) -> String {
        format!(
            "mux_frame protocol_id={} wire_protocol_id={} payload_bytes={} response={} timestamp={}",
            self.protocol_id,
            self.wire_protocol_id,
            self.payload_length,
            self.is_response,
            self.timestamp
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_MUX_FRAME_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoMuxFrameStreamSummary {
    pub fn summary_line(&self) -> String {
        format!(
            "mux_frame_stream frames={} request_frames={} response_frames={} payload_bytes={} frame_bytes={} protocols={} production_ready={} live_integrated={}",
            self.frame_count,
            self.request_frames,
            self.response_frames,
            self.total_payload_bytes,
            self.total_frame_bytes,
            self.protocol_count,
            self.production_ready,
            self.live_integrated
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_MUX_FRAME_STREAM_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoHandshakeResponse {
    pub fn summary_line(&self) -> String {
        let (
            kind,
            version,
            leios_overlay_negotiated,
            supported_versions,
            supported_min_version,
            supported_max_version,
            supported_leios_overlay_capable,
            message_present,
        ) = match &self.kind {
            CardanoHandshakeResponseKind::AcceptVersion { version, .. } => (
                "accept_version",
                Some(*version),
                cardano_ntn_version_leios_overlay_capable(*version).to_string(),
                0,
                0,
                0,
                "none".to_string(),
                false,
            ),
            CardanoHandshakeResponseKind::RefuseVersionMismatch { supported_versions } => (
                "refuse_version_mismatch",
                None,
                "none".to_string(),
                supported_versions.len(),
                cardano_ntn_min_version(supported_versions),
                cardano_ntn_max_version(supported_versions),
                cardano_ntn_leios_overlay_capable(supported_versions).to_string(),
                false,
            ),
            CardanoHandshakeResponseKind::RefuseDecodeError { version, message } => (
                "refuse_decode_error",
                Some(*version),
                "none".to_string(),
                0,
                0,
                0,
                "none".to_string(),
                !message.is_empty(),
            ),
            CardanoHandshakeResponseKind::RefuseRefused { version, message } => (
                "refuse_refused",
                Some(*version),
                "none".to_string(),
                0,
                0,
                0,
                "none".to_string(),
                !message.is_empty(),
            ),
        };
        let version = version
            .map(|version| version.to_string())
            .unwrap_or_else(|| "none".to_string());
        format!(
            "handshake_response kind={} version={} leios_overlay_min_version={} leios_overlay_negotiated={} supported_versions={} supported_min_version={} supported_max_version={} supported_leios_overlay_capable={} message_present={} production_ready={}",
            kind,
            version,
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            leios_overlay_negotiated,
            supported_versions,
            supported_min_version,
            supported_max_version,
            supported_leios_overlay_capable,
            message_present,
            self.production_ready
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_RESPONSE_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl ParsedCardanoNtNVersionData {
    pub fn summary_line(&self) -> String {
        let shape = match self.shape {
            CardanoNtNVersionDataShape::NtN7To10 => "ntn7_to_10",
            CardanoNtNVersionDataShape::NtN11To12 => "ntn11_to_12",
            CardanoNtNVersionDataShape::NtN13AndUp => "ntn13_and_up",
        };
        let diffusion = match self.diffusion_mode {
            CardanoNtNDiffusionMode::InitiatorOnly => "initiator_only",
            CardanoNtNDiffusionMode::InitiatorAndResponder => "initiator_and_responder",
        };
        format!(
            "handshake_version_data version={} network_magic={} shape={} diffusion={} peer_sharing_mode={} peer_sharing={} query={}",
            self.version,
            self.network_magic,
            shape,
            diffusion,
            self.peer_sharing_mode,
            self.peer_sharing,
            self.query
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_VERSION_DATA_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl CardanoHandshakeConformanceReport {
    pub fn action_items(&self) -> Vec<&'static str> {
        testnet_conformance_action_items(&self.blockers)
    }

    pub fn summary_line(&self) -> String {
        format!(
            "handshake_conformance network={} versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} passed={}/{} offline_complete={} live_ready={} blockers={}",
            self.profile.name,
            self.proposed_versions.len(),
            cardano_ntn_min_version(&self.proposed_versions),
            cardano_ntn_max_version(&self.proposed_versions),
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            cardano_ntn_leios_overlay_capable(&self.proposed_versions),
            self.passed_checks,
            self.offline_checks,
            self.offline_complete,
            self.live_ready,
            self.blockers.len()
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_CONFORMANCE_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl TestnetHandshakeConformanceMatrix {
    pub fn action_items(&self) -> Vec<&'static str> {
        testnet_conformance_action_items(&self.blockers)
    }

    pub fn summary_line(&self) -> String {
        format!(
            "handshake_conformance_matrix public_profiles={} passed_profiles={} offline_complete={} live_ready={} blockers={}",
            self.public_testnet_profiles,
            self.passed_profiles,
            self.offline_complete,
            self.live_ready,
            self.blockers.len()
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_CONFORMANCE_MATRIX_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        let mut events = vec![self.to_event()];
        for report in &self.reports {
            events.extend(report.event_batch());
        }
        events
    }
}

fn testnet_conformance_action_items(blockers: &[&'static str]) -> Vec<&'static str> {
    blockers
        .iter()
        .filter_map(|blocker| match *blocker {
            "live mux loop is not integrated" => Some("integrate reviewed live mux loop"),
            "network path review is incomplete" => Some("complete network path review"),
            "full protocol conformance is incomplete" => {
                Some("complete full protocol conformance harness")
            }
            _ => None,
        })
        .collect()
}

impl CardanoHandshakeStateMachinePlan {
    pub fn transition_count(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| entry.transitions.len())
            .sum()
    }

    pub fn timeout_state_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.timeout_secs.is_some())
            .count()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "handshake_state_machine states={} transitions={} timeout_states={} live_integrated={}",
            self.entries.len(),
            self.transition_count(),
            self.timeout_state_count(),
            self.live_integrated
        )
    }

    pub fn to_event(&self) -> Event {
        Event::new(
            NETWORK_HANDSHAKE_STATE_MACHINE_EVENT,
            EventPayload::Text(self.summary_line()),
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }

    pub fn next_state(
        &self,
        state: CardanoHandshakeState,
        message: CardanoHandshakeMessageType,
    ) -> Result<CardanoHandshakeState, NetworkError> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.state == state)
            .ok_or(NetworkError::InvalidHandshakeProposal(
                "unknown handshake state",
            ))?;
        entry
            .transitions
            .iter()
            .find(|transition| transition.message == message)
            .map(|transition| transition.next_state)
            .ok_or(NetworkError::InvalidHandshakeProposal(
                "invalid handshake state transition",
            ))
    }
}

pub fn local_handshake_sketch(
    profile: NetworkProfile,
    versions: &[u16],
) -> Result<LocalHandshakeSketch, NetworkError> {
    validate_handshake_versions(versions)?;
    let mut encoded = Vec::with_capacity(6 + 1 + (versions.len() * 2) + 4 + 1);
    encoded.extend_from_slice(b"ACRHS1");
    encoded.push(versions.len() as u8);
    for version in versions {
        encoded.extend_from_slice(&version.to_be_bytes());
    }
    encoded.extend_from_slice(&profile.network_magic.to_be_bytes());
    encoded.push(match profile.kind {
        NetworkProfileKind::Public => 1,
        NetworkProfileKind::Local => 0,
    });
    if encoded.len() > MAX_LOCAL_HANDSHAKE_SKETCH_BYTES {
        return Err(NetworkError::InvalidHandshakeProposal(
            "local handshake sketch exceeds byte cap",
        ));
    }
    Ok(LocalHandshakeSketch {
        profile,
        versions: versions.to_vec(),
        encoded,
        production_compatible: false,
    })
}

fn validate_handshake_versions(versions: &[u16]) -> Result<(), NetworkError> {
    if versions.is_empty() {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake versions must be non-empty",
        ));
    }
    if versions.len() > 8 {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake version table exceeds local cap",
        ));
    }
    if !versions.contains(&NETWORK_HANDSHAKE_VERSION) {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake version table must include local version",
        ));
    }
    let mut previous = None;
    for version in versions {
        if *version == 0 {
            return Err(NetworkError::InvalidHandshakeProposal(
                "handshake versions must be non-zero",
            ));
        }
        if previous.is_some_and(|previous| previous >= *version) {
            return Err(NetworkError::InvalidHandshakeProposal(
                "handshake versions must be strictly ascending",
            ));
        }
        previous = Some(*version);
    }
    Ok(())
}

pub fn cardano_ntn_handshake_protocol_vector(
    profile: NetworkProfile,
    versions: &[u16],
) -> Result<CardanoNtNHandshakeProtocolVector, NetworkError> {
    validate_cardano_ntn_versions(versions)?;
    let mut encoded = Vec::new();
    push_cbor_array_len(&mut encoded, 2)?;
    push_cbor_uint(&mut encoded, 0)?;
    push_cbor_map_len(&mut encoded, versions.len())?;
    for version in versions {
        push_cbor_uint(&mut encoded, u64::from(*version))?;
        encode_cardano_ntn_version_data(
            &mut encoded,
            cardano_ntn_version_data_plan(
                profile,
                *version,
                CardanoNtNDiffusionMode::InitiatorAndResponder,
                false,
                false,
            )?,
        )?;
    }
    if encoded.len() > MAX_LOCAL_HANDSHAKE_SKETCH_BYTES {
        return Err(NetworkError::InvalidHandshakeProposal(
            "Cardano NtN handshake protocol vector exceeds byte cap",
        ));
    }
    Ok(CardanoNtNHandshakeProtocolVector {
        profile,
        versions: versions.to_vec(),
        encoded,
        protocol_id: CARDANO_HANDSHAKE_PROTOCOL_ID,
        message_type: 0,
        production_ready: false,
    })
}

pub fn cardano_ntn_handshake_accept_protocol_vector(
    profile: NetworkProfile,
    version: u16,
) -> Result<CardanoNtNHandshakeAcceptProtocolVector, NetworkError> {
    let mut encoded = Vec::new();
    push_cbor_array_len(&mut encoded, 3)?;
    push_cbor_uint(&mut encoded, 1)?;
    push_cbor_uint(&mut encoded, u64::from(version))?;
    encode_cardano_ntn_version_data(
        &mut encoded,
        cardano_ntn_version_data_plan(
            profile,
            version,
            CardanoNtNDiffusionMode::InitiatorAndResponder,
            false,
            false,
        )?,
    )?;
    if encoded.len() > MAX_LOCAL_HANDSHAKE_SKETCH_BYTES {
        return Err(NetworkError::InvalidHandshakeProposal(
            "Cardano NtN handshake accept protocol vector exceeds byte cap",
        ));
    }
    Ok(CardanoNtNHandshakeAcceptProtocolVector {
        profile,
        version,
        encoded,
        protocol_id: CARDANO_HANDSHAKE_PROTOCOL_ID,
        message_type: 1,
        production_ready: false,
    })
}

pub fn cardano_ntn_handshake_version_mismatch_refusal_protocol_vector(
    supported_versions: &[u16],
) -> Result<CardanoNtNHandshakeRefusalProtocolVector, NetworkError> {
    validate_cardano_ntn_versions(supported_versions)?;
    let mut encoded = Vec::new();
    push_cbor_array_len(&mut encoded, 2)?;
    push_cbor_uint(&mut encoded, 2)?;
    push_cbor_array_len(&mut encoded, 2)?;
    push_cbor_uint(&mut encoded, 0)?;
    push_cbor_array_len(&mut encoded, supported_versions.len())?;
    for version in supported_versions {
        push_cbor_uint(&mut encoded, u64::from(*version))?;
    }
    if encoded.len() > MAX_LOCAL_HANDSHAKE_SKETCH_BYTES {
        return Err(NetworkError::InvalidHandshakeProposal(
            "Cardano NtN handshake refusal protocol vector exceeds byte cap",
        ));
    }
    Ok(CardanoNtNHandshakeRefusalProtocolVector {
        supported_versions: supported_versions.to_vec(),
        encoded,
        protocol_id: CARDANO_HANDSHAKE_PROTOCOL_ID,
        message_type: 2,
        production_ready: false,
    })
}

pub fn cardano_ntn_version_data_plan(
    profile: NetworkProfile,
    version: u16,
    diffusion_mode: CardanoNtNDiffusionMode,
    peer_sharing: bool,
    query: bool,
) -> Result<CardanoNtNVersionDataPlan, NetworkError> {
    let shape = match version {
        7..=10 => CardanoNtNVersionDataShape::NtN7To10,
        11..=12 => CardanoNtNVersionDataShape::NtN11To12,
        13..=15 => CardanoNtNVersionDataShape::NtN13AndUp,
        _ => {
            return Err(NetworkError::InvalidHandshakeProposal(
                "unsupported Cardano NtN handshake version",
            ));
        }
    };
    let peer_sharing_mode = match shape {
        CardanoNtNVersionDataShape::NtN7To10 => 0,
        CardanoNtNVersionDataShape::NtN11To12 => {
            if peer_sharing {
                2
            } else {
                0
            }
        }
        CardanoNtNVersionDataShape::NtN13AndUp => {
            if peer_sharing {
                1
            } else {
                0
            }
        }
    };
    Ok(CardanoNtNVersionDataPlan {
        version,
        network_magic: profile.network_magic,
        diffusion_mode,
        peer_sharing_mode,
        query,
        shape,
    })
}

fn validate_cardano_ntn_versions(versions: &[u16]) -> Result<(), NetworkError> {
    if versions.is_empty() {
        return Err(NetworkError::InvalidHandshakeProposal(
            "Cardano NtN versions must be non-empty",
        ));
    }
    let mut previous = None;
    for version in versions {
        if !CARDANO_NTN_SUPPORTED_VERSIONS.contains(version) {
            return Err(NetworkError::InvalidHandshakeProposal(
                "unsupported Cardano NtN handshake version",
            ));
        }
        if previous.is_some_and(|previous| previous >= *version) {
            return Err(NetworkError::InvalidHandshakeProposal(
                "Cardano NtN versions must be strictly ascending",
            ));
        }
        previous = Some(*version);
    }
    Ok(())
}

fn cardano_ntn_min_version(versions: &[u16]) -> u16 {
    versions.iter().copied().min().unwrap_or(0)
}

fn cardano_ntn_max_version(versions: &[u16]) -> u16 {
    versions.iter().copied().max().unwrap_or(0)
}

pub fn cardano_ntn_leios_overlay_capable(versions: &[u16]) -> bool {
    versions
        .iter()
        .any(|version| cardano_ntn_version_leios_overlay_capable(*version))
}

pub fn cardano_ntn_version_leios_overlay_capable(version: u16) -> bool {
    version >= CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION
}

fn encode_cardano_ntn_version_data(
    out: &mut Vec<u8>,
    plan: CardanoNtNVersionDataPlan,
) -> Result<(), NetworkError> {
    match plan.shape {
        CardanoNtNVersionDataShape::NtN7To10 => {
            push_cbor_array_len(out, 2)?;
            push_cbor_uint(out, u64::from(plan.network_magic))?;
            push_cbor_bool(out, plan.diffusion_mode.wire_bool());
        }
        CardanoNtNVersionDataShape::NtN11To12 | CardanoNtNVersionDataShape::NtN13AndUp => {
            push_cbor_array_len(out, 4)?;
            push_cbor_uint(out, u64::from(plan.network_magic))?;
            push_cbor_bool(out, plan.diffusion_mode.wire_bool());
            push_cbor_uint(out, u64::from(plan.peer_sharing_mode))?;
            push_cbor_bool(out, plan.query);
        }
    }
    Ok(())
}

fn push_cbor_array_len(out: &mut Vec<u8>, len: usize) -> Result<(), NetworkError> {
    push_cbor_major_len(out, 4, len)
}

fn push_cbor_map_len(out: &mut Vec<u8>, len: usize) -> Result<(), NetworkError> {
    push_cbor_major_len(out, 5, len)
}

fn push_cbor_major_len(out: &mut Vec<u8>, major: u8, len: usize) -> Result<(), NetworkError> {
    let len = u64::try_from(len).map_err(|_| {
        NetworkError::InvalidHandshakeProposal("CBOR container length exceeds supported range")
    })?;
    push_cbor_uint_with_major(out, major, len)
}

fn push_cbor_uint(out: &mut Vec<u8>, value: u64) -> Result<(), NetworkError> {
    push_cbor_uint_with_major(out, 0, value)
}

fn push_cbor_uint_with_major(out: &mut Vec<u8>, major: u8, value: u64) -> Result<(), NetworkError> {
    if major > 7 {
        return Err(NetworkError::InvalidHandshakeProposal(
            "CBOR major type exceeds supported range",
        ));
    }
    let prefix = major << 5;
    match value {
        0..=23 => out.push(prefix | value as u8),
        24..=0xff => {
            out.push(prefix | 24);
            out.push(value as u8);
        }
        0x100..=0xffff => {
            out.push(prefix | 25);
            out.extend_from_slice(&(value as u16).to_be_bytes());
        }
        0x1_0000..=0xffff_ffff => {
            out.push(prefix | 26);
            out.extend_from_slice(&(value as u32).to_be_bytes());
        }
        _ => {
            out.push(prefix | 27);
            out.extend_from_slice(&value.to_be_bytes());
        }
    }
    Ok(())
}

fn push_cbor_bool(out: &mut Vec<u8>, value: bool) {
    out.push(if value { 0xf5 } else { 0xf4 });
}

pub fn cardano_mux_frame_protocol_vector(
    protocol_id: u16,
    payload: &[u8],
    is_response: bool,
    timestamp: u32,
) -> Result<CardanoMuxFrameProtocolVector, NetworkError> {
    if protocol_id >= CARDANO_MUX_RESPONSE_FLAG {
        return Err(NetworkError::InvalidHandshakeProposal(
            "mux protocol id overlaps response flag",
        ));
    }
    if payload.is_empty() {
        return Err(NetworkError::InvalidHandshakeProposal(
            "mux frame payload must be non-empty",
        ));
    }
    if payload.len() > CARDANO_MUX_MAX_PAYLOAD_LENGTH {
        return Err(NetworkError::InvalidHandshakeProposal(
            "mux frame payload exceeds byte cap",
        ));
    }

    let payload_length = payload.len() as u16;
    let wire_protocol_id = if is_response {
        protocol_id | CARDANO_MUX_RESPONSE_FLAG
    } else {
        protocol_id
    };
    let mut encoded = Vec::with_capacity(CARDANO_MUX_HEADER_BYTES + payload.len());
    encoded.extend_from_slice(&timestamp.to_be_bytes());
    encoded.extend_from_slice(&wire_protocol_id.to_be_bytes());
    encoded.extend_from_slice(&payload_length.to_be_bytes());
    encoded.extend_from_slice(payload);

    Ok(CardanoMuxFrameProtocolVector {
        protocol_id,
        wire_protocol_id,
        timestamp,
        payload_length,
        is_response,
        encoded,
        production_ready: false,
    })
}

pub fn parse_cardano_mux_frame(data: &[u8]) -> Result<CardanoMuxFrame, NetworkError> {
    if data.len() < CARDANO_MUX_HEADER_BYTES {
        return Err(NetworkError::InvalidHandshakeProposal(
            "mux frame shorter than header",
        ));
    }
    let timestamp = u32::from_be_bytes(data[0..4].try_into().expect("slice length checked"));
    let wire_protocol_id = u16::from_be_bytes(data[4..6].try_into().expect("slice length checked"));
    let payload_length = u16::from_be_bytes(data[6..8].try_into().expect("slice length checked"));
    if payload_length == 0 {
        return Err(NetworkError::InvalidHandshakeProposal(
            "mux frame payload must be non-empty",
        ));
    }
    let expected_len = CARDANO_MUX_HEADER_BYTES + usize::from(payload_length);
    if data.len() != expected_len {
        return Err(NetworkError::InvalidHandshakeProposal(
            "mux frame payload length mismatch",
        ));
    }
    let is_response = (wire_protocol_id & CARDANO_MUX_RESPONSE_FLAG) != 0;
    let protocol_id = wire_protocol_id & !CARDANO_MUX_RESPONSE_FLAG;

    Ok(CardanoMuxFrame {
        protocol_id,
        wire_protocol_id,
        timestamp,
        payload_length,
        is_response,
        payload: data[CARDANO_MUX_HEADER_BYTES..].to_vec(),
    })
}

pub fn parse_cardano_mux_frame_stream(
    data: &[u8],
    max_frames: usize,
) -> Result<Vec<CardanoMuxFrame>, NetworkError> {
    if max_frames == 0 {
        return Err(NetworkError::InvalidHandshakeProposal(
            "mux frame stream max frames must be non-zero",
        ));
    }
    if data.is_empty() {
        return Err(NetworkError::InvalidHandshakeProposal(
            "mux frame stream must be non-empty",
        ));
    }

    let mut offset = 0;
    let mut frames = Vec::new();
    while offset < data.len() {
        if frames.len() == max_frames {
            return Err(NetworkError::InvalidHandshakeProposal(
                "mux frame stream exceeds frame cap",
            ));
        }
        let header_end = offset.checked_add(CARDANO_MUX_HEADER_BYTES).ok_or(
            NetworkError::InvalidHandshakeProposal("mux frame offset exceeds supported range"),
        )?;
        if header_end > data.len() {
            return Err(NetworkError::InvalidHandshakeProposal(
                "mux frame shorter than header",
            ));
        }
        let payload_length = u16::from_be_bytes(
            data[offset + 6..offset + 8]
                .try_into()
                .expect("slice length checked"),
        );
        let frame_len = CARDANO_MUX_HEADER_BYTES + usize::from(payload_length);
        let frame_end =
            offset
                .checked_add(frame_len)
                .ok_or(NetworkError::InvalidHandshakeProposal(
                    "mux frame offset exceeds supported range",
                ))?;
        let frame = parse_cardano_mux_frame(data.get(offset..frame_end).ok_or(
            NetworkError::InvalidHandshakeProposal("mux frame payload length mismatch"),
        )?)?;
        frames.push(frame);
        offset = frame_end;
    }
    Ok(frames)
}

pub fn cardano_mux_frame_stream_summary(
    frames: &[CardanoMuxFrame],
) -> Result<CardanoMuxFrameStreamSummary, NetworkError> {
    if frames.is_empty() {
        return Err(NetworkError::InvalidHandshakeProposal(
            "mux frame stream summary must be non-empty",
        ));
    }

    let request_frames = frames.iter().filter(|frame| !frame.is_response).count();
    let response_frames = frames.iter().filter(|frame| frame.is_response).count();
    let total_payload_bytes = frames
        .iter()
        .map(|frame| usize::from(frame.payload_length))
        .sum();
    let total_frame_bytes = frames
        .iter()
        .map(|frame| CARDANO_MUX_HEADER_BYTES + usize::from(frame.payload_length))
        .sum();
    let mut protocol_ids = Vec::new();
    for frame in frames {
        if !protocol_ids.contains(&frame.protocol_id) {
            protocol_ids.push(frame.protocol_id);
        }
    }

    Ok(CardanoMuxFrameStreamSummary {
        frame_count: frames.len(),
        request_frames,
        response_frames,
        total_payload_bytes,
        total_frame_bytes,
        protocol_count: protocol_ids.len(),
        production_ready: false,
        live_integrated: false,
    })
}

pub fn parse_cardano_handshake_response(
    data: &[u8],
) -> Result<CardanoHandshakeResponse, NetworkError> {
    let mut cbor = CborCursor::new(data);
    let len = cbor.read_array_len()?;
    let message_type = cbor.read_uint_u8()?;
    let kind = match message_type {
        1 => {
            if len != 3 {
                return Err(NetworkError::InvalidHandshakeProposal(
                    "accept-version message must have three fields",
                ));
            }
            let version = cbor.read_uint_u16()?;
            let version_data = parse_cardano_ntn_version_data(version, &mut cbor)?;
            CardanoHandshakeResponseKind::AcceptVersion {
                version,
                version_data,
            }
        }
        2 => {
            if len != 2 {
                return Err(NetworkError::InvalidHandshakeProposal(
                    "refuse message must have two fields",
                ));
            }
            parse_cardano_handshake_refusal(&mut cbor)?
        }
        _ => {
            return Err(NetworkError::InvalidHandshakeProposal(
                "unsupported handshake response message type",
            ));
        }
    };
    cbor.finish()?;
    Ok(CardanoHandshakeResponse {
        kind,
        production_ready: false,
    })
}

pub fn cardano_handshake_negotiation_report(
    profile: NetworkProfile,
    proposed_versions: &[u16],
    response: &CardanoHandshakeResponse,
) -> Result<CardanoHandshakeNegotiationReport, NetworkError> {
    validate_cardano_ntn_versions(proposed_versions)?;
    let outcome = match &response.kind {
        CardanoHandshakeResponseKind::AcceptVersion {
            version,
            version_data,
        } => {
            if !proposed_versions.contains(version) {
                return Err(NetworkError::InvalidHandshakeProposal(
                    "accepted version was not proposed",
                ));
            }
            if version_data.version != *version {
                return Err(NetworkError::InvalidHandshakeProposal(
                    "accepted version data does not match version",
                ));
            }
            if version_data.network_magic != profile.network_magic {
                return Err(NetworkError::NetworkMismatch {
                    local: profile.network_magic,
                    remote: version_data.network_magic,
                });
            }
            CardanoHandshakeNegotiationOutcome::Accepted {
                version: *version,
                network_magic: version_data.network_magic,
                diffusion_mode: version_data.diffusion_mode,
                peer_sharing: version_data.peer_sharing,
                query: version_data.query,
            }
        }
        CardanoHandshakeResponseKind::RefuseVersionMismatch { supported_versions } => {
            CardanoHandshakeNegotiationOutcome::Refused {
                reason: CardanoHandshakeRefusalReason::VersionMismatch,
                version: None,
                supported_versions: supported_versions.clone(),
                message: None,
            }
        }
        CardanoHandshakeResponseKind::RefuseDecodeError { version, message } => {
            CardanoHandshakeNegotiationOutcome::Refused {
                reason: CardanoHandshakeRefusalReason::DecodeError,
                version: Some(*version),
                supported_versions: Vec::new(),
                message: Some(message.clone()),
            }
        }
        CardanoHandshakeResponseKind::RefuseRefused { version, message } => {
            CardanoHandshakeNegotiationOutcome::Refused {
                reason: CardanoHandshakeRefusalReason::Refused,
                version: Some(*version),
                supported_versions: Vec::new(),
                message: Some(message.clone()),
            }
        }
    };

    Ok(CardanoHandshakeNegotiationReport {
        profile,
        proposed_versions: proposed_versions.to_vec(),
        outcome,
        production_ready: false,
    })
}

pub fn cardano_ntn_handshake_state_machine_plan() -> CardanoHandshakeStateMachinePlan {
    CardanoHandshakeStateMachinePlan {
        entries: vec![
            CardanoHandshakeStateEntry {
                state: CardanoHandshakeState::Propose,
                agency: CardanoHandshakeAgency::Client,
                timeout_secs: Some(CARDANO_NTN_HANDSHAKE_PROPOSE_TIMEOUT_SECS),
                transitions: vec![CardanoHandshakeTransition {
                    message: CardanoHandshakeMessageType::ProposeVersions,
                    next_state: CardanoHandshakeState::Confirm,
                }],
            },
            CardanoHandshakeStateEntry {
                state: CardanoHandshakeState::Confirm,
                agency: CardanoHandshakeAgency::Server,
                timeout_secs: Some(CARDANO_NTN_HANDSHAKE_CONFIRM_TIMEOUT_SECS),
                transitions: vec![
                    CardanoHandshakeTransition {
                        message: CardanoHandshakeMessageType::AcceptVersion,
                        next_state: CardanoHandshakeState::Done,
                    },
                    CardanoHandshakeTransition {
                        message: CardanoHandshakeMessageType::Refuse,
                        next_state: CardanoHandshakeState::Done,
                    },
                    CardanoHandshakeTransition {
                        message: CardanoHandshakeMessageType::QueryReply,
                        next_state: CardanoHandshakeState::Done,
                    },
                ],
            },
            CardanoHandshakeStateEntry {
                state: CardanoHandshakeState::Done,
                agency: CardanoHandshakeAgency::None,
                timeout_secs: None,
                transitions: Vec::new(),
            },
        ],
        production_ready: false,
        live_integrated: false,
    }
}

pub fn run_cardano_ntn_handshake_harness(
    profile: NetworkProfile,
    proposed_versions: &[u16],
    response_frame_bytes: &[u8],
) -> Result<CardanoHandshakeHarnessRun, NetworkError> {
    let machine = cardano_ntn_handshake_state_machine_plan();
    let proposal = cardano_ntn_handshake_protocol_vector(profile, proposed_versions)?;
    let proposal_frame =
        cardano_mux_frame_protocol_vector(proposal.protocol_id, &proposal.encoded, false, 0)?;
    let mut states = vec![CardanoHandshakeState::Propose];
    let mut messages = Vec::new();

    messages.push(CardanoHandshakeMessageType::ProposeVersions);
    let next = machine.next_state(
        *states.last().expect("state trace is initialized"),
        CardanoHandshakeMessageType::ProposeVersions,
    )?;
    states.push(next);

    let response_frame = parse_cardano_mux_frame(response_frame_bytes)?;
    if response_frame.protocol_id != CARDANO_HANDSHAKE_PROTOCOL_ID {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake response frame uses unexpected protocol id",
        ));
    }
    if !response_frame.is_response {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake response frame must set response flag",
        ));
    }
    let response = parse_cardano_handshake_response(&response_frame.payload)?;
    let response_message = match &response.kind {
        CardanoHandshakeResponseKind::AcceptVersion { .. } => {
            CardanoHandshakeMessageType::AcceptVersion
        }
        CardanoHandshakeResponseKind::RefuseVersionMismatch { .. }
        | CardanoHandshakeResponseKind::RefuseDecodeError { .. }
        | CardanoHandshakeResponseKind::RefuseRefused { .. } => CardanoHandshakeMessageType::Refuse,
    };
    messages.push(response_message);
    let next = machine.next_state(
        *states.last().expect("state trace advanced"),
        response_message,
    )?;
    states.push(next);

    let negotiation = cardano_handshake_negotiation_report(profile, proposed_versions, &response)?;
    Ok(CardanoHandshakeHarnessRun {
        profile,
        proposed_versions: proposed_versions.to_vec(),
        states,
        messages,
        proposal_frame,
        response_frame,
        response,
        production_ready: proposal.production_ready
            && machine.production_ready
            && negotiation.production_ready,
        live_integrated: machine.live_integrated,
        negotiation,
    })
}

pub fn cardano_ntn_handshake_transcript_protocol_vector(
    profile: NetworkProfile,
    proposed_versions: &[u16],
) -> Result<CardanoHandshakeTranscriptProtocolVector, NetworkError> {
    let proposal = cardano_ntn_handshake_protocol_vector(profile, proposed_versions)?;
    let request_frame =
        cardano_mux_frame_protocol_vector(proposal.protocol_id, &proposal.encoded, false, 0)?;
    let parsed_request = parse_cardano_mux_frame(&request_frame.encoded)?;
    if parsed_request.protocol_id != CARDANO_HANDSHAKE_PROTOCOL_ID || parsed_request.is_response {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake transcript request frame is invalid",
        ));
    }
    let accepted_version = proposed_versions
        .iter()
        .copied()
        .find(|version| *version == 10)
        .unwrap_or(proposed_versions[0]);
    let accept = cardano_ntn_handshake_accept_protocol_vector(profile, accepted_version)?;
    let response_frame =
        cardano_mux_frame_protocol_vector(accept.protocol_id, &accept.encoded, true, 0)?;
    let harness =
        run_cardano_ntn_handshake_harness(profile, proposed_versions, &response_frame.encoded)?;
    let frame_count = 2;
    let total_bytes = request_frame.encoded.len() + response_frame.encoded.len();
    Ok(CardanoHandshakeTranscriptProtocolVector {
        profile,
        proposed_versions: proposed_versions.to_vec(),
        accepted_version,
        production_ready: proposal.production_ready
            && accept.production_ready
            && request_frame.production_ready
            && response_frame.production_ready
            && harness.production_ready,
        live_integrated: harness.live_integrated,
        request_frame,
        response_frame,
        harness,
        frame_count,
        total_bytes,
    })
}

pub fn cardano_ntn_handshake_transcript_replay(
    profile: NetworkProfile,
    proposed_versions: &[u16],
) -> Result<CardanoHandshakeTranscriptReplay, NetworkError> {
    let transcript = cardano_ntn_handshake_transcript_protocol_vector(profile, proposed_versions)?;
    let mut stream = Vec::with_capacity(transcript.total_bytes);
    stream.extend_from_slice(&transcript.request_frame.encoded);
    stream.extend_from_slice(&transcript.response_frame.encoded);
    let frames = parse_cardano_mux_frame_stream(&stream, transcript.frame_count)?;
    if frames.len() != transcript.frame_count {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake transcript replay frame count mismatch",
        ));
    }
    let request = &frames[0];
    if request.protocol_id != CARDANO_HANDSHAKE_PROTOCOL_ID || request.is_response {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake transcript replay request frame is invalid",
        ));
    }
    let response_frame = &frames[1];
    if response_frame.protocol_id != CARDANO_HANDSHAKE_PROTOCOL_ID || !response_frame.is_response {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake transcript replay response frame is invalid",
        ));
    }
    let response = parse_cardano_handshake_response(&response_frame.payload)?;
    let negotiation = cardano_handshake_negotiation_report(profile, proposed_versions, &response)?;
    let accepted_version = match negotiation.outcome {
        CardanoHandshakeNegotiationOutcome::Accepted { version, .. } => version,
        CardanoHandshakeNegotiationOutcome::Refused { .. } => {
            return Err(NetworkError::InvalidHandshakeProposal(
                "handshake transcript replay expected accept response",
            ));
        }
    };
    let machine = cardano_ntn_handshake_state_machine_plan();
    let confirm = machine.next_state(
        CardanoHandshakeState::Propose,
        CardanoHandshakeMessageType::ProposeVersions,
    )?;
    let final_state = machine.next_state(confirm, CardanoHandshakeMessageType::AcceptVersion)?;
    let request_frames = frames.iter().filter(|frame| !frame.is_response).count();
    let response_frames = frames.iter().filter(|frame| frame.is_response).count();

    Ok(CardanoHandshakeTranscriptReplay {
        profile,
        proposed_versions: proposed_versions.to_vec(),
        request_frames,
        response_frames,
        total_bytes: stream.len(),
        accepted_version,
        final_state,
        production_ready: false,
        live_integrated: false,
        frames,
    })
}

pub fn cardano_ntn_handshake_refusal_transcript_replay(
    profile: NetworkProfile,
    proposed_versions: &[u16],
    supported_versions: &[u16],
) -> Result<CardanoHandshakeRefusalTranscriptReplay, NetworkError> {
    let proposal = cardano_ntn_handshake_protocol_vector(profile, proposed_versions)?;
    let request_frame =
        cardano_mux_frame_protocol_vector(proposal.protocol_id, &proposal.encoded, false, 0)?;
    let refusal =
        cardano_ntn_handshake_version_mismatch_refusal_protocol_vector(supported_versions)?;
    let response_vector =
        cardano_mux_frame_protocol_vector(refusal.protocol_id, &refusal.encoded, true, 0)?;
    let _harness =
        run_cardano_ntn_handshake_harness(profile, proposed_versions, &response_vector.encoded)?;
    let mut stream =
        Vec::with_capacity(request_frame.encoded.len() + response_vector.encoded.len());
    stream.extend_from_slice(&request_frame.encoded);
    stream.extend_from_slice(&response_vector.encoded);
    let frames = parse_cardano_mux_frame_stream(&stream, 2)?;
    if frames.len() != 2 {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake refusal transcript replay frame count mismatch",
        ));
    }
    let request = &frames[0];
    if request.protocol_id != CARDANO_HANDSHAKE_PROTOCOL_ID || request.is_response {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake refusal transcript replay request frame is invalid",
        ));
    }
    let response_frame = &frames[1];
    if response_frame.protocol_id != CARDANO_HANDSHAKE_PROTOCOL_ID || !response_frame.is_response {
        return Err(NetworkError::InvalidHandshakeProposal(
            "handshake refusal transcript replay response frame is invalid",
        ));
    }
    let response = parse_cardano_handshake_response(&response_frame.payload)?;
    let negotiation = cardano_handshake_negotiation_report(profile, proposed_versions, &response)?;
    let (refusal_reason, supported_versions) = match negotiation.outcome {
        CardanoHandshakeNegotiationOutcome::Refused {
            reason,
            supported_versions,
            ..
        } => (reason, supported_versions),
        CardanoHandshakeNegotiationOutcome::Accepted { .. } => {
            return Err(NetworkError::InvalidHandshakeProposal(
                "handshake refusal transcript replay expected refuse response",
            ));
        }
    };
    let machine = cardano_ntn_handshake_state_machine_plan();
    let confirm = machine.next_state(
        CardanoHandshakeState::Propose,
        CardanoHandshakeMessageType::ProposeVersions,
    )?;
    let final_state = machine.next_state(confirm, CardanoHandshakeMessageType::Refuse)?;
    let request_frames = frames.iter().filter(|frame| !frame.is_response).count();
    let response_frames = frames.iter().filter(|frame| frame.is_response).count();
    Ok(CardanoHandshakeRefusalTranscriptReplay {
        profile,
        proposed_versions: proposed_versions.to_vec(),
        request_frames,
        response_frames,
        total_bytes: stream.len(),
        final_state,
        refusal_reason,
        supported_versions,
        production_ready: false,
        live_integrated: false,
        frames,
    })
}

pub fn cardano_ntn_handshake_timeout_protocol_vector(
    state: CardanoHandshakeState,
    elapsed_secs: u64,
) -> Result<CardanoHandshakeTimeoutProtocolVector, NetworkError> {
    let machine = cardano_ntn_handshake_state_machine_plan();
    let entry = machine
        .entries
        .iter()
        .find(|entry| entry.state == state)
        .ok_or(NetworkError::InvalidHandshakeProposal(
            "unknown handshake state",
        ))?;
    let timeout_secs = entry
        .timeout_secs
        .ok_or(NetworkError::InvalidHandshakeProposal(
            "handshake state has no timeout",
        ))?;
    Ok(CardanoHandshakeTimeoutProtocolVector {
        state,
        agency: entry.agency,
        timeout_secs,
        elapsed_secs,
        timed_out: elapsed_secs >= timeout_secs,
        production_ready: false,
        live_integrated: machine.live_integrated,
    })
}

pub fn cardano_ntn_handshake_error_protocol_vector_report(
    profile: NetworkProfile,
    proposed_versions: &[u16],
) -> Result<CardanoHandshakeErrorProtocolVectorReport, NetworkError> {
    validate_cardano_ntn_versions(proposed_versions)?;
    let accepted_version = proposed_versions
        .iter()
        .copied()
        .find(|version| *version == 10)
        .unwrap_or(proposed_versions[0]);
    let accept = cardano_ntn_handshake_accept_protocol_vector(profile, accepted_version)?;
    let wrong_protocol_frame = cardano_mux_frame_protocol_vector(1, &accept.encoded, true, 0)?;
    let non_response_frame = cardano_mux_frame_protocol_vector(
        CARDANO_HANDSHAKE_PROTOCOL_ID,
        &accept.encoded,
        false,
        0,
    )?;
    let malformed_cbor_frame =
        cardano_mux_frame_protocol_vector(CARDANO_HANDSHAKE_PROTOCOL_ID, &[0x83], true, 0)?;
    let case_inputs = [
        (
            CardanoHandshakeErrorProtocolVectorKind::WrongProtocolId,
            wrong_protocol_frame.encoded,
            NetworkError::InvalidHandshakeProposal(
                "handshake response frame uses unexpected protocol id",
            ),
        ),
        (
            CardanoHandshakeErrorProtocolVectorKind::NonResponseFrame,
            non_response_frame.encoded,
            NetworkError::InvalidHandshakeProposal(
                "handshake response frame must set response flag",
            ),
        ),
        (
            CardanoHandshakeErrorProtocolVectorKind::MalformedCbor,
            malformed_cbor_frame.encoded,
            NetworkError::InvalidHandshakeProposal("truncated CBOR"),
        ),
    ];

    let mut cases = Vec::with_capacity(case_inputs.len());
    for (kind, frame, expected_error) in case_inputs {
        let observed_error =
            match run_cardano_ntn_handshake_harness(profile, proposed_versions, &frame) {
                Ok(_) => {
                    return Err(NetworkError::InvalidHandshakeProposal(
                        "error protocol vector unexpectedly completed handshake",
                    ));
                }
                Err(error) => error,
            };
        cases.push(CardanoHandshakeErrorProtocolVectorCase {
            kind,
            matched: observed_error == expected_error,
            expected_error,
            observed_error,
        });
    }

    Ok(CardanoHandshakeErrorProtocolVectorReport {
        profile,
        proposed_versions: proposed_versions.to_vec(),
        production_ready: false,
        live_integrated: false,
        cases,
    })
}

pub fn cardano_ntn_handshake_conformance_report(
    profile: NetworkProfile,
    proposed_versions: &[u16],
) -> Result<CardanoHandshakeConformanceReport, NetworkError> {
    let proposal = cardano_ntn_handshake_protocol_vector(profile, proposed_versions)?;
    let proposal_frame =
        cardano_mux_frame_protocol_vector(proposal.protocol_id, &proposal.encoded, false, 0)?;
    let parsed_proposal_frame = parse_cardano_mux_frame(&proposal_frame.encoded)?;
    let transcript = cardano_ntn_handshake_transcript_protocol_vector(profile, proposed_versions)?;
    let replay = cardano_ntn_handshake_transcript_replay(profile, proposed_versions)?;
    let refusal_replay = cardano_ntn_handshake_refusal_transcript_replay(
        profile,
        proposed_versions,
        &CARDANO_NTN_REFUSAL_SUPPORTED_VERSIONS,
    )?;
    let timeout = cardano_ntn_handshake_timeout_protocol_vector(
        CardanoHandshakeState::Confirm,
        CARDANO_NTN_HANDSHAKE_CONFIRM_TIMEOUT_SECS,
    )?;
    let errors = cardano_ntn_handshake_error_protocol_vector_report(profile, proposed_versions)?;
    let checks = [
        proposal.protocol_id == CARDANO_HANDSHAKE_PROTOCOL_ID && proposal.message_type == 0,
        parsed_proposal_frame.protocol_id == CARDANO_HANDSHAKE_PROTOCOL_ID
            && !parsed_proposal_frame.is_response,
        transcript.response_frame.protocol_id == CARDANO_HANDSHAKE_PROTOCOL_ID
            && transcript.response_frame.is_response,
        transcript.harness.states.last() == Some(&CardanoHandshakeState::Done),
        transcript.frame_count == 2
            && transcript.total_bytes
                == transcript.request_frame.encoded.len() + transcript.response_frame.encoded.len(),
        replay.frames.len() == 2
            && replay.request_frames == 1
            && replay.response_frames == 1
            && replay.final_state == CardanoHandshakeState::Done,
        refusal_replay.frames.len() == 2
            && refusal_replay.request_frames == 1
            && refusal_replay.response_frames == 1
            && refusal_replay.final_state == CardanoHandshakeState::Done
            && refusal_replay.refusal_reason == CardanoHandshakeRefusalReason::VersionMismatch
            && refusal_replay.supported_versions == CARDANO_NTN_REFUSAL_SUPPORTED_VERSIONS,
        timeout.timed_out,
        errors.cases.iter().all(|case| case.matched),
    ];
    let passed_checks = checks.iter().filter(|passed| **passed).count();
    let offline_complete = passed_checks == checks.len();

    Ok(CardanoHandshakeConformanceReport {
        profile,
        proposed_versions: proposed_versions.to_vec(),
        offline_checks: checks.len(),
        passed_checks,
        offline_complete,
        live_ready: false,
        blockers: TESTNET_HANDSHAKE_CONFORMANCE_BLOCKERS.to_vec(),
        production_ready: false,
        live_integrated: false,
    })
}

pub fn testnet_handshake_conformance_matrix(
    proposed_versions: &[u16],
) -> Result<TestnetHandshakeConformanceMatrix, NetworkError> {
    let reports = network_profiles()
        .iter()
        .filter(|profile| profile.kind == NetworkProfileKind::Public && profile.name != "mainnet")
        .map(|profile| cardano_ntn_handshake_conformance_report(*profile, proposed_versions))
        .collect::<Result<Vec<_>, _>>()?;
    let passed_profiles = reports
        .iter()
        .filter(|report| report.offline_complete)
        .count();
    let offline_complete = passed_profiles == reports.len() && !reports.is_empty();

    Ok(TestnetHandshakeConformanceMatrix {
        public_testnet_profiles: reports.len(),
        passed_profiles,
        offline_complete,
        live_ready: false,
        blockers: TESTNET_HANDSHAKE_CONFORMANCE_BLOCKERS.to_vec(),
        production_ready: false,
        live_integrated: false,
        reports,
    })
}

fn parse_cardano_handshake_refusal(
    cbor: &mut CborCursor<'_>,
) -> Result<CardanoHandshakeResponseKind, NetworkError> {
    let len = cbor.read_array_len()?;
    let reason = cbor.read_uint_u8()?;
    match reason {
        0 => {
            if len != 2 {
                return Err(NetworkError::InvalidHandshakeProposal(
                    "version-mismatch refusal must have two fields",
                ));
            }
            let version_count = cbor.read_array_len()?;
            let mut supported_versions = Vec::with_capacity(version_count);
            for _ in 0..version_count {
                supported_versions.push(cbor.read_uint_u16()?);
            }
            Ok(CardanoHandshakeResponseKind::RefuseVersionMismatch { supported_versions })
        }
        1 | 2 => {
            if len != 3 {
                return Err(NetworkError::InvalidHandshakeProposal(
                    "decode/refused refusal must have three fields",
                ));
            }
            let version = cbor.read_uint_u16()?;
            let message = cbor.read_text()?;
            if reason == 1 {
                Ok(CardanoHandshakeResponseKind::RefuseDecodeError { version, message })
            } else {
                Ok(CardanoHandshakeResponseKind::RefuseRefused { version, message })
            }
        }
        _ => Err(NetworkError::InvalidHandshakeProposal(
            "unsupported handshake refusal reason",
        )),
    }
}

fn parse_cardano_ntn_version_data(
    version: u16,
    cbor: &mut CborCursor<'_>,
) -> Result<ParsedCardanoNtNVersionData, NetworkError> {
    let shape = match version {
        7..=10 => CardanoNtNVersionDataShape::NtN7To10,
        11..=12 => CardanoNtNVersionDataShape::NtN11To12,
        13..=15 => CardanoNtNVersionDataShape::NtN13AndUp,
        _ => {
            return Err(NetworkError::InvalidHandshakeProposal(
                "unsupported Cardano NtN handshake version",
            ));
        }
    };
    let expected_len = match shape {
        CardanoNtNVersionDataShape::NtN7To10 => 2,
        CardanoNtNVersionDataShape::NtN11To12 | CardanoNtNVersionDataShape::NtN13AndUp => 4,
    };
    if cbor.read_array_len()? != expected_len {
        return Err(NetworkError::InvalidHandshakeProposal(
            "unexpected Cardano NtN version-data shape",
        ));
    }
    let network_magic = cbor.read_uint_u32()?;
    let diffusion_mode = if cbor.read_bool()? {
        CardanoNtNDiffusionMode::InitiatorOnly
    } else {
        CardanoNtNDiffusionMode::InitiatorAndResponder
    };
    let (peer_sharing_mode, query) = match shape {
        CardanoNtNVersionDataShape::NtN7To10 => (0, false),
        CardanoNtNVersionDataShape::NtN11To12 | CardanoNtNVersionDataShape::NtN13AndUp => {
            (cbor.read_uint_u8()?, cbor.read_bool()?)
        }
    };
    let peer_sharing = match shape {
        CardanoNtNVersionDataShape::NtN7To10 => false,
        CardanoNtNVersionDataShape::NtN11To12 => peer_sharing_mode != 0,
        CardanoNtNVersionDataShape::NtN13AndUp => peer_sharing_mode >= 1,
    };

    Ok(ParsedCardanoNtNVersionData {
        version,
        network_magic,
        diffusion_mode,
        peer_sharing_mode,
        peer_sharing,
        query,
        shape,
    })
}

struct CborCursor<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> CborCursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn finish(&self) -> Result<(), NetworkError> {
        if self.offset == self.data.len() {
            Ok(())
        } else {
            Err(NetworkError::InvalidHandshakeProposal(
                "CBOR message has trailing bytes",
            ))
        }
    }

    fn read_array_len(&mut self) -> Result<usize, NetworkError> {
        let (major, value) = self.read_type_value()?;
        if major != 4 {
            return Err(NetworkError::InvalidHandshakeProposal(
                "expected CBOR array",
            ));
        }
        usize::try_from(value).map_err(|_| {
            NetworkError::InvalidHandshakeProposal("CBOR array length exceeds supported range")
        })
    }

    fn read_uint_u8(&mut self) -> Result<u8, NetworkError> {
        let value = self.read_uint()?;
        u8::try_from(value)
            .map_err(|_| NetworkError::InvalidHandshakeProposal("CBOR uint exceeds u8 range"))
    }

    fn read_uint_u16(&mut self) -> Result<u16, NetworkError> {
        let value = self.read_uint()?;
        u16::try_from(value)
            .map_err(|_| NetworkError::InvalidHandshakeProposal("CBOR uint exceeds u16 range"))
    }

    fn read_uint_u32(&mut self) -> Result<u32, NetworkError> {
        let value = self.read_uint()?;
        u32::try_from(value)
            .map_err(|_| NetworkError::InvalidHandshakeProposal("CBOR uint exceeds u32 range"))
    }

    fn read_uint(&mut self) -> Result<u64, NetworkError> {
        let (major, value) = self.read_type_value()?;
        if major != 0 {
            return Err(NetworkError::InvalidHandshakeProposal("expected CBOR uint"));
        }
        Ok(value)
    }

    fn read_bool(&mut self) -> Result<bool, NetworkError> {
        match self.take_byte()? {
            0xf4 => Ok(false),
            0xf5 => Ok(true),
            _ => Err(NetworkError::InvalidHandshakeProposal("expected CBOR bool")),
        }
    }

    fn read_text(&mut self) -> Result<String, NetworkError> {
        let (major, value) = self.read_type_value()?;
        if major != 3 {
            return Err(NetworkError::InvalidHandshakeProposal("expected CBOR text"));
        }
        let len = usize::try_from(value).map_err(|_| {
            NetworkError::InvalidHandshakeProposal("CBOR text length exceeds supported range")
        })?;
        let bytes = self.take_slice(len)?;
        std::str::from_utf8(bytes)
            .map(|value| value.to_string())
            .map_err(|_| NetworkError::InvalidHandshakeProposal("CBOR text is not UTF-8"))
    }

    fn read_type_value(&mut self) -> Result<(u8, u64), NetworkError> {
        let byte = self.take_byte()?;
        let major = byte >> 5;
        let additional = byte & 0x1f;
        let value = match additional {
            0..=23 => u64::from(additional),
            24 => u64::from(self.take_byte()?),
            25 => u64::from(u16::from_be_bytes(self.take_array()?)),
            26 => u64::from(u32::from_be_bytes(self.take_array()?)),
            27 => u64::from_be_bytes(self.take_array()?),
            _ => {
                return Err(NetworkError::InvalidHandshakeProposal(
                    "unsupported CBOR additional information",
                ));
            }
        };
        Ok((major, value))
    }

    fn take_byte(&mut self) -> Result<u8, NetworkError> {
        let byte = *self
            .data
            .get(self.offset)
            .ok_or(NetworkError::InvalidHandshakeProposal("truncated CBOR"))?;
        self.offset += 1;
        Ok(byte)
    }

    fn take_array<const N: usize>(&mut self) -> Result<[u8; N], NetworkError> {
        let bytes = self.take_slice(N)?;
        Ok(bytes.try_into().expect("slice length checked"))
    }

    fn take_slice(&mut self, len: usize) -> Result<&'a [u8], NetworkError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(NetworkError::InvalidHandshakeProposal(
                "CBOR offset exceeds supported range",
            ))?;
        let slice = self
            .data
            .get(self.offset..end)
            .ok_or(NetworkError::InvalidHandshakeProposal("truncated CBOR"))?;
        self.offset = end;
        Ok(slice)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkError {
    PathsClosed,
    ProtocolNotImplemented,
    ProtocolRequiresReview(Vec<String>),
    UnknownNetwork(String),
    TestnetContactBlocked(String),
    UnboundedTestnetLimit(&'static str),
    EmptyTestnetRequest(&'static str),
    TestnetLimitExceeded {
        limit: &'static str,
        requested: u64,
        max: u64,
    },
    NetworkMismatch {
        local: u32,
        remote: u32,
    },
    UnsupportedVersion {
        local: u16,
        remote: u16,
    },
    InvalidHandshakeProposal(&'static str),
}

impl NetworkError {
    pub fn summary_line(&self) -> String {
        let (kind, details) = match self {
            Self::PathsClosed => ("paths_closed", 0),
            Self::ProtocolNotImplemented => ("protocol_not_implemented", 0),
            Self::ProtocolRequiresReview(blockers) => ("protocol_requires_review", blockers.len()),
            Self::UnknownNetwork(_) => ("unknown_network", 1),
            Self::TestnetContactBlocked(_) => ("testnet_contact_blocked", 1),
            Self::UnboundedTestnetLimit(_) => ("unbounded_testnet_limit", 1),
            Self::EmptyTestnetRequest(_) => ("empty_testnet_request", 1),
            Self::TestnetLimitExceeded { .. } => ("testnet_limit_exceeded", 3),
            Self::NetworkMismatch { .. } => ("network_mismatch", 2),
            Self::UnsupportedVersion { .. } => ("unsupported_version", 2),
            Self::InvalidHandshakeProposal(_) => ("invalid_handshake_proposal", 1),
        };
        format!("network_error kind={} details={}", kind, details)
    }

    pub fn to_event(&self) -> Event {
        Event::new(NETWORK_ERROR_EVENT, EventPayload::Text(self.summary_line()))
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![self.to_event()]
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathsClosed => f.write_str("path opening is disabled by guard pattern"),
            Self::ProtocolNotImplemented => {
                f.write_str("network path opening is not implemented yet")
            }
            Self::ProtocolRequiresReview(blockers) => {
                write!(
                    f,
                    "network path opening requires review: {}",
                    blockers.join("; ")
                )
            }
            Self::UnknownNetwork(network) => write!(f, "unknown network {network}"),
            Self::TestnetContactBlocked(reason) => {
                write!(f, "testnet contact blocked: {reason}")
            }
            Self::UnboundedTestnetLimit(limit) => {
                write!(f, "testnet contact limit {limit} must be bounded")
            }
            Self::EmptyTestnetRequest(limit) => {
                write!(f, "testnet contact request {limit} must be nonzero")
            }
            Self::TestnetLimitExceeded {
                limit,
                requested,
                max,
            } => write!(
                f,
                "testnet contact {limit} exceeds bound: requested={requested} max={max}"
            ),
            Self::NetworkMismatch { local, remote } => {
                write!(f, "network mismatch: local={local} remote={remote}")
            }
            Self::UnsupportedVersion { local, remote } => {
                write!(
                    f,
                    "unsupported network version: local={local} remote={remote}"
                )
            }
            Self::InvalidHandshakeProposal(reason) => {
                write!(f, "invalid handshake proposal: {reason}")
            }
        }
    }
}

impl std::error::Error for NetworkError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_error_summary_events_omit_error_detail_payloads() {
        let blocked = NetworkError::TestnetContactBlocked("operator supplied peer blocked".into());
        let invalid = NetworkError::InvalidHandshakeProposal("payload length mismatch");

        assert_eq!(blocked.event_batch().len(), 1);
        assert_eq!(blocked.event_batch()[0].name.as_str(), NETWORK_ERROR_EVENT);
        assert_eq!(
            blocked.summary_line(),
            "network_error kind=testnet_contact_blocked details=1"
        );
        assert_eq!(
            invalid.summary_line(),
            "network_error kind=invalid_handshake_proposal details=1"
        );
    }

    #[test]
    fn handshake_plan_validates_magic_without_opening_paths() {
        let local = HandshakeHello::new(2, ConnectionRole::Outer, PeerAddress::new("local", 3001));
        let remote =
            HandshakeHello::new(2, ConnectionRole::Inner, PeerAddress::new("remote", 3002));
        assert_eq!(local.event_batch().len(), 1);
        assert_eq!(
            local.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_HELLO_EVENT
        );
        assert_eq!(
            local.summary_line(),
            "handshake_hello magic=2 version=1 role=outer"
        );
        let plan = HandshakePlan {
            local,
            remote,
            share_peer: false,
            intersect_tip: false,
        };
        assert_eq!(plan.event_batch().len(), 1);
        assert_eq!(
            plan.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_PLAN_EVENT
        );
        assert_eq!(
            plan.summary_line(),
            "handshake_plan local_magic=2 remote_magic=2 local_version=1 remote_version=1 local_role=outer remote_role=inner share_peer=false intersect_tip=false"
        );
        assert_eq!(plan.validate(), Ok(()));
    }

    #[test]
    fn handshake_rejects_network_mismatch() {
        let local = HandshakeHello::new(2, ConnectionRole::Outer, PeerAddress::new("local", 3001));
        let remote =
            HandshakeHello::new(42, ConnectionRole::Inner, PeerAddress::new("remote", 3002));
        let plan = HandshakePlan {
            local,
            remote,
            share_peer: false,
            intersect_tip: false,
        };
        assert_eq!(
            plan.validate(),
            Err(NetworkError::NetworkMismatch {
                local: 2,
                remote: 42
            })
        );
    }

    #[test]
    fn open_review_lists_blockers_without_opening_paths() {
        let plan = NetworkPlan::from_config(&NodeConfig::default());
        let review = plan.open_review();

        assert_eq!(plan.event_batch().len(), 1);
        assert_eq!(plan.event_batch()[0].name.as_str(), NETWORK_PLAN_EVENT);
        assert_eq!(
            plan.summary_line(),
            format!(
                "network_plan magic=2 listeners={} sharing=false intersect_tip=false paths_enabled=false",
                NodeConfig::default().listener_plan().len()
            )
        );
        assert!(review.blocked());
        assert!(!review.paths_enabled);
        assert_eq!(
            review.listeners.len(),
            NodeConfig::default().listener_plan().len()
        );
        assert!(review
            .blockers
            .iter()
            .any(|blocker| blocker.contains("disabled by safety")));
        let events = review.events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_OPEN_BLOCKED_EVENT);
        assert_eq!(
            events[0].payload,
            EventPayload::Text(format!(
                "paths_enabled=false listeners={} blockers=2",
                NodeConfig::default().listener_plan().len()
            ))
        );
        assert_eq!(review.event_batch().len(), 2);
        assert_eq!(
            review.event_batch()[0].name.as_str(),
            NETWORK_OPEN_REVIEW_EVENT
        );
        assert_eq!(
            review.summary_line(),
            format!(
                "network_open_review paths_enabled=false listeners={} blockers=2 blocked=true",
                NodeConfig::default().listener_plan().len()
            )
        );
        assert_eq!(plan.assert_safe_to_open(), Err(NetworkError::PathsClosed));
    }

    #[test]
    fn open_review_still_requires_protocol_review_when_paths_are_allowed() {
        let mut config = NodeConfig::default();
        config.safety.allow_paths = true;
        let plan = NetworkPlan::from_config(&config);
        let review = plan.open_review();

        assert!(review.blocked());
        assert!(review.paths_enabled);
        assert_eq!(
            plan.assert_safe_to_open(),
            Err(NetworkError::ProtocolRequiresReview(vec![
                "network protocol path opening requires explicit review".to_string()
            ]))
        );
    }

    #[test]
    fn bounded_testnet_contact_plan_accepts_small_testnet_request() {
        let plan = plan_bounded_testnet_contact(
            TestnetContactRequest::new("testnet", 2, 100, 1024),
            TestnetContactLimits::smoke_test(),
        )
        .unwrap();

        assert_eq!(plan.profile.name, "preprod");
        assert_eq!(plan.profile.network_magic, 1);
        assert_eq!(plan.request.requested_blocks, 2);
        assert!(plan.limits.allow_testnet);
        assert_eq!(plan.request.event_batch().len(), 1);
        assert_eq!(
            plan.request.event_batch()[0].name.as_str(),
            NETWORK_TESTNET_CONTACT_REQUEST_EVENT
        );
        assert_eq!(
            plan.request.summary_line(),
            "testnet_contact_request network=testnet blocks=2 slots=100 bytes=1024"
        );
        assert_eq!(plan.limits.event_batch().len(), 1);
        assert_eq!(
            plan.limits.event_batch()[0].name.as_str(),
            NETWORK_TESTNET_CONTACT_LIMITS_EVENT
        );
        assert_eq!(
            plan.limits.summary_line(),
            "testnet_contact_limits allow_testnet=true max_blocks=32 max_slots=2000 max_bytes=8388608 timeout_secs=30 temp_bytes=33554432"
        );
        assert_eq!(plan.event_batch().len(), 1);
        assert_eq!(
            plan.event_batch()[0].name.as_str(),
            NETWORK_TESTNET_CONTACT_PLAN_EVENT
        );
        assert_eq!(
            plan.summary_line(),
            "testnet_contact_plan network=preprod blocks=2/32 slots=100/2000 bytes=1024/8388608 temp_bytes=33554432 timeout_secs=30"
        );
    }

    #[test]
    fn testnet_live_readiness_reports_blockers_without_opening_paths() {
        let readiness = testnet_live_readiness();

        assert!(readiness.tcp_probe_available);
        assert!(readiness.handshake_sketch_available);
        assert!(readiness.offline_conformance_complete);
        assert_eq!(readiness.public_testnet_profiles, 2);
        assert!(readiness.live_command_available);
        assert!(readiness.path_review_complete);
        assert!(!readiness.conformance_complete);
        assert!(readiness.live_contact_allowed);
        assert_eq!(readiness.blockers.len(), 1);
        assert!(readiness
            .blockers
            .iter()
            .any(|blocker| blocker.contains("conformance")));
        assert_eq!(
            readiness.action_items(),
            vec!["complete full testnet conformance harness"]
        );
        assert_eq!(readiness.event_batch().len(), 1);
        assert_eq!(
            readiness.event_batch()[0].name.as_str(),
            NETWORK_TESTNET_LIVE_READINESS_EVENT
        );
        assert_eq!(
            readiness.summary_line(),
            "testnet_live_readiness tcp_probe=true handshake_sketch=true offline_conformance=true public_profiles=2 live_contact=true blockers=1"
        );
    }

    #[test]
    fn local_handshake_sketch_encodes_deterministic_non_production_bytes() {
        let profile = network_profile("preview").unwrap();
        let sketch = local_handshake_sketch(profile, &[NETWORK_HANDSHAKE_VERSION]).unwrap();

        assert_eq!(sketch.profile.name, "preview");
        assert_eq!(sketch.versions, vec![NETWORK_HANDSHAKE_VERSION]);
        assert_eq!(
            sketch.encoded,
            vec![65, 67, 82, 72, 83, 49, 1, 0, 1, 0, 0, 0, 2, 1]
        );
        assert!(!sketch.production_compatible);
        assert!(sketch.encoded.len() <= MAX_LOCAL_HANDSHAKE_SKETCH_BYTES);
        assert_eq!(sketch.event_batch().len(), 1);
        assert_eq!(
            sketch.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_SKETCH_EVENT
        );
        assert_eq!(
            sketch.summary_line(),
            "handshake_sketch network=preview versions=1 encoded_bytes=14 production_compatible=false"
        );
    }

    #[test]
    fn local_handshake_sketch_rejects_unreviewed_version_tables() {
        let profile = network_profile("preview").unwrap();
        assert_eq!(
            local_handshake_sketch(profile, &[]),
            Err(NetworkError::InvalidHandshakeProposal(
                "handshake versions must be non-empty"
            ))
        );
        assert_eq!(
            local_handshake_sketch(profile, &[2]),
            Err(NetworkError::InvalidHandshakeProposal(
                "handshake version table must include local version"
            ))
        );
        assert_eq!(
            local_handshake_sketch(
                profile,
                &[NETWORK_HANDSHAKE_VERSION, NETWORK_HANDSHAKE_VERSION]
            ),
            Err(NetworkError::InvalidHandshakeProposal(
                "handshake versions must be strictly ascending"
            ))
        );
    }

    #[test]
    fn cardano_ntn_handshake_protocol_vector_matches_expected_bytes() {
        let profile = network_profile("preview").unwrap();
        let protocol_vector =
            cardano_ntn_handshake_protocol_vector(profile, &[7, 8, 9, 10]).unwrap();

        assert_eq!(protocol_vector.protocol_id, CARDANO_HANDSHAKE_PROTOCOL_ID);
        assert_eq!(protocol_vector.message_type, 0);
        assert!(!protocol_vector.production_ready);
        assert_eq!(
            hex(&protocol_vector.encoded),
            "8200a4078202f4088202f4098202f40a8202f4"
        );
        assert_eq!(protocol_vector.event_batch().len(), 1);
        assert_eq!(
            protocol_vector.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_PROPOSAL_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            protocol_vector.summary_line(),
            "handshake_proposal_protocol_vector network=preview versions=4 min_version=7 max_version=10 leios_overlay_min_version=15 leios_overlay_capable=false encoded_bytes=19 protocol_id=0 message_type=0 production_ready=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_protocol_vector_reports_supported_versions() {
        let profile = network_profile("preprod").unwrap();
        let protocol_vector =
            cardano_ntn_handshake_protocol_vector(profile, &CARDANO_NTN_SUPPORTED_VERSIONS)
                .unwrap();

        assert_eq!(protocol_vector.profile.name, "preprod");
        assert_eq!(
            protocol_vector.versions,
            CARDANO_NTN_SUPPORTED_VERSIONS.to_vec()
        );
        assert_eq!(CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION, 15);
        assert!(!cardano_ntn_version_leios_overlay_capable(14));
        assert!(cardano_ntn_version_leios_overlay_capable(15));
        assert!(cardano_ntn_version_leios_overlay_capable(16));
        assert!(cardano_ntn_leios_overlay_capable(&protocol_vector.versions));
        assert!(!cardano_ntn_leios_overlay_capable(&[7, 8, 9, 10]));
        assert_eq!(protocol_vector.encoded.len(), 49);
        assert!(protocol_vector.encoded.len() <= MAX_LOCAL_HANDSHAKE_SKETCH_BYTES);
        assert_eq!(
            hex(&protocol_vector.encoded),
            "8200a9078201f4088201f4098201f40a8201f40b8401f400f40c8401f400f40d8401f400f40e8401f400f40f8401f400f4"
        );
    }

    #[test]
    fn cardano_ntn_version_data_protocol_vectors_cover_peer_sharing_modes() {
        let profile = network_profile("preview").unwrap();
        let mut v11_public = Vec::new();
        let v11_plan = cardano_ntn_version_data_plan(
            profile,
            11,
            CardanoNtNDiffusionMode::InitiatorAndResponder,
            true,
            false,
        )
        .unwrap();
        encode_cardano_ntn_version_data(&mut v11_public, v11_plan).unwrap();

        let mut v13_public = Vec::new();
        let v13_plan = cardano_ntn_version_data_plan(
            profile,
            13,
            CardanoNtNDiffusionMode::InitiatorAndResponder,
            true,
            false,
        )
        .unwrap();
        encode_cardano_ntn_version_data(&mut v13_public, v13_plan).unwrap();

        assert_eq!(hex(&v11_public), "8402f402f4");
        assert_eq!(hex(&v13_public), "8402f401f4");
        assert_eq!(v13_plan.event_batch().len(), 1);
        assert_eq!(
            v13_plan.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_VERSION_DATA_PLAN_EVENT
        );
        assert_eq!(
            v13_plan.summary_line(),
            "handshake_version_data_plan version=13 network_magic=2 shape=ntn13_and_up diffusion=initiator_and_responder peer_sharing_mode=1 query=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_protocol_vector_rejects_unreviewed_versions() {
        let profile = network_profile("preview").unwrap();
        assert_eq!(
            cardano_ntn_handshake_protocol_vector(profile, &[1]),
            Err(NetworkError::InvalidHandshakeProposal(
                "unsupported Cardano NtN handshake version"
            ))
        );
        assert_eq!(
            cardano_ntn_handshake_protocol_vector(profile, &[7, 7]),
            Err(NetworkError::InvalidHandshakeProposal(
                "Cardano NtN versions must be strictly ascending"
            ))
        );
    }

    #[test]
    fn cardano_mux_frame_protocol_vector_wraps_handshake_cbor() {
        let profile = network_profile("preprod").unwrap();
        let handshake =
            cardano_ntn_handshake_protocol_vector(profile, &CARDANO_NTN_SUPPORTED_VERSIONS)
                .unwrap();
        let frame =
            cardano_mux_frame_protocol_vector(handshake.protocol_id, &handshake.encoded, false, 0)
                .unwrap();

        assert_eq!(frame.protocol_id, CARDANO_HANDSHAKE_PROTOCOL_ID);
        assert_eq!(frame.wire_protocol_id, CARDANO_HANDSHAKE_PROTOCOL_ID);
        assert_eq!(frame.timestamp, 0);
        assert_eq!(frame.payload_length, 49);
        assert!(!frame.is_response);
        assert!(!frame.production_ready);
        assert_eq!(frame.encoded.len(), CARDANO_MUX_HEADER_BYTES + 49);
        let events = frame.event_batch();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0].name.as_str(),
            NETWORK_MUX_FRAME_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            frame.summary_line(),
            "mux_frame_protocol_vector protocol_id=0 wire_protocol_id=0 payload_bytes=49 encoded_bytes=57 response=false production_ready=false"
        );
        assert_eq!(
            hex(&frame.encoded),
            "00000000000000318200a9078201f4088201f4098201f40a8201f40b8401f400f40c8401f400f40d8401f400f40e8401f400f40f8401f400f4"
        );
    }

    #[test]
    fn cardano_mux_frame_protocol_vector_applies_response_flag() {
        let frame = cardano_mux_frame_protocol_vector(1, &[1, 2, 3], true, 0x0102_0304).unwrap();

        assert_eq!(frame.protocol_id, 1);
        assert_eq!(frame.wire_protocol_id, 0x8001);
        assert_eq!(frame.payload_length, 3);
        assert!(frame.is_response);
        assert_eq!(hex(&frame.encoded), "0102030480010003010203");
    }

    #[test]
    fn cardano_mux_frame_protocol_vector_rejects_unsafe_shapes() {
        assert_eq!(
            cardano_mux_frame_protocol_vector(CARDANO_MUX_RESPONSE_FLAG, &[1], false, 0),
            Err(NetworkError::InvalidHandshakeProposal(
                "mux protocol id overlaps response flag"
            ))
        );
        assert_eq!(
            cardano_mux_frame_protocol_vector(1, &[], false, 0),
            Err(NetworkError::InvalidHandshakeProposal(
                "mux frame payload must be non-empty"
            ))
        );
        let oversized = vec![0; CARDANO_MUX_MAX_PAYLOAD_LENGTH + 1];
        assert_eq!(
            cardano_mux_frame_protocol_vector(1, &oversized, false, 0),
            Err(NetworkError::InvalidHandshakeProposal(
                "mux frame payload exceeds byte cap"
            ))
        );
    }

    #[test]
    fn cardano_mux_frame_parser_decodes_protocol_vector_without_io() {
        let profile = network_profile("preprod").unwrap();
        let handshake =
            cardano_ntn_handshake_protocol_vector(profile, &CARDANO_NTN_SUPPORTED_VERSIONS)
                .unwrap();
        let frame =
            cardano_mux_frame_protocol_vector(handshake.protocol_id, &handshake.encoded, false, 0)
                .unwrap();

        let parsed = parse_cardano_mux_frame(&frame.encoded).unwrap();
        assert_eq!(parsed.protocol_id, CARDANO_HANDSHAKE_PROTOCOL_ID);
        assert_eq!(parsed.wire_protocol_id, CARDANO_HANDSHAKE_PROTOCOL_ID);
        assert_eq!(parsed.timestamp, 0);
        assert_eq!(parsed.payload_length, 49);
        assert!(!parsed.is_response);
        assert_eq!(parsed.payload, handshake.encoded);
        let events = parsed.event_batch();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), NETWORK_MUX_FRAME_EVENT);
        assert_eq!(
            parsed.summary_line(),
            "mux_frame protocol_id=0 wire_protocol_id=0 payload_bytes=49 response=false timestamp=0"
        );
    }

    #[test]
    fn cardano_handshake_response_parser_matches_accept_protocol_vector() {
        let parsed = parse_cardano_handshake_response(&bytes("83010a8202f4")).unwrap();

        assert!(!parsed.production_ready);
        assert_eq!(parsed.event_batch().len(), 1);
        assert_eq!(
            parsed.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_RESPONSE_EVENT
        );
        assert_eq!(
            parsed.summary_line(),
            "handshake_response kind=accept_version version=10 leios_overlay_min_version=15 leios_overlay_negotiated=false supported_versions=0 supported_min_version=0 supported_max_version=0 supported_leios_overlay_capable=none message_present=false production_ready=false"
        );
        match parsed.kind {
            CardanoHandshakeResponseKind::AcceptVersion {
                version,
                version_data,
            } => {
                assert_eq!(version, 10);
                assert_eq!(version_data.version, 10);
                assert_eq!(version_data.network_magic, 2);
                assert_eq!(
                    version_data.diffusion_mode,
                    CardanoNtNDiffusionMode::InitiatorAndResponder
                );
                assert_eq!(version_data.peer_sharing_mode, 0);
                assert!(!version_data.peer_sharing);
                assert!(!version_data.query);
                assert_eq!(version_data.shape, CardanoNtNVersionDataShape::NtN7To10);
                assert_eq!(version_data.event_batch().len(), 1);
                assert_eq!(
                    version_data.event_batch()[0].name.as_str(),
                    NETWORK_HANDSHAKE_VERSION_DATA_EVENT
                );
                assert_eq!(
                    version_data.summary_line(),
                    "handshake_version_data version=10 network_magic=2 shape=ntn7_to_10 diffusion=initiator_and_responder peer_sharing_mode=0 peer_sharing=false query=false"
                );
            }
            other => panic!("expected accept-version response, got {other:?}"),
        }
    }

    #[test]
    fn cardano_handshake_response_parser_matches_refusal_protocol_vectors() {
        let mismatch = parse_cardano_handshake_response(&bytes("82028200840708090a")).unwrap();
        assert_eq!(
            mismatch.kind,
            CardanoHandshakeResponseKind::RefuseVersionMismatch {
                supported_versions: vec![7, 8, 9, 10]
            }
        );
        assert_eq!(
            mismatch.summary_line(),
            "handshake_response kind=refuse_version_mismatch version=none leios_overlay_min_version=15 leios_overlay_negotiated=none supported_versions=4 supported_min_version=7 supported_max_version=10 supported_leios_overlay_capable=false message_present=false production_ready=false"
        );

        let decode = parse_cardano_handshake_response(&bytes("820283010163666f6f")).unwrap();
        assert_eq!(
            decode.kind,
            CardanoHandshakeResponseKind::RefuseDecodeError {
                version: 1,
                message: "foo".to_string()
            }
        );
        assert_eq!(
            decode.summary_line(),
            "handshake_response kind=refuse_decode_error version=1 leios_overlay_min_version=15 leios_overlay_negotiated=none supported_versions=0 supported_min_version=0 supported_max_version=0 supported_leios_overlay_capable=none message_present=true production_ready=false"
        );

        let refused = parse_cardano_handshake_response(&bytes("820283020163666f6f")).unwrap();
        assert_eq!(
            refused.kind,
            CardanoHandshakeResponseKind::RefuseRefused {
                version: 1,
                message: "foo".to_string()
            }
        );
    }

    #[test]
    fn cardano_handshake_response_parser_rejects_bad_shapes() {
        assert_eq!(
            parse_cardano_mux_frame(&[0; CARDANO_MUX_HEADER_BYTES - 1]),
            Err(NetworkError::InvalidHandshakeProposal(
                "mux frame shorter than header"
            ))
        );
        assert_eq!(
            parse_cardano_handshake_response(&bytes("8101")),
            Err(NetworkError::InvalidHandshakeProposal(
                "accept-version message must have three fields"
            ))
        );
        assert_eq!(
            parse_cardano_handshake_response(&bytes("830118ff8202f4")),
            Err(NetworkError::InvalidHandshakeProposal(
                "unsupported Cardano NtN handshake version"
            ))
        );
    }

    #[test]
    fn cardano_handshake_negotiation_accepts_matching_preview_protocol_vector() {
        let profile = network_profile("preview").unwrap();
        let response = parse_cardano_handshake_response(&bytes("83010a8202f4")).unwrap();
        let report = cardano_handshake_negotiation_report(
            profile,
            &CARDANO_NTN_SUPPORTED_VERSIONS,
            &response,
        )
        .unwrap();

        assert_eq!(report.profile.name, "preview");
        assert_eq!(
            report.proposed_versions,
            CARDANO_NTN_SUPPORTED_VERSIONS.to_vec()
        );
        assert!(!report.production_ready);
        assert_eq!(
            report.outcome,
            CardanoHandshakeNegotiationOutcome::Accepted {
                version: 10,
                network_magic: 2,
                diffusion_mode: CardanoNtNDiffusionMode::InitiatorAndResponder,
                peer_sharing: false,
                query: false,
            }
        );
        assert_eq!(report.event_batch().len(), 1);
        assert_eq!(
            report.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_NEGOTIATION_EVENT
        );
        assert_eq!(
            report.summary_line(),
            "handshake_negotiation network=preview versions=9 outcome=accepted accepted_version=10 leios_overlay_min_version=15 leios_overlay_negotiated=false refused_supported_versions=0 refused_supported_min_version=0 refused_supported_max_version=0 refused_supported_leios_overlay_capable=none production_ready=false"
        );
    }

    #[test]
    fn cardano_handshake_negotiation_rejects_mismatched_magic_and_unproposed_accept() {
        let response = parse_cardano_handshake_response(&bytes("83010a8202f4")).unwrap();
        assert_eq!(
            cardano_handshake_negotiation_report(
                network_profile("preprod").unwrap(),
                &CARDANO_NTN_SUPPORTED_VERSIONS,
                &response,
            ),
            Err(NetworkError::NetworkMismatch {
                local: 1,
                remote: 2
            })
        );
        assert_eq!(
            cardano_handshake_negotiation_report(
                network_profile("preview").unwrap(),
                &[7, 8, 9],
                &response,
            ),
            Err(NetworkError::InvalidHandshakeProposal(
                "accepted version was not proposed"
            ))
        );
    }

    #[test]
    fn cardano_handshake_negotiation_reports_refusals_without_live_reads() {
        let response = parse_cardano_handshake_response(&bytes("82028200840708090a")).unwrap();
        let report = cardano_handshake_negotiation_report(
            network_profile("preview").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
            &response,
        )
        .unwrap();

        assert_eq!(
            report.outcome,
            CardanoHandshakeNegotiationOutcome::Refused {
                reason: CardanoHandshakeRefusalReason::VersionMismatch,
                version: None,
                supported_versions: vec![7, 8, 9, 10],
                message: None,
            }
        );
        assert_eq!(
            report.summary_line(),
            "handshake_negotiation network=preview versions=9 outcome=refused accepted_version=none leios_overlay_min_version=15 leios_overlay_negotiated=none refused_supported_versions=4 refused_supported_min_version=7 refused_supported_max_version=10 refused_supported_leios_overlay_capable=false production_ready=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_state_machine_matches_protocol_plan() {
        let plan = cardano_ntn_handshake_state_machine_plan();

        assert!(!plan.production_ready);
        assert!(!plan.live_integrated);
        assert_eq!(plan.entries.len(), 3);
        assert_eq!(plan.entries[0].state, CardanoHandshakeState::Propose);
        assert_eq!(plan.entries[0].agency, CardanoHandshakeAgency::Client);
        assert_eq!(plan.entries[0].timeout_secs, Some(10));
        assert_eq!(plan.entries[1].state, CardanoHandshakeState::Confirm);
        assert_eq!(plan.entries[1].agency, CardanoHandshakeAgency::Server);
        assert_eq!(plan.entries[1].timeout_secs, Some(10));
        assert_eq!(plan.entries[1].transitions.len(), 3);
        assert_eq!(plan.entries[2].state, CardanoHandshakeState::Done);
        assert_eq!(plan.entries[2].agency, CardanoHandshakeAgency::None);
        assert_eq!(plan.entries[2].timeout_secs, None);
        assert!(plan.entries[2].transitions.is_empty());
        assert_eq!(plan.transition_count(), 4);
        assert_eq!(plan.timeout_state_count(), 2);
        assert_eq!(plan.event_batch().len(), 1);
        assert_eq!(
            plan.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_STATE_MACHINE_EVENT
        );
        assert_eq!(
            plan.summary_line(),
            "handshake_state_machine states=3 transitions=4 timeout_states=2 live_integrated=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_state_machine_validates_protocol_transitions() {
        let plan = cardano_ntn_handshake_state_machine_plan();

        assert_eq!(
            plan.next_state(
                CardanoHandshakeState::Propose,
                CardanoHandshakeMessageType::ProposeVersions,
            ),
            Ok(CardanoHandshakeState::Confirm)
        );
        assert_eq!(
            plan.next_state(
                CardanoHandshakeState::Confirm,
                CardanoHandshakeMessageType::AcceptVersion,
            ),
            Ok(CardanoHandshakeState::Done)
        );
        assert_eq!(
            plan.next_state(
                CardanoHandshakeState::Confirm,
                CardanoHandshakeMessageType::Refuse,
            ),
            Ok(CardanoHandshakeState::Done)
        );
        assert_eq!(
            plan.next_state(
                CardanoHandshakeState::Confirm,
                CardanoHandshakeMessageType::QueryReply,
            ),
            Ok(CardanoHandshakeState::Done)
        );
        assert_eq!(
            plan.next_state(
                CardanoHandshakeState::Done,
                CardanoHandshakeMessageType::AcceptVersion,
            ),
            Err(NetworkError::InvalidHandshakeProposal(
                "invalid handshake state transition"
            ))
        );
    }

    #[test]
    fn cardano_ntn_handshake_accept_protocol_vector_matches_profile_magic() {
        let preview =
            cardano_ntn_handshake_accept_protocol_vector(network_profile("preview").unwrap(), 10)
                .unwrap();
        let preprod =
            cardano_ntn_handshake_accept_protocol_vector(network_profile("preprod").unwrap(), 10)
                .unwrap();

        assert_eq!(preview.profile.name, "preview");
        assert_eq!(preview.version, 10);
        assert_eq!(preview.protocol_id, CARDANO_HANDSHAKE_PROTOCOL_ID);
        assert_eq!(preview.message_type, 1);
        assert!(!preview.production_ready);
        assert_eq!(hex(&preview.encoded), "83010a8202f4");
        assert_eq!(hex(&preprod.encoded), "83010a8201f4");
        assert_eq!(preview.event_batch().len(), 1);
        assert_eq!(
            preview.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_ACCEPT_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            preview.summary_line(),
            "handshake_accept_protocol_vector network=preview version=10 encoded_bytes=6 protocol_id=0 message_type=1 production_ready=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_refusal_protocol_vector_matches_expected_bytes() {
        let refusal = cardano_ntn_handshake_version_mismatch_refusal_protocol_vector(
            &CARDANO_NTN_REFUSAL_SUPPORTED_VERSIONS,
        )
        .unwrap();

        assert_eq!(refusal.supported_versions, vec![7, 8, 9, 10]);
        assert_eq!(refusal.protocol_id, CARDANO_HANDSHAKE_PROTOCOL_ID);
        assert_eq!(refusal.message_type, 2);
        assert!(!refusal.production_ready);
        assert_eq!(hex(&refusal.encoded), "82028200840708090a");
        assert_eq!(refusal.event_batch().len(), 1);
        assert_eq!(
            refusal.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_REFUSAL_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            refusal.summary_line(),
            "handshake_refusal_protocol_vector supported_versions=4 supported_min_version=7 supported_max_version=10 leios_overlay_min_version=15 supported_leios_overlay_capable=false encoded_bytes=9 protocol_id=0 message_type=2 production_ready=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_harness_runs_accept_protocol_vector_without_io() {
        let profile = network_profile("preview").unwrap();
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

        assert_eq!(run.profile.name, "preview");
        assert_eq!(
            run.proposed_versions,
            CARDANO_NTN_SUPPORTED_VERSIONS.to_vec()
        );
        assert_eq!(
            run.states,
            vec![
                CardanoHandshakeState::Propose,
                CardanoHandshakeState::Confirm,
                CardanoHandshakeState::Done,
            ]
        );
        assert_eq!(
            run.messages,
            vec![
                CardanoHandshakeMessageType::ProposeVersions,
                CardanoHandshakeMessageType::AcceptVersion,
            ]
        );
        assert_eq!(
            run.proposal_frame.encoded.len(),
            CARDANO_MUX_HEADER_BYTES + 49
        );
        assert_eq!(run.response_frame.payload_length, 6);
        assert!(run.response_frame.is_response);
        assert!(!run.production_ready);
        assert!(!run.live_integrated);
        assert_eq!(
            run.negotiation.outcome,
            CardanoHandshakeNegotiationOutcome::Accepted {
                version: 10,
                network_magic: 2,
                diffusion_mode: CardanoNtNDiffusionMode::InitiatorAndResponder,
                peer_sharing: false,
                query: false,
            }
        );
        assert_eq!(run.event_batch().len(), 1);
        assert_eq!(
            run.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_HARNESS_EVENT
        );
        assert_eq!(
            run.summary_line(),
            "handshake_harness network=preview versions=9 states=3 messages=2 final_state=done outcome=accepted leios_overlay_min_version=15 leios_overlay_negotiated=false production_ready=false live_integrated=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_harness_reports_refusal_without_io() {
        let response_frame = cardano_mux_frame_protocol_vector(
            CARDANO_HANDSHAKE_PROTOCOL_ID,
            &bytes("82028200840708090a"),
            true,
            0,
        )
        .unwrap();
        let run = run_cardano_ntn_handshake_harness(
            network_profile("preview").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
            &response_frame.encoded,
        )
        .unwrap();

        assert_eq!(
            run.messages,
            vec![
                CardanoHandshakeMessageType::ProposeVersions,
                CardanoHandshakeMessageType::Refuse,
            ]
        );
        assert_eq!(
            run.negotiation.outcome,
            CardanoHandshakeNegotiationOutcome::Refused {
                reason: CardanoHandshakeRefusalReason::VersionMismatch,
                version: None,
                supported_versions: vec![7, 8, 9, 10],
                message: None,
            }
        );
        assert_eq!(
            run.summary_line(),
            "handshake_harness network=preview versions=9 states=3 messages=2 final_state=done outcome=refused leios_overlay_min_version=15 leios_overlay_negotiated=none production_ready=false live_integrated=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_harness_rejects_non_response_frames() {
        let profile = network_profile("preview").unwrap();
        let accept = cardano_ntn_handshake_accept_protocol_vector(profile, 10).unwrap();
        let request_frame =
            cardano_mux_frame_protocol_vector(accept.protocol_id, &accept.encoded, false, 0)
                .unwrap();

        assert_eq!(
            run_cardano_ntn_handshake_harness(
                profile,
                &CARDANO_NTN_SUPPORTED_VERSIONS,
                &request_frame.encoded,
            ),
            Err(NetworkError::InvalidHandshakeProposal(
                "handshake response frame must set response flag"
            ))
        );
    }

    #[test]
    fn cardano_ntn_handshake_transcript_protocol_vector_records_frames_without_io() {
        let transcript = cardano_ntn_handshake_transcript_protocol_vector(
            network_profile("preprod").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();

        assert_eq!(transcript.profile.name, "preprod");
        assert_eq!(transcript.accepted_version, 10);
        assert_eq!(transcript.frame_count, 2);
        assert_eq!(transcript.request_frame.encoded.len(), 57);
        assert_eq!(transcript.response_frame.encoded.len(), 14);
        assert_eq!(transcript.total_bytes, 71);
        assert_eq!(
            transcript.request_frame.protocol_id,
            CARDANO_HANDSHAKE_PROTOCOL_ID
        );
        assert!(!transcript.request_frame.is_response);
        assert_eq!(
            transcript.response_frame.protocol_id,
            CARDANO_HANDSHAKE_PROTOCOL_ID
        );
        assert!(transcript.response_frame.is_response);
        assert!(!transcript.production_ready);
        assert!(!transcript.live_integrated);
        assert_eq!(
            transcript.harness.states,
            vec![
                CardanoHandshakeState::Propose,
                CardanoHandshakeState::Confirm,
                CardanoHandshakeState::Done,
            ]
        );
        assert_eq!(transcript.event_batch().len(), 1);
        assert_eq!(
            transcript.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_TRANSCRIPT_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            transcript.summary_line(),
            "handshake_transcript_protocol_vector network=preprod versions=9 frames=2 total_bytes=71 accepted_version=10 leios_overlay_min_version=15 leios_overlay_negotiated=false production_ready=false live_integrated=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_transcript_replay_parses_concatenated_frames_without_io() {
        let replay = cardano_ntn_handshake_transcript_replay(
            network_profile("preprod").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();

        assert_eq!(replay.profile.name, "preprod");
        assert_eq!(
            replay.proposed_versions,
            CARDANO_NTN_SUPPORTED_VERSIONS.to_vec()
        );
        assert_eq!(replay.frames.len(), 2);
        assert_eq!(replay.request_frames, 1);
        assert_eq!(replay.response_frames, 1);
        assert_eq!(replay.total_bytes, 71);
        assert_eq!(replay.accepted_version, 10);
        assert_eq!(replay.final_state, CardanoHandshakeState::Done);
        assert!(!replay.production_ready);
        assert!(!replay.live_integrated);
        assert_eq!(replay.frames[0].payload_length, 49);
        assert!(!replay.frames[0].is_response);
        assert_eq!(replay.frames[1].payload_length, 6);
        assert!(replay.frames[1].is_response);
        assert_eq!(replay.event_batch().len(), 1);
        assert_eq!(
            replay.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_TRANSCRIPT_REPLAY_EVENT
        );
        assert_eq!(
            replay.summary_line(),
            "handshake_transcript_replay network=preprod versions=9 frames=2 request_frames=1 response_frames=1 total_bytes=71 accepted_version=10 leios_overlay_min_version=15 leios_overlay_negotiated=false final_state=done production_ready=false live_integrated=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_refusal_transcript_replay_reaches_done_without_io() {
        let replay = cardano_ntn_handshake_refusal_transcript_replay(
            network_profile("preview").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
            &CARDANO_NTN_REFUSAL_SUPPORTED_VERSIONS,
        )
        .unwrap();

        assert_eq!(replay.profile.name, "preview");
        assert_eq!(
            replay.proposed_versions,
            CARDANO_NTN_SUPPORTED_VERSIONS.to_vec()
        );
        assert_eq!(replay.frames.len(), 2);
        assert_eq!(replay.request_frames, 1);
        assert_eq!(replay.response_frames, 1);
        assert_eq!(replay.total_bytes, 74);
        assert_eq!(replay.final_state, CardanoHandshakeState::Done);
        assert_eq!(
            replay.refusal_reason,
            CardanoHandshakeRefusalReason::VersionMismatch
        );
        assert_eq!(replay.supported_versions, vec![7, 8, 9, 10]);
        assert!(!replay.production_ready);
        assert!(!replay.live_integrated);
        assert_eq!(replay.frames[0].payload_length, 49);
        assert!(!replay.frames[0].is_response);
        assert_eq!(replay.frames[1].payload_length, 9);
        assert!(replay.frames[1].is_response);
        assert_eq!(replay.event_batch().len(), 1);
        assert_eq!(
            replay.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_REFUSAL_TRANSCRIPT_REPLAY_EVENT
        );
        assert_eq!(
            replay.summary_line(),
            "handshake_refusal_transcript_replay network=preview versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true frames=2 request_frames=1 response_frames=1 total_bytes=74 final_state=done refusal_reason=version_mismatch supported_versions=4 supported_min_version=7 supported_max_version=10 supported_leios_overlay_capable=false production_ready=false live_integrated=false"
        );
    }

    #[test]
    fn cardano_mux_frame_stream_parser_rejects_empty_and_over_cap_inputs() {
        let transcript = cardano_ntn_handshake_transcript_protocol_vector(
            network_profile("preview").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();
        let mut stream = Vec::new();
        stream.extend_from_slice(&transcript.request_frame.encoded);
        stream.extend_from_slice(&transcript.response_frame.encoded);
        let frames = parse_cardano_mux_frame_stream(&stream, 2).unwrap();
        let summary = cardano_mux_frame_stream_summary(&frames).unwrap();

        assert_eq!(summary.frame_count, 2);
        assert_eq!(summary.request_frames, 1);
        assert_eq!(summary.response_frames, 1);
        assert_eq!(summary.total_payload_bytes, 55);
        assert_eq!(summary.total_frame_bytes, 71);
        assert_eq!(summary.protocol_count, 1);
        assert!(!summary.production_ready);
        assert!(!summary.live_integrated);
        assert_eq!(summary.event_batch().len(), 1);
        assert_eq!(
            summary.event_batch()[0].name.as_str(),
            NETWORK_MUX_FRAME_STREAM_EVENT
        );
        assert_eq!(
            summary.summary_line(),
            "mux_frame_stream frames=2 request_frames=1 response_frames=1 payload_bytes=55 frame_bytes=71 protocols=1 production_ready=false live_integrated=false"
        );

        assert_eq!(
            parse_cardano_mux_frame_stream(&[], 2),
            Err(NetworkError::InvalidHandshakeProposal(
                "mux frame stream must be non-empty"
            ))
        );
        assert_eq!(
            cardano_mux_frame_stream_summary(&[]),
            Err(NetworkError::InvalidHandshakeProposal(
                "mux frame stream summary must be non-empty"
            ))
        );
        assert_eq!(
            parse_cardano_mux_frame_stream(&stream, 1),
            Err(NetworkError::InvalidHandshakeProposal(
                "mux frame stream exceeds frame cap"
            ))
        );
    }

    #[test]
    fn cardano_ntn_handshake_timeout_protocol_vector_reports_expiry_without_io() {
        let confirm = cardano_ntn_handshake_timeout_protocol_vector(
            CardanoHandshakeState::Confirm,
            CARDANO_NTN_HANDSHAKE_CONFIRM_TIMEOUT_SECS,
        )
        .unwrap();
        let propose = cardano_ntn_handshake_timeout_protocol_vector(
            CardanoHandshakeState::Propose,
            CARDANO_NTN_HANDSHAKE_PROPOSE_TIMEOUT_SECS - 1,
        )
        .unwrap();

        assert_eq!(confirm.state, CardanoHandshakeState::Confirm);
        assert_eq!(confirm.agency, CardanoHandshakeAgency::Server);
        assert_eq!(confirm.timeout_secs, 10);
        assert_eq!(confirm.elapsed_secs, 10);
        assert!(confirm.timed_out);
        assert!(!confirm.production_ready);
        assert!(!confirm.live_integrated);
        assert_eq!(propose.state, CardanoHandshakeState::Propose);
        assert_eq!(propose.agency, CardanoHandshakeAgency::Client);
        assert_eq!(propose.elapsed_secs, 9);
        assert!(!propose.timed_out);
        assert_eq!(confirm.event_batch().len(), 1);
        assert_eq!(
            confirm.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_TIMEOUT_PROTOCOL_VECTOR_EVENT
        );
        assert_eq!(
            confirm.summary_line(),
            "handshake_timeout_protocol_vector state=confirm agency=server timeout_secs=10 elapsed_secs=10 timed_out=true production_ready=false live_integrated=false"
        );
        assert_eq!(
            propose.summary_line(),
            "handshake_timeout_protocol_vector state=propose agency=client timeout_secs=10 elapsed_secs=9 timed_out=false production_ready=false live_integrated=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_timeout_protocol_vector_rejects_done_state() {
        assert_eq!(
            cardano_ntn_handshake_timeout_protocol_vector(CardanoHandshakeState::Done, 10),
            Err(NetworkError::InvalidHandshakeProposal(
                "handshake state has no timeout"
            ))
        );
    }

    #[test]
    fn cardano_ntn_handshake_error_protocol_vectors_match_expected_failures_without_io() {
        let report = cardano_ntn_handshake_error_protocol_vector_report(
            network_profile("preview").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();

        assert_eq!(report.profile.name, "preview");
        assert_eq!(
            report.proposed_versions,
            CARDANO_NTN_SUPPORTED_VERSIONS.to_vec()
        );
        assert!(!report.production_ready);
        assert!(!report.live_integrated);
        assert_eq!(report.cases.len(), 3);
        assert!(report.cases.iter().all(|case| case.matched));
        assert_eq!(
            report
                .cases
                .iter()
                .map(|case| case.kind)
                .collect::<Vec<_>>(),
            vec![
                CardanoHandshakeErrorProtocolVectorKind::WrongProtocolId,
                CardanoHandshakeErrorProtocolVectorKind::NonResponseFrame,
                CardanoHandshakeErrorProtocolVectorKind::MalformedCbor,
            ]
        );
        assert_eq!(
            report.cases[0].observed_error,
            NetworkError::InvalidHandshakeProposal(
                "handshake response frame uses unexpected protocol id"
            )
        );
        assert_eq!(report.cases[0].event_batch().len(), 1);
        assert_eq!(
            report.cases[0].event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_ERROR_PROTOCOL_VECTOR_CASE_EVENT
        );
        assert_eq!(
            report.cases[0].summary_line(),
            "handshake_error_protocol_vector_case kind=wrong_protocol_id matched=true"
        );
        assert_eq!(
            report.cases[1].observed_error,
            NetworkError::InvalidHandshakeProposal(
                "handshake response frame must set response flag"
            )
        );
        assert_eq!(
            report.cases[2].observed_error,
            NetworkError::InvalidHandshakeProposal("truncated CBOR")
        );
        assert_eq!(report.matched_cases(), 3);
        assert_eq!(report.event_batch().len(), 1);
        assert_eq!(
            report.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_ERROR_PROTOCOL_VECTORS_EVENT
        );
        assert_eq!(
            report.summary_line(),
            "handshake_error_protocol_vectors network=preview versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true cases=3 matched=3 live_integrated=false"
        );
    }

    #[test]
    fn cardano_ntn_handshake_conformance_report_keeps_live_gate_closed() {
        let report = cardano_ntn_handshake_conformance_report(
            network_profile("preprod").unwrap(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();

        assert_eq!(report.profile.name, "preprod");
        assert_eq!(
            report.proposed_versions,
            CARDANO_NTN_SUPPORTED_VERSIONS.to_vec()
        );
        assert_eq!(report.offline_checks, 9);
        assert_eq!(report.passed_checks, 9);
        assert!(report.offline_complete);
        assert!(!report.live_ready);
        assert_eq!(report.blockers.len(), 3);
        assert!(report.blockers.contains(&"live mux loop is not integrated"));
        assert_eq!(
            report.action_items(),
            vec![
                "integrate reviewed live mux loop",
                "complete network path review",
                "complete full protocol conformance harness"
            ]
        );
        assert!(!report.production_ready);
        assert!(!report.live_integrated);
        assert_eq!(report.event_batch().len(), 1);
        assert_eq!(
            report.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_CONFORMANCE_EVENT
        );
        assert_eq!(
            report.summary_line(),
            "handshake_conformance network=preprod versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true passed=9/9 offline_complete=true live_ready=false blockers=3"
        );
    }

    #[test]
    fn testnet_handshake_conformance_matrix_covers_public_testnets_without_io() {
        let matrix = testnet_handshake_conformance_matrix(&CARDANO_NTN_SUPPORTED_VERSIONS).unwrap();

        assert_eq!(matrix.public_testnet_profiles, 2);
        assert_eq!(matrix.passed_profiles, 2);
        assert!(matrix.offline_complete);
        assert!(!matrix.live_ready);
        assert_eq!(matrix.blockers.len(), 3);
        assert_eq!(
            matrix.action_items(),
            vec![
                "integrate reviewed live mux loop",
                "complete network path review",
                "complete full protocol conformance harness"
            ]
        );
        assert!(!matrix.production_ready);
        assert!(!matrix.live_integrated);
        assert_eq!(
            matrix
                .reports
                .iter()
                .map(|report| report.profile.name)
                .collect::<Vec<_>>(),
            vec!["preprod", "preview"]
        );
        assert!(matrix.reports.iter().all(|report| report.offline_complete));
        assert!(matrix.reports.iter().all(|report| !report.live_ready));
        assert_eq!(matrix.event_batch().len(), 3);
        assert_eq!(
            matrix.event_batch()[0].name.as_str(),
            NETWORK_HANDSHAKE_CONFORMANCE_MATRIX_EVENT
        );
        assert_eq!(
            matrix.summary_line(),
            "handshake_conformance_matrix public_profiles=2 passed_profiles=2 offline_complete=true live_ready=false blockers=3"
        );
    }

    fn hex(bytes: &[u8]) -> String {
        let mut out = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            out.push(hex_char(byte >> 4));
            out.push(hex_char(byte & 0x0f));
        }
        out
    }

    fn hex_char(nibble: u8) -> char {
        match nibble {
            0..=9 => (b'0' + nibble) as char,
            10..=15 => (b'a' + nibble - 10) as char,
            _ => unreachable!("hex nibble must fit in four bits"),
        }
    }

    fn bytes(hex: &str) -> Vec<u8> {
        assert_eq!(hex.len() % 2, 0);
        hex.as_bytes()
            .chunks_exact(2)
            .map(|chunk| (hex_value(chunk[0]) << 4) | hex_value(chunk[1]))
            .collect()
    }

    fn hex_value(byte: u8) -> u8 {
        match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            b'A'..=b'F' => byte - b'A' + 10,
            _ => panic!("invalid hex byte"),
        }
    }

    #[test]
    fn testnet_tcp_probe_plan_accepts_single_public_testnet_peer() {
        let request = TestnetTcpProbeRequest::new("preview", "8.8.8.8:3001", true, 2);
        assert_eq!(request.event_batch().len(), 1);
        assert_eq!(
            request.event_batch()[0].name.as_str(),
            NETWORK_TESTNET_TCP_PROBE_REQUEST_EVENT
        );
        assert_eq!(
            request.summary_line(),
            "testnet_tcp_probe_request network=preview peer_supplied=true allow_live_testnet=true timeout_secs=2"
        );
        let plan = plan_testnet_tcp_probe(request, TestnetContactLimits::smoke_test()).unwrap();

        assert_eq!(plan.profile.name, "preview");
        assert_eq!(plan.peer.to_string(), "8.8.8.8:3001");
        assert_eq!(plan.timeout_secs, 2);
        assert_eq!(plan.event_batch().len(), 1);
        assert_eq!(
            plan.event_batch()[0].name.as_str(),
            NETWORK_TESTNET_TCP_PROBE_PLAN_EVENT
        );
        assert_eq!(
            plan.summary_line(),
            "testnet_tcp_probe_plan network=preview peer_port=3001 timeout_secs=2 tcp_only=true retries=0 protocol_bytes=false"
        );
    }

    #[test]
    fn testnet_tcp_probe_plan_requires_explicit_live_opt_in() {
        assert_eq!(
            plan_testnet_tcp_probe(
                TestnetTcpProbeRequest::new("preview", "8.8.8.8:3001", false, 2),
                TestnetContactLimits::smoke_test(),
            ),
            Err(NetworkError::TestnetContactBlocked(
                "live testnet TCP probe requires --allow-live-testnet".to_string()
            ))
        );
    }

    #[test]
    fn testnet_tcp_probe_plan_rejects_mainnet_local_and_unsafe_peer() {
        let limits = TestnetContactLimits::smoke_test();
        assert_eq!(
            plan_testnet_tcp_probe(
                TestnetTcpProbeRequest::new("mainnet", "8.8.8.8:3001", true, 2),
                limits,
            ),
            Err(NetworkError::TestnetContactBlocked(
                "mainnet contact is blocked".to_string()
            ))
        );
        assert_eq!(
            plan_testnet_tcp_probe(
                TestnetTcpProbeRequest::new("local", "8.8.8.8:3001", true, 2),
                limits,
            ),
            Err(NetworkError::TestnetContactBlocked(
                "live contact is limited to public testnet profiles".to_string()
            ))
        );
        assert_eq!(
            plan_testnet_tcp_probe(
                TestnetTcpProbeRequest::new("preview", "127.0.0.1:3001", true, 2),
                limits,
            ),
            Err(NetworkError::TestnetContactBlocked(
                "testnet TCP probe peer must be a public IP address".to_string()
            ))
        );
    }

    #[test]
    fn testnet_tcp_probe_plan_rejects_dns_wrong_port_and_large_timeout() {
        let limits = TestnetContactLimits::smoke_test();
        assert_eq!(
            plan_testnet_tcp_probe(
                TestnetTcpProbeRequest::new("preview", "relay.example.com:3001", true, 2),
                limits,
            ),
            Err(NetworkError::TestnetContactBlocked(
                "testnet TCP probe requires literal ip:port peer".to_string()
            ))
        );
        assert_eq!(
            plan_testnet_tcp_probe(
                TestnetTcpProbeRequest::new("preview", "8.8.8.8:1", true, 2),
                limits,
            ),
            Err(NetworkError::TestnetContactBlocked(
                "testnet TCP probe port must match profile default 3001".to_string()
            ))
        );
        assert_eq!(
            plan_testnet_tcp_probe(
                TestnetTcpProbeRequest::new("preview", "8.8.8.8:3001", true, 6),
                limits,
            ),
            Err(NetworkError::TestnetLimitExceeded {
                limit: "probe_timeout_secs",
                requested: 6,
                max: MAX_TESTNET_TCP_PROBE_TIMEOUT_SECS,
            })
        );
    }

    #[test]
    fn testnet_handshake_probe_plan_accepts_single_public_testnet_peer() {
        let request = TestnetHandshakeProbeRequest::new("preview", "8.8.8.8:3001", true, 2);
        assert_eq!(request.event_batch().len(), 1);
        assert_eq!(
            request.event_batch()[0].name.as_str(),
            NETWORK_TESTNET_HANDSHAKE_PROBE_REQUEST_EVENT
        );
        assert_eq!(
            request.summary_line(),
            "testnet_handshake_probe_request network=preview peer_supplied=true allow_live_testnet=true timeout_secs=2"
        );
        let plan = plan_testnet_handshake_probe(
            request,
            TestnetContactLimits::smoke_test(),
            &CARDANO_NTN_SUPPORTED_VERSIONS,
        )
        .unwrap();

        assert_eq!(plan.profile.name, "preview");
        assert_eq!(plan.peer.to_string(), "8.8.8.8:3001");
        assert_eq!(plan.timeout_secs, 2);
        assert_eq!(
            plan.proposed_versions,
            CARDANO_NTN_SUPPORTED_VERSIONS.to_vec()
        );
        assert_eq!(
            plan.request_frame.protocol_id,
            CARDANO_HANDSHAKE_PROTOCOL_ID
        );
        assert!(!plan.request_frame.is_response);
        assert_eq!(
            plan.request_frame.encoded.len(),
            CARDANO_MUX_HEADER_BYTES + 49
        );
        assert_eq!(
            plan.max_response_bytes,
            MAX_TESTNET_HANDSHAKE_PROBE_RESPONSE_BYTES
        );
        assert_eq!(plan.event_batch().len(), 1);
        assert_eq!(
            plan.event_batch()[0].name.as_str(),
            NETWORK_TESTNET_HANDSHAKE_PROBE_PLAN_EVENT
        );
        assert_eq!(
            plan.summary_line(),
            "testnet_handshake_probe_plan network=preview peer_port=3001 versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true timeout_secs=2 dials=1 retries=0 write_frames=1 read_frames=1 max_response_bytes=1024 mainnet_allowed=false"
        );
    }

    #[test]
    fn testnet_handshake_probe_plan_requires_explicit_live_opt_in() {
        assert_eq!(
            plan_testnet_handshake_probe(
                TestnetHandshakeProbeRequest::new("preview", "8.8.8.8:3001", false, 2),
                TestnetContactLimits::smoke_test(),
                &CARDANO_NTN_SUPPORTED_VERSIONS,
            ),
            Err(NetworkError::TestnetContactBlocked(
                "live testnet handshake probe requires --allow-live-testnet".to_string()
            ))
        );
    }

    #[test]
    fn bounded_testnet_contact_plan_rejects_mainnet_and_local() {
        let limits = TestnetContactLimits::smoke_test();
        assert_eq!(
            plan_bounded_testnet_contact(TestnetContactRequest::new("mainnet", 1, 1, 1), limits),
            Err(NetworkError::TestnetContactBlocked(
                "mainnet contact is blocked".to_string()
            ))
        );
        assert_eq!(
            plan_bounded_testnet_contact(TestnetContactRequest::new("local", 1, 1, 1), limits),
            Err(NetworkError::TestnetContactBlocked(
                "live contact is limited to public testnet profiles".to_string()
            ))
        );
    }

    #[test]
    fn bounded_testnet_contact_plan_requires_opt_in_and_finite_limits() {
        assert_eq!(
            plan_bounded_testnet_contact(
                TestnetContactRequest::new("preview", 1, 1, 1),
                TestnetContactLimits {
                    allow_testnet: false,
                    ..TestnetContactLimits::smoke_test()
                }
            ),
            Err(NetworkError::TestnetContactBlocked(
                "testnet contact requires explicit opt-in".to_string()
            ))
        );
        assert_eq!(
            plan_bounded_testnet_contact(
                TestnetContactRequest::new("preview", 1, 1, 1),
                TestnetContactLimits {
                    max_bytes: 0,
                    ..TestnetContactLimits::smoke_test()
                }
            ),
            Err(NetworkError::UnboundedTestnetLimit("max_bytes"))
        );
    }

    #[test]
    fn bounded_testnet_contact_plan_rejects_empty_requests() {
        let limits = TestnetContactLimits::smoke_test();
        assert_eq!(
            plan_bounded_testnet_contact(TestnetContactRequest::new("preview", 0, 1, 1), limits),
            Err(NetworkError::EmptyTestnetRequest("blocks"))
        );
        assert_eq!(
            plan_bounded_testnet_contact(TestnetContactRequest::new("preview", 1, 0, 1), limits),
            Err(NetworkError::EmptyTestnetRequest("slots"))
        );
        assert_eq!(
            plan_bounded_testnet_contact(TestnetContactRequest::new("preview", 1, 1, 0), limits),
            Err(NetworkError::EmptyTestnetRequest("bytes"))
        );
        assert_eq!(
            plan_bounded_testnet_contact(TestnetContactRequest::new("mainnet", 0, 0, 0), limits),
            Err(NetworkError::TestnetContactBlocked(
                "mainnet contact is blocked".to_string()
            ))
        );
    }

    #[test]
    fn bounded_testnet_contact_plan_rejects_large_requests() {
        assert_eq!(
            plan_bounded_testnet_contact(
                TestnetContactRequest::new("preview", 33, 100, 1024),
                TestnetContactLimits::smoke_test(),
            ),
            Err(NetworkError::TestnetLimitExceeded {
                limit: "blocks",
                requested: 33,
                max: 32,
            })
        );
        assert_eq!(
            plan_bounded_testnet_contact(
                TestnetContactRequest::new("preview", 1, 100, 2048),
                TestnetContactLimits {
                    max_bytes: 4096,
                    temp_storage_bytes: 1024,
                    ..TestnetContactLimits::smoke_test()
                },
            ),
            Err(NetworkError::TestnetLimitExceeded {
                limit: "temp_bytes",
                requested: 2048,
                max: 1024,
            })
        );
    }
}
