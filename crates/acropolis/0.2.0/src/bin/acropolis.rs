use acropolis::agentic::{agent_flow_snapshot, agentic_safety_gate};
use acropolis::config::cardano::{
    blake2b_256_hex, raw_genesis_file_digest_supported, CardanoNodeConfigManifest,
    GenesisFileDigest, GenesisHashStatus, NetworkMagicRequirementStatus, PeerTargetConfig,
    MAX_CARDANO_GENESIS_FILE_SIZE, MAX_CARDANO_NODE_CONFIG_SIZE,
};
use acropolis::config::{network_profile, network_profiles, ConfigPatch, NetworkProfileKind};
use acropolis::network::{
    cardano_mux_frame_protocol_vector, cardano_ntn_handshake_accept_protocol_vector,
    cardano_ntn_handshake_conformance_report, cardano_ntn_handshake_error_protocol_vector_report,
    cardano_ntn_handshake_protocol_vector, cardano_ntn_handshake_refusal_transcript_replay,
    cardano_ntn_handshake_state_machine_plan, cardano_ntn_handshake_timeout_protocol_vector,
    cardano_ntn_handshake_transcript_protocol_vector, cardano_ntn_handshake_transcript_replay,
    cardano_ntn_leios_overlay_capable, cardano_ntn_version_leios_overlay_capable,
    local_handshake_sketch, plan_bounded_testnet_contact, plan_testnet_handshake_probe,
    plan_testnet_tcp_probe, run_cardano_ntn_handshake_harness,
    testnet_handshake_conformance_matrix, testnet_live_readiness, CardanoHandshakeAgency,
    CardanoHandshakeErrorProtocolVectorKind, CardanoHandshakeNegotiationOutcome,
    CardanoHandshakeRefusalReason, CardanoHandshakeState, TestnetContactLimits,
    TestnetContactRequest, TestnetHandshakeConformanceMatrix, TestnetHandshakeProbeRequest,
    TestnetTcpProbeRequest, CARDANO_MUX_HEADER_BYTES, CARDANO_NTN_HANDSHAKE_CONFIRM_TIMEOUT_SECS,
    CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION, CARDANO_NTN_REFUSAL_SUPPORTED_VERSIONS,
    CARDANO_NTN_SUPPORTED_VERSIONS, MAX_TESTNET_HANDSHAKE_PROBE_RESPONSE_BYTES,
    MAX_TESTNET_TCP_PROBE_TIMEOUT_SECS, NETWORK_HANDSHAKE_VERSION,
};
use acropolis::peers::{PeerDiscoveryPlan, PeerDiscoverySource, PeerSet, PeerTargets};
use acropolis::topology::{
    parse_cardano_peer_snapshot_json, parse_cardano_topology_json, PeerSnapshotError,
    PeerSnapshotRules, TopologyConfig, MAX_PEER_SNAPSHOT_SIZE, MAX_TOPOLOGY_SIZE,
};
use acropolis::{Node, NodeConfig};
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let request = CliRequest::parse(std::env::args().skip(1))?;
    if request.command == Command::Help {
        print_help();
        return Ok(());
    }
    if request.command == Command::Networks {
        print_networks();
        return Ok(());
    }
    if request.command == Command::CardanoPlan {
        print_cardano_plan(&request.cardano_plan)?;
        return Ok(());
    }
    if request.command == Command::TestnetConformance {
        print_testnet_conformance()?;
        return Ok(());
    }
    if request.command == Command::TestnetReadiness {
        print_testnet_readiness();
        return Ok(());
    }
    if request.command == Command::Status {
        print_status()?;
        return Ok(());
    }

    let file_patch = match &request.config_path {
        Some(path) => Some(read_config_patch(path)?),
        None => None,
    };
    let config = NodeConfig::from_sources(file_patch, Some(request.patch))?;
    let node = Node::new(config)?;

    match request.command {
        Command::Plan => {
            print!("{}", node.startup_plan().render_text());
            Ok(())
        }
        Command::Guard => {
            println!(
                "paths_allowed={} state_mutation_allowed={}",
                node.config().safety.allow_paths,
                node.config().safety.allow_state_mutation
            );
            Ok(())
        }
        Command::Open => {
            node.open_paths()?;
            Ok(())
        }
        Command::TestnetPlan => {
            print_testnet_plan(node.config(), &request.testnet_plan)?;
            Ok(())
        }
        Command::AgentFlow => {
            print!(
                "{}",
                agent_flow_snapshot(&node.startup_plan()).render_text()
            );
            Ok(())
        }
        Command::AgentSafety => {
            let snapshot = agent_flow_snapshot(&node.startup_plan());
            print!("{}", agentic_safety_gate(&snapshot).render_text());
            Ok(())
        }
        Command::TestnetProbe => run_testnet_probe(node.config(), &request.testnet_probe),
        Command::TestnetHandshakeProbe => {
            run_testnet_handshake_probe(node.config(), &request.testnet_probe)
        }
        Command::Help
        | Command::Networks
        | Command::CardanoPlan
        | Command::TestnetConformance
        | Command::TestnetReadiness
        | Command::Status => Ok(()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Plan,
    Guard,
    Open,
    Status,
    Networks,
    TestnetPlan,
    TestnetProbe,
    TestnetHandshakeProbe,
    TestnetConformance,
    TestnetReadiness,
    CardanoPlan,
    AgentFlow,
    AgentSafety,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliRequest {
    command: Command,
    config_path: Option<PathBuf>,
    patch: ConfigPatch,
    testnet_plan: TestnetPlanArgs,
    testnet_probe: TestnetProbeArgs,
    cardano_plan: CardanoPlanArgs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestnetPlanArgs {
    requested_blocks: u64,
    requested_slots: u64,
    requested_bytes: u64,
    limits: TestnetContactLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TestnetProbeArgs {
    peer: Option<String>,
    allow_live_testnet: bool,
    timeout_secs: u64,
}

impl Default for TestnetProbeArgs {
    fn default() -> Self {
        Self {
            peer: None,
            allow_live_testnet: false,
            timeout_secs: 2,
        }
    }
}

impl Default for TestnetPlanArgs {
    fn default() -> Self {
        Self {
            requested_blocks: 1,
            requested_slots: 100,
            requested_bytes: 1024,
            limits: TestnetContactLimits::smoke_test(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct CardanoPlanArgs {
    config_path: Option<PathBuf>,
    topology_path: Option<PathBuf>,
    peer_snapshot_path: Option<PathBuf>,
    genesis_digests: Vec<GenesisFileDigest>,
    read_genesis_files: bool,
    network_name: Option<String>,
    socket_path: Option<PathBuf>,
}

impl CliRequest {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut command = None;
        let mut config_path = None;
        let mut patch = ConfigPatch::new();
        let mut testnet_plan = TestnetPlanArgs::default();
        let mut testnet_probe = TestnetProbeArgs::default();
        let mut cardano_plan = CardanoPlanArgs::default();
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            if let Some((flag, value)) = arg.split_once('=') {
                apply_flag_value(
                    flag,
                    value,
                    &mut config_path,
                    &mut patch,
                    &mut testnet_plan,
                    &mut testnet_probe,
                    &mut cardano_plan,
                )?;
                continue;
            }

            match arg.as_str() {
                "plan" => set_command(&mut command, Command::Plan)?,
                "guard" | "safety" => set_command(&mut command, Command::Guard)?,
                "open" | "serve" | "run" => set_command(&mut command, Command::Open)?,
                "status" | "progress" | "local-status" => {
                    set_command(&mut command, Command::Status)?
                }
                "networks" | "network-profiles" => set_command(&mut command, Command::Networks)?,
                "testnet-plan" | "testnet-check-plan" => {
                    set_command(&mut command, Command::TestnetPlan)?
                }
                "testnet-probe" | "testnet-tcp-probe" => {
                    set_command(&mut command, Command::TestnetProbe)?
                }
                "testnet-handshake-probe" | "testnet-live-handshake" => {
                    set_command(&mut command, Command::TestnetHandshakeProbe)?
                }
                "testnet-conformance" | "testnet-conformance-plan" => {
                    set_command(&mut command, Command::TestnetConformance)?
                }
                "testnet-readiness" | "testnet-readiness-plan" => {
                    set_command(&mut command, Command::TestnetReadiness)?
                }
                "cardano-plan" | "cardano-config-plan" => {
                    set_command(&mut command, Command::CardanoPlan)?
                }
                "agent-flow" | "agentic-flow" => set_command(&mut command, Command::AgentFlow)?,
                "agent-safety" | "agentic-safety" => {
                    set_command(&mut command, Command::AgentSafety)?
                }
                "help" | "--help" | "-h" => set_command(&mut command, Command::Help)?,
                "--config" | "-c" => {
                    let value = args
                        .next()
                        .ok_or_else(|| format!("{arg} requires a path value"))?;
                    config_path = Some(PathBuf::from(value));
                }
                "--set" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--set requires key=value".to_string())?;
                    patch.extend(ConfigPatch::parse(&value).map_err(|err| err.to_string())?);
                }
                "--network" => {
                    let value = args
                        .next()
                        .ok_or_else(|| format!("{arg} requires a value"))?;
                    patch.push("network", value.clone());
                    cardano_plan.network_name = Some(value);
                }
                "--network-magic" => push_next(&mut args, &mut patch, "network_magic", &arg)?,
                "--data-mode" => push_next(&mut args, &mut patch, "data_mode", &arg)?,
                "--mode" => push_next(&mut args, &mut patch, "operating_mode", &arg)?,
                "--store-dir" => push_next(&mut args, &mut patch, "store_dir", &arg)?,
                "--topology" => push_next(&mut args, &mut patch, "topology_path", &arg)?,
                "--socket" => {
                    let value = args
                        .next()
                        .ok_or_else(|| format!("{arg} requires a path value"))?;
                    patch.push("local_socket_path", value.clone());
                    cardano_plan.socket_path = Some(PathBuf::from(value));
                }
                "--outer-bind" => push_next(&mut args, &mut patch, "outer_bind_addr", &arg)?,
                "--inner-bind" => push_next(&mut args, &mut patch, "inner_bind_addr", &arg)?,
                "--outer-port" => push_next(&mut args, &mut patch, "outer_port", &arg)?,
                "--inner-port" => push_next(&mut args, &mut patch, "inner_port", &arg)?,
                "--metrics-port" => push_next(&mut args, &mut patch, "metrics_port", &arg)?,
                "--transaction-port" => push_next(&mut args, &mut patch, "transaction_port", &arg)?,
                "--state-port" => push_next(&mut args, &mut patch, "state_port", &arg)?,
                "--batch-port" => push_next(&mut args, &mut patch, "batch_port", &arg)?,
                "--archive-port" => push_next(&mut args, &mut patch, "archive_port", &arg)?,
                "--archive-base-url" => push_next(&mut args, &mut patch, "archive_base_url", &arg)?,
                "--testnet-blocks" => {
                    testnet_plan.requested_blocks = parse_u64_flag(&arg, args.next())?
                }
                "--testnet-slots" => {
                    testnet_plan.requested_slots = parse_u64_flag(&arg, args.next())?
                }
                "--testnet-bytes" => {
                    testnet_plan.requested_bytes = parse_u64_flag(&arg, args.next())?
                }
                "--max-blocks" => {
                    testnet_plan.limits.max_blocks = parse_u64_flag(&arg, args.next())?
                }
                "--max-slots" => testnet_plan.limits.max_slots = parse_u64_flag(&arg, args.next())?,
                "--max-bytes" => testnet_plan.limits.max_bytes = parse_u64_flag(&arg, args.next())?,
                "--timeout-secs" => {
                    let value = parse_u64_flag(&arg, args.next())?;
                    testnet_plan.limits.timeout_secs = value;
                    testnet_probe.timeout_secs = value;
                }
                "--probe-timeout-secs" => {
                    testnet_probe.timeout_secs = parse_u64_flag(&arg, args.next())?
                }
                "--temp-bytes" => {
                    testnet_plan.limits.temp_storage_bytes = parse_u64_flag(&arg, args.next())?
                }
                "--testnet-peer" => {
                    testnet_probe.peer = Some(
                        args.next()
                            .ok_or_else(|| format!("{arg} requires ip:port value"))?,
                    )
                }
                "--allow-live-testnet" => testnet_probe.allow_live_testnet = true,
                "--cardano-config" | "--node-config" => {
                    cardano_plan.config_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| format!("{arg} requires a path value"))?,
                    ));
                }
                "--cardano-topology" => {
                    cardano_plan.topology_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| format!("{arg} requires a path value"))?,
                    ));
                }
                "--cardano-peer-snapshot" => {
                    cardano_plan.peer_snapshot_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| format!("{arg} requires a path value"))?,
                    ));
                }
                "--cardano-network" => {
                    cardano_plan.network_name = Some(
                        args.next()
                            .ok_or_else(|| format!("{arg} requires a network name"))?,
                    );
                }
                "--genesis-digest" => {
                    let digest = parse_genesis_digest(
                        &arg,
                        &args
                            .next()
                            .ok_or_else(|| "--genesis-digest requires era=hash".to_string())?,
                    )?;
                    push_genesis_digest(&mut cardano_plan.genesis_digests, digest, &arg)?;
                }
                "--read-genesis-files" | "--read-genesis" => {
                    cardano_plan.read_genesis_files = true;
                }
                "--allow-paths" => patch.push("allow_paths", "true"),
                "--allow-state-mutation" => patch.push("allow_state_mutation", "true"),
                "--block-producer" => patch.push("block_producer_enabled", "true"),
                other if other.starts_with('-') => return Err(format!("unknown flag {other}")),
                other => {
                    return Err(format!(
                        "unknown command {other}; use plan, guard, status, networks, testnet-plan, testnet-probe, testnet-handshake-probe, testnet-conformance, testnet-readiness, cardano-plan, agent-flow, agent-safety, or help"
                    ))
                }
            }
        }

        Ok(Self {
            command: command.unwrap_or(Command::Plan),
            config_path,
            patch,
            testnet_plan,
            testnet_probe,
            cardano_plan,
        })
    }
}

