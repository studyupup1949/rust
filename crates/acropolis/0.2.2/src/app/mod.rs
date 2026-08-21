use crate::chain::Chain;
use crate::config::{ConfigError, NodeConfig};
use crate::events::{Event, EventBus, EventPayload};
use crate::interfaces::InterfacePlan;
use crate::ledger::blocks::local_ledger_fixture_catalog;
use crate::mempool::Mempool;
use crate::network::{NetworkError, NetworkOpenReview, NetworkPlan};
use crate::peers::{PeerPlan, PeerTargets};
use std::fmt;

pub const DEFAULT_GRACEFUL_SHUTDOWN_SECS: u64 = 30;
pub const LOCAL_RELOAD_HOOKS: [&str; 3] = ["config", "topology", "tracing"];
pub const STARTUP_PLAN_EVENT: &str = "startup.plan";
pub const STARTUP_CATEGORY_EVENT: &str = "startup.categories";
pub const RUNTIME_LIFECYCLE_PLAN_EVENT: &str = "runtime.lifecycle_plan";
pub const RUNTIME_RELOAD_PREFLIGHT_EVENT: &str = "runtime.reload_preflight";
pub const RUNTIME_SHUTDOWN_PREFLIGHT_EVENT: &str = "runtime.shutdown_preflight";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupStepKind {
    BlockProduction,
    Config,
    LocalState,
    Storage,
    Ledger,
    Network,
    Interfaces,
    Safety,
    Sync,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupStep {
    pub kind: StartupStepKind,
    pub name: String,
    pub enabled: bool,
    pub detail: String,
}

impl StartupStep {
    pub fn new(
        kind: StartupStepKind,
        name: impl Into<String>,
        enabled: bool,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            name: name.into(),
            enabled,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupPlan {
    pub network_name: String,
    pub network_magic: u32,
    pub steps: Vec<StartupStep>,
}

impl StartupPlan {
    pub fn lifecycle_plan(&self) -> RuntimeLifecyclePlan {
        RuntimeLifecyclePlan::from_startup_plan(self)
    }

    pub fn event_summary_line(&self) -> String {
        let open_steps = self.steps.iter().filter(|step| step.enabled).count();
        format!(
            "startup_plan local_only=true network={} magic={} steps={} open_steps={} closed_steps={} paths_opened=false state_mutation=false",
            self.network_name,
            self.network_magic,
            self.steps.len(),
            open_steps,
            self.steps.len().saturating_sub(open_steps),
        )
    }

    pub fn event_category_line(&self) -> String {
        let mut block_production = (0usize, 0usize);
        let mut config = (0usize, 0usize);
        let mut interfaces = (0usize, 0usize);
        let mut ledger = (0usize, 0usize);
        let mut local_state = (0usize, 0usize);
        let mut network = (0usize, 0usize);
        let mut safety = (0usize, 0usize);
        let mut storage = (0usize, 0usize);
        let mut sync = (0usize, 0usize);

        for step in &self.steps {
            let counter = match step.kind {
                StartupStepKind::BlockProduction => &mut block_production,
                StartupStepKind::Config => &mut config,
                StartupStepKind::Interfaces => &mut interfaces,
                StartupStepKind::Ledger => &mut ledger,
                StartupStepKind::LocalState => &mut local_state,
                StartupStepKind::Network => &mut network,
                StartupStepKind::Safety => &mut safety,
                StartupStepKind::Storage => &mut storage,
                StartupStepKind::Sync => &mut sync,
            };
            counter.0 += 1;
            if step.enabled {
                counter.1 += 1;
            }
        }

        format!(
            "startup_categories local_only=true format=total/open block_production={}/{} config={}/{} interfaces={}/{} ledger={}/{} local_state={}/{} network={}/{} safety={}/{} storage={}/{} sync={}/{} paths_opened=false state_mutation=false",
            block_production.0,
            block_production.1,
            config.0,
            config.1,
            interfaces.0,
            interfaces.1,
            ledger.0,
            ledger.1,
            local_state.0,
            local_state.1,
            network.0,
            network.1,
            safety.0,
            safety.1,
            storage.0,
            storage.1,
            sync.0,
            sync.1,
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![
            Event::new(
                STARTUP_PLAN_EVENT,
                EventPayload::Text(self.event_summary_line()),
            ),
            Event::new(
                STARTUP_CATEGORY_EVENT,
                EventPayload::Text(self.event_category_line()),
            ),
        ]
    }

    pub fn render_text(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "Acropolis offline startup plan\nnetwork={} magic={}\n",
            self.network_name, self.network_magic
        ));
        for step in &self.steps {
            let status = if step.enabled { "open" } else { "closed" };
            out.push_str(&format!("- {} [{}]: {}\n", step.name, status, step.detail));
        }
        let lifecycle = self.lifecycle_plan();
        out.push_str(&lifecycle.summary_line());
        out.push('\n');
        out.push_str(&lifecycle.reload_preflight(&[]).summary_line());
        out.push('\n');
        out.push_str(&lifecycle.shutdown_preflight().summary_line());
        out.push('\n');
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLifecyclePlan {
    pub local_only: bool,
    pub startup_order: Vec<String>,
    pub shutdown_order: Vec<String>,
    pub reload_hooks: Vec<&'static str>,
    pub graceful_shutdown_secs: u64,
    pub signal_handlers_installed: bool,
    pub path_activation_allowed: bool,
}

impl RuntimeLifecyclePlan {
    pub fn from_startup_plan(plan: &StartupPlan) -> Self {
        let startup_order = plan
            .steps
            .iter()
            .map(|step| step.name.clone())
            .collect::<Vec<_>>();
        let shutdown_order = startup_order.iter().rev().cloned().collect::<Vec<_>>();
        Self {
            local_only: true,
            startup_order,
            shutdown_order,
            reload_hooks: LOCAL_RELOAD_HOOKS.to_vec(),
            graceful_shutdown_secs: DEFAULT_GRACEFUL_SHUTDOWN_SECS,
            signal_handlers_installed: false,
            path_activation_allowed: false,
        }
    }

    pub fn summary_line(&self) -> String {
        format!(
            "lifecycle local_only={} startup_steps={} shutdown_steps={} reload_hooks={} graceful_shutdown_secs={} signal_handlers_installed={} path_activation_allowed={}",
            self.local_only,
            self.startup_order.len(),
            self.shutdown_order.len(),
            self.reload_hooks.join(","),
            self.graceful_shutdown_secs,
            self.signal_handlers_installed,
            self.path_activation_allowed,
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![Event::new(
            RUNTIME_LIFECYCLE_PLAN_EVENT,
            EventPayload::Text(self.summary_line()),
        )]
    }

    pub fn reload_preflight(&self, requested_hooks: &[&str]) -> RuntimeReloadPreflight {
        let requested = if requested_hooks.is_empty() {
            self.reload_hooks.clone()
        } else {
            requested_hooks.to_vec()
        };
        let requested_count = requested.len();
        let mut accepted_hooks = Vec::new();
        let mut rejected_hooks = Vec::new();
        for hook in requested {
            if self.reload_hooks.contains(&hook) {
                accepted_hooks.push(hook.to_string());
            } else {
                rejected_hooks.push(hook.to_string());
            }
        }
        RuntimeReloadPreflight {
            local_only: true,
            requested_hooks: requested_count,
            accepted_hooks,
            rejected_hooks,
            signal_handlers_installed: self.signal_handlers_installed,
            path_activation_allowed: self.path_activation_allowed,
        }
    }

    pub fn shutdown_preflight(&self) -> RuntimeShutdownPreflight {
        RuntimeShutdownPreflight {
            local_only: true,
            shutdown_steps: self.shutdown_order.len(),
            graceful_shutdown_secs: self.graceful_shutdown_secs,
            force_after_secs: self.graceful_shutdown_secs,
            signal_handlers_installed: self.signal_handlers_installed,
            path_activation_allowed: self.path_activation_allowed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeReloadPreflight {
    pub local_only: bool,
    pub requested_hooks: usize,
    pub accepted_hooks: Vec<String>,
    pub rejected_hooks: Vec<String>,
    pub signal_handlers_installed: bool,
    pub path_activation_allowed: bool,
}

impl RuntimeReloadPreflight {
    pub fn summary_line(&self) -> String {
        format!(
            "reload_preflight local_only={} requested_hooks={} accepted_hooks={} rejected_hooks={} signal_handlers_installed={} path_activation_allowed={}",
            self.local_only,
            self.requested_hooks,
            self.accepted_hooks.len(),
            self.rejected_hooks.len(),
            self.signal_handlers_installed,
            self.path_activation_allowed,
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![Event::new(
            RUNTIME_RELOAD_PREFLIGHT_EVENT,
            EventPayload::Text(self.summary_line()),
        )]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeShutdownPreflight {
    pub local_only: bool,
    pub shutdown_steps: usize,
    pub graceful_shutdown_secs: u64,
    pub force_after_secs: u64,
    pub signal_handlers_installed: bool,
    pub path_activation_allowed: bool,
}

impl RuntimeShutdownPreflight {
    pub fn summary_line(&self) -> String {
        format!(
            "shutdown_preflight local_only={} shutdown_steps={} graceful_shutdown_secs={} force_after_secs={} signal_handlers_installed={} path_activation_allowed={}",
            self.local_only,
            self.shutdown_steps,
            self.graceful_shutdown_secs,
            self.force_after_secs,
            self.signal_handlers_installed,
            self.path_activation_allowed,
        )
    }

    pub fn event_batch(&self) -> Vec<Event> {
        vec![Event::new(
            RUNTIME_SHUTDOWN_PREFLIGHT_EVENT,
            EventPayload::Text(self.summary_line()),
        )]
    }
}

pub struct Node {
    config: NodeConfig,
    events: EventBus,
    chain: Chain,
    mempool: Mempool,
    network_plan: NetworkPlan,
    peer_plan: PeerPlan,
    interface_plan: InterfacePlan,
}

impl Node {
    pub fn new(config: NodeConfig) -> Result<Self, NodeError> {
        config.validate()?;
        let network_plan = NetworkPlan::from_config(&config);
        let interface_plan = InterfacePlan::from_config(&config.interfaces, config.data_mode);
        Ok(Self {
            config,
            events: EventBus::new(),
            chain: Chain::new(),
            mempool: Mempool::new(64 * 1024 * 1024),
            network_plan,
            peer_plan: PeerPlan::new(PeerTargets::default()),
            interface_plan,
        })
    }

    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    pub fn events(&self) -> &EventBus {
        &self.events
    }

    pub fn startup_plan(&self) -> StartupPlan {
        let ledger_catalog = local_ledger_fixture_catalog();
        let (ledger_enabled, ledger_detail) = match ledger_catalog.validate() {
            Ok(()) => (
                true,
                format!(
                    "local fixture era catalog={} ranges available; production parity pending",
                    ledger_catalog.eras.len()
                ),
            ),
            Err(err) => (false, format!("local fixture era catalog invalid: {err}")),
        };
        let mut steps = vec![
            StartupStep::new(
                StartupStepKind::Safety,
                "safety",
                true,
                format!(
                    "paths={} state_mutation={}",
                    self.config.safety.allow_paths, self.config.safety.allow_state_mutation
                ),
            ),
            StartupStep::new(
                StartupStepKind::Safety,
                "production-readiness",
                false,
                "production_ready=false network_protocols_blocked=true ledger_parity_blocked=true genesis_hashing_blocked=true path_review_blocked=true agentic_runtime_blocked=true mainnet_contact_allowed=false",
            ),
            StartupStep::new(
                StartupStepKind::Config,
                "config-assets",
                false,
                format!(
                    "config={} topology={} reads=false",
                    self.config.source_config_path.display(),
                    self.config
                        .topology_path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "none".to_string())
                ),
            ),
            StartupStep::new(
                StartupStepKind::Config,
                "genesis-lifecycle",
                false,
                "cardano_plan=local genesis_reads=false raw_hashing=opt-in byron_canonical_hash=false production_genesis=false remote_fetch=false",
            ),
            StartupStep::new(
                StartupStepKind::Config,
                "topology-lifecycle",
                false,
                "cardano_plan=local topology_reads=false dns=false peer_snapshot_reads=false bootstrap_peers=false ledger_peers=false dials=false",
            ),
            StartupStep::new(
                StartupStepKind::Storage,
                "storage",
                false,
                format!(
                    "planned store_dir={} data_mode={} (not opened by offline plan)",
                    self.config.store_dir.display(),
                    self.config.data_mode
                ),
            ),
            StartupStep::new(
                StartupStepKind::Storage,
                "storage-lifecycle",
                false,
                format!(
                    "migration=false fade_enabled={} fade_secs={} archive_fallback=false",
                    self.config.pruning.enabled,
                    self.config.pruning.frequency.as_secs(),
                ),
            ),
            StartupStep::new(
                StartupStepKind::Storage,
                "bootstrap-lifecycle",
                false,
                "remote_fetch=false mithril=false snapshot_load=false state_mutation=false paths=false",
            ),
            StartupStep::new(
                StartupStepKind::Interfaces,
                "local-socket",
                false,
                format!(
                    "planned path={} activation=false",
                    self.config.local_socket_path.display()
                ),
            ),
            StartupStep::new(
                StartupStepKind::LocalState,
                "events",
                true,
                "in-memory local subscribers only",
            ),
            StartupStep::new(
                StartupStepKind::LocalState,
                "observability-lifecycle",
                false,
                "collector=in-memory metrics_server=false tracing_export=false artifacts=false paths=false",
            ),
            StartupStep::new(
                StartupStepKind::LocalState,
                "chain",
                true,
                format!("current block={}", self.chain.tip().bundle_number),
            ),
            StartupStep::new(
                StartupStepKind::LocalState,
                "mempool",
                true,
                format!("{} items waiting", self.mempool.len()),
            ),
            StartupStep::new(
                StartupStepKind::LocalState,
                "mempool-lifecycle",
                true,
                format!(
                    "capacity_bytes={} semantic_rules=local tx_submission=false mutation=false",
                    self.mempool.capacity_bytes()
                ),
            ),
            StartupStep::new(
                StartupStepKind::Ledger,
                "ledger",
                ledger_enabled,
                ledger_detail,
            ),
            StartupStep::new(
                StartupStepKind::Ledger,
                "ledger-lifecycle",
                false,
                "production_rules=false genesis_hooks=false checkpoints=false hard_fork_triggers=false",
            ),
            StartupStep::new(
                StartupStepKind::Sync,
                "sync-lifecycle",
                false,
                "mode=local-fixture chain_sync=false block_fetch=false paths=false",
            ),
            StartupStep::new(
                StartupStepKind::BlockProduction,
                "block-producer",
                false,
                format!(
                    "enabled={} tree_key={} producer_key={} certificate={} reads=false signing=false",
                    self.config.block_producer.enabled,
                    self.config.block_producer.tree_key.is_some(),
                    self.config.block_producer.producer_key.is_some(),
                    self.config.block_producer.operational_certificate.is_some(),
                ),
            ),
            StartupStep::new(
                StartupStepKind::Network,
                "peers",
                false,
                format!(
                    "targets known={} established={} active={} (no paths opened)",
                    self.peer_plan.targets.known,
                    self.peer_plan.targets.established,
                    self.peer_plan.targets.active
                ),
            ),
            StartupStep::new(
                StartupStepKind::Network,
                "peer-lifecycle",
                false,
                "topology_sources=local-fixture ledger_peers=false peer_sharing=false governor=false warm_hot_cold=false dials=false",
            ),
            StartupStep::new(
                StartupStepKind::Network,
                "mini-protocols",
                false,
                "handshake=bounded-probe-only chain_sync=false block_fetch=false tx_submission=false keepalive=false peer_sharing=false",
            ),
        ];

        for door in &self.network_plan.listeners {
            steps.push(StartupStep::new(
                StartupStepKind::Network,
                door.name.clone(),
                false,
                format!("{} planned but not opened", door.endpoint()),
            ));
        }

        steps.push(StartupStep::new(
            StartupStepKind::Interfaces,
            "interfaces",
            self.interface_plan.transactions_enabled
                || self.interface_plan.state_enabled
                || self.interface_plan.batches_enabled
                || self.interface_plan.archive_enabled,
            self.interface_plan
                .blocked_reason
                .clone()
                .unwrap_or_else(|| "all interfaces remain closed by offline plan".to_string()),
        ));

        steps.push(StartupStep::new(
            StartupStepKind::Interfaces,
            "api-lifecycle",
            false,
            "local_tx_submission=false local_state_query=false local_tx_monitor=false blockfrost=false mesh=false utxorpc=false",
        ));

        StartupPlan {
            network_name: self.config.network_name.clone(),
            network_magic: self.config.network_magic,
            steps,
        }
    }

    pub fn open_paths(&self) -> Result<(), NodeError> {
        self.network_plan.assert_safe_to_open()?;
        Ok(())
    }

    pub fn path_opening_review(&self) -> NetworkOpenReview {
        self.network_plan.open_review()
    }
}

#[derive(Debug)]
pub enum NodeError {
    Config(ConfigError),
    Network(NetworkError),
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(err) => write!(f, "config error: {err}"),
            Self::Network(err) => write!(f, "network error: {err}"),
        }
    }
}

impl std::error::Error for NodeError {}

impl From<ConfigError> for NodeError {
    fn from(value: ConfigError) -> Self {
        Self::Config(value)
    }
}

impl From<NetworkError> for NodeError {
    fn from(value: NetworkError) -> Self {
        Self::Network(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_plan_keeps_paths_closed_by_default() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        assert!(plan
            .steps
            .iter()
            .filter(|step| step.kind == StartupStepKind::Network)
            .all(|step| !step.enabled));
        assert!(matches!(
            node.open_paths(),
            Err(NodeError::Network(NetworkError::PathsClosed))
        ));
    }

    #[test]
    fn startup_plan_reports_local_ledger_fixture_catalog() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let ledger = plan
            .steps
            .iter()
            .find(|step| step.kind == StartupStepKind::Ledger)
            .unwrap();

        assert!(ledger.enabled);
        assert!(ledger.detail.contains("local fixture era catalog=3"));
        assert!(ledger.detail.contains("production parity pending"));
    }

    #[test]
    fn startup_plan_emits_compact_events_without_step_details() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let events = plan.event_batch();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].name.as_str(), STARTUP_PLAN_EVENT);
        let EventPayload::Text(text) = &events[0].payload else {
            panic!("startup plan event should use text payload");
        };
        assert_eq!(
            text,
            "startup_plan local_only=true network=local magic=2 steps=30 open_steps=6 closed_steps=24 paths_opened=false state_mutation=false"
        );
        assert!(!text.contains(".acropolis"));
        assert!(!text.contains("config="));

        assert_eq!(events[1].name.as_str(), STARTUP_CATEGORY_EVENT);
        let EventPayload::Text(text) = &events[1].payload else {
            panic!("startup category event should use text payload");
        };
        assert_eq!(
            text,
            "startup_categories local_only=true format=total/open block_production=1/0 config=3/0 interfaces=3/0 ledger=2/1 local_state=5/4 network=10/0 safety=2/1 storage=3/0 sync=1/0 paths_opened=false state_mutation=false"
        );
        assert!(!text.contains(".acropolis"));
        assert!(!text.contains("config=./"));
    }

    #[test]
    fn startup_lifecycle_plan_is_local_and_ordered() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let lifecycle = plan.lifecycle_plan();

        assert!(lifecycle.local_only);
        assert!(!lifecycle.signal_handlers_installed);
        assert!(!lifecycle.path_activation_allowed);
        assert_eq!(
            lifecycle.reload_hooks,
            vec!["config", "topology", "tracing"]
        );
        assert_eq!(lifecycle.graceful_shutdown_secs, 30);
        assert_eq!(lifecycle.startup_order.first().unwrap(), "safety");
        assert_eq!(lifecycle.shutdown_order.first().unwrap(), "api-lifecycle");
        assert_eq!(
            lifecycle.startup_order.len(),
            lifecycle.shutdown_order.len()
        );
        assert_eq!(
            lifecycle.summary_line(),
            "lifecycle local_only=true startup_steps=30 shutdown_steps=30 reload_hooks=config,topology,tracing graceful_shutdown_secs=30 signal_handlers_installed=false path_activation_allowed=false"
        );
        let events = lifecycle.event_batch();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), RUNTIME_LIFECYCLE_PLAN_EVENT);
    }

    #[test]
    fn startup_plan_render_text_includes_lifecycle_preflight_summaries() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let text = node.startup_plan().render_text();

        assert!(text.contains(
            "lifecycle local_only=true startup_steps=30 shutdown_steps=30 reload_hooks=config,topology,tracing graceful_shutdown_secs=30 signal_handlers_installed=false path_activation_allowed=false\n"
        ));
        assert!(text.contains(
            "reload_preflight local_only=true requested_hooks=3 accepted_hooks=3 rejected_hooks=0 signal_handlers_installed=false path_activation_allowed=false\n"
        ));
        assert!(text.contains(
            "shutdown_preflight local_only=true shutdown_steps=30 graceful_shutdown_secs=30 force_after_secs=30 signal_handlers_installed=false path_activation_allowed=false\n"
        ));
    }

    #[test]
    fn reload_preflight_accepts_only_known_local_hooks() {
        let lifecycle = Node::new(NodeConfig::default())
            .unwrap()
            .startup_plan()
            .lifecycle_plan();
        let preflight = lifecycle.reload_preflight(&["config", "topology", "unknown"]);

        assert!(preflight.local_only);
        assert!(!preflight.signal_handlers_installed);
        assert!(!preflight.path_activation_allowed);
        assert_eq!(preflight.accepted_hooks, vec!["config", "topology"]);
        assert_eq!(preflight.rejected_hooks, vec!["unknown"]);
        assert_eq!(
            preflight.summary_line(),
            "reload_preflight local_only=true requested_hooks=3 accepted_hooks=2 rejected_hooks=1 signal_handlers_installed=false path_activation_allowed=false"
        );
        let events = preflight.event_batch();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), RUNTIME_RELOAD_PREFLIGHT_EVENT);

        let default_preflight = lifecycle.reload_preflight(&[]);
        assert_eq!(default_preflight.requested_hooks, 3);
        assert_eq!(default_preflight.accepted_hooks.len(), 3);
        assert!(default_preflight.rejected_hooks.is_empty());
    }

    #[test]
    fn shutdown_preflight_is_local_and_bounded() {
        let shutdown = Node::new(NodeConfig::default())
            .unwrap()
            .startup_plan()
            .lifecycle_plan()
            .shutdown_preflight();

        assert!(shutdown.local_only);
        assert_eq!(shutdown.shutdown_steps, 30);
        assert_eq!(shutdown.graceful_shutdown_secs, 30);
        assert_eq!(shutdown.force_after_secs, 30);
        assert!(!shutdown.signal_handlers_installed);
        assert!(!shutdown.path_activation_allowed);
        assert_eq!(
            shutdown.summary_line(),
            "shutdown_preflight local_only=true shutdown_steps=30 graceful_shutdown_secs=30 force_after_secs=30 signal_handlers_installed=false path_activation_allowed=false"
        );
        let events = shutdown.event_batch();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name.as_str(), RUNTIME_SHUTDOWN_PREFLIGHT_EVENT);
    }

    #[test]
    fn startup_plan_reports_local_socket_activation_preflight() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let socket = plan
            .steps
            .iter()
            .find(|step| step.name == "local-socket")
            .unwrap();

        assert_eq!(socket.kind, StartupStepKind::Interfaces);
        assert!(!socket.enabled);
        assert_eq!(
            socket.detail,
            "planned path=acropolis.socket activation=false"
        );
    }

    #[test]
    fn startup_plan_reports_production_readiness_blockers() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let readiness = plan
            .steps
            .iter()
            .find(|step| step.name == "production-readiness")
            .unwrap();

        assert_eq!(readiness.kind, StartupStepKind::Safety);
        assert!(!readiness.enabled);
        assert_eq!(
            readiness.detail,
            "production_ready=false network_protocols_blocked=true ledger_parity_blocked=true genesis_hashing_blocked=true path_review_blocked=true agentic_runtime_blocked=true mainnet_contact_allowed=false"
        );
    }

    #[test]
    fn startup_plan_reports_config_asset_preflight_without_reads() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let config = plan
            .steps
            .iter()
            .find(|step| step.name == "config-assets")
            .unwrap();

        assert_eq!(config.kind, StartupStepKind::Config);
        assert!(!config.enabled);
        assert_eq!(
            config.detail,
            "config=./config/local.json topology=none reads=false"
        );
    }

    #[test]
    fn startup_plan_reports_genesis_lifecycle_preflight_without_reads() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let genesis = plan
            .steps
            .iter()
            .find(|step| step.name == "genesis-lifecycle")
            .unwrap();

        assert_eq!(genesis.kind, StartupStepKind::Config);
        assert!(!genesis.enabled);
        assert_eq!(
            genesis.detail,
            "cardano_plan=local genesis_reads=false raw_hashing=opt-in byron_canonical_hash=false production_genesis=false remote_fetch=false"
        );
    }

    #[test]
    fn startup_plan_reports_topology_lifecycle_preflight_without_reads_or_dials() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let topology = plan
            .steps
            .iter()
            .find(|step| step.name == "topology-lifecycle")
            .unwrap();

        assert_eq!(topology.kind, StartupStepKind::Config);
        assert!(!topology.enabled);
        assert_eq!(
            topology.detail,
            "cardano_plan=local topology_reads=false dns=false peer_snapshot_reads=false bootstrap_peers=false ledger_peers=false dials=false"
        );
    }

    #[test]
    fn startup_plan_reports_block_producer_preflight_without_reads_or_signing() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let producer = plan
            .steps
            .iter()
            .find(|step| step.name == "block-producer")
            .unwrap();

        assert_eq!(producer.kind, StartupStepKind::BlockProduction);
        assert!(!producer.enabled);
        assert_eq!(
            producer.detail,
            "enabled=false tree_key=false producer_key=false certificate=false reads=false signing=false"
        );
    }

    #[test]
    fn startup_plan_reports_storage_lifecycle_preflight_without_paths() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let storage = plan
            .steps
            .iter()
            .find(|step| step.name == "storage-lifecycle")
            .unwrap();

        assert_eq!(storage.kind, StartupStepKind::Storage);
        assert!(!storage.enabled);
        assert_eq!(
            storage.detail,
            "migration=false fade_enabled=false fade_secs=3600 archive_fallback=false"
        );
    }

    #[test]
    fn startup_plan_reports_bootstrap_lifecycle_preflight_without_fetch_or_paths() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let bootstrap = plan
            .steps
            .iter()
            .find(|step| step.name == "bootstrap-lifecycle")
            .unwrap();

        assert_eq!(bootstrap.kind, StartupStepKind::Storage);
        assert!(!bootstrap.enabled);
        assert_eq!(
            bootstrap.detail,
            "remote_fetch=false mithril=false snapshot_load=false state_mutation=false paths=false"
        );
    }

    #[test]
    fn startup_plan_reports_mempool_lifecycle_preflight_without_mutation() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let mempool = plan
            .steps
            .iter()
            .find(|step| step.name == "mempool-lifecycle")
            .unwrap();

        assert_eq!(mempool.kind, StartupStepKind::LocalState);
        assert!(mempool.enabled);
        assert_eq!(
            mempool.detail,
            "capacity_bytes=67108864 semantic_rules=local tx_submission=false mutation=false"
        );
    }

    #[test]
    fn startup_plan_reports_observability_lifecycle_preflight_without_paths() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let observability = plan
            .steps
            .iter()
            .find(|step| step.name == "observability-lifecycle")
            .unwrap();

        assert_eq!(observability.kind, StartupStepKind::LocalState);
        assert!(!observability.enabled);
        assert_eq!(
            observability.detail,
            "collector=in-memory metrics_server=false tracing_export=false artifacts=false paths=false"
        );
    }

    #[test]
    fn startup_plan_reports_sync_lifecycle_preflight_without_paths() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let sync = plan
            .steps
            .iter()
            .find(|step| step.name == "sync-lifecycle")
            .unwrap();

        assert_eq!(sync.kind, StartupStepKind::Sync);
        assert!(!sync.enabled);
        assert_eq!(
            sync.detail,
            "mode=local-fixture chain_sync=false block_fetch=false paths=false"
        );
    }

    #[test]
    fn startup_plan_reports_ledger_lifecycle_preflight_without_production_rules() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let ledger = plan
            .steps
            .iter()
            .find(|step| step.name == "ledger-lifecycle")
            .unwrap();

        assert_eq!(ledger.kind, StartupStepKind::Ledger);
        assert!(!ledger.enabled);
        assert_eq!(
            ledger.detail,
            "production_rules=false genesis_hooks=false checkpoints=false hard_fork_triggers=false"
        );
    }

    #[test]
    fn startup_plan_reports_mini_protocol_preflight_without_paths() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let protocols = plan
            .steps
            .iter()
            .find(|step| step.name == "mini-protocols")
            .unwrap();

        assert_eq!(protocols.kind, StartupStepKind::Network);
        assert!(!protocols.enabled);
        assert_eq!(
            protocols.detail,
            "handshake=bounded-probe-only chain_sync=false block_fetch=false tx_submission=false keepalive=false peer_sharing=false"
        );
    }

    #[test]
    fn startup_plan_reports_peer_lifecycle_preflight_without_dials() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let peers = plan
            .steps
            .iter()
            .find(|step| step.name == "peer-lifecycle")
            .unwrap();

        assert_eq!(peers.kind, StartupStepKind::Network);
        assert!(!peers.enabled);
        assert_eq!(
            peers.detail,
            "topology_sources=local-fixture ledger_peers=false peer_sharing=false governor=false warm_hot_cold=false dials=false"
        );
    }

    #[test]
    fn startup_plan_reports_api_lifecycle_preflight_without_listeners() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let plan = node.startup_plan();
        let api = plan
            .steps
            .iter()
            .find(|step| step.name == "api-lifecycle")
            .unwrap();

        assert_eq!(api.kind, StartupStepKind::Interfaces);
        assert!(!api.enabled);
        assert_eq!(
            api.detail,
            "local_tx_submission=false local_state_query=false local_tx_monitor=false blockfrost=false mesh=false utxorpc=false"
        );
    }

    #[test]
    fn node_exposes_path_opening_review_without_opening_paths() {
        let node = Node::new(NodeConfig::default()).unwrap();
        let review = node.path_opening_review();

        assert!(review.blocked());
        assert!(!review.paths_enabled);
        assert!(review
            .blockers
            .iter()
            .any(|blocker| blocker.contains("disabled by safety")));
    }
}
