pub mod cardano;

use crate::ledger::eras::AgeParams;
use crate::sync::first_light_window_slots;
use std::env;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DataMode {
    #[default]
    Core,
    Full,
}

impl DataMode {
    pub fn carries_interface_data(self) -> bool {
        matches!(self, Self::Full)
    }
}

impl fmt::Display for DataMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core => f.write_str("core"),
            Self::Full => f.write_str("full"),
        }
    }
}

impl FromStr for DataMode {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "core" => Ok(Self::Core),
            "full" => Ok(Self::Full),
            other => Err(ConfigError::InvalidDataMode(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OperatingMode {
    #[default]
    Listen,
    Load,
    Produce,
    Braid,
}

impl fmt::Display for OperatingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Listen => f.write_str("listen"),
            Self::Load => f.write_str("load"),
            Self::Produce => f.write_str("produce"),
            Self::Braid => f.write_str("batch"),
        }
    }
}

impl FromStr for OperatingMode {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "listen" | "serve" => Ok(Self::Listen),
            "load" => Ok(Self::Load),
            "produce" | "forge" | "dev" => Ok(Self::Produce),
            "batch" | "braid" => Ok(Self::Braid),
            other => Err(ConfigError::InvalidOperatingMode(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenerPlan {
    pub name: String,
    pub bind_addr: String,
    pub port: u16,
    pub enabled: bool,
}

impl ListenerPlan {
    pub fn closed(name: impl Into<String>, bind_addr: impl Into<String>, port: u16) -> Self {
        Self {
            name: name.into(),
            bind_addr: bind_addr.into(),
            port,
            enabled: false,
        }
    }

    pub fn endpoint(&self) -> String {
        format!("{}:{}", self.bind_addr, self.port)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FadePattern {
    pub enabled: bool,
    pub frequency: Duration,
}

impl Default for FadePattern {
    fn default() -> Self {
        Self {
            enabled: false,
            frequency: Duration::from_secs(60 * 60),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlockProducerConfig {
    pub enabled: bool,
    pub tree_key: Option<PathBuf>,
    pub producer_key: Option<PathBuf>,
    pub operational_certificate: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InterfaceConfig {
    pub transaction_port: u16,
    pub state_port: u16,
    pub batch_port: u16,
    pub archive_port: u16,
    pub archive_base_url: Option<String>,
}

impl InterfaceConfig {
    pub fn any_open(&self) -> bool {
        self.transaction_port != 0
            || self.state_port != 0
            || self.batch_port != 0
            || self.archive_port != 0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SafetyConfig {
    pub allow_paths: bool,
    pub allow_state_mutation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    pub network_name: String,
    pub network_magic: u32,
    pub store_dir: PathBuf,
    pub source_config_path: PathBuf,
    pub topology_path: Option<PathBuf>,
    pub local_socket_path: PathBuf,
    pub outer_bind_addr: String,
    pub inner_bind_addr: String,
    pub outer_port: u16,
    pub inner_port: u16,
    pub seer_port: u16,
    pub data_mode: DataMode,
    pub operating_mode: OperatingMode,
    pub pruning: FadePattern,
    pub block_producer: BlockProducerConfig,
    pub interfaces: InterfaceConfig,
    pub safety: SafetyConfig,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            network_name: "local".to_string(),
            network_magic: 2,
            store_dir: PathBuf::from(".acropolis"),
            source_config_path: PathBuf::from("./config/local.json"),
            topology_path: None,
            local_socket_path: PathBuf::from("acropolis.socket"),
            outer_bind_addr: "0.0.0.0".to_string(),
            inner_bind_addr: "127.0.0.1".to_string(),
            outer_port: 3001,
            inner_port: 3002,
            seer_port: 12798,
            data_mode: DataMode::Core,
            operating_mode: OperatingMode::Listen,
            pruning: FadePattern::default(),
            block_producer: BlockProducerConfig::default(),
            interfaces: InterfaceConfig::default(),
            safety: SafetyConfig::default(),
        }
    }
}

impl NodeConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_sources(None, None)
    }

    pub fn from_sources(
        file_patch: Option<ConfigPatch>,
        cli_patch: Option<ConfigPatch>,
    ) -> Result<Self, ConfigError> {
        Self::from_ordered_patches(file_patch, Some(ConfigPatch::from_env()?), cli_patch)
    }

    pub fn from_ordered_patches(
        file_patch: Option<ConfigPatch>,
        env_patch: Option<ConfigPatch>,
        cli_patch: Option<ConfigPatch>,
    ) -> Result<Self, ConfigError> {
        let mut cfg = Self::default();
        let mut overlays = Vec::new();
        if let Some(patch) = file_patch.filter(|patch| !patch.is_empty()) {
            overlays.push(ConfigOverlay {
                source: ConfigSource::File,
                patch,
            });
        }
        if let Some(patch) = env_patch.filter(|patch| !patch.is_empty()) {
            overlays.push(ConfigOverlay {
                source: ConfigSource::Env,
                patch,
            });
        }
        if let Some(patch) = cli_patch.filter(|patch| !patch.is_empty()) {
            overlays.push(ConfigOverlay {
                source: ConfigSource::Cli,
                patch,
            });
        }
        cfg.apply_overlays(&overlays)?;
        cfg.populate_network_magic()?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.network_name.trim().is_empty() {
            return Err(ConfigError::Missing("network_name"));
        }
        if self.network_magic == 0 {
            return Err(ConfigError::Missing("network_magic"));
        }
        if self.interfaces.any_open() && !self.data_mode.carries_interface_data() {
            return Err(ConfigError::InterfacesRequireFullData);
        }
        if self.block_producer.enabled
            && (self.block_producer.tree_key.is_none()
                || self.block_producer.producer_key.is_none()
                || self.block_producer.operational_certificate.is_none())
        {
            return Err(ConfigError::BlockProducerKeysRequired);
        }
        Ok(())
    }

    pub fn populate_network_magic(&mut self) -> Result<(), ConfigError> {
        if self.network_magic == 0 {
            self.network_magic = network_magic(&self.network_name)
                .ok_or_else(|| ConfigError::UnknownNetwork(self.network_name.clone()))?;
        }
        Ok(())
    }

    pub fn listener_plan(&self) -> Vec<ListenerPlan> {
        vec![
            ListenerPlan::closed(
                "network-outer",
                self.outer_bind_addr.clone(),
                self.outer_port,
            ),
            ListenerPlan::closed(
                "network-inner",
                self.inner_bind_addr.clone(),
                self.inner_port,
            ),
            ListenerPlan::closed("metrics", self.inner_bind_addr.clone(), self.seer_port),
            ListenerPlan::closed(
                "transaction-interface",
                self.inner_bind_addr.clone(),
                self.interfaces.transaction_port,
            ),
            ListenerPlan::closed(
                "state-interface",
                self.inner_bind_addr.clone(),
                self.interfaces.state_port,
            ),
            ListenerPlan::closed(
                "batch-interface",
                self.inner_bind_addr.clone(),
                self.interfaces.batch_port,
            ),
            ListenerPlan::closed(
                "archive-interface",
                self.inner_bind_addr.clone(),
                self.interfaces.archive_port,
            ),
        ]
    }

    pub fn apply_patch(&mut self, patch: &ConfigPatch) -> Result<(), ConfigError> {
        for entry in &patch.entries {
            self.apply_patch_entry(entry)?;
        }
        self.validate()
    }

    pub fn apply_overlays(&mut self, overlays: &[ConfigOverlay]) -> Result<(), ConfigError> {
        let mut overlays = overlays.to_vec();
        overlays.sort_by_key(|overlay| overlay.source.priority());
        for overlay in &overlays {
            for entry in &overlay.patch.entries {
                self.apply_patch_entry(entry)?;
            }
        }
        self.validate()
    }

    fn apply_patch_entry(&mut self, entry: &ConfigPatchEntry) -> Result<(), ConfigError> {
        match entry.key.as_str() {
            "network" | "network_name" => {
                self.network_name = entry.value.clone();
                self.network_magic = network_magic(&self.network_name)
                    .ok_or_else(|| ConfigError::UnknownNetwork(self.network_name.clone()))?;
            }
            "network_magic" => self.network_magic = parse_u32_value("network_magic", &entry.value)?,
            "source_config" | "source_config_path" => {
                self.source_config_path = PathBuf::from(&entry.value)
            }
            "store_dir" => self.store_dir = PathBuf::from(&entry.value),
            "topology" | "topology_path" => {
                self.topology_path = parse_optional_path_value(&entry.value)
            }
            "socket" | "local_socket_path" => self.local_socket_path = PathBuf::from(&entry.value),
            "outer_bind" | "outer_bind_addr" => self.outer_bind_addr = entry.value.clone(),
            "inner_bind" | "inner_bind_addr" => self.inner_bind_addr = entry.value.clone(),
            "outer_port" => self.outer_port = parse_port_value(&entry.value)?,
            "inner_port" => self.inner_port = parse_port_value(&entry.value)?,
            "metrics_port" | "seer_port" => self.seer_port = parse_port_value(&entry.value)?,
            "data_mode" => self.data_mode = entry.value.parse()?,
            "mode" | "operating_mode" => self.operating_mode = entry.value.parse()?,
            "pruning_enabled" => self.pruning.enabled = parse_bool_value(&entry.value)?,
            "block_producer_enabled" => {
                self.block_producer.enabled = parse_bool_value(&entry.value)?
            }
            "producer_tree_key" | "tree_key" => {
                self.block_producer.tree_key = Some(PathBuf::from(&entry.value))
            }
            "producer_key" => self.block_producer.producer_key = Some(PathBuf::from(&entry.value)),
            "producer_certificate" | "operational_certificate" => {
                self.block_producer.operational_certificate = Some(PathBuf::from(&entry.value))
            }
            "allow_paths" => self.safety.allow_paths = parse_bool_value(&entry.value)?,
            "allow_state_mutation" => {
                self.safety.allow_state_mutation = parse_bool_value(&entry.value)?
            }
            "transaction_port" => {
                self.interfaces.transaction_port = parse_port_value(&entry.value)?
            }
            "state_port" => self.interfaces.state_port = parse_port_value(&entry.value)?,
            "batch_port" => self.interfaces.batch_port = parse_port_value(&entry.value)?,
            "archive_port" => self.interfaces.archive_port = parse_port_value(&entry.value)?,
            "archive_base_url" => {
                self.interfaces.archive_base_url = parse_optional_string_value(&entry.value)
            }
            other => return Err(ConfigError::UnknownPatchKey(other.to_string())),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPatch {
    pub entries: Vec<ConfigPatchEntry>,
}

impl ConfigPatch {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn push(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        self.entries.push(ConfigPatchEntry {
            key: normalize_patch_key(&key),
            value: value.into().trim().to_string(),
        });
    }

    pub fn extend(&mut self, other: ConfigPatch) {
        self.entries.extend(other.entries);
    }

    pub fn parse(input: &str) -> Result<Self, ConfigError> {
        if input.trim_start().starts_with('{') {
            return StructuredPatchParser::new(input).parse();
        }
        let mut patch = Self::new();
        for (line_no, raw_line) in input.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or(ConfigError::InvalidPatchLine { line: line_no + 1 })?;
            patch.push(key, value);
        }
        Ok(patch)
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let mut patch = Self::new();
        push_env_string(&mut patch, "network", "ACROPOLIS_NETWORK");
        push_env_string(&mut patch, "source_config_path", "ACROPOLIS_SOURCE_CONFIG");
        push_env_string(&mut patch, "store_dir", "ACROPOLIS_STORE_DIR");
        push_env_string(&mut patch, "topology_path", "ACROPOLIS_TOPOLOGY");
        push_env_string(&mut patch, "local_socket_path", "ACROPOLIS_SOCKET");
        push_env_string(&mut patch, "outer_bind_addr", "ACROPOLIS_OUTER_BIND");
        push_env_string(&mut patch, "inner_bind_addr", "ACROPOLIS_INNER_BIND");
        push_env_u16(&mut patch, "outer_port", "ACROPOLIS_OUTER_PORT")?;
        push_env_u16(&mut patch, "inner_port", "ACROPOLIS_INNER_PORT")?;
        push_env_u16(&mut patch, "metrics_port", "ACROPOLIS_METRICS_PORT")?;
        push_env_string(&mut patch, "data_mode", "ACROPOLIS_DATA_MODE");
        push_env_string(&mut patch, "operating_mode", "ACROPOLIS_MODE");
        push_env_bool(&mut patch, "pruning_enabled", "ACROPOLIS_PRUNING_ENABLED")?;
        push_env_u16(&mut patch, "transaction_port", "ACROPOLIS_TRANSACTION_PORT")?;
        push_env_u16(&mut patch, "state_port", "ACROPOLIS_STATE_PORT")?;
        push_env_u16(&mut patch, "batch_port", "ACROPOLIS_BATCH_PORT")?;
        push_env_u16(&mut patch, "archive_port", "ACROPOLIS_ARCHIVE_PORT")?;
        push_env_string(&mut patch, "archive_base_url", "ACROPOLIS_ARCHIVE_BASE_URL");
        push_env_bool(
            &mut patch,
            "block_producer_enabled",
            "ACROPOLIS_BLOCK_PRODUCER_ENABLED",
        )?;
        push_env_string(
            &mut patch,
            "producer_tree_key",
            "ACROPOLIS_PRODUCER_TREE_KEY",
        );
        push_env_string(&mut patch, "producer_key", "ACROPOLIS_PRODUCER_KEY");
        push_env_string(
            &mut patch,
            "producer_certificate",
            "ACROPOLIS_PRODUCER_CERTIFICATE",
        );
        push_env_bool(&mut patch, "allow_paths", "ACROPOLIS_ALLOW_PATHS")?;
        push_env_bool(
            &mut patch,
            "allow_state_mutation",
            "ACROPOLIS_ALLOW_STATE_MUTATION",
        )?;
        Ok(patch)
    }
}

impl Default for ConfigPatch {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPatchEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigSource {
    Defaults,
    File,
    Env,
    Cli,
}

impl ConfigSource {
    pub fn priority(self) -> u8 {
        match self {
            Self::Defaults => 0,
            Self::File => 1,
            Self::Env => 2,
            Self::Cli => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigOverlay {
    pub source: ConfigSource,
    pub patch: ConfigPatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Missing(&'static str),
    InvalidBool { name: String, value: String },
    InvalidPort { name: String, value: String },
    InvalidNumber { name: String, value: String },
    InvalidDataMode(String),
    InvalidOperatingMode(String),
    UnknownNetwork(String),
    InterfacesRequireFullData,
    BlockProducerKeysRequired,
    UnknownPatchKey(String),
    InvalidPatchLine { line: usize },
    InvalidStructuredConfig(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(name) => write!(f, "missing required config field {name}"),
            Self::InvalidBool { name, value } => write!(f, "invalid boolean {name}={value}"),
            Self::InvalidPort { name, value } => write!(f, "invalid port {name}={value}"),
            Self::InvalidNumber { name, value } => write!(f, "invalid number {name}={value}"),
            Self::InvalidDataMode(value) => write!(f, "invalid data mode {value}"),
            Self::InvalidOperatingMode(value) => write!(f, "invalid operating mode {value}"),
            Self::UnknownNetwork(value) => write!(f, "unknown network {value}"),
            Self::InterfacesRequireFullData => {
                f.write_str("open interfaces require data mode full")
            }
            Self::BlockProducerKeysRequired => {
                f.write_str("block production requires tree, producer, and certificate paths")
            }
            Self::UnknownPatchKey(key) => write!(f, "unknown config patch key {key}"),
            Self::InvalidPatchLine { line } => write!(f, "invalid config patch line {line}"),
            Self::InvalidStructuredConfig(message) => {
                write!(f, "invalid structured config: {message}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn network_magic(network: &str) -> Option<u32> {
    network_profile(network).map(|profile| profile.network_magic)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkProfileKind {
    Public,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapMode {
    RequiresOptIn,
    LocalOnly,
}

impl fmt::Display for BootstrapMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequiresOptIn => f.write_str("opt-in"),
            Self::LocalOnly => f.write_str("local"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkProfile {
    pub name: &'static str,
    pub network_magic: u32,
    pub kind: NetworkProfileKind,
    pub default_node_port: u16,
    pub era: &'static str,
    pub slot_length_ms: u64,
    pub epoch_length_slots: u64,
    pub security_parameter: u64,
    pub active_slots_coefficient: (u64, u64),
    pub bootstrap_mode: BootstrapMode,
    pub bootstrap_requires_opt_in: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkProfileRuntimeParams {
    pub profile: NetworkProfile,
    pub age_params: AgeParams,
    pub first_light_window_slots: u64,
}

impl NetworkProfile {
    pub fn validate_offline(&self) -> Result<(), NetworkProfileError> {
        if self.name.trim().is_empty() {
            return Err(NetworkProfileError::MissingName);
        }
        if self.network_magic == 0 {
            return Err(NetworkProfileError::MissingNetworkMagic { name: self.name });
        }
        if self.default_node_port == 0 {
            return Err(NetworkProfileError::MissingDefaultPort { name: self.name });
        }
        if self.era.trim().is_empty() {
            return Err(NetworkProfileError::MissingEra { name: self.name });
        }
        if self.slot_length_ms == 0 {
            return Err(NetworkProfileError::MissingSlotLength { name: self.name });
        }
        if self.epoch_length_slots == 0 {
            return Err(NetworkProfileError::MissingEpochLength { name: self.name });
        }
        if self.security_parameter == 0 {
            return Err(NetworkProfileError::MissingSecurityParameter { name: self.name });
        }
        let (numerator, denominator) = self.active_slots_coefficient;
        if numerator == 0 || denominator == 0 || numerator > denominator {
            return Err(NetworkProfileError::InvalidActiveSlotsCoefficient { name: self.name });
        }
        if self.bootstrap_requires_opt_in
            != matches!(self.bootstrap_mode, BootstrapMode::RequiresOptIn)
        {
            return Err(NetworkProfileError::BootstrapModeMismatch { name: self.name });
        }
        Ok(())
    }

    pub fn active_slots_ratio_text(&self) -> String {
        format!(
            "{}/{}",
            self.active_slots_coefficient.0, self.active_slots_coefficient.1
        )
    }

    pub fn active_slots_ratio(&self) -> f64 {
        self.active_slots_coefficient.0 as f64 / self.active_slots_coefficient.1 as f64
    }

    pub fn runtime_params(&self) -> Result<NetworkProfileRuntimeParams, NetworkProfileError> {
        self.validate_offline()?;
        let first_light_window_slots =
            first_light_window_slots(self.security_parameter, self.active_slots_ratio());
        Ok(NetworkProfileRuntimeParams {
            profile: *self,
            age_params: AgeParams {
                epoch_size: self.epoch_length_slots,
                slot_length: Duration::from_millis(self.slot_length_ms),
                safe_zone_slots: self.security_parameter,
                first_light_window: first_light_window_slots,
            },
            first_light_window_slots,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkProfileError {
    MissingName,
    MissingNetworkMagic { name: &'static str },
    MissingDefaultPort { name: &'static str },
    MissingEra { name: &'static str },
    MissingSlotLength { name: &'static str },
    MissingEpochLength { name: &'static str },
    MissingSecurityParameter { name: &'static str },
    InvalidActiveSlotsCoefficient { name: &'static str },
    BootstrapModeMismatch { name: &'static str },
}

impl fmt::Display for NetworkProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingName => f.write_str("network profile is missing a name"),
            Self::MissingNetworkMagic { name } => {
                write!(f, "network profile {name} is missing network magic")
            }
            Self::MissingDefaultPort { name } => {
                write!(f, "network profile {name} is missing default port")
            }
            Self::MissingEra { name } => write!(f, "network profile {name} is missing era"),
            Self::MissingSlotLength { name } => {
                write!(f, "network profile {name} is missing slot length")
            }
            Self::MissingEpochLength { name } => {
                write!(f, "network profile {name} is missing epoch length")
            }
            Self::MissingSecurityParameter { name } => {
                write!(f, "network profile {name} is missing security parameter")
            }
            Self::InvalidActiveSlotsCoefficient { name } => {
                write!(
                    f,
                    "network profile {name} has invalid active slots coefficient"
                )
            }
            Self::BootstrapModeMismatch { name } => {
                write!(f, "network profile {name} bootstrap flags disagree")
            }
        }
    }
}

impl std::error::Error for NetworkProfileError {}

pub const NETWORK_PROFILES: [NetworkProfile; 5] = [
    NetworkProfile {
        name: "mainnet",
        network_magic: 764_824_073,
        kind: NetworkProfileKind::Public,
        default_node_port: 3001,
        era: "conway",
        slot_length_ms: 1_000,
        epoch_length_slots: 432_000,
        security_parameter: 2_160,
        active_slots_coefficient: (5, 100),
        bootstrap_mode: BootstrapMode::RequiresOptIn,
        bootstrap_requires_opt_in: true,
    },
    NetworkProfile {
        name: "preprod",
        network_magic: 1,
        kind: NetworkProfileKind::Public,
        default_node_port: 3001,
        era: "conway",
        slot_length_ms: 1_000,
        epoch_length_slots: 432_000,
        security_parameter: 2_160,
        active_slots_coefficient: (5, 100),
        bootstrap_mode: BootstrapMode::RequiresOptIn,
        bootstrap_requires_opt_in: true,
    },
    NetworkProfile {
        name: "preview",
        network_magic: 2,
        kind: NetworkProfileKind::Public,
        default_node_port: 3001,
        era: "conway",
        slot_length_ms: 1_000,
        epoch_length_slots: 86_400,
        security_parameter: 432,
        active_slots_coefficient: (5, 100),
        bootstrap_mode: BootstrapMode::RequiresOptIn,
        bootstrap_requires_opt_in: true,
    },
    NetworkProfile {
        name: "local",
        network_magic: 2,
        kind: NetworkProfileKind::Local,
        default_node_port: 3001,
        era: "local-conway",
        slot_length_ms: 1_000,
        epoch_length_slots: 1_000,
        security_parameter: 10,
        active_slots_coefficient: (1, 1),
        bootstrap_mode: BootstrapMode::LocalOnly,
        bootstrap_requires_opt_in: false,
    },
    NetworkProfile {
        name: "staging",
        network_magic: 4,
        kind: NetworkProfileKind::Local,
        default_node_port: 3001,
        era: "local-conway",
        slot_length_ms: 1_000,
        epoch_length_slots: 1_000,
        security_parameter: 10,
        active_slots_coefficient: (1, 1),
        bootstrap_mode: BootstrapMode::LocalOnly,
        bootstrap_requires_opt_in: false,
    },
];

pub fn network_profiles() -> &'static [NetworkProfile] {
    &NETWORK_PROFILES
}

pub fn network_profile(network: &str) -> Option<NetworkProfile> {
    match network.trim().to_ascii_lowercase().as_str() {
        "mainnet" => Some(NETWORK_PROFILES[0]),
        "preprod" | "testnet" => Some(NETWORK_PROFILES[1]),
        "preview" => Some(NETWORK_PROFILES[2]),
        "local" => Some(NETWORK_PROFILES[3]),
        "staging" => Some(NETWORK_PROFILES[4]),
        _ => None,
    }
}

fn env_string(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn push_env_string(patch: &mut ConfigPatch, key: &'static str, name: &'static str) {
    if let Some(value) = env_string(name) {
        patch.push(key, value);
    }
}

fn push_env_bool(
    patch: &mut ConfigPatch,
    key: &'static str,
    name: &'static str,
) -> Result<(), ConfigError> {
    if let Some(value) = env_bool(name)? {
        patch.push(key, value.to_string());
    }
    Ok(())
}

fn push_env_u16(
    patch: &mut ConfigPatch,
    key: &'static str,
    name: &'static str,
) -> Result<(), ConfigError> {
    if let Some(value) = env_u16(name)? {
        patch.push(key, value.to_string());
    }
    Ok(())
}

fn env_bool(name: &str) -> Result<Option<bool>, ConfigError> {
    let Some(value) = env_string(name) else {
        return Ok(None);
    };
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Ok(Some(true)),
        "0" | "false" | "no" | "n" | "off" => Ok(Some(false)),
        _ => Err(ConfigError::InvalidBool {
            name: name.to_string(),
            value,
        }),
    }
}

fn parse_bool_value(value: &str) -> Result<bool, ConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "no" | "n" | "off" => Ok(false),
        _ => Err(ConfigError::InvalidBool {
            name: "patch".to_string(),
            value: value.to_string(),
        }),
    }
}

fn parse_port_value(value: &str) -> Result<u16, ConfigError> {
    value.parse().map_err(|_| ConfigError::InvalidPort {
        name: "patch".to_string(),
        value: value.to_string(),
    })
}

fn parse_u32_value(name: &'static str, value: &str) -> Result<u32, ConfigError> {
    value.parse().map_err(|_| ConfigError::InvalidNumber {
        name: name.to_string(),
        value: value.to_string(),
    })
}

fn parse_optional_path_value(value: &str) -> Option<PathBuf> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "none" | "null" | "off" => None,
        _ => Some(PathBuf::from(value)),
    }
}

fn parse_optional_string_value(value: &str) -> Option<String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "none" | "null" | "off" => None,
        _ => Some(value.to_string()),
    }
}

fn normalize_patch_key(key: &str) -> String {
    let key = key.trim().to_ascii_lowercase().replace('-', "_");
    match key.as_str() {
        "network.name" => "network".to_string(),
        "network.magic" => "network_magic".to_string(),
        "network.outer_bind" | "network.outer_bind_addr" => "outer_bind_addr".to_string(),
        "network.inner_bind" | "network.inner_bind_addr" => "inner_bind_addr".to_string(),
        "network.outer_port" => "outer_port".to_string(),
        "network.inner_port" => "inner_port".to_string(),
        "network.metrics_port" | "network.seer_port" => "metrics_port".to_string(),
        "storage.source_config" | "storage.source_config_path" => "source_config_path".to_string(),
        "storage.store_dir" => "store_dir".to_string(),
        "storage.topology" | "storage.topology_path" => "topology_path".to_string(),
        "storage.socket" | "storage.local_socket_path" => "local_socket_path".to_string(),
        "interfaces.transaction_port" => "transaction_port".to_string(),
        "interfaces.state_port" => "state_port".to_string(),
        "interfaces.batch_port" => "batch_port".to_string(),
        "interfaces.archive_port" => "archive_port".to_string(),
        "interfaces.archive_base_url" => "archive_base_url".to_string(),
        "safety.allow_paths" => "allow_paths".to_string(),
        "safety.allow_state_mutation" => "allow_state_mutation".to_string(),
        "pruning.enabled" => "pruning_enabled".to_string(),
        "block_producer.enabled" => "block_producer_enabled".to_string(),
        "block_producer.tree_key" => "producer_tree_key".to_string(),
        "block_producer.producer_key" => "producer_key".to_string(),
        "block_producer.certificate" | "block_producer.operational_certificate" => {
            "producer_certificate".to_string()
        }
        _ => key,
    }
}

struct StructuredPatchParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> StructuredPatchParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn parse(mut self) -> Result<ConfigPatch, ConfigError> {
        let mut patch = ConfigPatch::new();
        let mut prefix = Vec::new();
        self.parse_object(&mut patch, &mut prefix)?;
        self.skip_ws();
        if self.peek().is_some() {
            return self.err("trailing input after object");
        }
        Ok(patch)
    }

    fn parse_object(
        &mut self,
        patch: &mut ConfigPatch,
        prefix: &mut Vec<String>,
    ) -> Result<(), ConfigError> {
        self.expect('{')?;
        loop {
            self.skip_ws();
            if self.consume('}') {
                return Ok(());
            }
            let key = self.parse_key()?;
            self.skip_ws();
            self.expect(':')?;
            self.skip_ws();
            if self.peek() == Some('{') {
                prefix.push(key);
                self.parse_object(patch, prefix)?;
                prefix.pop();
            } else {
                let value = self.parse_scalar()?;
                let mut parts = prefix.clone();
                parts.push(key);
                patch.push(parts.join("."), value);
            }
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

    fn parse_key(&mut self) -> Result<String, ConfigError> {
        self.skip_ws();
        if self.peek() == Some('"') {
            return self.parse_string();
        }
        let key = self.parse_bare_token();
        if key.is_empty() {
            return self.err("expected object key");
        }
        Ok(key)
    }

    fn parse_scalar(&mut self) -> Result<String, ConfigError> {
        self.skip_ws();
        if self.peek() == Some('"') {
            return self.parse_string();
        }
        let value = self.parse_bare_token();
        if value.is_empty() {
            return self.err("expected scalar value");
        }
        Ok(if value == "null" {
            "none".to_string()
        } else {
            value
        })
    }

    fn parse_string(&mut self) -> Result<String, ConfigError> {
        self.expect('"')?;
        let mut out = String::new();
        while let Some(ch) = self.next() {
            match ch {
                '"' => return Ok(out),
                '\\' => {
                    let Some(escaped) = self.next() else {
                        return self.err("unterminated escape sequence");
                    };
                    match escaped {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        other => {
                            return self.err(&format!("unsupported escape sequence \\{other}"));
                        }
                    }
                }
                other => out.push(other),
            }
        }
        self.err("unterminated string")
    }

    fn parse_bare_token(&mut self) -> String {
        let mut out = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || ch == ',' || ch == '}' {
                break;
            }
            out.push(ch);
            self.next();
        }
        out
    }

    fn expect(&mut self, expected: char) -> Result<(), ConfigError> {
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

    fn err<T>(&self, message: &str) -> Result<T, ConfigError> {
        Err(ConfigError::InvalidStructuredConfig(format!(
            "{message} at byte {}",
            self.pos
        )))
    }
}

fn env_u16(name: &str) -> Result<Option<u16>, ConfigError> {
    let Some(value) = env_string(name) else {
        return Ok(None);
    };
    value
        .parse()
        .map(Some)
        .map_err(|_| ConfigError::InvalidPort {
            name: name.to_string(),
            value,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_local_core_and_guarded() {
        let cfg = NodeConfig::default();
        assert_eq!(cfg.network_name, "local");
        assert_eq!(cfg.network_magic, 2);
        assert_eq!(cfg.data_mode, DataMode::Core);
        assert!(!cfg.safety.allow_paths);
    }

    #[test]
    fn open_interfaces_require_full_data() {
        let mut cfg = NodeConfig::default();
        cfg.interfaces.state_port = 3000;
        assert_eq!(cfg.validate(), Err(ConfigError::InterfacesRequireFullData));
        cfg.data_mode = DataMode::Full;
        assert_eq!(cfg.validate(), Ok(()));
    }

    #[test]
    fn known_network_magic_matches_supported_aliases() {
        assert_eq!(network_magic("mainnet"), Some(764_824_073));
        assert_eq!(network_magic("testnet"), Some(1));
        assert_eq!(network_magic("local"), Some(2));
    }

    #[test]
    fn offline_network_profiles_have_valid_metadata() {
        for profile in network_profiles() {
            profile.validate_offline().unwrap();
            assert!(!profile.era.is_empty());
            assert!(profile.epoch_length_slots >= profile.security_parameter);
            assert!(profile.runtime_params().unwrap().first_light_window_slots > 0);
        }
        assert_eq!(
            network_profile("mainnet").unwrap().security_parameter,
            2_160
        );
        assert_eq!(
            network_profile("preview").unwrap().epoch_length_slots,
            86_400
        );
        assert_eq!(
            network_profile("local").unwrap().bootstrap_mode,
            BootstrapMode::LocalOnly
        );
    }

    #[test]
    fn network_profiles_derive_local_runtime_params_without_genesis_fetch() {
        let mainnet = network_profile("mainnet")
            .unwrap()
            .runtime_params()
            .unwrap();
        assert_eq!(mainnet.profile.name, "mainnet");
        assert_eq!(mainnet.age_params.epoch_size, 432_000);
        assert_eq!(mainnet.age_params.slot_length, Duration::from_millis(1_000));
        assert_eq!(mainnet.age_params.safe_zone_slots, 2_160);
        assert_eq!(mainnet.first_light_window_slots, 129_600);
        assert_eq!(mainnet.age_params.first_light_window, 129_600);

        let local = network_profile("local").unwrap().runtime_params().unwrap();
        assert_eq!(local.profile.name, "local");
        assert_eq!(local.age_params.epoch_size, 1_000);
        assert_eq!(local.first_light_window_slots, 30);
    }

    #[test]
    fn pattern_patch_applies_local_text_overrides() {
        let patch =
            ConfigPatch::parse("network=staging\ndata_mode=full\ntransaction_port=4000\n").unwrap();
        let mut config = NodeConfig::default();
        config.apply_patch(&patch).unwrap();
        assert_eq!(config.network_name, "staging");
        assert_eq!(config.network_magic, 4);
        assert_eq!(config.data_mode, DataMode::Full);
        assert_eq!(config.interfaces.transaction_port, 4000);
    }

    #[test]
    fn structured_config_object_applies_nested_local_overrides() {
        let patch = ConfigPatch::parse(
            r#"{
                "network": { "name": "staging" },
                "data_mode": "full",
                "interfaces": {
                    "transaction_port": 4000,
                    "archive_base_url": null
                },
                "safety": { "allow_paths": false }
            }"#,
        )
        .unwrap();
        let mut config = NodeConfig::default();
        config.apply_patch(&patch).unwrap();

        assert_eq!(config.network_name, "staging");
        assert_eq!(config.network_magic, 4);
        assert_eq!(config.data_mode, DataMode::Full);
        assert_eq!(config.interfaces.transaction_port, 4000);
        assert_eq!(config.interfaces.archive_base_url, None);
        assert!(!config.safety.allow_paths);
    }

    #[test]
    fn structured_config_rejects_malformed_object() {
        assert!(matches!(
            ConfigPatch::parse(r#"{ "network": { "name": "staging" }"#),
            Err(ConfigError::InvalidStructuredConfig(_))
        ));
    }

    #[test]
    fn pattern_overlays_apply_cli_env_file_priority_before_final_validation() {
        let overlays = vec![
            ConfigOverlay {
                source: ConfigSource::Cli,
                patch: ConfigPatch::parse("network=staging\ndata_mode=full\n").unwrap(),
            },
            ConfigOverlay {
                source: ConfigSource::File,
                patch: ConfigPatch::parse("network=testnet\ntransaction_port=4000\n").unwrap(),
            },
            ConfigOverlay {
                source: ConfigSource::Env,
                patch: ConfigPatch::parse("network=local\n").unwrap(),
            },
        ];
        let mut config = NodeConfig::default();
        config.apply_overlays(&overlays).unwrap();
        assert_eq!(config.network_name, "staging");
        assert_eq!(config.network_magic, 4);
        assert_eq!(config.data_mode, DataMode::Full);
        assert_eq!(config.interfaces.transaction_port, 4000);
    }
}