fn set_command(command: &mut Option<Command>, value: Command) -> Result<(), String> {
    if command.replace(value).is_some() {
        return Err("only one command may be provided".to_string());
    }
    Ok(())
}

fn push_next(
    args: &mut impl Iterator<Item = String>,
    patch: &mut ConfigPatch,
    key: &'static str,
    flag: &str,
) -> Result<(), String> {
    let value = args
        .next()
        .ok_or_else(|| format!("{flag} requires a value"))?;
    patch.push(key, value);
    Ok(())
}

fn apply_flag_value(
    flag: &str,
    value: &str,
    config_path: &mut Option<PathBuf>,
    patch: &mut ConfigPatch,
    testnet_plan: &mut TestnetPlanArgs,
    testnet_probe: &mut TestnetProbeArgs,
    cardano_plan: &mut CardanoPlanArgs,
) -> Result<(), String> {
    match flag {
        "--config" | "-c" => *config_path = Some(PathBuf::from(value)),
        "--set" => patch.extend(ConfigPatch::parse(value).map_err(|err| err.to_string())?),
        "--network" => {
            patch.push("network", value);
            cardano_plan.network_name = Some(value.to_string());
        }
        "--cardano-network" => cardano_plan.network_name = Some(value.to_string()),
        "--network-magic" => patch.push("network_magic", value),
        "--data-mode" => patch.push("data_mode", value),
        "--mode" => patch.push("operating_mode", value),
        "--store-dir" => patch.push("store_dir", value),
        "--topology" => patch.push("topology_path", value),
        "--socket" => {
            patch.push("local_socket_path", value);
            cardano_plan.socket_path = Some(PathBuf::from(value));
        }
        "--outer-bind" => patch.push("outer_bind_addr", value),
        "--inner-bind" => patch.push("inner_bind_addr", value),
        "--outer-port" => patch.push("outer_port", value),
        "--inner-port" => patch.push("inner_port", value),
        "--metrics-port" => patch.push("metrics_port", value),
        "--transaction-port" => patch.push("transaction_port", value),
        "--state-port" => patch.push("state_port", value),
        "--batch-port" => patch.push("batch_port", value),
        "--archive-port" => patch.push("archive_port", value),
        "--archive-base-url" => patch.push("archive_base_url", value),
        "--testnet-blocks" => testnet_plan.requested_blocks = parse_u64_value(flag, value)?,
        "--testnet-slots" => testnet_plan.requested_slots = parse_u64_value(flag, value)?,
        "--testnet-bytes" => testnet_plan.requested_bytes = parse_u64_value(flag, value)?,
        "--max-blocks" => testnet_plan.limits.max_blocks = parse_u64_value(flag, value)?,
        "--max-slots" => testnet_plan.limits.max_slots = parse_u64_value(flag, value)?,
        "--max-bytes" => testnet_plan.limits.max_bytes = parse_u64_value(flag, value)?,
        "--timeout-secs" => {
            let parsed = parse_u64_value(flag, value)?;
            testnet_plan.limits.timeout_secs = parsed;
            testnet_probe.timeout_secs = parsed;
        }
        "--probe-timeout-secs" => testnet_probe.timeout_secs = parse_u64_value(flag, value)?,
        "--temp-bytes" => testnet_plan.limits.temp_storage_bytes = parse_u64_value(flag, value)?,
        "--testnet-peer" => testnet_probe.peer = Some(value.to_string()),
        "--allow-live-testnet" => testnet_probe.allow_live_testnet = parse_cli_bool(flag, value)?,
        "--cardano-config" | "--node-config" => {
            cardano_plan.config_path = Some(PathBuf::from(value))
        }
        "--cardano-topology" => cardano_plan.topology_path = Some(PathBuf::from(value)),
        "--cardano-peer-snapshot" => cardano_plan.peer_snapshot_path = Some(PathBuf::from(value)),
        "--genesis-digest" => {
            let digest = parse_genesis_digest(flag, value)?;
            push_genesis_digest(&mut cardano_plan.genesis_digests, digest, flag)?;
        }
        "--read-genesis-files" | "--read-genesis" => {
            cardano_plan.read_genesis_files = parse_cli_bool(flag, value)?
        }
        "--allow-paths" => patch.push("allow_paths", value),
        "--allow-state-mutation" => patch.push("allow_state_mutation", value),
        "--block-producer" => patch.push("block_producer_enabled", value),
        other => return Err(format!("unknown flag {other}")),
    }
    Ok(())
}

fn parse_cli_bool(flag: &str, value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "no" | "n" | "off" => Ok(false),
        _ => Err(format!("{flag} requires a boolean value")),
    }
}

fn parse_genesis_digest(flag: &str, value: &str) -> Result<GenesisFileDigest, String> {
    let (era, hash) = value
        .split_once('=')
        .ok_or_else(|| format!("{flag} requires era=hash"))?;
    if era.trim().is_empty() {
        return Err(format!("{flag} requires a non-empty era"));
    }
    let hash = hash.trim();
    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{flag} requires a 64-character hexadecimal hash"));
    }
    Ok(GenesisFileDigest::new(
        era.trim(),
        hash.to_ascii_lowercase(),
    ))
}

fn push_genesis_digest(
    digests: &mut Vec<GenesisFileDigest>,
    digest: GenesisFileDigest,
    flag: &str,
) -> Result<(), String> {
    if digests
        .iter()
        .any(|existing| existing.era.eq_ignore_ascii_case(&digest.era))
    {
        return Err(format!("{flag} duplicate era {}", digest.era));
    }
    digests.push(digest);
    Ok(())
}

fn parse_u64_flag(flag: &str, value: Option<String>) -> Result<u64, String> {
    let value = value.ok_or_else(|| format!("{flag} requires a value"))?;
    parse_u64_value(flag, &value)
}

fn parse_u64_value(flag: &str, value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| format!("{flag} requires a non-negative integer"))
}

fn read_config_patch(path: &PathBuf) -> Result<ConfigPatch, Box<dyn std::error::Error>> {
    let text = std::fs::read_to_string(path)?;
    Ok(ConfigPatch::parse(&text)?)
}

fn print_networks() {
    println!("known offline network profiles");
    println!("name magic kind port era slot_ms epoch_slots k active_slots bootstrap");
    for profile in network_profiles() {
        println!(
            "{} {} {:?} {} {} {} {} {} {} {}",
            profile.name,
            profile.network_magic,
            profile.kind,
            profile.default_node_port,
            profile.era,
            profile.slot_length_ms,
            profile.epoch_length_slots,
            profile.security_parameter,
            profile.active_slots_ratio_text(),
            profile.bootstrap_mode,
        );
    }
}

fn print_status() -> Result<(), Box<dyn std::error::Error>> {
    let config = NodeConfig::default();
    let node = Node::new(config)?;
    let profiles = network_profiles();
    let public_profiles = profiles
        .iter()
        .filter(|profile| profile.kind == NetworkProfileKind::Public)
        .count();
    let local_profiles = profiles.len().saturating_sub(public_profiles);
    let readiness = testnet_live_readiness();
    let matrix = testnet_handshake_conformance_matrix(&CARDANO_NTN_SUPPORTED_VERSIONS)?;
    let readiness_profile_names = public_testnet_profile_names();
    let readiness_profile_ports = public_testnet_profile_ports();
    let matrix_profile_names = testnet_conformance_matrix_profiles(&matrix);
    let matrix_profile_ports = testnet_conformance_matrix_profile_ports(&matrix);
    let startup_plan = node.startup_plan();
    let lifecycle_plan = startup_plan.lifecycle_plan();
    let reload_preflight = lifecycle_plan.reload_preflight(&[]);
    let shutdown_preflight = lifecycle_plan.shutdown_preflight();
    let agent_snapshot = agent_flow_snapshot(&startup_plan);
    let agent_gate = agentic_safety_gate(&agent_snapshot);

    println!("Acropolis local status");
    println!(
        "local_only=true paths_opened=false dials=false remote_fetch=false state_mutation=false live_protocol_send=false"
    );
    println!(
        "startup network={} magic={} paths_allowed={} state_mutation_allowed={} data_mode={}",
        node.config().network_name,
        node.config().network_magic,
        node.config().safety.allow_paths,
        node.config().safety.allow_state_mutation,
        node.config().data_mode,
    );
    println!("{}", startup_plan.event_summary_line());
    println!("{}", startup_plan.event_category_line());
    println!("{}", lifecycle_plan.summary_line());
    println!("{}", reload_preflight.summary_line());
    println!("{}", shutdown_preflight.summary_line());
    println!(
        "profiles total={} public={} local={}",
        profiles.len(),
        public_profiles,
        local_profiles
    );
    println!(
        "testnet_readiness tcp_probe_available={} handshake_probe_available={} offline_conformance_complete={} public_testnets={} profiles={} profile_ports={} bounded_live_contact_allowed={} blockers={} actions={}",
        readiness.tcp_probe_available,
        readiness.live_command_available,
        readiness.offline_conformance_complete,
        readiness.public_testnet_profiles,
        readiness_profile_names,
        readiness_profile_ports,
        readiness.live_contact_allowed,
        readiness.blockers.len(),
        readiness.action_items().len()
    );
    println!(
        "testnet_conformance public_testnets={} passed_profiles={} offline_complete={} live_ready={} profiles={} profile_ports={} blockers={} actions={}",
        matrix.public_testnet_profiles,
        matrix.passed_profiles,
        matrix.offline_complete,
        matrix.live_ready,
        matrix_profile_names,
        matrix_profile_ports,
        matrix.blockers.len(),
        matrix.action_items().len()
    );
    println!("{}", testnet_live_capable_path_line());
    println!("{}", tcp_probe_gate_line());
    println!("{}", testnet_handshake_live_capable_path_line());
    println!("{}", handshake_probe_gate_line());
    println!(
        "agentic_safety safety_gate={} blockers={} actions={} live_agents_running={} provider_calls={} sidecar_spawn={}",
        if agent_gate.is_clear() { "clear" } else { "blocked" },
        agent_gate.blockers.len(),
        agent_gate.action_items().len(),
        agent_gate.live_agents_running,
        agent_gate.provider_calls,
        agent_gate.sidecar_spawn,
    );
    println!("production_ready=false mainnet_contact_allowed=false live_protocol_send=false");
    println!(
        "production_blockers network_protocols=true ledger_parity=true genesis_hashing=true path_review=true agentic_runtime=true"
    );
    Ok(())
}

