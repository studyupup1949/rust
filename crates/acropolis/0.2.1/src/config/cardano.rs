use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::NetworkProfile;

pub const MAX_CARDANO_NODE_CONFIG_SIZE: usize = 10 * 1024 * 1024;
pub const MAX_CARDANO_GENESIS_FILE_SIZE: usize = 10 * 1024 * 1024;

const BLAKE2B_IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];

const BLAKE2B_SIGMA: [[usize; 16]; 12] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiresNetworkMagic {
    RequiresMagic,
    RequiresNoMagic,
}

impl fmt::Display for RequiresNetworkMagic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequiresMagic => f.write_str("RequiresMagic"),
            Self::RequiresNoMagic => f.write_str("RequiresNoMagic"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenesisFileFixture {
    pub era: String,
    pub file: String,
    pub expected_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenesisFileDigest {
    pub era: String,
    pub hash: String,
}

impl GenesisFileDigest {
    pub fn new(era: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            era: era.into(),
            hash: hash.into(),
        }
    }
}

pub fn raw_genesis_file_digest_supported(era: &str) -> bool {
    !era.eq_ignore_ascii_case("Byron")
}

pub fn blake2b_256_hex(input: &[u8]) -> String {
    bytes_to_lower_hex(&blake2b_256(input))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenesisHashCheck {
    pub era: String,
    pub file: String,
    pub expected_hash: Option<String>,
    pub actual_hash: Option<String>,
    pub status: GenesisHashStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenesisHashStatus {
    Match,
    Mismatch,
    MissingExpected,
    MissingActual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenesisHashReport {
    pub checks: Vec<GenesisHashCheck>,
}

impl GenesisHashReport {
    pub fn all_matched(&self) -> bool {
        !self.checks.is_empty()
            && self
                .checks
                .iter()
                .all(|check| check.status == GenesisHashStatus::Match)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkMagicRequirementCheck {
    pub profile_name: String,
    pub expected: RequiresNetworkMagic,
    pub actual: Option<RequiresNetworkMagic>,
    pub status: NetworkMagicRequirementStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMagicRequirementStatus {
    Match,
    Mismatch,
    MissingConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardanoNodeConfigManifest {
    pub protocol: Option<String>,
    pub consensus_mode: Option<String>,
    pub protocol_features: ProtocolFeatureConfig,
    pub application: ApplicationConfig,
    pub test_hard_forks: TestHardForkConfig,
    pub max_known_major_protocol_version: Option<u64>,
    pub min_node_version: Option<String>,
    pub requires_network_magic: Option<RequiresNetworkMagic>,
    pub ledger_db: LedgerDbConfig,
    pub byron_protocol: ByronProtocolConfig,
    pub checkpoints: CheckpointsConfig,
    pub p2p_governor: P2pGovernorConfig,
    pub mempool: MempoolConfig,
    pub tracing: TraceConfig,
    pub genesis_files: Vec<GenesisFileFixture>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ApplicationConfig {
    pub name: Option<String>,
    pub version: Option<u64>,
    pub max_concurrency_deadline: Option<u64>,
}

impl ApplicationConfig {
    pub fn configured_count(&self) -> usize {
        [
            self.name.as_ref().map(|_| ()),
            self.version.map(|_| ()),
            self.max_concurrency_deadline.map(|_| ()),
        ]
        .into_iter()
        .filter(Option::is_some)
        .count()
    }

    pub fn config_complete(&self) -> bool {
        self.configured_count() == 3
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProtocolFeatureConfig {
    pub experimental_hard_forks_enabled: Option<bool>,
    pub experimental_protocols_enabled: Option<bool>,
}

impl ProtocolFeatureConfig {
    pub fn configured_count(&self) -> usize {
        [
            self.experimental_hard_forks_enabled.map(|_| ()),
            self.experimental_protocols_enabled.map(|_| ()),
        ]
        .into_iter()
        .filter(Option::is_some)
        .count()
    }

    pub fn config_complete(&self) -> bool {
        self.configured_count() == 2
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestHardForkConfig {
    pub shelley_epoch: Option<u64>,
    pub allegra_epoch: Option<u64>,
    pub mary_epoch: Option<u64>,
    pub alonzo_epoch: Option<u64>,
}

impl TestHardForkConfig {
    pub fn configured_count(&self) -> usize {
        [
            self.shelley_epoch,
            self.allegra_epoch,
            self.mary_epoch,
            self.alonzo_epoch,
        ]
        .into_iter()
        .filter(Option::is_some)
        .count()
    }

    pub fn config_complete(&self) -> bool {
        self.configured_count() == 4
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LedgerDbConfig {
    pub backend: Option<String>,
    pub num_disk_snapshots: Option<u64>,
    pub query_batch_size: Option<u64>,
    pub snapshot_interval: Option<u64>,
}

impl LedgerDbConfig {
    pub fn configured_count(&self) -> usize {
        [
            self.backend.as_ref().map(|_| ()),
            self.num_disk_snapshots.map(|_| ()),
            self.query_batch_size.map(|_| ()),
            self.snapshot_interval.map(|_| ()),
        ]
        .into_iter()
        .filter(Option::is_some)
        .count()
    }

    pub fn config_complete(&self) -> bool {
        self.configured_count() == 4
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ByronProtocolConfig {
    pub last_known_block_version_major: Option<u64>,
    pub last_known_block_version_minor: Option<u64>,
    pub last_known_block_version_alt: Option<u64>,
    pub pbft_signature_threshold: Option<String>,
}

impl ByronProtocolConfig {
    pub fn configured_count(&self) -> usize {
        [
            self.last_known_block_version_major.map(|_| ()),
            self.last_known_block_version_minor.map(|_| ()),
            self.last_known_block_version_alt.map(|_| ()),
            self.pbft_signature_threshold.as_ref().map(|_| ()),
        ]
        .into_iter()
        .filter(Option::is_some)
        .count()
    }

    pub fn config_complete(&self) -> bool {
        self.configured_count() == 4
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckpointsConfig {
    pub file: Option<String>,
    pub expected_hash: Option<String>,
}

impl CheckpointsConfig {
    pub fn configured_count(&self) -> usize {
        [self.file.as_ref(), self.expected_hash.as_ref()]
            .into_iter()
            .filter(Option::is_some)
            .count()
    }

    pub fn config_complete(&self) -> bool {
        self.configured_count() == 2
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct P2pGovernorConfig {
    pub deadline: PeerTargetConfig,
    pub sync: PeerTargetConfig,
    pub min_big_ledger_peers_for_trusted_state: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PeerTargetConfig {
    pub root_peers: Option<u64>,
    pub known_peers: Option<u64>,
    pub established_peers: Option<u64>,
    pub active_peers: Option<u64>,
    pub known_big_ledger_peers: Option<u64>,
    pub established_big_ledger_peers: Option<u64>,
    pub active_big_ledger_peers: Option<u64>,
}

impl PeerTargetConfig {
    pub fn configured_count(&self) -> usize {
        [
            self.root_peers,
            self.known_peers,
            self.established_peers,
            self.active_peers,
            self.known_big_ledger_peers,
            self.established_big_ledger_peers,
            self.active_big_ledger_peers,
        ]
        .into_iter()
        .filter(Option::is_some)
        .count()
    }

    pub fn config_complete(&self) -> bool {
        self.configured_count() == 7
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MempoolConfig {
    pub timeout_soft: Option<String>,
    pub timeout_hard: Option<String>,
    pub timeout_capacity: Option<String>,
}

impl MempoolConfig {
    pub fn timeout_configured_count(&self) -> usize {
        [
            self.timeout_soft.as_ref(),
            self.timeout_hard.as_ref(),
            self.timeout_capacity.as_ref(),
        ]
        .into_iter()
        .filter(Option::is_some)
        .count()
    }

    pub fn timeout_config_complete(&self) -> bool {
        self.timeout_configured_count() == 3
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TraceConfig {
    pub turn_on_log_metrics: Option<bool>,
    pub turn_on_logging: Option<bool>,
    pub use_trace_dispatcher: Option<bool>,
    pub min_severity: Option<String>,
    pub metrics_prefix: Option<String>,
    pub resource_frequency: Option<u64>,
    pub forwarder_conn_queue_size: Option<u64>,
    pub forwarder_disconn_queue_size: Option<u64>,
    pub forwarder_max_reconnect_delay: Option<u64>,
    pub trace_scalar_fields: usize,
    pub trace_option_entries: usize,
    pub trace_severity_overrides: usize,
    pub trace_severity_silence: usize,
    pub trace_severity_debug: usize,
    pub trace_severity_info: usize,
    pub trace_severity_notice: usize,
    pub trace_severity_warning: usize,
    pub trace_severity_error: usize,
    pub trace_severity_critical: usize,
    pub trace_severity_other: usize,
    pub trace_detail_overrides: usize,
    pub trace_frequency_limits: usize,
    pub trace_backend_entries: usize,
    pub trace_backend_ekg: usize,
    pub trace_backend_forwarder: usize,
    pub trace_backend_prometheus: usize,
    pub trace_backend_stdout: usize,
    pub trace_backend_katip: usize,
    pub trace_backend_other: usize,
    pub trace_prometheus_host: Option<String>,
    pub trace_prometheus_port: Option<u64>,
    pub default_backends: usize,
    pub default_backend_ekg: usize,
    pub default_backend_katip: usize,
    pub default_backend_other: usize,
    pub default_scribes: usize,
    pub default_scribe_stdout: usize,
    pub default_scribe_file: usize,
    pub default_scribe_other: usize,
    pub setup_backends: usize,
    pub setup_backend_ekg: usize,
    pub setup_backend_katip: usize,
    pub setup_backend_other: usize,
    pub setup_scribes: usize,
    pub setup_scribe_stdout: usize,
    pub setup_scribe_file: usize,
    pub setup_scribe_other: usize,
    pub legacy_trace_flags: usize,
    pub legacy_trace_enabled: usize,
    pub legacy_trace_disabled: usize,
    pub has_ekg_port: Option<u64>,
    pub has_prometheus_host: Option<String>,
    pub has_prometheus_port: Option<u64>,
    pub has_prometheus_items: usize,
    pub tracing_verbosity: Option<String>,
    pub rotation_keep_files_num: Option<u64>,
    pub rotation_log_limit_bytes: Option<u64>,
    pub rotation_max_age_hours: Option<u64>,
    pub legacy_map_backend_entries: usize,
    pub legacy_map_backend_items: usize,
    pub legacy_map_subtrace_entries: usize,
}

impl TraceConfig {
    pub fn runtime_configured_count(&self) -> usize {
        [
            self.turn_on_log_metrics.map(|_| ()),
            self.turn_on_logging.map(|_| ()),
            self.use_trace_dispatcher.map(|_| ()),
            self.min_severity.as_ref().map(|_| ()),
        ]
        .into_iter()
        .filter(Option::is_some)
        .count()
    }

    pub fn runtime_config_complete(&self) -> bool {
        self.runtime_configured_count() == 4
    }
}

impl CardanoNodeConfigManifest {
    pub fn protocol_identity_configured_count(&self) -> usize {
        [
            self.protocol.as_ref().map(|_| ()),
            self.consensus_mode.as_ref().map(|_| ()),
            self.requires_network_magic.map(|_| ()),
        ]
        .into_iter()
        .filter(Option::is_some)
        .count()
    }

    pub fn protocol_identity_config_complete(&self) -> bool {
        self.protocol_identity_configured_count() == 3
    }

    pub fn parse(input: &str) -> Result<Self, CardanoConfigError> {
        if input.len() > MAX_CARDANO_NODE_CONFIG_SIZE {
            return Err(CardanoConfigError::OversizedConfig {
                max: MAX_CARDANO_NODE_CONFIG_SIZE,
                actual: input.len(),
            });
        }

        let fields = parse_top_level_json_scalars(input)?;
        let requires_network_magic =
            match optional_text_field(&fields, "RequiresNetworkMagic").as_deref() {
                Some("RequiresMagic") => Some(RequiresNetworkMagic::RequiresMagic),
                Some("RequiresNoMagic") => Some(RequiresNetworkMagic::RequiresNoMagic),
                Some(value) => {
                    return Err(CardanoConfigError::InvalidRequiresNetworkMagic(
                        value.to_string(),
                    ))
                }
                None => None,
            };

        let mut genesis_files = Vec::new();
        for era in ["Byron", "Shelley", "Alonzo", "Conway", "Dijkstra"] {
            let file_key = format!("{era}GenesisFile");
            let Some(file) = optional_text_field(&fields, &file_key) else {
                continue;
            };
            if file.trim().is_empty() {
                return Err(CardanoConfigError::EmptyGenesisFile { era: era.into() });
            }
            let hash_key = format!("{era}GenesisHash");
            let expected_hash = optional_text_field(&fields, &hash_key)
                .as_ref()
                .map(|value| normalize_hex_hash(era, value))
                .transpose()?;
            genesis_files.push(GenesisFileFixture {
                era: era.to_string(),
                file,
                expected_hash,
            });
        }

        if genesis_files.is_empty() {
            return Err(CardanoConfigError::MissingGenesisFiles);
        }

        let (
            trace_option_entries,
            trace_severity_overrides,
            trace_detail_overrides,
            trace_frequency_limits,
            trace_backend_entries,
        ) = count_trace_option_fields(&fields);
        let trace_severity_levels = count_trace_severity_levels(&fields);
        let trace_backend_sinks = count_trace_backend_sinks(&fields);
        let (trace_prometheus_host, trace_prometheus_port) =
            first_trace_prometheus_endpoint(&fields);
        let default_backend_sinks = count_legacy_backend_sinks(&fields, "defaultBackends.#");
        let setup_backend_sinks = count_legacy_backend_sinks(&fields, "setupBackends.#");
        let default_scribe_sinks = count_legacy_scribe_sinks(&fields, "defaultScribes.");
        let setup_scribe_sinks = count_legacy_scribe_sinks(&fields, "setupScribes.");
        let (legacy_trace_flags, legacy_trace_enabled, legacy_trace_disabled) =
            count_legacy_trace_flags(&fields);
        let (legacy_map_backend_entries, legacy_map_backend_items) =
            count_array_length_fields(&fields, "options.mapBackends.");

        Ok(Self {
            protocol: optional_text_field(&fields, "Protocol"),
            consensus_mode: optional_text_field(&fields, "ConsensusMode"),
            protocol_features: ProtocolFeatureConfig {
                experimental_hard_forks_enabled: parse_optional_bool_field(
                    &fields,
                    "ExperimentalHardForksEnabled",
                )?,
                experimental_protocols_enabled: parse_optional_bool_field(
                    &fields,
                    "ExperimentalProtocolsEnabled",
                )?,
            },
            application: ApplicationConfig {
                name: optional_text_field(&fields, "ApplicationName"),
                version: parse_optional_u64_field(&fields, "ApplicationVersion")?,
                max_concurrency_deadline: parse_optional_u64_field(
                    &fields,
                    "MaxConcurrencyDeadline",
                )?,
            },
            test_hard_forks: TestHardForkConfig {
                shelley_epoch: parse_optional_u64_field(&fields, "TestShelleyHardForkAtEpoch")?,
                allegra_epoch: parse_optional_u64_field(&fields, "TestAllegraHardForkAtEpoch")?,
                mary_epoch: parse_optional_u64_field(&fields, "TestMaryHardForkAtEpoch")?,
                alonzo_epoch: parse_optional_u64_field(&fields, "TestAlonzoHardForkAtEpoch")?,
            },
            max_known_major_protocol_version: parse_optional_u64_field(
                &fields,
                "MaxKnownMajorProtocolVersion",
            )?,
            min_node_version: optional_text_field(&fields, "MinNodeVersion"),
            requires_network_magic,
            ledger_db: LedgerDbConfig {
                backend: optional_text_field(&fields, "LedgerDB.Backend"),
                num_disk_snapshots: parse_optional_u64_field(
                    &fields,
                    "LedgerDB.NumOfDiskSnapshots",
                )?,
                query_batch_size: parse_optional_u64_field(&fields, "LedgerDB.QueryBatchSize")?,
                snapshot_interval: parse_optional_u64_field(&fields, "LedgerDB.SnapshotInterval")?,
            },
            byron_protocol: ByronProtocolConfig {
                last_known_block_version_major: parse_optional_u64_field(
                    &fields,
                    "LastKnownBlockVersion-Major",
                )?,
                last_known_block_version_minor: parse_optional_u64_field(
                    &fields,
                    "LastKnownBlockVersion-Minor",
                )?,
                last_known_block_version_alt: parse_optional_u64_field(
                    &fields,
                    "LastKnownBlockVersion-Alt",
                )?,
                pbft_signature_threshold: parse_optional_decimal_field(
                    &fields,
                    "PBftSignatureThreshold",
                )?,
            },
            checkpoints: CheckpointsConfig {
                file: optional_text_field(&fields, "CheckpointsFile"),
                expected_hash: optional_text_field(&fields, "CheckpointsFileHash")
                    .as_ref()
                    .map(|value| normalize_config_hash("CheckpointsFileHash", value))
                    .transpose()?,
            },
            p2p_governor: P2pGovernorConfig {
                deadline: PeerTargetConfig {
                    root_peers: parse_optional_u64_field(&fields, "TargetNumberOfRootPeers")?,
                    known_peers: parse_optional_u64_field(&fields, "TargetNumberOfKnownPeers")?,
                    established_peers: parse_optional_u64_field(
                        &fields,
                        "TargetNumberOfEstablishedPeers",
                    )?,
                    active_peers: parse_optional_u64_field(&fields, "TargetNumberOfActivePeers")?,
                    known_big_ledger_peers: parse_optional_u64_field(
                        &fields,
                        "TargetNumberOfKnownBigLedgerPeers",
                    )?,
                    established_big_ledger_peers: parse_optional_u64_field(
                        &fields,
                        "TargetNumberOfEstablishedBigLedgerPeers",
                    )?,
                    active_big_ledger_peers: parse_optional_u64_field(
                        &fields,
                        "TargetNumberOfActiveBigLedgerPeers",
                    )?,
                },
                sync: PeerTargetConfig {
                    root_peers: parse_optional_u64_field(&fields, "SyncTargetNumberOfRootPeers")?,
                    known_peers: parse_optional_u64_field(&fields, "SyncTargetNumberOfKnownPeers")?,
                    established_peers: parse_optional_u64_field(
                        &fields,
                        "SyncTargetNumberOfEstablishedPeers",
                    )?,
                    active_peers: parse_optional_u64_field(
                        &fields,
                        "SyncTargetNumberOfActivePeers",
                    )?,
                    known_big_ledger_peers: parse_optional_u64_field(
                        &fields,
                        "SyncTargetNumberOfKnownBigLedgerPeers",
                    )?,
                    established_big_ledger_peers: parse_optional_u64_field(
                        &fields,
                        "SyncTargetNumberOfEstablishedBigLedgerPeers",
                    )?,
                    active_big_ledger_peers: parse_optional_u64_field(
                        &fields,
                        "SyncTargetNumberOfActiveBigLedgerPeers",
                    )?,
                },
                min_big_ledger_peers_for_trusted_state: parse_optional_u64_field(
                    &fields,
                    "MinBigLedgerPeersForTrustedState",
                )?,
            },
            mempool: MempoolConfig {
                timeout_soft: parse_optional_decimal_field(&fields, "MempoolTimeoutSoft")?,
                timeout_hard: parse_optional_decimal_field(&fields, "MempoolTimeoutHard")?,
                timeout_capacity: parse_optional_decimal_field(&fields, "MempoolTimeoutCapacity")?,
            },
            tracing: TraceConfig {
                turn_on_log_metrics: parse_optional_bool_field(&fields, "TurnOnLogMetrics")?,
                turn_on_logging: parse_optional_bool_field(&fields, "TurnOnLogging")?,
                use_trace_dispatcher: parse_optional_bool_field(&fields, "UseTraceDispatcher")?,
                min_severity: optional_text_field(&fields, "minSeverity"),
                metrics_prefix: optional_text_field(&fields, "TraceOptionMetricsPrefix"),
                resource_frequency: parse_optional_u64_field(
                    &fields,
                    "TraceOptionResourceFrequency",
                )?,
                forwarder_conn_queue_size: parse_optional_u64_field(
                    &fields,
                    "TraceOptionForwarder.connQueueSize",
                )?,
                forwarder_disconn_queue_size: parse_optional_u64_field(
                    &fields,
                    "TraceOptionForwarder.disconnQueueSize",
                )?,
                forwarder_max_reconnect_delay: parse_optional_u64_field(
                    &fields,
                    "TraceOptionForwarder.maxReconnectDelay",
                )?,
                trace_scalar_fields: count_trace_scalar_fields(&fields),
                trace_option_entries,
                trace_severity_overrides,
                trace_severity_silence: trace_severity_levels.silence,
                trace_severity_debug: trace_severity_levels.debug,
                trace_severity_info: trace_severity_levels.info,
                trace_severity_notice: trace_severity_levels.notice,
                trace_severity_warning: trace_severity_levels.warning,
                trace_severity_error: trace_severity_levels.error,
                trace_severity_critical: trace_severity_levels.critical,
                trace_severity_other: trace_severity_levels.other,
                trace_detail_overrides,
                trace_frequency_limits,
                trace_backend_entries,
                trace_backend_ekg: trace_backend_sinks.ekg,
                trace_backend_forwarder: trace_backend_sinks.forwarder,
                trace_backend_prometheus: trace_backend_sinks.prometheus,
                trace_backend_stdout: trace_backend_sinks.stdout,
                trace_backend_katip: trace_backend_sinks.katip,
                trace_backend_other: trace_backend_sinks.other,
                trace_prometheus_host,
                trace_prometheus_port,
                default_backends: parse_array_length_field(&fields, "defaultBackends.#len")?,
                default_backend_ekg: default_backend_sinks.ekg,
                default_backend_katip: default_backend_sinks.katip,
                default_backend_other: default_backend_sinks.other,
                default_scribes: parse_array_length_field(&fields, "defaultScribes.#len")?,
                default_scribe_stdout: default_scribe_sinks.stdout,
                default_scribe_file: default_scribe_sinks.file,
                default_scribe_other: default_scribe_sinks.other,
                setup_backends: parse_array_length_field(&fields, "setupBackends.#len")?,
                setup_backend_ekg: setup_backend_sinks.ekg,
                setup_backend_katip: setup_backend_sinks.katip,
                setup_backend_other: setup_backend_sinks.other,
                setup_scribes: parse_array_length_field(&fields, "setupScribes.#len")?,
                setup_scribe_stdout: setup_scribe_sinks.stdout,
                setup_scribe_file: setup_scribe_sinks.file,
                setup_scribe_other: setup_scribe_sinks.other,
                legacy_trace_flags,
                legacy_trace_enabled,
                legacy_trace_disabled,
                has_ekg_port: parse_optional_u64_field(&fields, "hasEKG")?,
                has_prometheus_host: optional_text_field(&fields, "hasPrometheus.#0"),
                has_prometheus_port: parse_optional_u64_field(&fields, "hasPrometheus.#1")?,
                has_prometheus_items: parse_array_length_field(&fields, "hasPrometheus.#len")?,
                tracing_verbosity: optional_text_field(&fields, "TracingVerbosity"),
                rotation_keep_files_num: parse_optional_u64_field(
                    &fields,
                    "rotation.rpKeepFilesNum",
                )?,
                rotation_log_limit_bytes: parse_optional_u64_field(
                    &fields,
                    "rotation.rpLogLimitBytes",
                )?,
                rotation_max_age_hours: parse_optional_u64_field(
                    &fields,
                    "rotation.rpMaxAgeHours",
                )?,
                legacy_map_backend_entries,
                legacy_map_backend_items,
                legacy_map_subtrace_entries: count_scalar_fields(
                    &fields,
                    "options.mapSubtrace.",
                    ".subtrace",
                ),
            },
            genesis_files,
        })
    }

    pub fn verify_genesis_hashes(&self, digests: &[GenesisFileDigest]) -> GenesisHashReport {
        let mut checks = Vec::with_capacity(self.genesis_files.len());
        for fixture in &self.genesis_files {
            let actual_hash = digests
                .iter()
                .find(|digest| digest.era.eq_ignore_ascii_case(&fixture.era))
                .map(|digest| digest.hash.trim().to_ascii_lowercase());
            let status = match (&fixture.expected_hash, &actual_hash) {
                (Some(expected), Some(actual)) if expected == actual => GenesisHashStatus::Match,
                (Some(_), Some(_)) => GenesisHashStatus::Mismatch,
                (Some(_), None) => GenesisHashStatus::MissingActual,
                (None, _) => GenesisHashStatus::MissingExpected,
            };
            checks.push(GenesisHashCheck {
                era: fixture.era.clone(),
                file: fixture.file.clone(),
                expected_hash: fixture.expected_hash.clone(),
                actual_hash,
                status,
            });
        }
        GenesisHashReport { checks }
    }

    pub fn check_network_magic_requirement(
        &self,
        profile: NetworkProfile,
    ) -> NetworkMagicRequirementCheck {
        let expected = expected_requires_network_magic(profile);
        let actual = self.requires_network_magic;
        let status = match actual {
            Some(value) if value == expected => NetworkMagicRequirementStatus::Match,
            Some(_) => NetworkMagicRequirementStatus::Mismatch,
            None => NetworkMagicRequirementStatus::MissingConfig,
        };
        NetworkMagicRequirementCheck {
            profile_name: profile.name.to_string(),
            expected,
            actual,
            status,
        }
    }
}

fn expected_requires_network_magic(profile: NetworkProfile) -> RequiresNetworkMagic {
    if profile.name == "mainnet" {
        RequiresNetworkMagic::RequiresNoMagic
    } else {
        RequiresNetworkMagic::RequiresMagic
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardanoConfigError {
    OversizedConfig { max: usize, actual: usize },
    InvalidJson(String),
    DuplicateField(String),
    MissingGenesisFiles,
    EmptyGenesisFile { era: String },
    InvalidGenesisHash { era: String, hash: String },
    InvalidConfigHash { field: String, hash: String },
    InvalidNumber { field: String, value: String },
    InvalidBoolean { field: String, value: String },
    InvalidRequiresNetworkMagic(String),
}

impl fmt::Display for CardanoConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OversizedConfig { max, actual } => {
                write!(f, "cardano config too large: max={max} actual={actual}")
            }
            Self::InvalidJson(message) => write!(f, "invalid cardano config JSON: {message}"),
            Self::DuplicateField(field) => write!(f, "duplicate cardano config field {field}"),
            Self::MissingGenesisFiles => f.write_str("cardano config contains no genesis files"),
            Self::EmptyGenesisFile { era } => write!(f, "{era} genesis file is empty"),
            Self::InvalidGenesisHash { era, hash } => {
                write!(f, "invalid {era} genesis hash {hash}")
            }
            Self::InvalidConfigHash { field, hash } => {
                write!(f, "invalid cardano config hash {field}={hash}")
            }
            Self::InvalidNumber { field, value } => {
                write!(f, "invalid numeric cardano config field {field}={value}")
            }
            Self::InvalidBoolean { field, value } => {
                write!(f, "invalid boolean cardano config field {field}={value}")
            }
            Self::InvalidRequiresNetworkMagic(value) => {
                write!(f, "invalid RequiresNetworkMagic value {value}")
            }
        }
    }
}

impl std::error::Error for CardanoConfigError {}

fn normalize_hex_hash(era: &str, value: &str) -> Result<String, CardanoConfigError> {
    let hash = value.trim();
    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CardanoConfigError::InvalidGenesisHash {
            era: era.to_string(),
            hash: value.to_string(),
        });
    }
    Ok(hash.to_ascii_lowercase())
}

fn normalize_config_hash(field: &str, value: &str) -> Result<String, CardanoConfigError> {
    let hash = value.trim();
    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(CardanoConfigError::InvalidConfigHash {
            field: field.to_string(),
            hash: value.to_string(),
        });
    }
    Ok(hash.to_ascii_lowercase())
}

fn optional_text_field(fields: &BTreeMap<String, String>, field: &str) -> Option<String> {
    fields
        .get(field)
        .filter(|value| !is_json_null_scalar(value))
        .cloned()
}

fn is_json_null_scalar(value: &str) -> bool {
    value == "null"
}

fn parse_optional_u64_field(
    fields: &BTreeMap<String, String>,
    field: &'static str,
) -> Result<Option<u64>, CardanoConfigError> {
    let Some(value) = fields.get(field) else {
        return Ok(None);
    };
    if is_json_null_scalar(value) {
        return Ok(None);
    }
    value
        .parse::<u64>()
        .map(Some)
        .map_err(|_| CardanoConfigError::InvalidNumber {
            field: field.to_string(),
            value: value.clone(),
        })
}

fn parse_optional_decimal_field(
    fields: &BTreeMap<String, String>,
    field: &'static str,
) -> Result<Option<String>, CardanoConfigError> {
    let Some(value) = fields.get(field) else {
        return Ok(None);
    };
    if is_json_null_scalar(value) {
        return Ok(None);
    }
    value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .map(|_| Some(value.clone()))
        .ok_or_else(|| CardanoConfigError::InvalidNumber {
            field: field.to_string(),
            value: value.clone(),
        })
}

fn parse_optional_bool_field(
    fields: &BTreeMap<String, String>,
    field: &'static str,
) -> Result<Option<bool>, CardanoConfigError> {
    let Some(value) = fields.get(field) else {
        return Ok(None);
    };
    if is_json_null_scalar(value) {
        return Ok(None);
    }
    match value.as_str() {
        "true" | "True" => Ok(Some(true)),
        "false" | "False" => Ok(Some(false)),
        _ => Err(CardanoConfigError::InvalidBoolean {
            field: field.to_string(),
            value: value.clone(),
        }),
    }
}

fn count_trace_scalar_fields(fields: &BTreeMap<String, String>) -> usize {
    fields
        .keys()
        .filter(|key| key.starts_with("TraceOptions.") && !key.contains(".#"))
        .count()
}

fn parse_array_length_field(
    fields: &BTreeMap<String, String>,
    field: &'static str,
) -> Result<usize, CardanoConfigError> {
    let Some(value) = fields.get(field) else {
        return Ok(0);
    };
    value
        .parse::<usize>()
        .map_err(|_| CardanoConfigError::InvalidNumber {
            field: field.to_string(),
            value: value.clone(),
        })
}

fn count_trace_option_fields(
    fields: &BTreeMap<String, String>,
) -> (usize, usize, usize, usize, usize) {
    let mut entries = BTreeSet::new();
    let mut severity_overrides = 0;
    let mut detail_overrides = 0;
    let mut frequency_limits = 0;
    let mut backend_entries = 0;
    for key in fields.keys().filter(|key| key.starts_with("TraceOptions.")) {
        let suffix = &key["TraceOptions.".len()..];
        if let Some(entry) = suffix.strip_suffix(".backends.#len") {
            entries.insert(entry.to_string());
            backend_entries += fields
                .get(key)
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            continue;
        }
        if key.contains(".#") {
            continue;
        }
        let Some((entry, field)) = suffix.rsplit_once('.') else {
            continue;
        };
        entries.insert(entry.to_string());
        match field {
            "severity" => severity_overrides += 1,
            "detail" => detail_overrides += 1,
            "maxFrequency" => frequency_limits += 1,
            _ => {}
        }
    }
    (
        entries.len(),
        severity_overrides,
        detail_overrides,
        frequency_limits,
        backend_entries,
    )
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TraceBackendSinks {
    ekg: usize,
    forwarder: usize,
    prometheus: usize,
    stdout: usize,
    katip: usize,
    other: usize,
}

fn count_trace_backend_sinks(fields: &BTreeMap<String, String>) -> TraceBackendSinks {
    let mut sinks = TraceBackendSinks::default();
    for value in fields.iter().filter_map(|(key, value)| {
        (key.starts_with("TraceOptions.") && key.contains(".backends.#") && !key.ends_with(".#len"))
            .then_some(value)
    }) {
        match value.as_str() {
            "EKGBackend" => sinks.ekg += 1,
            "Forwarder" => sinks.forwarder += 1,
            value if value.starts_with("PrometheusSimple") => sinks.prometheus += 1,
            value if value.starts_with("Stdout") => sinks.stdout += 1,
            "KatipBK" => sinks.katip += 1,
            _ => sinks.other += 1,
        }
    }
    sinks
}

fn first_trace_prometheus_endpoint(
    fields: &BTreeMap<String, String>,
) -> (Option<String>, Option<u64>) {
    for value in fields.iter().filter_map(|(key, value)| {
        (key.starts_with("TraceOptions.") && key.contains(".backends.#") && !key.ends_with(".#len"))
            .then_some(value)
    }) {
        let parts = value.split_whitespace().collect::<Vec<_>>();
        if parts.len() < 4 || parts.first() != Some(&"PrometheusSimple") {
            continue;
        }
        let host = parts[parts.len() - 2].to_string();
        let port = parts[parts.len() - 1].parse::<u64>().ok();
        return (Some(host), port);
    }
    (None, None)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct LegacyBackendSinks {
    ekg: usize,
    katip: usize,
    other: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct LegacyScribeSinks {
    stdout: usize,
    file: usize,
    other: usize,
}

fn count_legacy_backend_sinks(
    fields: &BTreeMap<String, String>,
    prefix: &str,
) -> LegacyBackendSinks {
    let mut sinks = LegacyBackendSinks::default();
    for value in fields.iter().filter_map(|(key, value)| {
        (key.starts_with(prefix) && !key.ends_with(".#len")).then_some(value)
    }) {
        match value.as_str() {
            "EKGBackend" | "EKGViewBK" => sinks.ekg += 1,
            "KatipBK" => sinks.katip += 1,
            _ => sinks.other += 1,
        }
    }
    sinks
}

fn count_legacy_scribe_sinks(fields: &BTreeMap<String, String>, prefix: &str) -> LegacyScribeSinks {
    let mut sinks = LegacyScribeSinks::default();
    for (key, value) in fields {
        if !key.starts_with(prefix) || !(key.ends_with(".scKind") || key.ends_with(".#0")) {
            continue;
        }
        match value.as_str() {
            "StdoutSK" => sinks.stdout += 1,
            value if value.starts_with("FileSK") => sinks.file += 1,
            _ => sinks.other += 1,
        }
    }
    sinks
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TraceSeverityLevels {
    silence: usize,
    debug: usize,
    info: usize,
    notice: usize,
    warning: usize,
    error: usize,
    critical: usize,
    other: usize,
}

fn count_trace_severity_levels(fields: &BTreeMap<String, String>) -> TraceSeverityLevels {
    let mut levels = TraceSeverityLevels::default();
    for (_, value) in fields
        .iter()
        .filter(|(key, _)| key.starts_with("TraceOptions.") && key.ends_with(".severity"))
    {
        match value.as_str() {
            "Silence" => levels.silence += 1,
            "Debug" => levels.debug += 1,
            "Info" => levels.info += 1,
            "Notice" => levels.notice += 1,
            "Warning" => levels.warning += 1,
            "Error" => levels.error += 1,
            "Critical" => levels.critical += 1,
            _ => levels.other += 1,
        }
    }
    levels
}

fn count_legacy_trace_flags(fields: &BTreeMap<String, String>) -> (usize, usize, usize) {
    let mut total = 0;
    let mut enabled = 0;
    let mut disabled = 0;
    for (key, value) in fields {
        if !key.starts_with("Trace") || key.starts_with("TraceOption") {
            continue;
        }
        match value.as_str() {
            "true" | "True" => {
                total += 1;
                enabled += 1;
            }
            "false" | "False" => {
                total += 1;
                disabled += 1;
            }
            _ => {}
        }
    }
    (total, enabled, disabled)
}

fn count_array_length_fields(fields: &BTreeMap<String, String>, prefix: &str) -> (usize, usize) {
    let mut entries = 0;
    let mut items = 0;
    for (key, value) in fields {
        if key.starts_with(prefix) && key.ends_with(".#len") {
            entries += 1;
            items += value.parse::<usize>().unwrap_or(0);
        }
    }
    (entries, items)
}

fn count_scalar_fields(fields: &BTreeMap<String, String>, prefix: &str, suffix: &str) -> usize {
    fields
        .keys()
        .filter(|key| key.starts_with(prefix) && key.ends_with(suffix))
        .count()
}

fn blake2b_256(input: &[u8]) -> [u8; 32] {
    let mut h = BLAKE2B_IV;
    h[0] ^= 0x0101_0020;

    let mut counter = 0u128;
    let mut remaining = input;
    while remaining.len() > 128 {
        counter += 128;
        blake2b_compress(&mut h, &remaining[..128], counter, false);
        remaining = &remaining[128..];
    }

    counter += remaining.len() as u128;
    let mut final_block = [0u8; 128];
    final_block[..remaining.len()].copy_from_slice(remaining);
    blake2b_compress(&mut h, &final_block, counter, true);

    let mut out = [0u8; 32];
    for (index, word) in h.iter().enumerate() {
        let bytes = word.to_le_bytes();
        let start = index * 8;
        if start >= out.len() {
            break;
        }
        let len = (out.len() - start).min(8);
        out[start..start + len].copy_from_slice(&bytes[..len]);
    }
    out
}

fn blake2b_compress(h: &mut [u64; 8], block: &[u8], counter: u128, final_block: bool) {
    debug_assert_eq!(block.len(), 128);
    let mut m = [0u64; 16];
    for (index, chunk) in block.chunks_exact(8).enumerate() {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(chunk);
        m[index] = u64::from_le_bytes(bytes);
    }

    let mut v = [0u64; 16];
    v[..8].copy_from_slice(h);
    v[8..].copy_from_slice(&BLAKE2B_IV);
    v[12] ^= counter as u64;
    v[13] ^= (counter >> 64) as u64;
    if final_block {
        v[14] = !v[14];
    }

    for schedule in BLAKE2B_SIGMA {
        blake2b_g(&mut v, 0, 4, 8, 12, m[schedule[0]], m[schedule[1]]);
        blake2b_g(&mut v, 1, 5, 9, 13, m[schedule[2]], m[schedule[3]]);
        blake2b_g(&mut v, 2, 6, 10, 14, m[schedule[4]], m[schedule[5]]);
        blake2b_g(&mut v, 3, 7, 11, 15, m[schedule[6]], m[schedule[7]]);
        blake2b_g(&mut v, 0, 5, 10, 15, m[schedule[8]], m[schedule[9]]);
        blake2b_g(&mut v, 1, 6, 11, 12, m[schedule[10]], m[schedule[11]]);
        blake2b_g(&mut v, 2, 7, 8, 13, m[schedule[12]], m[schedule[13]]);
        blake2b_g(&mut v, 3, 4, 9, 14, m[schedule[14]], m[schedule[15]]);
    }

    for index in 0..8 {
        h[index] ^= v[index] ^ v[index + 8];
    }
}

fn blake2b_g(v: &mut [u64; 16], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = (v[d] ^ v[a]).rotate_right(32);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(24);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(63);
}

fn bytes_to_lower_hex(bytes: &[u8]) -> String {
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
        _ => unreachable!("hex nibble must be in 0..=15"),
    }
}

fn parse_top_level_json_scalars(
    input: &str,
) -> Result<BTreeMap<String, String>, CardanoConfigError> {
    let mut parser = JsonScalarParser::new(input);
    parser.parse_object()
}

fn insert_flattened_field(
    fields: &mut BTreeMap<String, String>,
    prefix: &[String],
    value: String,
) -> Result<(), CardanoConfigError> {
    let key = prefix.join(".");
    if fields.insert(key.clone(), value).is_some() {
        return Err(CardanoConfigError::DuplicateField(key));
    }
    Ok(())
}

fn insert_array_length_field(
    fields: &mut BTreeMap<String, String>,
    prefix: &[String],
    len: usize,
) -> Result<(), CardanoConfigError> {
    let key = format!("{}.#len", prefix.join("."));
    if fields.insert(key.clone(), len.to_string()).is_some() {
        return Err(CardanoConfigError::DuplicateField(key));
    }
    Ok(())
}

fn field_path(prefix: &[String], key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{}.{}", prefix.join("."), key)
    }
}

struct JsonScalarParser<'a> {
    pos: usize,
    input: &'a str,
}

impl<'a> JsonScalarParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { pos: 0, input }
    }

    fn parse_object(&mut self) -> Result<BTreeMap<String, String>, CardanoConfigError> {
        let mut fields = BTreeMap::new();
        let mut prefix = Vec::new();
        self.parse_object_into(&mut fields, &mut prefix)?;
        self.skip_ws();
        if self.peek().is_some() {
            return self.err("trailing input after object");
        }
        Ok(fields)
    }

    fn parse_object_into(
        &mut self,
        fields: &mut BTreeMap<String, String>,
        prefix: &mut Vec<String>,
    ) -> Result<(), CardanoConfigError> {
        self.skip_ws();
        self.expect('{')?;
        let mut seen_keys = BTreeSet::new();
        loop {
            self.skip_ws();
            if self.consume('}') {
                return Ok(());
            }
            let key = self.parse_string()?;
            if !seen_keys.insert(key.clone()) {
                return Err(CardanoConfigError::DuplicateField(field_path(prefix, &key)));
            }
            self.skip_ws();
            self.expect(':')?;
            self.skip_ws();
            prefix.push(key.clone());
            match self.peek() {
                Some('"') => {
                    let value = self.parse_string()?;
                    insert_flattened_field(fields, prefix, value)?;
                }
                Some('{') => self.parse_object_into(fields, prefix)?,
                Some('[') => {
                    let len = self.parse_array_into(fields, prefix)?;
                    insert_array_length_field(fields, prefix, len)?;
                }
                Some(_) => {
                    let value = self.parse_bare_scalar()?;
                    insert_flattened_field(fields, prefix, value)?;
                }
                None => return self.err("expected value"),
            }
            prefix.pop();
            self.skip_ws();
            if self.consume(',') {
                continue;
            }
            if self.consume('}') {
                return Ok(());
            }
            return self.err("expected ',' or '}'");
        }
    }

    fn parse_string(&mut self) -> Result<String, CardanoConfigError> {
        self.expect('"')?;
        let mut out = String::new();
        while let Some(ch) = self.next() {
            match ch {
                '"' => return Ok(out),
                '\\' => out.push(self.parse_escape()?),
                other => out.push(other),
            }
        }
        self.err("unterminated string")
    }

    fn parse_escape(&mut self) -> Result<char, CardanoConfigError> {
        let Some(escaped) = self.next() else {
            return self.err("unterminated escape sequence");
        };
        match escaped {
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            '/' => Ok('/'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            other => self.err(&format!("unsupported escape sequence \\{other}")),
        }
    }

    fn parse_bare_scalar(&mut self) -> Result<String, CardanoConfigError> {
        let mut out = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || ch == ',' || ch == '}' {
                break;
            }
            out.push(ch);
            self.next();
        }
        if out.is_empty() {
            return self.err("expected scalar value");
        }
        Ok(out)
    }

    fn parse_array_into(
        &mut self,
        fields: &mut BTreeMap<String, String>,
        prefix: &[String],
    ) -> Result<usize, CardanoConfigError> {
        self.expect('[')?;
        self.skip_ws();
        if self.consume(']') {
            return Ok(0);
        }

        let mut len = 0;
        loop {
            self.skip_ws();
            match self.peek() {
                Some('"') => {
                    let value = self.parse_string()?;
                    let mut item_prefix = prefix.to_vec();
                    item_prefix.push(format!("#{len}"));
                    insert_flattened_field(fields, &item_prefix, value)?;
                }
                Some('{') => {
                    let mut item_prefix = prefix.to_vec();
                    item_prefix.push(format!("#{len}"));
                    self.parse_object_into(fields, &mut item_prefix)?;
                }
                Some('[') => {
                    let mut item_prefix = prefix.to_vec();
                    item_prefix.push(format!("#{len}"));
                    let nested_len = self.parse_array_into(fields, &item_prefix)?;
                    insert_array_length_field(fields, &item_prefix, nested_len)?;
                }
                Some(_) => {
                    let value = self.parse_bare_array_scalar()?;
                    let mut item_prefix = prefix.to_vec();
                    item_prefix.push(format!("#{len}"));
                    insert_flattened_field(fields, &item_prefix, value)?;
                }
                None => return self.err("expected array value"),
            }
            len += 1;
            self.skip_ws();
            if self.consume(',') {
                continue;
            }
            if self.consume(']') {
                return Ok(len);
            }
            return self.err("expected ',' or ']'");
        }
    }

    fn parse_bare_array_scalar(&mut self) -> Result<String, CardanoConfigError> {
        let mut out = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || ch == ',' || ch == ']' {
                break;
            }
            out.push(ch);
            self.next();
        }
        if out.is_empty() {
            self.err("expected array scalar")
        } else {
            Ok(out)
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), CardanoConfigError> {
        if self.consume(expected) {
            Ok(())
        } else {
            self.err(&format!("expected '{expected}'"))
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.pos += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn next(&mut self) -> Option<char> {
        let value = self.peek()?;
        self.pos += value.len_utf8();
        Some(value)
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos..)?.chars().next()
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.next();
        }
    }

    fn err<T>(&self, message: &str) -> Result<T, CardanoConfigError> {
        Err(CardanoConfigError::InvalidJson(format!(
            "{message} at byte {} of {}",
            self.pos,
            self.input.len()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::network_profile;

    #[test]
    fn cardano_node_config_manifest_parses_mainnet_genesis_hashes() {
        let manifest = CardanoNodeConfigManifest::parse(
            r#"{
              "AlonzoGenesisFile": "mainnet-alonzo-genesis.json",
              "AlonzoGenesisHash": "7e94a15f55d1e82d10f09203fa1d40f8eede58fd8066542cf6566008068ed874",
              "ApplicationName": "acropolis",
              "ApplicationVersion": 0,
              "ByronGenesisFile": "mainnet-byron-genesis.json",
              "ByronGenesisHash": "5f20df933584822601f9e3f8c024eb5eb252fe8cefb24d1317dc3d432e940ebb",
              "CheckpointsFile": "mainnet-checkpoints.json",
              "CheckpointsFileHash": "3e6dee5bae7acc6d870187e72674b37c929be8c66e62a552cf6a876b1af31ade",
              "ConsensusMode": "PraosMode",
              "ConwayGenesisFile": "mainnet-conway-genesis.json",
              "ConwayGenesisHash": "15a199f895e461ec0ffc6dd4e4028af28a492ab4e806d39cb674c88f7643ef62",
              "ExperimentalHardForksEnabled": false,
              "ExperimentalProtocolsEnabled": true,
              "LastKnownBlockVersion-Alt": 0,
              "LastKnownBlockVersion-Major": 3,
              "LastKnownBlockVersion-Minor": 0,
              "LedgerDB": {
                "Backend": "V2InMemory",
                "NumOfDiskSnapshots": 2,
                "QueryBatchSize": 100000,
                "SnapshotInterval": 4320
              },
              "MaxConcurrencyDeadline": 4,
              "MaxKnownMajorProtocolVersion": 2,
              "MempoolTimeoutCapacity": 5.0,
              "MempoolTimeoutHard": 1.5,
              "MempoolTimeoutSoft": 1.0,
              "MinNodeVersion": "10.7.0",
              "PBftSignatureThreshold": 0.6,
              "Protocol": "Cardano",
              "RequiresNetworkMagic": "RequiresNoMagic",
              "ShelleyGenesisFile": "mainnet-shelley-genesis.json",
              "ShelleyGenesisHash": "1a3be38bcbb7911969283716ad7aa550250226b76a61fc51cc9a9a35d9276d81",
              "SyncTargetNumberOfActiveBigLedgerPeers": 1,
              "SyncTargetNumberOfActivePeers": 2,
              "SyncTargetNumberOfEstablishedBigLedgerPeers": 3,
              "SyncTargetNumberOfEstablishedPeers": 4,
              "SyncTargetNumberOfKnownBigLedgerPeers": 5,
              "SyncTargetNumberOfKnownPeers": 6,
              "SyncTargetNumberOfRootPeers": 7,
              "TargetNumberOfActiveBigLedgerPeers": 8,
              "TargetNumberOfActivePeers": 9,
              "TargetNumberOfEstablishedBigLedgerPeers": 10,
              "TargetNumberOfEstablishedPeers": 11,
              "TargetNumberOfKnownBigLedgerPeers": 12,
              "TargetNumberOfKnownPeers": 13,
              "TargetNumberOfRootPeers": 14,
              "MinBigLedgerPeersForTrustedState": 15,
              "TraceOptionForwarder": {
                "connQueueSize": 64,
                "disconnQueueSize": 128,
                "maxReconnectDelay": 30
              },
              "TraceOptionMetricsPrefix": "cardano.node.metrics.",
              "TraceOptionResourceFrequency": 1000,
              "TraceOptions": {
                "": {
                  "backends": ["EKGBackend", "PrometheusSimple suffix 127.0.0.1 12798"],
                  "detail": "DNormal",
                  "severity": "Notice"
                },
                "Net.PeerSelection": {
                  "maxFrequency": 2.0,
                  "severity": "Info"
                }
              },
              "TraceAcceptPolicy": true,
              "TraceMempool": false,
              "TestAllegraHardForkAtEpoch": 0,
              "TestAlonzoHardForkAtEpoch": 0,
              "TestMaryHardForkAtEpoch": 0,
              "TestShelleyHardForkAtEpoch": 0,
              "TracingVerbosity": "NormalVerbosity",
              "TurnOnLogMetrics": true,
              "TurnOnLogging": true,
              "UseTraceDispatcher": true,
              "defaultBackends": ["EKGBackend"],
              "defaultScribes": [["StdoutSK", "stdout"]],
              "hasEKG": 12788,
              "hasPrometheus": ["127.0.0.1", 12798],
              "minSeverity": "Critical",
              "options": {
                "mapBackends": {
                  "cardano.node.metrics": ["EKGViewBK"],
                  "cardano.node.resources": ["EKGViewBK"]
                },
                "mapSubtrace": {
                  "cardano.node.metrics": {
                    "subtrace": "Neutral"
                  }
                }
              },
              "rotation": {
                "rpKeepFilesNum": 10,
                "rpLogLimitBytes": 5000000,
                "rpMaxAgeHours": 24
              },
              "setupBackends": ["KatipBK"],
              "setupScribes": [
                {
                  "scFormat": "ScText",
                  "scKind": "StdoutSK",
                  "scName": "stdout",
                  "scRotation": null
                }
              ]
            }"#,
        )
        .unwrap();

        assert_eq!(manifest.protocol.as_deref(), Some("Cardano"));
        assert_eq!(manifest.consensus_mode.as_deref(), Some("PraosMode"));
        assert_eq!(
            manifest.protocol_features.experimental_hard_forks_enabled,
            Some(false)
        );
        assert_eq!(
            manifest.protocol_features.experimental_protocols_enabled,
            Some(true)
        );
        assert_eq!(manifest.protocol_features.configured_count(), 2);
        assert!(manifest.protocol_features.config_complete());
        assert_eq!(manifest.application.name.as_deref(), Some("acropolis"));
        assert_eq!(manifest.application.version, Some(0));
        assert_eq!(manifest.application.max_concurrency_deadline, Some(4));
        assert_eq!(manifest.application.configured_count(), 3);
        assert!(manifest.application.config_complete());
        assert_eq!(manifest.test_hard_forks.shelley_epoch, Some(0));
        assert_eq!(manifest.test_hard_forks.allegra_epoch, Some(0));
        assert_eq!(manifest.test_hard_forks.mary_epoch, Some(0));
        assert_eq!(manifest.test_hard_forks.alonzo_epoch, Some(0));
        assert_eq!(manifest.test_hard_forks.configured_count(), 4);
        assert!(manifest.test_hard_forks.config_complete());
        assert_eq!(manifest.max_known_major_protocol_version, Some(2));
        assert_eq!(manifest.min_node_version.as_deref(), Some("10.7.0"));
        assert_eq!(manifest.ledger_db.backend.as_deref(), Some("V2InMemory"));
        assert_eq!(manifest.ledger_db.num_disk_snapshots, Some(2));
        assert_eq!(manifest.ledger_db.query_batch_size, Some(100000));
        assert_eq!(manifest.ledger_db.snapshot_interval, Some(4320));
        assert_eq!(manifest.ledger_db.configured_count(), 4);
        assert!(manifest.ledger_db.config_complete());
        assert_eq!(manifest.p2p_governor.deadline.root_peers, Some(14));
        assert_eq!(manifest.p2p_governor.deadline.known_peers, Some(13));
        assert_eq!(manifest.p2p_governor.deadline.configured_count(), 7);
        assert!(manifest.p2p_governor.deadline.config_complete());
        assert_eq!(manifest.p2p_governor.sync.root_peers, Some(7));
        assert_eq!(manifest.p2p_governor.sync.known_peers, Some(6));
        assert_eq!(manifest.p2p_governor.sync.configured_count(), 7);
        assert!(manifest.p2p_governor.sync.config_complete());
        assert_eq!(
            manifest.p2p_governor.min_big_ledger_peers_for_trusted_state,
            Some(15)
        );
        assert_eq!(manifest.mempool.timeout_soft.as_deref(), Some("1.0"));
        assert_eq!(manifest.mempool.timeout_hard.as_deref(), Some("1.5"));
        assert_eq!(manifest.mempool.timeout_capacity.as_deref(), Some("5.0"));
        assert_eq!(manifest.mempool.timeout_configured_count(), 3);
        assert!(manifest.mempool.timeout_config_complete());
        assert_eq!(
            manifest.byron_protocol.last_known_block_version_major,
            Some(3)
        );
        assert_eq!(
            manifest.byron_protocol.last_known_block_version_minor,
            Some(0)
        );
        assert_eq!(
            manifest.byron_protocol.last_known_block_version_alt,
            Some(0)
        );
        assert_eq!(
            manifest.byron_protocol.pbft_signature_threshold.as_deref(),
            Some("0.6")
        );
        assert_eq!(manifest.byron_protocol.configured_count(), 4);
        assert!(manifest.byron_protocol.config_complete());
        assert_eq!(
            manifest.checkpoints.file.as_deref(),
            Some("mainnet-checkpoints.json")
        );
        assert_eq!(
            manifest.checkpoints.expected_hash.as_deref(),
            Some("3e6dee5bae7acc6d870187e72674b37c929be8c66e62a552cf6a876b1af31ade")
        );
        assert_eq!(manifest.checkpoints.configured_count(), 2);
        assert!(manifest.checkpoints.config_complete());
        assert_eq!(manifest.tracing.turn_on_log_metrics, Some(true));
        assert_eq!(manifest.tracing.turn_on_logging, Some(true));
        assert_eq!(manifest.tracing.use_trace_dispatcher, Some(true));
        assert_eq!(manifest.tracing.min_severity.as_deref(), Some("Critical"));
        assert_eq!(manifest.tracing.runtime_configured_count(), 4);
        assert!(manifest.tracing.runtime_config_complete());
        assert_eq!(
            manifest.tracing.metrics_prefix.as_deref(),
            Some("cardano.node.metrics.")
        );
        assert_eq!(manifest.tracing.resource_frequency, Some(1000));
        assert_eq!(manifest.tracing.forwarder_conn_queue_size, Some(64));
        assert_eq!(manifest.tracing.forwarder_disconn_queue_size, Some(128));
        assert_eq!(manifest.tracing.forwarder_max_reconnect_delay, Some(30));
        assert_eq!(manifest.tracing.trace_scalar_fields, 4);
        assert_eq!(manifest.tracing.trace_option_entries, 2);
        assert_eq!(manifest.tracing.trace_severity_overrides, 2);
        assert_eq!(manifest.tracing.trace_severity_silence, 0);
        assert_eq!(manifest.tracing.trace_severity_debug, 0);
        assert_eq!(manifest.tracing.trace_severity_info, 1);
        assert_eq!(manifest.tracing.trace_severity_notice, 1);
        assert_eq!(manifest.tracing.trace_severity_warning, 0);
        assert_eq!(manifest.tracing.trace_severity_error, 0);
        assert_eq!(manifest.tracing.trace_severity_critical, 0);
        assert_eq!(manifest.tracing.trace_severity_other, 0);
        assert_eq!(manifest.tracing.trace_detail_overrides, 1);
        assert_eq!(manifest.tracing.trace_frequency_limits, 1);
        assert_eq!(manifest.tracing.trace_backend_entries, 2);
        assert_eq!(manifest.tracing.trace_backend_ekg, 1);
        assert_eq!(manifest.tracing.trace_backend_forwarder, 0);
        assert_eq!(manifest.tracing.trace_backend_prometheus, 1);
        assert_eq!(manifest.tracing.trace_backend_stdout, 0);
        assert_eq!(manifest.tracing.trace_backend_katip, 0);
        assert_eq!(manifest.tracing.trace_backend_other, 0);
        assert_eq!(
            manifest.tracing.trace_prometheus_host.as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(manifest.tracing.trace_prometheus_port, Some(12798));
        assert_eq!(manifest.tracing.default_backends, 1);
        assert_eq!(manifest.tracing.default_backend_ekg, 1);
        assert_eq!(manifest.tracing.default_backend_katip, 0);
        assert_eq!(manifest.tracing.default_backend_other, 0);
        assert_eq!(manifest.tracing.default_scribes, 1);
        assert_eq!(manifest.tracing.default_scribe_stdout, 1);
        assert_eq!(manifest.tracing.default_scribe_file, 0);
        assert_eq!(manifest.tracing.default_scribe_other, 0);
        assert_eq!(manifest.tracing.setup_backends, 1);
        assert_eq!(manifest.tracing.setup_backend_ekg, 0);
        assert_eq!(manifest.tracing.setup_backend_katip, 1);
        assert_eq!(manifest.tracing.setup_backend_other, 0);
        assert_eq!(manifest.tracing.setup_scribes, 1);
        assert_eq!(manifest.tracing.setup_scribe_stdout, 1);
        assert_eq!(manifest.tracing.setup_scribe_file, 0);
        assert_eq!(manifest.tracing.setup_scribe_other, 0);
        assert_eq!(manifest.tracing.legacy_trace_flags, 2);
        assert_eq!(manifest.tracing.legacy_trace_enabled, 1);
        assert_eq!(manifest.tracing.legacy_trace_disabled, 1);
        assert_eq!(manifest.tracing.has_ekg_port, Some(12788));
        assert_eq!(
            manifest.tracing.has_prometheus_host.as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(manifest.tracing.has_prometheus_port, Some(12798));
        assert_eq!(manifest.tracing.has_prometheus_items, 2);
        assert_eq!(
            manifest.tracing.tracing_verbosity.as_deref(),
            Some("NormalVerbosity")
        );
        assert_eq!(manifest.tracing.rotation_keep_files_num, Some(10));
        assert_eq!(manifest.tracing.rotation_log_limit_bytes, Some(5_000_000));
        assert_eq!(manifest.tracing.rotation_max_age_hours, Some(24));
        assert_eq!(manifest.tracing.legacy_map_backend_entries, 2);
        assert_eq!(manifest.tracing.legacy_map_backend_items, 2);
        assert_eq!(manifest.tracing.legacy_map_subtrace_entries, 1);
        assert_eq!(
            manifest.requires_network_magic,
            Some(RequiresNetworkMagic::RequiresNoMagic)
        );
        assert_eq!(manifest.genesis_files.len(), 4);

        let report = manifest.verify_genesis_hashes(&[
            GenesisFileDigest::new(
                "Byron",
                "5f20df933584822601f9e3f8c024eb5eb252fe8cefb24d1317dc3d432e940ebb",
            ),
            GenesisFileDigest::new(
                "Shelley",
                "1a3be38bcbb7911969283716ad7aa550250226b76a61fc51cc9a9a35d9276d81",
            ),
            GenesisFileDigest::new(
                "Alonzo",
                "7e94a15f55d1e82d10f09203fa1d40f8eede58fd8066542cf6566008068ed874",
            ),
            GenesisFileDigest::new(
                "Conway",
                "15a199f895e461ec0ffc6dd4e4028af28a492ab4e806d39cb674c88f7643ef62",
            ),
        ]);
        assert!(report.all_matched());
    }

    #[test]
    fn cardano_node_config_manifest_accepts_template_without_hashes() {
        let manifest = CardanoNodeConfigManifest::parse(
            r#"{
              "AlonzoGenesisFile": "alonzo-genesis.json",
              "ByronGenesisFile": "byron-genesis.json",
              "ConwayGenesisFile": "conway-genesis.json",
              "DijkstraGenesisFile": "dijkstra-genesis.json",
              "Protocol": "Cardano",
              "RequiresNetworkMagic": "RequiresMagic",
              "ShelleyGenesisFile": "shelley-genesis.json"
            }"#,
        )
        .unwrap();

        assert_eq!(manifest.genesis_files.len(), 5);
        let report = manifest.verify_genesis_hashes(&[]);
        assert!(!report.all_matched());
        assert!(report
            .checks
            .iter()
            .all(|check| check.status == GenesisHashStatus::MissingExpected));
    }

    #[test]
    fn cardano_node_config_manifest_accepts_null_optional_metadata() {
        let manifest = CardanoNodeConfigManifest::parse(
            r#"{
              "ApplicationName": null,
              "ApplicationVersion": null,
              "CheckpointsFile": null,
              "CheckpointsFileHash": null,
              "ExperimentalHardForksEnabled": null,
              "LedgerDB": { "SnapshotInterval": null },
              "PBftSignatureThreshold": null,
              "RequiresNetworkMagic": null,
              "ShelleyGenesisFile": "shelley-genesis.json",
              "ShelleyGenesisHash": null,
              "TraceOptionMetricsPrefix": null,
              "TurnOnLogging": null
            }"#,
        )
        .unwrap();

        assert_eq!(manifest.application.name, None);
        assert_eq!(manifest.application.version, None);
        assert_eq!(manifest.checkpoints.file, None);
        assert_eq!(manifest.checkpoints.expected_hash, None);
        assert_eq!(manifest.ledger_db.snapshot_interval, None);
        assert_eq!(manifest.byron_protocol.pbft_signature_threshold, None);
        assert_eq!(manifest.requires_network_magic, None);
        assert_eq!(
            manifest.protocol_features.experimental_hard_forks_enabled,
            None
        );
        assert_eq!(manifest.tracing.metrics_prefix, None);
        assert_eq!(manifest.tracing.turn_on_logging, None);
        assert_eq!(manifest.genesis_files[0].expected_hash, None);
    }

    #[test]
    fn cardano_node_config_manifest_reports_incomplete_mempool_timeouts() {
        let manifest = CardanoNodeConfigManifest::parse(
            r#"{
              "MempoolTimeoutSoft": 1.0,
              "ShelleyGenesisFile": "shelley-genesis.json"
            }"#,
        )
        .unwrap();

        assert_eq!(manifest.mempool.timeout_configured_count(), 1);
        assert!(!manifest.mempool.timeout_config_complete());
    }

    #[test]
    fn cardano_node_config_manifest_rejects_invalid_hash_shape() {
        assert_eq!(
            CardanoNodeConfigManifest::parse(
                r#"{
                  "ByronGenesisFile": "byron-genesis.json",
                  "ByronGenesisHash": "not-a-hash"
                }"#,
            ),
            Err(CardanoConfigError::InvalidGenesisHash {
                era: "Byron".to_string(),
                hash: "not-a-hash".to_string(),
            })
        );
    }

    #[test]
    fn cardano_node_config_manifest_rejects_duplicate_fields() {
        assert_eq!(
            CardanoNodeConfigManifest::parse(
                r#"{
                  "ShelleyGenesisFile": "shelley-a.json",
                  "ShelleyGenesisFile": "shelley-b.json"
                }"#,
            ),
            Err(CardanoConfigError::DuplicateField(
                "ShelleyGenesisFile".to_string()
            ))
        );
    }

    #[test]
    fn cardano_node_config_manifest_rejects_duplicate_array_fields() {
        assert_eq!(
            CardanoNodeConfigManifest::parse(
                r#"{
                  "ShelleyGenesisFile": "shelley-genesis.json",
                  "TraceOptions": {
                    "Net.PeerSelection": {
                      "backends": [],
                      "backends": ["EKGBackend"]
                    }
                  }
                }"#,
            ),
            Err(CardanoConfigError::DuplicateField(
                "TraceOptions.Net.PeerSelection.backends".to_string()
            ))
        );
    }

    #[test]
    fn cardano_node_config_manifest_rejects_invalid_numeric_metadata() {
        assert_eq!(
            CardanoNodeConfigManifest::parse(
                r#"{
                  "ShelleyGenesisFile": "shelley-genesis.json",
                  "LedgerDB": { "SnapshotInterval": "often" }
                }"#,
            ),
            Err(CardanoConfigError::InvalidNumber {
                field: "LedgerDB.SnapshotInterval".to_string(),
                value: "often".to_string(),
            })
        );
    }

    #[test]
    fn cardano_node_config_manifest_rejects_invalid_checkpoint_hash() {
        assert_eq!(
            CardanoNodeConfigManifest::parse(
                r#"{
                  "ShelleyGenesisFile": "shelley-genesis.json",
                  "CheckpointsFileHash": "bad"
                }"#,
            ),
            Err(CardanoConfigError::InvalidConfigHash {
                field: "CheckpointsFileHash".to_string(),
                hash: "bad".to_string(),
            })
        );
    }

    #[test]
    fn cardano_node_config_manifest_rejects_invalid_boolean_metadata() {
        assert_eq!(
            CardanoNodeConfigManifest::parse(
                r#"{
                  "ShelleyGenesisFile": "shelley-genesis.json",
                  "UseTraceDispatcher": "sometimes"
                }"#,
            ),
            Err(CardanoConfigError::InvalidBoolean {
                field: "UseTraceDispatcher".to_string(),
                value: "sometimes".to_string(),
            })
        );
    }

    #[test]
    fn cardano_node_config_manifest_reports_hash_mismatches_and_missing_actuals() {
        let manifest = CardanoNodeConfigManifest::parse(
            r#"{
              "ByronGenesisFile": "byron-genesis.json",
              "ByronGenesisHash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
              "ShelleyGenesisFile": "shelley-genesis.json",
              "ShelleyGenesisHash": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            }"#,
        )
        .unwrap();

        let report = manifest.verify_genesis_hashes(&[GenesisFileDigest::new(
            "Byron",
            "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )]);
        assert_eq!(report.checks[0].status, GenesisHashStatus::Mismatch);
        assert_eq!(report.checks[1].status, GenesisHashStatus::MissingActual);
        assert!(!report.all_matched());
    }

    #[test]
    fn blake2b_256_hex_matches_known_vectors() {
        assert_eq!(
            blake2b_256_hex(b""),
            "0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8"
        );
        assert_eq!(
            blake2b_256_hex(b"abc"),
            "bddd813c634239723171ef3fee98579b94964e3bb1cb3e427262c8c068d52319"
        );
    }

    #[test]
    fn raw_genesis_file_digest_support_skips_byron_canonical_hash() {
        assert!(!raw_genesis_file_digest_supported("Byron"));
        assert!(raw_genesis_file_digest_supported("Shelley"));
        assert!(raw_genesis_file_digest_supported("Alonzo"));
        assert!(raw_genesis_file_digest_supported("Conway"));
    }

    #[test]
    fn cardano_node_config_manifest_checks_network_magic_requirement() {
        let mainnet = CardanoNodeConfigManifest::parse(
            r#"{
              "ByronGenesisFile": "mainnet-byron-genesis.json",
              "RequiresNetworkMagic": "RequiresNoMagic"
            }"#,
        )
        .unwrap();
        assert_eq!(
            mainnet
                .check_network_magic_requirement(network_profile("mainnet").unwrap())
                .status,
            NetworkMagicRequirementStatus::Match
        );
        assert_eq!(
            mainnet
                .check_network_magic_requirement(network_profile("preprod").unwrap())
                .status,
            NetworkMagicRequirementStatus::Mismatch
        );

        let missing =
            CardanoNodeConfigManifest::parse(r#"{ "ShelleyGenesisFile": "shelley-genesis.json" }"#)
                .unwrap();
        assert_eq!(
            missing
                .check_network_magic_requirement(network_profile("preview").unwrap())
                .status,
            NetworkMagicRequirementStatus::MissingConfig
        );
    }
}