fn print_testnet_plan(
    config: &NodeConfig,
    args: &TestnetPlanArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let plan = plan_bounded_testnet_contact(
        TestnetContactRequest::new(
            config.network_name.clone(),
            args.requested_blocks,
            args.requested_slots,
            args.requested_bytes,
        ),
        args.limits,
    )?;
    println!("bounded testnet dry-run plan");
    println!(
        "network={} magic={} live_contact=false paths_opened=false dials=false remote_fetch=false state_mutation=false",
        plan.profile.name, plan.profile.network_magic
    );
    println!(
        "request blocks={} slots={} bytes={}",
        plan.request.requested_blocks, plan.request.requested_slots, plan.request.requested_bytes
    );
    println!(
        "limits max_blocks={} max_slots={} max_bytes={} timeout_secs={} temp_bytes={}",
        plan.limits.max_blocks,
        plan.limits.max_slots,
        plan.limits.max_bytes,
        plan.limits.timeout_secs,
        plan.limits.temp_storage_bytes
    );
    println!(
        "remaining blocks={} slots={} bytes={} temp_bytes={}",
        plan.limits
            .max_blocks
            .saturating_sub(plan.request.requested_blocks),
        plan.limits
            .max_slots
            .saturating_sub(plan.request.requested_slots),
        plan.limits
            .max_bytes
            .saturating_sub(plan.request.requested_bytes),
        plan.limits
            .temp_storage_bytes
            .saturating_sub(plan.request.requested_bytes)
    );
    let readiness = testnet_live_readiness();
    let public_testnet_profile_names = public_testnet_profile_names();
    let public_testnet_profile_ports = public_testnet_profile_ports();
    println!(
        "live_readiness tcp_probe_available={} handshake_sketch_available={} offline_conformance_complete={} public_testnet_profiles={} public_testnet_profile_names={} public_testnet_profile_ports={} command_available={} path_review_complete={} conformance_complete={} live_contact_allowed={} blockers={}",
        readiness.tcp_probe_available,
        readiness.handshake_sketch_available,
        readiness.offline_conformance_complete,
        readiness.public_testnet_profiles,
        public_testnet_profile_names,
        public_testnet_profile_ports,
        readiness.live_command_available,
        readiness.path_review_complete,
        readiness.conformance_complete,
        readiness.live_contact_allowed,
        readiness.blockers.join(";")
    );
    println!(
        "live_readiness_actions actions={} live_contact_allowed={}",
        readiness.action_items().join(";"),
        readiness.live_contact_allowed
    );
    println!("{}", testnet_live_capable_path_line());
    println!("{}", tcp_probe_gate_line());
    println!("{}", testnet_handshake_live_capable_path_line());
    println!("{}", handshake_probe_gate_line());
    let sketch = local_handshake_sketch(plan.profile, &[NETWORK_HANDSHAKE_VERSION])?;
    println!(
        "handshake_sketch versions={} bytes={} production_compatible={} live_protocol_send=false",
        sketch.versions.len(),
        sketch.encoded.len(),
        sketch.production_compatible
    );
    let protocol_vector =
        cardano_ntn_handshake_protocol_vector(plan.profile, &CARDANO_NTN_SUPPORTED_VERSIONS)?;
    println!(
        "handshake_proposal_protocol_vector protocol_id={} message_type=propose_versions versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} bytes={} production_ready={} mux_framed=false live_protocol_send=false",
        protocol_vector.protocol_id,
        protocol_vector.versions.len(),
        protocol_vector.versions.iter().copied().min().unwrap_or(0),
        protocol_vector.versions.iter().copied().max().unwrap_or(0),
        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
        cardano_ntn_leios_overlay_capable(&protocol_vector.versions),
        protocol_vector.encoded.len(),
        protocol_vector.production_ready
    );
    let mux_protocol_vector = cardano_mux_frame_protocol_vector(
        protocol_vector.protocol_id,
        &protocol_vector.encoded,
        false,
        0,
    )?;
    println!(
        "handshake_mux_protocol_vector header_bytes={} frame_bytes={} payload_bytes={} timestamp={} protocol_id={} response={} production_ready={} live_protocol_send=false",
        CARDANO_MUX_HEADER_BYTES,
        mux_protocol_vector.encoded.len(),
        mux_protocol_vector.payload_length,
        mux_protocol_vector.timestamp,
        mux_protocol_vector.protocol_id,
        mux_protocol_vector.is_response,
        mux_protocol_vector.production_ready
    );
    let state_machine = cardano_ntn_handshake_state_machine_plan();
    let transition_count: usize = state_machine
        .entries
        .iter()
        .map(|entry| entry.transitions.len())
        .sum();
    let timeout_count = state_machine
        .entries
        .iter()
        .filter(|entry| entry.timeout_secs.is_some())
        .count();
    println!(
        "handshake_state_machine states={} transitions={} timeout_states={} production_ready={} live_integrated=false live_protocol_send=false",
        state_machine.entries.len(),
        transition_count,
        timeout_count,
        state_machine.production_ready,
    );
    let transcript = cardano_ntn_handshake_transcript_protocol_vector(
        plan.profile,
        &CARDANO_NTN_SUPPORTED_VERSIONS,
    )?;
    println!(
        "handshake_transcript local_only=true frames={} total_bytes={} request_bytes={} response_bytes={} accepted_version={} leios_overlay_min_version={} leios_overlay_negotiated={} production_ready={} live_integrated={} live_protocol_send=false",
        transcript.frame_count,
        transcript.total_bytes,
        transcript.request_frame.encoded.len(),
        transcript.response_frame.encoded.len(),
        transcript.accepted_version,
        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
        cardano_ntn_version_leios_overlay_capable(transcript.accepted_version),
        transcript.production_ready,
        transcript.live_integrated,
    );
    let replay =
        cardano_ntn_handshake_transcript_replay(plan.profile, &CARDANO_NTN_SUPPORTED_VERSIONS)?;
    let replay_final_state = match replay.final_state {
        CardanoHandshakeState::Propose => "propose",
        CardanoHandshakeState::Confirm => "confirm",
        CardanoHandshakeState::Done => "done",
    };
    println!(
        "handshake_replay local_only=true frames={} request_frames={} response_frames={} stream_bytes={} final_state={} accepted_version={} leios_overlay_min_version={} leios_overlay_negotiated={} production_ready={} live_integrated={} live_protocol_send=false",
        replay.frames.len(),
        replay.request_frames,
        replay.response_frames,
        replay.total_bytes,
        replay_final_state,
        replay.accepted_version,
        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
        cardano_ntn_version_leios_overlay_capable(replay.accepted_version),
        replay.production_ready,
        replay.live_integrated,
    );
    let refusal_replay = cardano_ntn_handshake_refusal_transcript_replay(
        plan.profile,
        &CARDANO_NTN_SUPPORTED_VERSIONS,
        &CARDANO_NTN_REFUSAL_SUPPORTED_VERSIONS,
    )?;
    let refusal_final_state = match refusal_replay.final_state {
        CardanoHandshakeState::Propose => "propose",
        CardanoHandshakeState::Confirm => "confirm",
        CardanoHandshakeState::Done => "done",
    };
    let refusal_reason = match refusal_replay.refusal_reason {
        CardanoHandshakeRefusalReason::VersionMismatch => "version_mismatch",
        CardanoHandshakeRefusalReason::DecodeError => "decode_error",
        CardanoHandshakeRefusalReason::Refused => "refused",
    };
    let refusal_supported_versions = refusal_replay
        .supported_versions
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "handshake_refusal_replay local_only=true frames={} request_frames={} response_frames={} stream_bytes={} final_state={} refusal_reason={} supported_versions={} supported_min_version={} supported_max_version={} leios_overlay_min_version={} supported_leios_overlay_capable={} production_ready={} live_integrated={} live_protocol_send=false",
        refusal_replay.frames.len(),
        refusal_replay.request_frames,
        refusal_replay.response_frames,
        refusal_replay.total_bytes,
        refusal_final_state,
        refusal_reason,
        refusal_supported_versions,
        refusal_replay
            .supported_versions
            .iter()
            .copied()
            .min()
            .unwrap_or(0),
        refusal_replay
            .supported_versions
            .iter()
            .copied()
            .max()
            .unwrap_or(0),
        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
        cardano_ntn_leios_overlay_capable(&refusal_replay.supported_versions),
        refusal_replay.production_ready,
        refusal_replay.live_integrated,
    );
    let accept_protocol_vector = cardano_ntn_handshake_accept_protocol_vector(plan.profile, 10)?;
    let accept_frame = cardano_mux_frame_protocol_vector(
        accept_protocol_vector.protocol_id,
        &accept_protocol_vector.encoded,
        true,
        0,
    )?;
    let harness = run_cardano_ntn_handshake_harness(
        plan.profile,
        &CARDANO_NTN_SUPPORTED_VERSIONS,
        &accept_frame.encoded,
    )?;
    let (outcome, accepted_version, leios_overlay_negotiated) = match &harness.negotiation.outcome {
        CardanoHandshakeNegotiationOutcome::Accepted { version, .. } => (
            "accepted",
            version.to_string(),
            cardano_ntn_version_leios_overlay_capable(*version).to_string(),
        ),
        CardanoHandshakeNegotiationOutcome::Refused { .. } => {
            ("refused", "none".to_string(), "none".to_string())
        }
    };
    println!(
        "handshake_harness local_only=true scenario=accept_protocol_vector states={} messages={} proposal_frame_bytes={} response_frame_bytes={} outcome={} accepted_version={} leios_overlay_min_version={} leios_overlay_negotiated={} production_ready={} live_integrated={} live_protocol_send=false",
        harness.states.len(),
        harness.messages.len(),
        harness.proposal_frame.encoded.len(),
        CARDANO_MUX_HEADER_BYTES + usize::from(harness.response_frame.payload_length),
        outcome,
        accepted_version,
        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
        leios_overlay_negotiated,
        harness.production_ready,
        harness.live_integrated,
    );
    let timeout_protocol_vector = cardano_ntn_handshake_timeout_protocol_vector(
        CardanoHandshakeState::Confirm,
        CARDANO_NTN_HANDSHAKE_CONFIRM_TIMEOUT_SECS,
    )?;
    let timeout_state = match timeout_protocol_vector.state {
        CardanoHandshakeState::Propose => "propose",
        CardanoHandshakeState::Confirm => "confirm",
        CardanoHandshakeState::Done => "done",
    };
    let timeout_agency = match timeout_protocol_vector.agency {
        CardanoHandshakeAgency::Client => "client",
        CardanoHandshakeAgency::Server => "server",
        CardanoHandshakeAgency::None => "none",
    };
    println!(
        "handshake_timeout_protocol_vector state={} agency={} timeout_secs={} elapsed_secs={} timed_out={} production_ready={} live_integrated={} live_protocol_send=false",
        timeout_state,
        timeout_agency,
        timeout_protocol_vector.timeout_secs,
        timeout_protocol_vector.elapsed_secs,
        timeout_protocol_vector.timed_out,
        timeout_protocol_vector.production_ready,
        timeout_protocol_vector.live_integrated,
    );
    let error_report = cardano_ntn_handshake_error_protocol_vector_report(
        plan.profile,
        &CARDANO_NTN_SUPPORTED_VERSIONS,
    )?;
    let matched_errors = error_report
        .cases
        .iter()
        .filter(|case| case.matched)
        .count();
    let error_kinds = error_report
        .cases
        .iter()
        .map(|case| match case.kind {
            CardanoHandshakeErrorProtocolVectorKind::WrongProtocolId => "wrong_protocol_id",
            CardanoHandshakeErrorProtocolVectorKind::NonResponseFrame => "non_response_frame",
            CardanoHandshakeErrorProtocolVectorKind::MalformedCbor => "malformed_cbor",
        })
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "handshake_error_protocol_vectors local_only=true versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} cases={} matched={} kinds={} production_ready={} live_integrated={} live_protocol_send=false",
        error_report.proposed_versions.len(),
        error_report
            .proposed_versions
            .iter()
            .copied()
            .min()
            .unwrap_or(0),
        error_report
            .proposed_versions
            .iter()
            .copied()
            .max()
            .unwrap_or(0),
        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
        cardano_ntn_leios_overlay_capable(&error_report.proposed_versions),
        error_report.cases.len(),
        matched_errors,
        error_kinds,
        error_report.production_ready,
        error_report.live_integrated,
    );
    let conformance =
        cardano_ntn_handshake_conformance_report(plan.profile, &CARDANO_NTN_SUPPORTED_VERSIONS)?;
    println!(
        "handshake_conformance local_only=true versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} offline_checks={} passed={} offline_complete={} live_ready={} blockers={} production_ready={} live_integrated={} live_protocol_send=false",
        conformance.proposed_versions.len(),
        conformance.proposed_versions.iter().copied().min().unwrap_or(0),
        conformance.proposed_versions.iter().copied().max().unwrap_or(0),
        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
        cardano_ntn_leios_overlay_capable(&conformance.proposed_versions),
        conformance.offline_checks,
        conformance.passed_checks,
        conformance.offline_complete,
        conformance.live_ready,
        conformance.blockers.join(";"),
        conformance.production_ready,
        conformance.live_integrated,
    );
    let matrix = testnet_handshake_conformance_matrix(&CARDANO_NTN_SUPPORTED_VERSIONS)?;
    let matrix_profiles = testnet_conformance_matrix_profiles(&matrix);
    let matrix_profile_ports = testnet_conformance_matrix_profile_ports(&matrix);
    println!(
        "testnet_conformance_matrix local_only=true versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} public_testnets={} passed_profiles={} offline_complete={} live_ready={} profiles={} profile_ports={} blockers={} production_ready={} live_integrated={} live_protocol_send=false",
        CARDANO_NTN_SUPPORTED_VERSIONS.len(),
        CARDANO_NTN_SUPPORTED_VERSIONS.iter().copied().min().unwrap_or(0),
        CARDANO_NTN_SUPPORTED_VERSIONS.iter().copied().max().unwrap_or(0),
        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
        cardano_ntn_leios_overlay_capable(&CARDANO_NTN_SUPPORTED_VERSIONS),
        matrix.public_testnet_profiles,
        matrix.passed_profiles,
        matrix.offline_complete,
        matrix.live_ready,
        matrix_profiles,
        matrix_profile_ports,
        matrix.blockers.join(";"),
        matrix.production_ready,
        matrix.live_integrated,
    );
    println!(
        "testnet_conformance_actions actions={} live_protocol_send=false",
        matrix.action_items().join(";")
    );
    Ok(())
}

fn print_testnet_conformance() -> Result<(), Box<dyn std::error::Error>> {
    let matrix = testnet_handshake_conformance_matrix(&CARDANO_NTN_SUPPORTED_VERSIONS)?;
    let matrix_profiles = testnet_conformance_matrix_profiles(&matrix);
    let matrix_profile_ports = testnet_conformance_matrix_profile_ports(&matrix);
    println!("offline public testnet conformance matrix");
    println!("local_only=true live_protocol_send=false");
    println!(
        "public_testnets={} passed_profiles={} offline_complete={} live_ready={} profiles={} profile_ports={} versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={}",
        matrix.public_testnet_profiles,
        matrix.passed_profiles,
        matrix.offline_complete,
        matrix.live_ready,
        matrix_profiles,
        matrix_profile_ports,
        CARDANO_NTN_SUPPORTED_VERSIONS.len(),
        CARDANO_NTN_SUPPORTED_VERSIONS.iter().copied().min().unwrap_or(0),
        CARDANO_NTN_SUPPORTED_VERSIONS.iter().copied().max().unwrap_or(0),
        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
        cardano_ntn_leios_overlay_capable(&CARDANO_NTN_SUPPORTED_VERSIONS)
    );
    println!("blockers={}", matrix.blockers.join(";"));
    println!("actions={}", matrix.action_items().join(";"));
    println!("{}", testnet_live_capable_path_line());
    println!("{}", tcp_probe_gate_line());
    println!("{}", testnet_handshake_live_capable_path_line());
    println!("{}", handshake_probe_gate_line());
    for report in &matrix.reports {
        println!(
            "profile={} magic={} port={} versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} offline_checks={} passed={} offline_complete={} live_ready={} actions={} production_ready={} live_integrated={}",
            report.profile.name,
            report.profile.network_magic,
            report.profile.default_node_port,
            report.proposed_versions.len(),
            report.proposed_versions.iter().copied().min().unwrap_or(0),
            report.proposed_versions.iter().copied().max().unwrap_or(0),
            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
            cardano_ntn_leios_overlay_capable(&report.proposed_versions),
            report.offline_checks,
            report.passed_checks,
            report.offline_complete,
            report.live_ready,
            report.action_items().join(";"),
            report.production_ready,
            report.live_integrated,
        );
    }
    Ok(())
}

fn testnet_conformance_matrix_profiles(matrix: &TestnetHandshakeConformanceMatrix) -> String {
    matrix
        .reports
        .iter()
        .map(|report| report.profile.name)
        .collect::<Vec<_>>()
        .join(",")
}

fn testnet_conformance_matrix_profile_ports(matrix: &TestnetHandshakeConformanceMatrix) -> String {
    matrix
        .reports
        .iter()
        .map(|report| {
            format!(
                "{}:{}",
                report.profile.name, report.profile.default_node_port
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn print_testnet_readiness() {
    let readiness = testnet_live_readiness();
    let public_testnet_profile_names = public_testnet_profile_names();
    let public_testnet_profile_ports = public_testnet_profile_ports();
    println!("offline testnet live-readiness report");
    println!(
        "local_only=true paths_opened=false dials=false remote_fetch=false state_mutation=false live_protocol_send=false"
    );
    println!(
        "tcp_probe_available={} handshake_sketch_available={} offline_conformance_complete={} public_testnet_profiles={}",
        readiness.tcp_probe_available,
        readiness.handshake_sketch_available,
        readiness.offline_conformance_complete,
        readiness.public_testnet_profiles
    );
    println!("public_testnet_profile_names={public_testnet_profile_names}");
    println!("public_testnet_profile_ports={public_testnet_profile_ports}");
    println!(
        "command_available={} path_review_complete={} conformance_complete={} live_contact_allowed={}",
        readiness.live_command_available,
        readiness.path_review_complete,
        readiness.conformance_complete,
        readiness.live_contact_allowed
    );
    println!("{}", testnet_live_capable_path_line());
    println!("{}", tcp_probe_gate_line());
    println!("{}", testnet_handshake_live_capable_path_line());
    println!("{}", handshake_probe_gate_line());
    println!("blockers={}", readiness.blockers.join(";"));
    println!("actions={}", readiness.action_items().join(";"));
}

fn public_testnet_profile_names() -> String {
    network_profiles()
        .iter()
        .filter(|profile| profile.kind == NetworkProfileKind::Public && profile.name != "mainnet")
        .map(|profile| profile.name)
        .collect::<Vec<_>>()
        .join(",")
}

fn public_testnet_profile_ports() -> String {
    network_profiles()
        .iter()
        .filter(|profile| profile.kind == NetworkProfileKind::Public && profile.name != "mainnet")
        .map(|profile| format!("{}:{}", profile.name, profile.default_node_port))
        .collect::<Vec<_>>()
        .join(",")
}

fn tcp_probe_gate_line() -> String {
    format!(
        "tcp_probe_gate command=testnet-probe requires_allow_live_testnet=true peer_literal_ip=true public_testnet_only=true default_port_only=true max_timeout_secs={} retries=0 protocol_bytes_allowed=false mainnet_allowed=false",
        MAX_TESTNET_TCP_PROBE_TIMEOUT_SECS
    )
}

fn testnet_live_capable_path_line() -> &'static str {
    "live_capable_path command=testnet-probe tcp_only=true requires_allow_live_testnet=true protocol_command_available=false protocol_bytes_allowed=false mainnet_allowed=false"
}

fn testnet_handshake_live_capable_path_line() -> &'static str {
    "live_capable_path command=testnet-handshake-probe tcp_only=false requires_allow_live_testnet=true protocol_command_available=true protocol_bytes_allowed=true mainnet_allowed=false"
}

fn handshake_probe_gate_line() -> String {
    format!(
        "handshake_probe_gate command=testnet-handshake-probe requires_allow_live_testnet=true peer_literal_ip=true public_testnet_only=true default_port_only=true versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} max_timeout_secs={} dials=1 retries=0 write_frames=1 read_frames=1 max_response_bytes={} mainnet_allowed=false",
        CARDANO_NTN_SUPPORTED_VERSIONS.len(),
        CARDANO_NTN_SUPPORTED_VERSIONS.iter().copied().min().unwrap_or(0),
        CARDANO_NTN_SUPPORTED_VERSIONS.iter().copied().max().unwrap_or(0),
        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
        cardano_ntn_leios_overlay_capable(&CARDANO_NTN_SUPPORTED_VERSIONS),
        MAX_TESTNET_TCP_PROBE_TIMEOUT_SECS,
        MAX_TESTNET_HANDSHAKE_PROBE_RESPONSE_BYTES
    )
}

fn run_testnet_probe(
    config: &NodeConfig,
    args: &TestnetProbeArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let peer = args
        .peer
        .as_ref()
        .ok_or("testnet-probe requires --testnet-peer <ip:port>")?;
    let limits = TestnetContactLimits {
        timeout_secs: args.timeout_secs,
        ..TestnetContactLimits::smoke_test()
    };
    let plan = plan_testnet_tcp_probe(
        TestnetTcpProbeRequest::new(
            config.network_name.clone(),
            peer.clone(),
            args.allow_live_testnet,
            args.timeout_secs,
        ),
        limits,
    )?;

    println!("bounded testnet TCP probe");
    println!(
        "network={} magic={} peer={} live_contact=true tcp_connect=true dials=1 retries=0 sends_protocol_bytes=false remote_fetch=false state_mutation=false timeout_secs={}",
        plan.profile.name,
        plan.profile.network_magic,
        plan.peer,
        plan.timeout_secs
    );
    let start = Instant::now();
    match TcpStream::connect_timeout(&plan.peer, Duration::from_secs(plan.timeout_secs)) {
        Ok(stream) => {
            let _ = stream.shutdown(Shutdown::Both);
            println!(
                "result status=connected elapsed_ms={}",
                start.elapsed().as_millis()
            );
        }
        Err(err) => {
            println!(
                "result status=failed elapsed_ms={} error_kind={:?}",
                start.elapsed().as_millis(),
                err.kind()
            );
        }
    }
    Ok(())
}

fn run_testnet_handshake_probe(
    config: &NodeConfig,
    args: &TestnetProbeArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let peer = args
        .peer
        .as_ref()
        .ok_or("testnet-handshake-probe requires --testnet-peer <ip:port>")?;
    let limits = TestnetContactLimits {
        timeout_secs: args.timeout_secs,
        ..TestnetContactLimits::smoke_test()
    };
    let plan = plan_testnet_handshake_probe(
        TestnetHandshakeProbeRequest::new(
            config.network_name.clone(),
            peer.clone(),
            args.allow_live_testnet,
            args.timeout_secs,
        ),
        limits,
        &CARDANO_NTN_SUPPORTED_VERSIONS,
    )?;

    println!("bounded testnet handshake probe");
    println!(
        "network={} magic={} peer={} live_contact=true tcp_connect=true protocol_handshake=true versions={} min_version={} max_version={} leios_overlay_min_version={} leios_overlay_capable={} dials=1 retries=0 write_frames=1 read_frames=1 max_response_bytes={} remote_fetch=false state_mutation=false timeout_secs={}",
        plan.profile.name,
        plan.profile.network_magic,
        plan.peer,
        plan.proposed_versions.len(),
        plan.proposed_versions.iter().copied().min().unwrap_or(0),
        plan.proposed_versions.iter().copied().max().unwrap_or(0),
        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
        cardano_ntn_leios_overlay_capable(&plan.proposed_versions),
        plan.max_response_bytes,
        plan.timeout_secs
    );
    let start = Instant::now();
    match TcpStream::connect_timeout(&plan.peer, Duration::from_secs(plan.timeout_secs)) {
        Ok(mut stream) => {
            let timeout = Some(Duration::from_secs(plan.timeout_secs));
            if let Err(err) = stream.set_read_timeout(timeout) {
                println!(
                    "result status=failed phase=configure elapsed_ms={} error_kind={:?}",
                    start.elapsed().as_millis(),
                    err.kind()
                );
                let _ = stream.shutdown(Shutdown::Both);
                return Ok(());
            }
            if let Err(err) = stream.set_write_timeout(timeout) {
                println!(
                    "result status=failed phase=configure elapsed_ms={} error_kind={:?}",
                    start.elapsed().as_millis(),
                    err.kind()
                );
                let _ = stream.shutdown(Shutdown::Both);
                return Ok(());
            }
            if let Err(err) = stream.write_all(&plan.request_frame.encoded) {
                println!(
                    "result status=failed phase=write elapsed_ms={} error_kind={:?}",
                    start.elapsed().as_millis(),
                    err.kind()
                );
                let _ = stream.shutdown(Shutdown::Both);
                return Ok(());
            }

            let mut header = [0u8; CARDANO_MUX_HEADER_BYTES];
            if let Err(err) = stream.read_exact(&mut header) {
                println!(
                    "result status=failed phase=read_header elapsed_ms={} error_kind={:?}",
                    start.elapsed().as_millis(),
                    err.kind()
                );
                let _ = stream.shutdown(Shutdown::Both);
                return Ok(());
            }
            let payload_len = u16::from_be_bytes([header[6], header[7]]) as usize;
            if payload_len == 0 || payload_len > plan.max_response_bytes {
                println!(
                    "result status=failed phase=read_payload elapsed_ms={} error_kind=response_size payload_bytes={} max_response_bytes={}",
                    start.elapsed().as_millis(),
                    payload_len,
                    plan.max_response_bytes
                );
                let _ = stream.shutdown(Shutdown::Both);
                return Ok(());
            }

            let mut response_frame = Vec::with_capacity(CARDANO_MUX_HEADER_BYTES + payload_len);
            response_frame.extend_from_slice(&header);
            let mut payload = vec![0; payload_len];
            if let Err(err) = stream.read_exact(&mut payload) {
                println!(
                    "result status=failed phase=read_payload elapsed_ms={} error_kind={:?}",
                    start.elapsed().as_millis(),
                    err.kind()
                );
                let _ = stream.shutdown(Shutdown::Both);
                return Ok(());
            }
            response_frame.extend_from_slice(&payload);

            match run_cardano_ntn_handshake_harness(
                plan.profile,
                &plan.proposed_versions,
                &response_frame,
            ) {
                Ok(run) => match &run.negotiation.outcome {
                    CardanoHandshakeNegotiationOutcome::Accepted {
                        version,
                        network_magic,
                        peer_sharing,
                        query,
                        ..
                    } => {
                        let leios_overlay_negotiated =
                            *version >= CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION;
                        println!(
                            "result status=accepted elapsed_ms={} final_state=done accepted_version={} leios_overlay_min_version={} leios_overlay_negotiated={} network_magic={} peer_sharing={} query={}",
                            start.elapsed().as_millis(),
                            version,
                            CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
                            leios_overlay_negotiated,
                            network_magic,
                            peer_sharing,
                            query
                        )
                    }
                    CardanoHandshakeNegotiationOutcome::Refused {
                        reason,
                        supported_versions,
                        ..
                    } => println!(
                        "result status=refused elapsed_ms={} final_state=done refusal_reason={} supported_versions={} supported_min_version={} supported_max_version={} leios_overlay_min_version={} supported_leios_overlay_capable={}",
                        start.elapsed().as_millis(),
                        handshake_refusal_reason_text(*reason),
                        supported_versions.len(),
                        supported_versions.iter().copied().min().unwrap_or(0),
                        supported_versions.iter().copied().max().unwrap_or(0),
                        CARDANO_NTN_LEIOS_OVERLAY_MIN_VERSION,
                        cardano_ntn_leios_overlay_capable(supported_versions)
                    ),
                },
                Err(err) => println!(
                    "result status=failed phase=parse elapsed_ms={} error_kind={}",
                    start.elapsed().as_millis(),
                    network_error_kind_text(&err)
                ),
            }
            let _ = stream.shutdown(Shutdown::Both);
        }
        Err(err) => {
            println!(
                "result status=failed phase=connect elapsed_ms={} error_kind={:?}",
                start.elapsed().as_millis(),
                err.kind()
            );
        }
    }
    Ok(())
}

fn handshake_refusal_reason_text(reason: CardanoHandshakeRefusalReason) -> &'static str {
    match reason {
        CardanoHandshakeRefusalReason::VersionMismatch => "version_mismatch",
        CardanoHandshakeRefusalReason::DecodeError => "decode_error",
        CardanoHandshakeRefusalReason::Refused => "refused",
    }
}

fn network_error_kind_text(err: &acropolis::network::NetworkError) -> &'static str {
    match err {
        acropolis::network::NetworkError::PathsClosed => "paths_closed",
        acropolis::network::NetworkError::ProtocolNotImplemented => "protocol_not_implemented",
        acropolis::network::NetworkError::ProtocolRequiresReview(_) => "protocol_requires_review",
        acropolis::network::NetworkError::UnknownNetwork(_) => "unknown_network",
        acropolis::network::NetworkError::TestnetContactBlocked(_) => "testnet_contact_blocked",
        acropolis::network::NetworkError::UnboundedTestnetLimit(_) => "unbounded_testnet_limit",
        acropolis::network::NetworkError::EmptyTestnetRequest(_) => "empty_testnet_request",
        acropolis::network::NetworkError::TestnetLimitExceeded { .. } => "testnet_limit_exceeded",
        acropolis::network::NetworkError::NetworkMismatch { .. } => "network_mismatch",
        acropolis::network::NetworkError::UnsupportedVersion { .. } => "unsupported_version",
        acropolis::network::NetworkError::InvalidHandshakeProposal(_) => {
            "invalid_handshake_proposal"
        }
    }
}

fn print_cardano_plan(args: &CardanoPlanArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = args
        .config_path
        .as_ref()
        .ok_or("cardano-plan requires --cardano-config <path>")?;
    let config_text = read_bounded_text(
        config_path,
        MAX_CARDANO_NODE_CONFIG_SIZE,
        "cardano node config",
    )?;
    let manifest = CardanoNodeConfigManifest::parse(&config_text)?;
    let genesis_digests = plan_genesis_digests(&manifest, args, config_path)?;
    let report = manifest.verify_genesis_hashes(&genesis_digests);
    let default_config = NodeConfig::default();
    let socket_path = args
        .socket_path
        .as_deref()
        .unwrap_or(default_config.local_socket_path.as_path());
    let socket_source = if args.socket_path.is_some() {
        "cli"
    } else {
        "default"
    };
    let mut discovery_topology = None;
    let mut discovery_rules = PeerSnapshotRules::default();
    let mut peer_snapshot_validation_status = "none";
    let mut priority_snapshot_relays = 0;
    let mut regular_snapshot_relays = 0;
    let mut topology_bootstrap_peers_configured = false;
    let mut topology_ledger_peers_enabled = false;
    let mut topology_ledger_after_slot = None;
    let mut topology_peer_snapshot_configured = false;
    let mut topology_peer_snapshot_usable_by_topology = false;

    println!("Cardano local config plan");
    println!(
        "config={} paths_opened=false genesis_files_read={} state_mutation=false",
        config_path.display(),
        args.read_genesis_files
    );
    println!(
        "genesis_hash_mode={}",
        if args.read_genesis_files {
            "raw-blake2b-256-by-era"
        } else {
            "caller-digest-only"
        }
    );
    println!(
        "socket path={} source={} activation=false bind=false node_to_client=false stale_cleanup=false paths_opened=false",
        socket_path.display(),
        socket_source
    );
    println!(
        "protocol={} consensus={} requires_network_magic={} configured={} complete={}",
        manifest.protocol.as_deref().unwrap_or("unknown"),
        manifest.consensus_mode.as_deref().unwrap_or("unknown"),
        manifest
            .requires_network_magic
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        manifest.protocol_identity_configured_count(),
        manifest.protocol_identity_config_complete()
    );
    println!(
        "protocol_features experimental_hard_forks={} experimental_protocols={} configured={} complete={}",
        optional_bool_text(manifest.protocol_features.experimental_hard_forks_enabled),
        optional_bool_text(manifest.protocol_features.experimental_protocols_enabled),
        manifest.protocol_features.configured_count(),
        manifest.protocol_features.config_complete()
    );
    println!(
        "application name={} version={} max_concurrency_deadline={} configured={} complete={}",
        manifest.application.name.as_deref().unwrap_or("unknown"),
        optional_u64_text(manifest.application.version),
        optional_u64_text(manifest.application.max_concurrency_deadline),
        manifest.application.configured_count(),
        manifest.application.config_complete()
    );
    println!(
        "test_hard_forks shelley_epoch={} allegra_epoch={} mary_epoch={} alonzo_epoch={} configured={} complete={}",
        optional_u64_text(manifest.test_hard_forks.shelley_epoch),
        optional_u64_text(manifest.test_hard_forks.allegra_epoch),
        optional_u64_text(manifest.test_hard_forks.mary_epoch),
        optional_u64_text(manifest.test_hard_forks.alonzo_epoch),
        manifest.test_hard_forks.configured_count(),
        manifest.test_hard_forks.config_complete()
    );
    println!(
        "metadata max_known_major_protocol_version={} min_node_version={}",
        optional_u64_text(manifest.max_known_major_protocol_version),
        manifest.min_node_version.as_deref().unwrap_or("unknown")
    );
    println!(
        "ledger_db backend={} snapshots={} query_batch_size={} snapshot_interval={} configured={} complete={}",
        manifest.ledger_db.backend.as_deref().unwrap_or("unknown"),
        optional_u64_text(manifest.ledger_db.num_disk_snapshots),
        optional_u64_text(manifest.ledger_db.query_batch_size),
        optional_u64_text(manifest.ledger_db.snapshot_interval),
        manifest.ledger_db.configured_count(),
        manifest.ledger_db.config_complete()
    );
    println!(
        "p2p_governor_deadline root_peers={} known_peers={} established_peers={} active_peers={} known_big_ledger_peers={} established_big_ledger_peers={} active_big_ledger_peers={} configured={} complete={}",
        optional_u64_text(manifest.p2p_governor.deadline.root_peers),
        optional_u64_text(manifest.p2p_governor.deadline.known_peers),
        optional_u64_text(manifest.p2p_governor.deadline.established_peers),
        optional_u64_text(manifest.p2p_governor.deadline.active_peers),
        optional_u64_text(manifest.p2p_governor.deadline.known_big_ledger_peers),
        optional_u64_text(manifest.p2p_governor.deadline.established_big_ledger_peers),
        optional_u64_text(manifest.p2p_governor.deadline.active_big_ledger_peers),
        manifest.p2p_governor.deadline.configured_count(),
        manifest.p2p_governor.deadline.config_complete()
    );
    println!(
        "p2p_governor_sync root_peers={} known_peers={} established_peers={} active_peers={} known_big_ledger_peers={} established_big_ledger_peers={} active_big_ledger_peers={} min_big_ledger_peers_for_trusted_state={} configured={} complete={}",
        optional_u64_text(manifest.p2p_governor.sync.root_peers),
        optional_u64_text(manifest.p2p_governor.sync.known_peers),
        optional_u64_text(manifest.p2p_governor.sync.established_peers),
        optional_u64_text(manifest.p2p_governor.sync.active_peers),
        optional_u64_text(manifest.p2p_governor.sync.known_big_ledger_peers),
        optional_u64_text(manifest.p2p_governor.sync.established_big_ledger_peers),
        optional_u64_text(manifest.p2p_governor.sync.active_big_ledger_peers),
        optional_u64_text(manifest.p2p_governor.min_big_ledger_peers_for_trusted_state),
        manifest.p2p_governor.sync.configured_count(),
        manifest.p2p_governor.sync.config_complete()
    );
    println!(
        "mempool_runtime timeout_soft={} timeout_hard={} timeout_capacity={} timeouts_configured={} timeouts_complete={}",
        manifest.mempool.timeout_soft.as_deref().unwrap_or("unknown"),
        manifest.mempool.timeout_hard.as_deref().unwrap_or("unknown"),
        manifest
            .mempool
            .timeout_capacity
            .as_deref()
            .unwrap_or("unknown"),
        manifest.mempool.timeout_configured_count(),
        manifest.mempool.timeout_config_complete()
    );
    println!(
        "byron_protocol last_known_block_version={}.{}.{} pbft_signature_threshold={} configured={} complete={}",
        optional_u64_text(manifest.byron_protocol.last_known_block_version_major),
        optional_u64_text(manifest.byron_protocol.last_known_block_version_minor),
        optional_u64_text(manifest.byron_protocol.last_known_block_version_alt),
        manifest
            .byron_protocol
            .pbft_signature_threshold
            .as_deref()
            .unwrap_or("unknown"),
        manifest.byron_protocol.configured_count(),
        manifest.byron_protocol.config_complete()
    );
    println!(
        "checkpoints file={} expected_hash={} configured={} complete={}",
        manifest.checkpoints.file.as_deref().unwrap_or("none"),
        if manifest.checkpoints.expected_hash.is_some() {
            "yes"
        } else {
            "no"
        },
        manifest.checkpoints.configured_count(),
        manifest.checkpoints.config_complete()
    );
    println!(
        "tracing log_metrics={} logging={} dispatcher={} min_severity={} trace_scalar_fields={} runtime_configured={} runtime_complete={}",
        optional_bool_text(manifest.tracing.turn_on_log_metrics),
        optional_bool_text(manifest.tracing.turn_on_logging),
        optional_bool_text(manifest.tracing.use_trace_dispatcher),
        manifest
            .tracing
            .min_severity
            .as_deref()
            .unwrap_or("unknown"),
        manifest.tracing.trace_scalar_fields,
        manifest.tracing.runtime_configured_count(),
        manifest.tracing.runtime_config_complete()
    );
    println!(
        "trace_options entries={} severity_overrides={} detail_overrides={} frequency_limits={} metrics_prefix={} resource_frequency={} forwarder_conn_queue={} forwarder_disconn_queue={} forwarder_max_reconnect_delay={}",
        manifest.tracing.trace_option_entries,
        manifest.tracing.trace_severity_overrides,
        manifest.tracing.trace_detail_overrides,
        manifest.tracing.trace_frequency_limits,
        manifest
            .tracing
            .metrics_prefix
            .as_deref()
            .unwrap_or("unknown"),
        optional_u64_text(manifest.tracing.resource_frequency),
        optional_u64_text(manifest.tracing.forwarder_conn_queue_size),
        optional_u64_text(manifest.tracing.forwarder_disconn_queue_size),
        optional_u64_text(manifest.tracing.forwarder_max_reconnect_delay)
    );
    println!(
        "trace_severities silence={} debug={} info={} notice={} warning={} error={} critical={} other={}",
        manifest.tracing.trace_severity_silence,
        manifest.tracing.trace_severity_debug,
        manifest.tracing.trace_severity_info,
        manifest.tracing.trace_severity_notice,
        manifest.tracing.trace_severity_warning,
        manifest.tracing.trace_severity_error,
        manifest.tracing.trace_severity_critical,
        manifest.tracing.trace_severity_other
    );
    println!(
        "trace_arrays trace_backends={} default_backends={} default_scribes={} setup_backends={} setup_scribes={}",
        manifest.tracing.trace_backend_entries,
        manifest.tracing.default_backends,
        manifest.tracing.default_scribes,
        manifest.tracing.setup_backends,
        manifest.tracing.setup_scribes
    );
    println!(
        "trace_backend_sinks ekg={} forwarder={} prometheus={} stdout={} katip={} other={} prometheus_host={} prometheus_port={}",
        manifest.tracing.trace_backend_ekg,
        manifest.tracing.trace_backend_forwarder,
        manifest.tracing.trace_backend_prometheus,
        manifest.tracing.trace_backend_stdout,
        manifest.tracing.trace_backend_katip,
        manifest.tracing.trace_backend_other,
        manifest
            .tracing
            .trace_prometheus_host
            .as_deref()
            .unwrap_or("unknown"),
        optional_u64_text(manifest.tracing.trace_prometheus_port)
    );
    println!(
        "legacy_backend_sinks default_ekg={} default_katip={} default_other={} setup_ekg={} setup_katip={} setup_other={}",
        manifest.tracing.default_backend_ekg,
        manifest.tracing.default_backend_katip,
        manifest.tracing.default_backend_other,
        manifest.tracing.setup_backend_ekg,
        manifest.tracing.setup_backend_katip,
        manifest.tracing.setup_backend_other
    );
    println!(
        "legacy_scribe_sinks default_stdout={} default_file={} default_other={} setup_stdout={} setup_file={} setup_other={}",
        manifest.tracing.default_scribe_stdout,
        manifest.tracing.default_scribe_file,
        manifest.tracing.default_scribe_other,
        manifest.tracing.setup_scribe_stdout,
        manifest.tracing.setup_scribe_file,
        manifest.tracing.setup_scribe_other
    );
    println!(
        "legacy_trace_flags total={} enabled={} disabled={}",
        manifest.tracing.legacy_trace_flags,
        manifest.tracing.legacy_trace_enabled,
        manifest.tracing.legacy_trace_disabled
    );
    println!(
        "legacy_telemetry ekg_port={} prometheus_host={} prometheus_port={} prometheus_items={}",
        optional_u64_text(manifest.tracing.has_ekg_port),
        manifest
            .tracing
            .has_prometheus_host
            .as_deref()
            .unwrap_or("unknown"),
        optional_u64_text(manifest.tracing.has_prometheus_port),
        manifest.tracing.has_prometheus_items
    );
    println!(
        "legacy_trace_config verbosity={} rotation_keep_files={} rotation_log_limit_bytes={} rotation_max_age_hours={} map_backends={} map_backend_items={} map_subtraces={}",
        manifest
            .tracing
            .tracing_verbosity
            .as_deref()
            .unwrap_or("unknown"),
        optional_u64_text(manifest.tracing.rotation_keep_files_num),
        optional_u64_text(manifest.tracing.rotation_log_limit_bytes),
        optional_u64_text(manifest.tracing.rotation_max_age_hours),
        manifest.tracing.legacy_map_backend_entries,
        manifest.tracing.legacy_map_backend_items,
        manifest.tracing.legacy_map_subtrace_entries
    );
    if let Some(network_name) = &args.network_name {
        let profile = network_profile(network_name)
            .ok_or_else(|| format!("unknown cardano-plan network {network_name}"))?;
        let check = manifest.check_network_magic_requirement(profile);
        println!(
            "network_profile={} network_magic={} requires_network_magic_expected={} actual={} status={}",
            check.profile_name,
            profile.network_magic,
            check.expected,
            check
                .actual
                .map(|value| value.to_string())
                .unwrap_or_else(|| "missing".to_string()),
            network_magic_status_text(check.status)
        );
    } else {
        println!("network_profile=none");
    }
    println!("genesis_files={}", manifest.genesis_files.len());
    let has_genesis_file = |era: &str| {
        manifest
            .genesis_files
            .iter()
            .any(|fixture| fixture.era.eq_ignore_ascii_case(era))
    };
    let genesis_expected_hashes = manifest
        .genesis_files
        .iter()
        .filter(|fixture| fixture.expected_hash.is_some())
        .count();
    let raw_file_hash_eras = manifest
        .genesis_files
        .iter()
        .filter(|fixture| raw_genesis_file_digest_supported(&fixture.era))
        .map(|fixture| fixture.era.as_str())
        .collect::<Vec<_>>();
    let raw_file_hash_era_text = if raw_file_hash_eras.is_empty() {
        "none".to_string()
    } else {
        raw_file_hash_eras.join(",")
    };
    let canonical_hash_blocked_eras = manifest
        .genesis_files
        .iter()
        .filter(|fixture| !raw_genesis_file_digest_supported(&fixture.era))
        .map(|fixture| fixture.era.as_str())
        .collect::<Vec<_>>();
    let canonical_hash_blocked_era_text = if canonical_hash_blocked_eras.is_empty() {
        "none".to_string()
    } else {
        canonical_hash_blocked_eras.join(",")
    };
    println!(
        "genesis_era_coverage byron={} shelley={} alonzo={} conway={} dijkstra={} expected_hashes={} missing_hashes={}",
        has_genesis_file("Byron"),
        has_genesis_file("Shelley"),
        has_genesis_file("Alonzo"),
        has_genesis_file("Conway"),
        has_genesis_file("Dijkstra"),
        genesis_expected_hashes,
        manifest
            .genesis_files
            .len()
            .saturating_sub(genesis_expected_hashes)
    );
    println!(
        "genesis_digest_support raw_file_hash_supported={} raw_file_hash_eras={} canonical_hash_blocked={} canonical_hash_blocked_eras={}",
        raw_file_hash_eras.len(),
        raw_file_hash_era_text,
        manifest
            .genesis_files
            .len()
            .saturating_sub(raw_file_hash_eras.len()),
        canonical_hash_blocked_era_text
    );
    let mut genesis_matches = 0;
    let mut genesis_mismatches = 0;
    let mut genesis_missing_expected = 0;
    let mut genesis_missing_actual = 0;
    for check in &report.checks {
        match check.status {
            GenesisHashStatus::Match => genesis_matches += 1,
            GenesisHashStatus::Mismatch => genesis_mismatches += 1,
            GenesisHashStatus::MissingExpected => genesis_missing_expected += 1,
            GenesisHashStatus::MissingActual => genesis_missing_actual += 1,
        }
    }
    println!(
        "genesis_hash_summary matched={} mismatched={} missing_expected={} missing_actual={} all_matched={}",
        genesis_matches,
        genesis_mismatches,
        genesis_missing_expected,
        genesis_missing_actual,
        report.all_matched()
    );
    for check in &report.checks {
        let expected = if check.expected_hash.is_some() {
            "yes"
        } else {
            "no"
        };
        let actual = if check.actual_hash.is_some() {
            "yes"
        } else {
            "no"
        };
        println!(
            "genesis era={} file={} expected_hash={} actual_digest={} status={}",
            check.era,
            check.file,
            expected,
            actual,
            genesis_status_text(check.status)
        );
    }
    if args.read_genesis_files {
        for fixture in &manifest.genesis_files {
            if !raw_genesis_file_digest_supported(&fixture.era) {
                println!(
                    "genesis_digest_support era={} raw_file_hash=false reason=canonical-hash-blocked",
                    fixture.era
                );
            }
        }
    }

    if let Some(topology_path) = &args.topology_path {
        let topology_text =
            read_bounded_text(topology_path, MAX_TOPOLOGY_SIZE, "cardano topology")?;
        let topology = parse_cardano_topology_json(&topology_text)?;
        let summary = topology.summary();
        println!(
            "topology={} local_roots={} public_roots={} local_peers={} public_peers={} bootstrap_peers={} unique_peers={} duplicate_peer_entries={} local_valency={} public_valency={} local_warm_valency={} public_warm_valency={} ledger_after_slot={} peer_snapshot={}",
            topology_path.display(),
            summary.local_roots,
            summary.public_roots,
            summary.local_peers,
            summary.public_peers,
            summary.bootstrap_peers,
            summary.unique_peers,
            summary.duplicate_peer_entries,
            summary.local_valency,
            summary.public_valency,
            summary.local_warm_valency,
            summary.public_warm_valency,
            topology.use_bootstrap_after_slot,
            topology.peer_snapshot_file.as_deref().unwrap_or("none")
        );
        println!(
            "topology_flags advertised_local_roots={} advertised_public_roots={} trustable_local_roots={} empty_local_roots={} empty_public_roots={} peer_snapshot_configured={}",
            summary.advertised_local_roots,
            summary.advertised_public_roots,
            summary.trustable_local_roots,
            summary.empty_local_roots,
            summary.empty_public_roots,
            summary.peer_snapshot_configured
        );
        println!(
            "topology_ledger ledger_peers_enabled={} peer_snapshot_usable_by_topology={} bootstrap_peers_configured={}",
            summary.ledger_peers_enabled,
            summary.peer_snapshot_usable_by_topology,
            summary.bootstrap_peers_configured
        );
        println!(
            "topology_trust trustable_local_peers={} trusted_peer_source_configured={}",
            summary.trustable_local_peers, summary.trusted_peer_source_configured
        );
        topology_bootstrap_peers_configured = summary.bootstrap_peers_configured;
        topology_ledger_peers_enabled = summary.ledger_peers_enabled;
        topology_ledger_after_slot = Some(topology.use_bootstrap_after_slot);
        topology_peer_snapshot_configured = summary.peer_snapshot_configured;
        topology_peer_snapshot_usable_by_topology = summary.peer_snapshot_usable_by_topology;
        discovery_topology = Some(topology);
    } else {
        println!("topology=none");
    }
    if let Some(peer_snapshot_path) = &args.peer_snapshot_path {
        let peer_snapshot_text = read_bounded_text(
            peer_snapshot_path,
            MAX_PEER_SNAPSHOT_SIZE,
            "cardano peer snapshot",
        )?;
        let peer_snapshot = parse_cardano_peer_snapshot_json(&peer_snapshot_text)?;
        let expected_network_magic = args
            .network_name
            .as_ref()
            .map(|network_name| {
                network_profile(network_name)
                    .ok_or_else(|| format!("unknown cardano-plan network {network_name}"))
            })
            .transpose()?
            .map(|profile| profile.network_magic);
        let validation_rules = PeerSnapshotRules {
            network_magic: expected_network_magic,
            ..PeerSnapshotRules::default()
        };
        let validation_result = peer_snapshot.validate(&validation_rules);
        let (validation, validation_error) = match &validation_result {
            Ok(()) => ("valid", "none"),
            Err(err) => ("invalid", peer_snapshot_error_kind(err)),
        };
        peer_snapshot_validation_status = validation;
        let expected_network_magic_text = expected_network_magic
            .map(|magic| magic.to_string())
            .unwrap_or_else(|| "none".to_string());
        let network_magic_status = match expected_network_magic {
            Some(expected) if expected == peer_snapshot.network_magic => "match",
            Some(_) => "mismatch",
            None => "not-checked",
        };
        println!(
            "peer_snapshot={} network_magic={} expected_network_magic={} network_magic_status={} node_to_client_version={} point_slot={} point_hash_present={} priority_pools={} pools={} relays={} validation={} validation_error={} paths_opened=false dials=false peer_sharing=false peer_governor=false state_mutation=false",
            peer_snapshot_path.display(),
            peer_snapshot.network_magic,
            expected_network_magic_text,
            network_magic_status,
            peer_snapshot.client_version,
            peer_snapshot.point.point_slot,
            !peer_snapshot.point.point_hash.trim().is_empty(),
            peer_snapshot.priority_pools.len(),
            peer_snapshot.pools.len(),
            peer_snapshot.relay_peers().len(),
            validation,
            validation_error,
        );
        discovery_rules = validation_rules;
        if validation_result.is_ok() {
            priority_snapshot_relays = peer_snapshot
                .priority_pools
                .iter()
                .map(|pool| pool.relays.len())
                .sum();
            regular_snapshot_relays = peer_snapshot
                .pools
                .iter()
                .map(|pool| pool.relays.len())
                .sum();
            discovery_topology
                .get_or_insert_with(|| TopologyConfig {
                    local_roots: Vec::new(),
                    public_roots: Vec::new(),
                    seed_peers: Vec::new(),
                    use_bootstrap_after_slot: -1,
                    peer_snapshot_file: None,
                    peer_snapshot: None,
                })
                .peer_snapshot = Some(peer_snapshot);
        }
    } else {
        println!(
            "peer_snapshot=none reads=false paths_opened=false dials=false peer_sharing=false peer_governor=false state_mutation=false"
        );
    }
    let (peer_targets, peer_target_source) = cardano_peer_targets(&manifest);
    let discovery = if let Some(topology) = &discovery_topology {
        let discovery = PeerDiscoveryPlan::from_topology(topology, &discovery_rules)?;
        println!(
            "{} snapshot_validation={} paths_opened=false dials=false peer_governor=false state_mutation=false",
            discovery.summary_line(),
            peer_snapshot_validation_status,
        );
        discovery
    } else {
        println!(
            "peer_discovery=none topology=false snapshot_validation={} paths_opened=false dials=false peer_governor=false state_mutation=false",
            peer_snapshot_validation_status,
        );
        PeerDiscoveryPlan {
            entries: Vec::new(),
        }
    };
    let discovery_counts = discovery.source_counts();
    let ledger_peer_candidates = if topology_ledger_peers_enabled {
        discovery_counts.snapshot_relays
    } else {
        0
    };
    println!(
        "peer_source_status bootstrap_peers={} bootstrap_configured={} bootstrap_blocked={} ledger_peers_enabled={} ledger_after_slot={} peer_snapshot_configured={} peer_snapshot_usable_by_topology={} snapshot_relays={} ledger_candidates={} snapshot_validation={} ledger_peers_blocked={} paths_opened=false dials=false peer_governor=false state_mutation=false",
        discovery_counts.seed_peers,
        topology_bootstrap_peers_configured,
        topology_bootstrap_peers_configured,
        topology_ledger_peers_enabled,
        topology_ledger_after_slot
            .map(|slot| slot.to_string())
            .unwrap_or_else(|| "none".to_string()),
        topology_peer_snapshot_configured,
        topology_peer_snapshot_usable_by_topology,
        discovery_counts.snapshot_relays,
        ledger_peer_candidates,
        peer_snapshot_validation_status,
        topology_ledger_peers_enabled,
    );
    let (root_peer_target, root_peer_target_source) = cardano_root_peer_target(&manifest);
    let root_peer_candidates = discovery_counts.local_roots + discovery_counts.public_roots;
    println!(
        "peer_root_status local_root_peers={} public_root_peers={} root_candidates={} root_target={} root_met={} root_deficit={} seed_peers={} snapshot_relays={} target_source={} paths_opened=false dials=false peer_governor=false state_mutation=false",
        discovery_counts.local_roots,
        discovery_counts.public_roots,
        root_peer_candidates,
        root_peer_target,
        peer_target_met(root_peer_candidates, root_peer_target),
        peer_target_deficit(root_peer_candidates, root_peer_target),
        discovery_counts.seed_peers,
        discovery_counts.snapshot_relays,
        root_peer_target_source,
    );
    let sharing_candidates = peer_sharing_candidate_counts(&discovery);
    println!(
        "peer_sharing_plan advertise_candidates={} local_advertise_candidates={} public_advertise_candidates={} trustable_candidates={} peer_sharing=false paths_opened=false dials=false peer_governor=false state_mutation=false",
        sharing_candidates.advertise,
        sharing_candidates.local_advertise,
        sharing_candidates.public_advertise,
        sharing_candidates.trustable,
    );
    let mut peer_set = PeerSet::new(peer_targets);
    let lifecycle = peer_set.apply_discovery_plan(&discovery);
    println!(
        "{} target_source={} targets_known={} targets_established={} targets_active={} paths_opened=false dials=false peer_governor=false state_mutation=false",
        lifecycle.summary_line(),
        peer_target_source,
        peer_targets.known,
        peer_targets.established,
        peer_targets.active,
    );
    let peer_counts = peer_set.counts();
    println!(
        "peer_target_status known={} warm={} hot={} known_met={} established_met={} active_met={} known_deficit={} established_deficit={} active_deficit={} target_source={} paths_opened=false dials=false peer_governor=false state_mutation=false",
        peer_counts.known,
        peer_counts.warm,
        peer_counts.hot,
        peer_target_met(peer_counts.known, peer_targets.known),
        peer_target_met(peer_counts.warm, peer_targets.established),
        peer_target_met(peer_counts.hot, peer_targets.active),
        peer_target_deficit(peer_counts.known, peer_targets.known),
        peer_target_deficit(peer_counts.warm, peer_targets.established),
        peer_target_deficit(peer_counts.hot, peer_targets.active),
        peer_target_source,
    );
    let (big_ledger_targets, big_ledger_target_source) = cardano_big_ledger_peer_targets(&manifest);
    println!(
        "peer_big_ledger_availability priority_relays={} regular_relays={} snapshot_validation={} known_target={} established_target={} active_target={} min_trusted_state={} known_candidates_met={} established_candidates_met={} active_candidates_met={} trusted_state_candidates_met={} known_deficit={} established_deficit={} active_deficit={} trusted_state_deficit={} target_source={} paths_opened=false dials=false peer_governor=false state_mutation=false",
        priority_snapshot_relays,
        regular_snapshot_relays,
        peer_snapshot_validation_status,
        big_ledger_targets.known,
        big_ledger_targets.established,
        big_ledger_targets.active,
        big_ledger_targets.min_trusted_state,
        peer_target_met(priority_snapshot_relays, big_ledger_targets.known),
        peer_target_met(priority_snapshot_relays, big_ledger_targets.established),
        peer_target_met(priority_snapshot_relays, big_ledger_targets.active),
        peer_target_met(priority_snapshot_relays, big_ledger_targets.min_trusted_state),
        peer_target_deficit(priority_snapshot_relays, big_ledger_targets.known),
        peer_target_deficit(priority_snapshot_relays, big_ledger_targets.established),
        peer_target_deficit(priority_snapshot_relays, big_ledger_targets.active),
        peer_target_deficit(priority_snapshot_relays, big_ledger_targets.min_trusted_state),
        big_ledger_target_source,
    );
    let connection_plan = peer_set.connection_plan(false);
    println!(
        "{} target_source={} targets_known={} targets_established={} targets_active={} dials=false peer_governor=false state_mutation=false",
        connection_plan.summary_line(),
        peer_target_source,
        peer_targets.known,
        peer_targets.established,
        peer_targets.active,
    );
    Ok(())
}

fn cardano_peer_targets(manifest: &CardanoNodeConfigManifest) -> (PeerTargets, &'static str) {
    let defaults = PeerTargets::default();
    let (targets, source) =
        if peer_target_config_has_connection_targets(&manifest.p2p_governor.sync) {
            (&manifest.p2p_governor.sync, "p2p_sync")
        } else if peer_target_config_has_connection_targets(&manifest.p2p_governor.deadline) {
            (&manifest.p2p_governor.deadline, "p2p_deadline")
        } else {
            return (defaults, "default");
        };

    (
        PeerTargets {
            known: peer_target_i32(targets.known_peers, defaults.known),
            established: peer_target_i32(targets.established_peers, defaults.established),
            active: peer_target_i32(targets.active_peers, defaults.active),
        },
        source,
    )
}

fn peer_target_config_has_connection_targets(targets: &PeerTargetConfig) -> bool {
    targets.known_peers.is_some()
        || targets.established_peers.is_some()
        || targets.active_peers.is_some()
}

fn cardano_root_peer_target(manifest: &CardanoNodeConfigManifest) -> (i32, &'static str) {
    if let Some(target) = manifest.p2p_governor.sync.root_peers {
        (peer_target_i32(Some(target), 0), "p2p_sync")
    } else if let Some(target) = manifest.p2p_governor.deadline.root_peers {
        (peer_target_i32(Some(target), 0), "p2p_deadline")
    } else {
        (0, "default")
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct PeerSharingCandidateCounts {
    advertise: usize,
    local_advertise: usize,
    public_advertise: usize,
    trustable: usize,
}

fn peer_sharing_candidate_counts(discovery: &PeerDiscoveryPlan) -> PeerSharingCandidateCounts {
    let mut counts = PeerSharingCandidateCounts::default();
    for entry in &discovery.entries {
        if entry.trustable {
            counts.trustable += 1;
        }
        if !entry.advertise {
            continue;
        }
        counts.advertise += 1;
        match entry.source {
            PeerDiscoverySource::LocalRoot => counts.local_advertise += 1,
            PeerDiscoverySource::PublicRoot => counts.public_advertise += 1,
            PeerDiscoverySource::Seed | PeerDiscoverySource::Snapshot => {}
        }
    }
    counts
}

fn peer_target_i32(value: Option<u64>, default: i32) -> i32 {
    match value {
        Some(value) => i32::try_from(value).unwrap_or(i32::MAX),
        None => default,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BigLedgerPeerTargets {
    known: i32,
    established: i32,
    active: i32,
    min_trusted_state: i32,
}

fn cardano_big_ledger_peer_targets(
    manifest: &CardanoNodeConfigManifest,
) -> (BigLedgerPeerTargets, &'static str) {
    let defaults = BigLedgerPeerTargets {
        known: 0,
        established: 0,
        active: 0,
        min_trusted_state: 0,
    };
    let (targets, source) =
        if peer_target_config_has_big_ledger_targets(&manifest.p2p_governor.sync) {
            (&manifest.p2p_governor.sync, "p2p_sync")
        } else if peer_target_config_has_big_ledger_targets(&manifest.p2p_governor.deadline) {
            (&manifest.p2p_governor.deadline, "p2p_deadline")
        } else {
            return (defaults, "default");
        };

    (
        BigLedgerPeerTargets {
            known: peer_target_i32(targets.known_big_ledger_peers, defaults.known),
            established: peer_target_i32(
                targets.established_big_ledger_peers,
                defaults.established,
            ),
            active: peer_target_i32(targets.active_big_ledger_peers, defaults.active),
            min_trusted_state: peer_target_i32(
                manifest.p2p_governor.min_big_ledger_peers_for_trusted_state,
                defaults.min_trusted_state,
            ),
        },
        source,
    )
}

fn peer_target_config_has_big_ledger_targets(targets: &PeerTargetConfig) -> bool {
    targets.known_big_ledger_peers.is_some()
        || targets.established_big_ledger_peers.is_some()
        || targets.active_big_ledger_peers.is_some()
}

fn peer_target_met(count: usize, target: i32) -> bool {
    count >= peer_target_usize(target)
}

fn peer_target_deficit(count: usize, target: i32) -> usize {
    peer_target_usize(target).saturating_sub(count)
}

fn peer_target_usize(target: i32) -> usize {
    target.max(0) as usize
}

fn peer_snapshot_error_kind(err: &PeerSnapshotError) -> &'static str {
    match err {
        PeerSnapshotError::NetworkMismatch { .. } => "network_mismatch",
        PeerSnapshotError::EmptyPointHash => "empty_point_hash",
        PeerSnapshotError::TooManyPools { .. } => "too_many_pools",
        PeerSnapshotError::TooManyRelays { .. } => "too_many_relays",
        PeerSnapshotError::InvalidStake => "invalid_stake",
        PeerSnapshotError::InvalidRelay(_) => "invalid_relay",
        PeerSnapshotError::DuplicateRelay(_) => "duplicate_relay",
    }
}

fn plan_genesis_digests(
    manifest: &CardanoNodeConfigManifest,
    args: &CardanoPlanArgs,
    config_path: &Path,
) -> Result<Vec<GenesisFileDigest>, Box<dyn std::error::Error>> {
    if !args.read_genesis_files {
        return Ok(args.genesis_digests.clone());
    }

    let mut digests = Vec::new();
    for fixture in &manifest.genesis_files {
        if !raw_genesis_file_digest_supported(&fixture.era) {
            continue;
        }
        let path = resolve_genesis_path(config_path, &fixture.file);
        let bytes =
            read_bounded_bytes(&path, MAX_CARDANO_GENESIS_FILE_SIZE, "cardano genesis file")?;
        digests.push(GenesisFileDigest::new(
            fixture.era.clone(),
            blake2b_256_hex(&bytes),
        ));
    }
    for digest in &args.genesis_digests {
        if !digests
            .iter()
            .any(|computed| computed.era.eq_ignore_ascii_case(&digest.era))
        {
            digests.push(digest.clone());
        }
    }
    Ok(digests)
}

fn resolve_genesis_path(config_path: &Path, genesis_file: &str) -> PathBuf {
    let genesis_path = PathBuf::from(genesis_file);
    if genesis_path.is_absolute() {
        genesis_path
    } else {
        config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(genesis_path)
    }
}

fn read_bounded_text(
    path: &Path,
    max_bytes: usize,
    label: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > max_bytes as u64 {
        return Err(format!(
            "{label} too large: max={max_bytes} actual={}",
            metadata.len()
        )
        .into());
    }
    let mut text = String::new();
    std::fs::File::open(path)?
        .take(max_bytes.saturating_add(1) as u64)
        .read_to_string(&mut text)?;
    if text.len() > max_bytes {
        return Err(format!("{label} too large: max={max_bytes} actual={}", text.len()).into());
    }
    Ok(text)
}

fn read_bounded_bytes(
    path: &Path,
    max_bytes: usize,
    label: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > max_bytes as u64 {
        return Err(format!(
            "{label} too large: max={max_bytes} actual={}",
            metadata.len()
        )
        .into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(max_bytes.saturating_add(1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(format!("{label} too large: max={max_bytes} actual={}", bytes.len()).into());
    }
    Ok(bytes)
}

fn genesis_status_text(status: GenesisHashStatus) -> &'static str {
    match status {
        GenesisHashStatus::Match => "match",
        GenesisHashStatus::Mismatch => "mismatch",
        GenesisHashStatus::MissingExpected => "missing-expected",
        GenesisHashStatus::MissingActual => "missing-actual",
    }
}

fn network_magic_status_text(status: NetworkMagicRequirementStatus) -> &'static str {
    match status {
        NetworkMagicRequirementStatus::Match => "match",
        NetworkMagicRequirementStatus::Mismatch => "mismatch",
        NetworkMagicRequirementStatus::MissingConfig => "missing-config",
    }
}

fn optional_u64_text(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn optional_bool_text(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

fn print_help() {
    println!("usage: acropolis [plan|guard|networks|testnet-plan|testnet-probe|testnet-handshake-probe|testnet-conformance|testnet-readiness|cardano-plan|agent-flow|agent-safety|open] [options]");
    println!("default command: plan");
    println!("options:");
    println!("  --config <path>             read local key=value or JSON-object config patch");
    println!("  --set <key=value>           apply CLI config override; repeatable");
    println!(
        "  --network <name>            select offline profile, e.g. mainnet/preprod/preview/local"
    );
    println!("  --data-mode <core|full>     choose local data mode");
    println!("  --transaction-port <port>   plan transaction interface port");
    println!("  --testnet-blocks <n>        bounded testnet-plan requested blocks");
    println!("  --testnet-slots <n>         bounded testnet-plan requested slots");
    println!("  --testnet-bytes <n>         bounded testnet-plan requested bytes");
    println!("  --max-blocks <n>            bounded testnet-plan max blocks");
    println!("  --max-slots <n>             bounded testnet-plan max slots");
    println!("  --max-bytes <n>             bounded testnet-plan max bytes");
    println!("  --timeout-secs <n>          bounded testnet-plan timeout seconds");
    println!("  --temp-bytes <n>            bounded testnet-plan temporary storage bytes");
    println!("  --testnet-peer <ip:port>    explicit public testnet peer for probe commands");
    println!("  --probe-timeout-secs <n>    testnet probe timeout, max 5 seconds");
    println!("  --allow-live-testnet        allow one bounded live testnet probe");
    println!("  --cardano-config <path>     cardano-plan local node config JSON");
    println!("  --cardano-topology <path>   cardano-plan local topology JSON");
    println!("  --cardano-peer-snapshot <path> cardano-plan local peer snapshot JSON");
    println!("  --cardano-network <name>    cardano-plan network magic requirement check");
    println!("  --socket <path>             plan local socket path without binding it");
    println!("  --genesis-digest <era=hash> cardano-plan caller-provided genesis digest");
    println!(
        "  --read-genesis-files        cardano-plan read bounded local non-Byron genesis files"
    );
    println!("  --allow-paths[=bool]        record path opt-in; network open still fails until implemented");
    println!("agent-flow prints a local-only Agent Flow snapshot without launching runtime agents");
    println!("agent-safety prints the local Agent Flow safety gate without opening runtime paths");
    println!("testnet-conformance prints the offline public-testnet conformance matrix");
    println!("testnet-readiness prints the offline live-readiness blockers and actions");
    println!(
        "testnet-handshake-probe performs one opt-in bounded public-testnet handshake and reports v15/Leios negotiation"
    );
    println!("open is intentionally blocked until path opening is implemented and reviewed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parser_keeps_default_plan_command() {
        let request = CliRequest::parse(Vec::<String>::new()).unwrap();
        assert_eq!(request.command, Command::Plan);
        assert!(request.patch.is_empty());
    }

    #[test]
    fn cli_parser_accepts_status_command() {
        let request = CliRequest::parse(["status".to_string()]).unwrap();

        assert_eq!(request.command, Command::Status);
        assert!(request.patch.is_empty());
    }

    #[test]
    fn cli_parser_accepts_status_aliases() {
        for command in ["progress", "local-status"] {
            let request = CliRequest::parse([command.to_string()]).unwrap();

            assert_eq!(request.command, Command::Status);
            assert!(request.patch.is_empty());
        }
    }

    #[test]
    fn cli_parser_accepts_remaining_command_aliases() {
        for (command, expected) in [
            ("safety", Command::Guard),
            ("serve", Command::Open),
            ("run", Command::Open),
            ("network-profiles", Command::Networks),
            ("testnet-check-plan", Command::TestnetPlan),
            ("testnet-tcp-probe", Command::TestnetProbe),
            ("testnet-live-handshake", Command::TestnetHandshakeProbe),
            ("cardano-config-plan", Command::CardanoPlan),
            ("agentic-flow", Command::AgentFlow),
            ("agentic-safety", Command::AgentSafety),
        ] {
            let request = CliRequest::parse([command.to_string()]).unwrap();

            assert_eq!(request.command, expected);
            assert!(request.patch.is_empty());
        }
    }

    #[test]
    fn cli_parser_collects_flags_into_patch() {
        let request = CliRequest::parse([
            "plan".to_string(),
            "--network".to_string(),
            "mainnet".to_string(),
            "--data-mode=full".to_string(),
            "--set".to_string(),
            "transaction_port=3000".to_string(),
        ])
        .unwrap();
        assert_eq!(request.command, Command::Plan);
        assert_eq!(request.patch.entries.len(), 3);
    }

    #[test]
    fn cli_parser_collects_testnet_plan_bounds() {
        let request = CliRequest::parse([
            "testnet-plan".to_string(),
            "--network=testnet".to_string(),
            "--testnet-blocks=2".to_string(),
            "--max-blocks".to_string(),
            "3".to_string(),
            "--timeout-secs=4".to_string(),
        ])
        .unwrap();
        assert_eq!(request.command, Command::TestnetPlan);
        assert_eq!(request.testnet_plan.requested_blocks, 2);
        assert_eq!(request.testnet_plan.limits.max_blocks, 3);
        assert_eq!(request.testnet_plan.limits.timeout_secs, 4);
        assert_eq!(request.patch.entries.len(), 1);
    }

    #[test]
    fn cli_parser_collects_testnet_probe_gates() {
        let request = CliRequest::parse([
            "testnet-probe".to_string(),
            "--network=preview".to_string(),
            "--testnet-peer".to_string(),
            "8.8.8.8:3001".to_string(),
            "--allow-live-testnet".to_string(),
            "--probe-timeout-secs=2".to_string(),
        ])
        .unwrap();

        assert_eq!(request.command, Command::TestnetProbe);
        assert_eq!(request.patch.entries.len(), 1);
        assert_eq!(request.testnet_probe.peer.as_deref(), Some("8.8.8.8:3001"));
        assert!(request.testnet_probe.allow_live_testnet);
        assert_eq!(request.testnet_probe.timeout_secs, 2);
    }

    #[test]
    fn cli_parser_collects_testnet_handshake_probe_gates() {
        let request = CliRequest::parse([
            "testnet-handshake-probe".to_string(),
            "--network=preview".to_string(),
            "--testnet-peer".to_string(),
            "8.8.8.8:3001".to_string(),
            "--allow-live-testnet".to_string(),
            "--probe-timeout-secs=2".to_string(),
        ])
        .unwrap();

        assert_eq!(request.command, Command::TestnetHandshakeProbe);
        assert_eq!(request.patch.entries.len(), 1);
        assert_eq!(request.testnet_probe.peer.as_deref(), Some("8.8.8.8:3001"));
        assert!(request.testnet_probe.allow_live_testnet);
        assert_eq!(request.testnet_probe.timeout_secs, 2);
    }

    #[test]
    fn cli_parser_accepts_testnet_conformance_command() {
        let request = CliRequest::parse(["testnet-conformance".to_string()]).unwrap();

        assert_eq!(request.command, Command::TestnetConformance);
        assert!(request.patch.is_empty());
    }

    #[test]
    fn cli_parser_accepts_testnet_conformance_alias() {
        let request = CliRequest::parse(["testnet-conformance-plan".to_string()]).unwrap();

        assert_eq!(request.command, Command::TestnetConformance);
        assert!(request.patch.is_empty());
    }

    #[test]
    fn cli_parser_accepts_testnet_readiness_command() {
        let request = CliRequest::parse(["testnet-readiness".to_string()]).unwrap();

        assert_eq!(request.command, Command::TestnetReadiness);
        assert!(request.patch.is_empty());
    }

    #[test]
    fn cli_parser_accepts_testnet_readiness_alias() {
        let request = CliRequest::parse(["testnet-readiness-plan".to_string()]).unwrap();

        assert_eq!(request.command, Command::TestnetReadiness);
        assert!(request.patch.is_empty());
    }

    #[test]
    fn cli_parser_accepts_agent_flow_command() {
        let request = CliRequest::parse(["agent-flow".to_string()]).unwrap();

        assert_eq!(request.command, Command::AgentFlow);
        assert!(request.patch.is_empty());
    }

    #[test]
    fn cli_parser_accepts_agent_safety_command() {
        let request = CliRequest::parse(["agent-safety".to_string()]).unwrap();

        assert_eq!(request.command, Command::AgentSafety);
        assert!(request.patch.is_empty());
    }

    #[test]
    fn cli_parser_collects_cardano_plan_paths_and_digests() {
        let request = CliRequest::parse([
            "cardano-plan".to_string(),
            "--network=mainnet".to_string(),
            "--cardano-config=configuration/cardano/mainnet-config.json".to_string(),
            "--cardano-topology".to_string(),
            "configuration/cardano/mainnet-topology.json".to_string(),
            "--cardano-peer-snapshot=configuration/cardano/mainnet-peer-snapshot.json".to_string(),
            "--socket=cardano.socket".to_string(),
            "--genesis-digest".to_string(),
            "Byron=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            "--read-genesis-files".to_string(),
        ])
        .unwrap();

        assert_eq!(request.command, Command::CardanoPlan);
        assert_eq!(
            request.cardano_plan.config_path.as_deref(),
            Some(std::path::Path::new(
                "configuration/cardano/mainnet-config.json"
            ))
        );
        assert_eq!(
            request.cardano_plan.topology_path.as_deref(),
            Some(std::path::Path::new(
                "configuration/cardano/mainnet-topology.json"
            ))
        );
        assert_eq!(
            request.cardano_plan.peer_snapshot_path.as_deref(),
            Some(std::path::Path::new(
                "configuration/cardano/mainnet-peer-snapshot.json"
            ))
        );
        assert_eq!(request.cardano_plan.genesis_digests.len(), 1);
        assert_eq!(request.cardano_plan.genesis_digests[0].era, "Byron");
        assert!(request.cardano_plan.read_genesis_files);
        assert_eq!(
            request.cardano_plan.socket_path.as_deref(),
            Some(std::path::Path::new("cardano.socket"))
        );
        assert_eq!(
            request.cardano_plan.network_name.as_deref(),
            Some("mainnet")
        );
    }

    #[test]
    fn cli_parser_rejects_two_commands() {
        assert!(CliRequest::parse(["plan".to_string(), "guard".to_string()]).is_err());
    }

    #[test]
    fn cli_parser_rejects_bad_genesis_digest() {
        assert!(CliRequest::parse([
            "cardano-plan".to_string(),
            "--genesis-digest=Byron=bad".to_string(),
        ])
        .is_err());
    }

    #[test]
    fn cli_parser_rejects_duplicate_genesis_digest_eras() {
        assert_eq!(
            CliRequest::parse([
                "cardano-plan".to_string(),
                "--genesis-digest=Byron=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                "--genesis-digest".to_string(),
                "byron=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
            ]),
            Err("--genesis-digest duplicate era byron".to_string())
        );
    }

    #[test]
    fn bounded_text_reader_rejects_oversized_local_file() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "acropolis-bounded-read-{}-{nonce}.txt",
            std::process::id()
        ));
        std::fs::write(&path, "abcd").unwrap();

        let err = read_bounded_text(&path, 3, "fixture").unwrap_err();
        let _ = std::fs::remove_file(&path);

        assert_eq!(err.to_string(), "fixture too large: max=3 actual=4");
    }

    #[test]
    fn genesis_digest_planner_rejects_oversized_local_file() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "acropolis-genesis-read-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&dir).unwrap();
        let config_path = dir.join("config.json");
        let genesis_path = dir.join("shelley-genesis.json");
        std::fs::write(
            &config_path,
            r#"{ "ShelleyGenesisFile": "shelley-genesis.json" }"#,
        )
        .unwrap();
        let file = std::fs::File::create(&genesis_path).unwrap();
        file.set_len((MAX_CARDANO_GENESIS_FILE_SIZE + 1) as u64)
            .unwrap();
        drop(file);

        let manifest =
            CardanoNodeConfigManifest::parse(r#"{ "ShelleyGenesisFile": "shelley-genesis.json" }"#)
                .unwrap();
        let args = CardanoPlanArgs {
            config_path: Some(config_path.clone()),
            topology_path: None,
            peer_snapshot_path: None,
            genesis_digests: Vec::new(),
            read_genesis_files: true,
            network_name: None,
            socket_path: None,
        };

        let err = plan_genesis_digests(&manifest, &args, &config_path).unwrap_err();
        let _ = std::fs::remove_file(&genesis_path);
        let _ = std::fs::remove_file(&config_path);
        let _ = std::fs::remove_dir(&dir);

        assert_eq!(
            err.to_string(),
            format!(
                "cardano genesis file too large: max={} actual={}",
                MAX_CARDANO_GENESIS_FILE_SIZE,
                MAX_CARDANO_GENESIS_FILE_SIZE + 1
            )
        );
    }
}
