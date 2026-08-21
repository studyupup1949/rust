use acropolis::config::network_profiles;
use acropolis::{Node, NodeConfig};
use std::process::Command;

#[test]
fn default_startup_plan_text_matches_local_golden() {
    let node = Node::new(NodeConfig::default()).unwrap();

    assert_eq!(
        node.startup_plan().render_text(),
        concat!(
            "Acropolis offline startup plan\n",
            "network=local magic=2\n",
            "- safety [open]: paths=false state_mutation=false\n",
            "- production-readiness [closed]: production_ready=false network_protocols_blocked=true ledger_parity_blocked=true genesis_hashing_blocked=true path_review_blocked=true agentic_runtime_blocked=true mainnet_contact_allowed=false\n",
            "- config-assets [closed]: config=./config/local.json topology=none reads=false\n",
            "- genesis-lifecycle [closed]: cardano_plan=local genesis_reads=false raw_hashing=opt-in byron_canonical_hash=false production_genesis=false remote_fetch=false\n",
            "- topology-lifecycle [closed]: cardano_plan=local topology_reads=false dns=false peer_snapshot_reads=false bootstrap_peers=false ledger_peers=false dials=false\n",
            "- storage [closed]: planned store_dir=.acropolis data_mode=core (not opened by offline plan)\n",
            "- storage-lifecycle [closed]: migration=false fade_enabled=false fade_secs=3600 archive_fallback=false\n",
            "- bootstrap-lifecycle [closed]: remote_fetch=false mithril=false snapshot_load=false state_mutation=false paths=false\n",
            "- local-socket [closed]: planned path=acropolis.socket activation=false\n",
            "- events [open]: in-memory local subscribers only\n",
            "- observability-lifecycle [closed]: collector=in-memory metrics_server=false tracing_export=false artifacts=false paths=false\n",
            "- chain [open]: current block=0\n",
            "- mempool [open]: 0 items waiting\n",
            "- mempool-lifecycle [open]: capacity_bytes=67108864 semantic_rules=local tx_submission=false mutation=false\n",
            "- ledger [open]: local fixture era catalog=3 ranges available; production parity pending\n",
            "- ledger-lifecycle [closed]: production_rules=false genesis_hooks=false checkpoints=false hard_fork_triggers=false\n",
            "- sync-lifecycle [closed]: mode=local-fixture chain_sync=false block_fetch=false paths=false\n",
            "- block-producer [closed]: enabled=false tree_key=false producer_key=false certificate=false reads=false signing=false\n",
            "- peers [closed]: targets known=100 established=20 active=5 (no paths opened)\n",
            "- peer-lifecycle [closed]: topology_sources=local-fixture ledger_peers=false peer_sharing=false governor=false warm_hot_cold=false dials=false\n",
            "- mini-protocols [closed]: handshake=bounded-probe-only chain_sync=false block_fetch=false tx_submission=false keepalive=false peer_sharing=false\n",
            "- network-outer [closed]: 0.0.0.0:3001 planned but not opened\n",
            "- network-inner [closed]: 127.0.0.1:3002 planned but not opened\n",
            "- metrics [closed]: 127.0.0.1:12798 planned but not opened\n",
            "- transaction-interface [closed]: 127.0.0.1:0 planned but not opened\n",
            "- state-interface [closed]: 127.0.0.1:0 planned but not opened\n",
            "- batch-interface [closed]: 127.0.0.1:0 planned but not opened\n",
            "- archive-interface [closed]: 127.0.0.1:0 planned but not opened\n",
            "- interfaces [closed]: all interfaces remain closed by offline plan\n",
            "- api-lifecycle [closed]: local_tx_submission=false local_state_query=false local_tx_monitor=false blockfrost=false mesh=false utxorpc=false\n",
            "lifecycle local_only=true startup_steps=30 shutdown_steps=30 reload_hooks=config,topology,tracing graceful_shutdown_secs=30 signal_handlers_installed=false path_activation_allowed=false\n",
            "reload_preflight local_only=true requested_hooks=3 accepted_hooks=3 rejected_hooks=0 signal_handlers_installed=false path_activation_allowed=false\n",
            "shutdown_preflight local_only=true shutdown_steps=30 graceful_shutdown_secs=30 force_after_secs=30 signal_handlers_installed=false path_activation_allowed=false\n",
        )
    );
}

#[test]
fn offline_network_profile_table_matches_local_golden() {
    let rows = network_profiles()
        .iter()
        .map(|profile| {
            let runtime = profile.runtime_params().unwrap();
            format!(
                "{} {} {:?} {} {} {} {} {} {} {} {}",
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
                runtime.first_light_window_slots,
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        rows,
        vec![
            "mainnet 764824073 Public 3001 conway 1000 432000 2160 5/100 opt-in 129600",
            "preprod 1 Public 3001 conway 1000 432000 2160 5/100 opt-in 129600",
            "preview 2 Public 3001 conway 1000 86400 432 5/100 opt-in 25920",
            "local 2 Local 3001 local-conway 1000 1000 10 1/1 local 30",
            "staging 4 Local 3001 local-conway 1000 1000 10 1/1 local 30",
        ]
    );
}

#[test]
fn network_profiles_alias_matches_networks_report() {
    let primary = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("networks")
        .output()
        .unwrap();
    let alias = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("network-profiles")
        .output()
        .unwrap();

    assert!(primary.status.success());
    assert!(primary.stderr.is_empty());
    assert!(alias.status.success());
    assert!(alias.stderr.is_empty());
    assert_eq!(alias.stdout, primary.stdout);
}

#[test]
fn status_cli_reports_local_progress_without_runtime_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("status")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        concat!(
            "Acropolis local status\n",
            "local_only=true paths_opened=false dials=false remote_fetch=false state_mutation=false live_protocol_send=false\n",
            "startup network=local magic=2 paths_allowed=false state_mutation_allowed=false data_mode=core\n",
            "startup_plan local_only=true network=local magic=2 steps=30 open_steps=6 closed_steps=24 paths_opened=false state_mutation=false\n",
            "startup_categories local_only=true format=total/open block_production=1/0 config=3/0 interfaces=3/0 ledger=2/1 local_state=5/4 network=10/0 safety=2/1 storage=3/0 sync=1/0 paths_opened=false state_mutation=false\n",
            "lifecycle local_only=true startup_steps=30 shutdown_steps=30 reload_hooks=config,topology,tracing graceful_shutdown_secs=30 signal_handlers_installed=false path_activation_allowed=false\n",
            "reload_preflight local_only=true requested_hooks=3 accepted_hooks=3 rejected_hooks=0 signal_handlers_installed=false path_activation_allowed=false\n",
            "shutdown_preflight local_only=true shutdown_steps=30 graceful_shutdown_secs=30 force_after_secs=30 signal_handlers_installed=false path_activation_allowed=false\n",
            "profiles total=5 public=3 local=2\n",
            "testnet_readiness tcp_probe_available=true handshake_probe_available=true offline_conformance_complete=true public_testnets=2 profiles=preprod,preview profile_ports=preprod:3001,preview:3001 bounded_live_contact_allowed=true blockers=1 actions=1\n",
            "testnet_conformance public_testnets=2 passed_profiles=2 offline_complete=true live_ready=false profiles=preprod,preview profile_ports=preprod:3001,preview:3001 blockers=3 actions=3\n",
            "live_capable_path command=testnet-probe tcp_only=true requires_allow_live_testnet=true protocol_command_available=false protocol_bytes_allowed=false mainnet_allowed=false\n",
            "tcp_probe_gate command=testnet-probe requires_allow_live_testnet=true peer_literal_ip=true public_testnet_only=true default_port_only=true max_timeout_secs=5 retries=0 protocol_bytes_allowed=false mainnet_allowed=false\n",
            "live_capable_path command=testnet-handshake-probe tcp_only=false requires_allow_live_testnet=true protocol_command_available=true protocol_bytes_allowed=true mainnet_allowed=false\n",
            "handshake_probe_gate command=testnet-handshake-probe requires_allow_live_testnet=true peer_literal_ip=true public_testnet_only=true default_port_only=true versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true max_timeout_secs=5 dials=1 retries=0 write_frames=1 read_frames=1 max_response_bytes=1024 mainnet_allowed=false\n",
            "agentic_safety safety_gate=clear blockers=0 actions=0 live_agents_running=false provider_calls=false sidecar_spawn=false\n",
            "production_ready=false mainnet_contact_allowed=false live_protocol_send=false\n",
            "production_blockers network_protocols=true ledger_parity=true genesis_hashing=true path_review=true agentic_runtime=true\n",
        )
    );
}

#[test]
fn status_aliases_match_local_progress_report() {
    let status = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("status")
        .output()
        .unwrap();

    assert!(status.status.success());
    assert!(status.stderr.is_empty());

    for command in ["progress", "local-status"] {
        let alias = Command::new(env!("CARGO_BIN_EXE_acropolis"))
            .arg(command)
            .output()
            .unwrap();

        assert!(alias.status.success());
        assert!(alias.stderr.is_empty());
        assert_eq!(alias.stdout, status.stdout);
    }
}

#[test]
fn status_cli_ignores_live_path_and_network_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args([
            "status",
            "--network",
            "mainnet",
            "--allow-live-testnet",
            "--testnet-peer",
            "8.8.8.8:3001",
            "--allow-paths",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("local_only=true"));
    assert!(text.contains("paths_opened=false"));
    assert!(text.contains("dials=false"));
    assert!(text.contains("startup network=local"));
    assert!(text.contains("bounded_live_contact_allowed=true"));
    assert!(text.contains("protocol_bytes_allowed=false"));
    assert!(text.contains("mainnet_contact_allowed=false"));
    assert!(text.contains("live_protocol_send=false"));
    assert!(text.contains("production_blockers network_protocols=true"));
}

#[test]
fn testnet_plan_cli_reports_dry_run_safety_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args(["testnet-plan", "--network", "testnet"])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        concat!(
            "bounded testnet dry-run plan\n",
            "network=preprod magic=1 live_contact=false paths_opened=false dials=false remote_fetch=false state_mutation=false\n",
            "request blocks=1 slots=100 bytes=1024\n",
            "limits max_blocks=32 max_slots=2000 max_bytes=8388608 timeout_secs=30 temp_bytes=33554432\n",
            "remaining blocks=31 slots=1900 bytes=8387584 temp_bytes=33553408\n",
            "live_readiness tcp_probe_available=true handshake_sketch_available=true offline_conformance_complete=true public_testnet_profiles=2 public_testnet_profile_names=preprod,preview public_testnet_profile_ports=preprod:3001,preview:3001 command_available=true path_review_complete=true conformance_complete=false live_contact_allowed=true blockers=full testnet conformance harness is not complete\n",
            "live_readiness_actions actions=complete full testnet conformance harness live_contact_allowed=true\n",
            "live_capable_path command=testnet-probe tcp_only=true requires_allow_live_testnet=true protocol_command_available=false protocol_bytes_allowed=false mainnet_allowed=false\n",
            "tcp_probe_gate command=testnet-probe requires_allow_live_testnet=true peer_literal_ip=true public_testnet_only=true default_port_only=true max_timeout_secs=5 retries=0 protocol_bytes_allowed=false mainnet_allowed=false\n",
            "live_capable_path command=testnet-handshake-probe tcp_only=false requires_allow_live_testnet=true protocol_command_available=true protocol_bytes_allowed=true mainnet_allowed=false\n",
            "handshake_probe_gate command=testnet-handshake-probe requires_allow_live_testnet=true peer_literal_ip=true public_testnet_only=true default_port_only=true versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true max_timeout_secs=5 dials=1 retries=0 write_frames=1 read_frames=1 max_response_bytes=1024 mainnet_allowed=false\n",
            "handshake_sketch versions=1 bytes=14 production_compatible=false live_protocol_send=false\n",
            "handshake_proposal_protocol_vector protocol_id=0 message_type=propose_versions versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true bytes=49 production_ready=false mux_framed=false live_protocol_send=false\n",
            "handshake_mux_protocol_vector header_bytes=8 frame_bytes=57 payload_bytes=49 timestamp=0 protocol_id=0 response=false production_ready=false live_protocol_send=false\n",
            "handshake_state_machine states=3 transitions=4 timeout_states=2 production_ready=false live_integrated=false live_protocol_send=false\n",
            "handshake_transcript local_only=true frames=2 total_bytes=71 request_bytes=57 response_bytes=14 accepted_version=10 leios_overlay_min_version=15 leios_overlay_negotiated=false production_ready=false live_integrated=false live_protocol_send=false\n",
            "handshake_replay local_only=true frames=2 request_frames=1 response_frames=1 stream_bytes=71 final_state=done accepted_version=10 leios_overlay_min_version=15 leios_overlay_negotiated=false production_ready=false live_integrated=false live_protocol_send=false\n",
            "handshake_refusal_replay local_only=true frames=2 request_frames=1 response_frames=1 stream_bytes=74 final_state=done refusal_reason=version_mismatch supported_versions=7,8,9,10 supported_min_version=7 supported_max_version=10 leios_overlay_min_version=15 supported_leios_overlay_capable=false production_ready=false live_integrated=false live_protocol_send=false\n",
            "handshake_harness local_only=true scenario=accept_protocol_vector states=3 messages=2 proposal_frame_bytes=57 response_frame_bytes=14 outcome=accepted accepted_version=10 leios_overlay_min_version=15 leios_overlay_negotiated=false production_ready=false live_integrated=false live_protocol_send=false\n",
            "handshake_timeout_protocol_vector state=confirm agency=server timeout_secs=10 elapsed_secs=10 timed_out=true production_ready=false live_integrated=false live_protocol_send=false\n",
            "handshake_error_protocol_vectors local_only=true versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true cases=3 matched=3 kinds=wrong_protocol_id,non_response_frame,malformed_cbor production_ready=false live_integrated=false live_protocol_send=false\n",
            "handshake_conformance local_only=true versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true offline_checks=9 passed=9 offline_complete=true live_ready=false blockers=live mux loop is not integrated;network path review is incomplete;full protocol conformance is incomplete production_ready=false live_integrated=false live_protocol_send=false\n",
            "testnet_conformance_matrix local_only=true versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true public_testnets=2 passed_profiles=2 offline_complete=true live_ready=false profiles=preprod,preview profile_ports=preprod:3001,preview:3001 blockers=live mux loop is not integrated;network path review is incomplete;full protocol conformance is incomplete production_ready=false live_integrated=false live_protocol_send=false\n",
            "testnet_conformance_actions actions=integrate reviewed live mux loop;complete network path review;complete full protocol conformance harness live_protocol_send=false\n",
        )
    );
}

#[test]
fn testnet_check_plan_alias_matches_dry_run_report() {
    let primary = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args(["testnet-plan", "--network", "testnet"])
        .output()
        .unwrap();
    let alias = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args(["testnet-check-plan", "--network", "testnet"])
        .output()
        .unwrap();

    assert!(primary.status.success());
    assert!(primary.stderr.is_empty());
    assert!(alias.status.success());
    assert!(alias.stderr.is_empty());
    assert_eq!(alias.stdout, primary.stdout);
}

#[test]
fn testnet_conformance_cli_reports_offline_matrix_without_runtime_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("testnet-conformance")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        concat!(
            "offline public testnet conformance matrix\n",
            "local_only=true live_protocol_send=false\n",
            "public_testnets=2 passed_profiles=2 offline_complete=true live_ready=false profiles=preprod,preview profile_ports=preprod:3001,preview:3001 versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true\n",
            "blockers=live mux loop is not integrated;network path review is incomplete;full protocol conformance is incomplete\n",
            "actions=integrate reviewed live mux loop;complete network path review;complete full protocol conformance harness\n",
            "live_capable_path command=testnet-probe tcp_only=true requires_allow_live_testnet=true protocol_command_available=false protocol_bytes_allowed=false mainnet_allowed=false\n",
            "tcp_probe_gate command=testnet-probe requires_allow_live_testnet=true peer_literal_ip=true public_testnet_only=true default_port_only=true max_timeout_secs=5 retries=0 protocol_bytes_allowed=false mainnet_allowed=false\n",
            "live_capable_path command=testnet-handshake-probe tcp_only=false requires_allow_live_testnet=true protocol_command_available=true protocol_bytes_allowed=true mainnet_allowed=false\n",
            "handshake_probe_gate command=testnet-handshake-probe requires_allow_live_testnet=true peer_literal_ip=true public_testnet_only=true default_port_only=true versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true max_timeout_secs=5 dials=1 retries=0 write_frames=1 read_frames=1 max_response_bytes=1024 mainnet_allowed=false\n",
            "profile=preprod magic=1 port=3001 versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true offline_checks=9 passed=9 offline_complete=true live_ready=false actions=integrate reviewed live mux loop;complete network path review;complete full protocol conformance harness production_ready=false live_integrated=false\n",
            "profile=preview magic=2 port=3001 versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true offline_checks=9 passed=9 offline_complete=true live_ready=false actions=integrate reviewed live mux loop;complete network path review;complete full protocol conformance harness production_ready=false live_integrated=false\n",
        )
    );
}

#[test]
fn testnet_conformance_alias_matches_offline_matrix_report() {
    let primary = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("testnet-conformance")
        .output()
        .unwrap();
    let alias = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("testnet-conformance-plan")
        .output()
        .unwrap();

    assert!(primary.status.success());
    assert!(primary.stderr.is_empty());
    assert!(alias.status.success());
    assert!(alias.stderr.is_empty());
    assert_eq!(alias.stdout, primary.stdout);
}

#[test]
fn testnet_readiness_cli_reports_blockers_without_runtime_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("testnet-readiness")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        concat!(
            "offline testnet live-readiness report\n",
            "local_only=true paths_opened=false dials=false remote_fetch=false state_mutation=false live_protocol_send=false\n",
            "tcp_probe_available=true handshake_sketch_available=true offline_conformance_complete=true public_testnet_profiles=2\n",
            "public_testnet_profile_names=preprod,preview\n",
            "public_testnet_profile_ports=preprod:3001,preview:3001\n",
            "command_available=true path_review_complete=true conformance_complete=false live_contact_allowed=true\n",
            "live_capable_path command=testnet-probe tcp_only=true requires_allow_live_testnet=true protocol_command_available=false protocol_bytes_allowed=false mainnet_allowed=false\n",
            "tcp_probe_gate command=testnet-probe requires_allow_live_testnet=true peer_literal_ip=true public_testnet_only=true default_port_only=true max_timeout_secs=5 retries=0 protocol_bytes_allowed=false mainnet_allowed=false\n",
            "live_capable_path command=testnet-handshake-probe tcp_only=false requires_allow_live_testnet=true protocol_command_available=true protocol_bytes_allowed=true mainnet_allowed=false\n",
            "handshake_probe_gate command=testnet-handshake-probe requires_allow_live_testnet=true peer_literal_ip=true public_testnet_only=true default_port_only=true versions=9 min_version=7 max_version=15 leios_overlay_min_version=15 leios_overlay_capable=true max_timeout_secs=5 dials=1 retries=0 write_frames=1 read_frames=1 max_response_bytes=1024 mainnet_allowed=false\n",
            "blockers=full testnet conformance harness is not complete\n",
            "actions=complete full testnet conformance harness\n",
        )
    );
}

#[test]
fn testnet_readiness_alias_matches_blocker_report() {
    let primary = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("testnet-readiness")
        .output()
        .unwrap();
    let alias = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("testnet-readiness-plan")
        .output()
        .unwrap();

    assert!(primary.status.success());
    assert!(primary.stderr.is_empty());
    assert!(alias.status.success());
    assert!(alias.stderr.is_empty());
    assert_eq!(alias.stdout, primary.stdout);
}

#[test]
fn testnet_readiness_cli_ignores_live_and_path_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args([
            "testnet-readiness",
            "--allow-live-testnet",
            "--testnet-peer",
            "8.8.8.8:3001",
            "--allow-paths",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("local_only=true"));
    assert!(text.contains("paths_opened=false"));
    assert!(text.contains("dials=false"));
    assert!(text.contains("live_protocol_send=false"));
    assert!(text.contains("requires_allow_live_testnet=true"));
    assert!(text.contains("protocol_bytes_allowed=false"));
    assert!(text.contains("mainnet_allowed=false"));
}

#[test]
fn testnet_probe_cli_requires_live_opt_in_before_dialing() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args([
            "testnet-probe",
            "--network",
            "preview",
            "--testnet-peer",
            "8.8.8.8:3001",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "testnet contact blocked: live testnet TCP probe requires --allow-live-testnet\n"
    );
}

#[test]
fn testnet_tcp_probe_alias_requires_live_opt_in_before_dialing() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args([
            "testnet-tcp-probe",
            "--network",
            "preview",
            "--testnet-peer",
            "8.8.8.8:3001",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "testnet contact blocked: live testnet TCP probe requires --allow-live-testnet\n"
    );
}

#[test]
fn testnet_handshake_probe_cli_requires_live_opt_in_before_dialing() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args([
            "testnet-handshake-probe",
            "--network",
            "preview",
            "--testnet-peer",
            "8.8.8.8:3001",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "testnet contact blocked: live testnet handshake probe requires --allow-live-testnet\n"
    );
}

#[test]
fn testnet_handshake_probe_cli_blocks_mainnet_before_dialing() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args([
            "testnet-handshake-probe",
            "--network",
            "mainnet",
            "--testnet-peer",
            "8.8.8.8:3001",
            "--allow-live-testnet",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "testnet contact blocked: mainnet contact is blocked\n"
    );
}

#[test]
fn agent_flow_cli_reports_local_snapshot_without_runtime_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("agent-flow")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Agentic flow snapshot"));
    assert!(text.contains("local_only=true"));
    assert!(text.contains("live_agents_running=false"));
    assert!(text.contains("safety_gate=clear blockers=0"));
    assert!(text.contains("safety_actions=none"));
}

#[test]
fn agentic_flow_alias_matches_local_snapshot_report() {
    let primary = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("agent-flow")
        .output()
        .unwrap();
    let alias = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("agentic-flow")
        .output()
        .unwrap();

    assert!(primary.status.success());
    assert!(primary.stderr.is_empty());
    assert!(alias.status.success());
    assert!(alias.stderr.is_empty());
    assert_eq!(alias.stdout, primary.stdout);
}

#[test]
fn agent_safety_cli_reports_clear_gate_without_runtime_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("agent-safety")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        concat!(
            "Agentic safety gate\n",
            "safety_gate=clear blockers=0\n",
            "local_only=true sidecar_spawn=false provider_calls=false live_agents_running=false agent_runtime_state_writes=false\n",
            "loop_diagnostics_clear=true loop_replay_clear=true\n",
            "blockers: none\n",
            "actions: none",
        )
    );
}

#[test]
fn agentic_safety_alias_matches_clear_gate_report() {
    let primary = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("agent-safety")
        .output()
        .unwrap();
    let alias = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("agentic-safety")
        .output()
        .unwrap();

    assert!(primary.status.success());
    assert!(primary.stderr.is_empty());
    assert!(alias.status.success());
    assert!(alias.stderr.is_empty());
    assert_eq!(alias.stdout, primary.stdout);
}

#[test]
fn cardano_config_plan_alias_requires_local_config_path() {
    let primary = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("cardano-plan")
        .output()
        .unwrap();
    let alias = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .arg("cardano-config-plan")
        .output()
        .unwrap();

    assert!(!primary.status.success());
    assert!(primary.stdout.is_empty());
    assert!(!alias.status.success());
    assert!(alias.stdout.is_empty());
    assert_eq!(alias.stderr, primary.stderr);
    assert_eq!(
        String::from_utf8(alias.stderr).unwrap(),
        "cardano-plan requires --cardano-config <path>\n"
    );
}

#[test]
fn testnet_plan_cli_blocks_mainnet() {
    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args(["testnet-plan", "--network", "mainnet"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "testnet contact blocked: mainnet contact is blocked\n"
    );
}

#[test]
fn cardano_plan_cli_reports_local_config_and_topology_golden() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "acropolis-cardano-plan-golden-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&dir).unwrap();
    let config_path = dir.join("config.json");
    let topology_path = dir.join("topology.json");
    let peer_snapshot_path = dir.join("peer-snapshot.json");
    std::fs::write(
        &config_path,
        r#"{
          "ApplicationName": "acropolis",
          "ApplicationVersion": 0,
          "CheckpointsFile": "checkpoints.json",
          "CheckpointsFileHash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          "ConsensusMode": "PraosMode",
          "DijkstraGenesisFile": "dijkstra-genesis.json",
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
          "RequiresNetworkMagic": "RequiresMagic",
          "ShelleyGenesisFile": "shelley-genesis.json",
          "ShelleyGenesisHash": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
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
              "backends": ["EKGBackend", "Forwarder", "PrometheusSimple suffix 127.0.0.1 12798"],
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
    std::fs::write(
        &topology_path,
        r#"{
          "bootstrapPeers": [
            { "address": "boot.local", "port": 3001 }
          ],
          "localRoots": [
            {
              "accessPoints": [ { "address": "alpha.local", "port": 3001 } ],
              "advertise": true,
              "trustable": true,
              "valency": 1
            }
          ],
          "peerSnapshotFile": "peers.json",
          "publicRoots": [
            { "accessPoints": [], "advertise": false, "valency": 1 }
          ],
          "useLedgerAfterSlot": 42
        }"#,
    )
    .unwrap();
    std::fs::write(
        &peer_snapshot_path,
        r#"{
          "NetworkMagic": 1,
          "NodeToClientVersion": 23,
          "Point": {
            "blockPointHash": "abcdef",
            "blockPointSlot": 99
          },
          "bigLedgerPools": [
            {
              "relativeStake": 0.6,
              "accumulatedStake": 0.6,
              "relays": [ { "address": "big.local", "port": 3001 } ]
            }
          ],
          "ledgerPools": [
            {
              "relativeStake": 0.4,
              "accumulatedStake": 1.0,
              "relays": [ { "address": "regular.local", "port": 3001 } ]
            }
          ]
        }"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args([
            "cardano-plan",
            "--network",
            "preprod",
            "--cardano-config",
            config_path.to_str().unwrap(),
            "--cardano-topology",
            topology_path.to_str().unwrap(),
            "--cardano-peer-snapshot",
            peer_snapshot_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let _ = std::fs::remove_file(&peer_snapshot_path);
    let _ = std::fs::remove_file(&topology_path);
    let _ = std::fs::remove_file(&config_path);
    let _ = std::fs::remove_dir(&dir);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            concat!(
                "Cardano local config plan\n",
                "config={} paths_opened=false genesis_files_read=false state_mutation=false\n",
                "genesis_hash_mode=caller-digest-only\n",
                "socket path=acropolis.socket source=default activation=false bind=false node_to_client=false stale_cleanup=false paths_opened=false\n",
                "protocol=Cardano consensus=PraosMode requires_network_magic=RequiresMagic configured=3 complete=true\n",
                "protocol_features experimental_hard_forks=false experimental_protocols=true configured=2 complete=true\n",
                "application name=acropolis version=0 max_concurrency_deadline=4 configured=3 complete=true\n",
                "test_hard_forks shelley_epoch=0 allegra_epoch=0 mary_epoch=0 alonzo_epoch=0 configured=4 complete=true\n",
                "metadata max_known_major_protocol_version=2 min_node_version=10.7.0\n",
                "ledger_db backend=V2InMemory snapshots=2 query_batch_size=100000 snapshot_interval=4320 configured=4 complete=true\n",
                "p2p_governor_deadline root_peers=14 known_peers=13 established_peers=11 active_peers=9 known_big_ledger_peers=12 established_big_ledger_peers=10 active_big_ledger_peers=8 configured=7 complete=true\n",
                "p2p_governor_sync root_peers=7 known_peers=6 established_peers=4 active_peers=2 known_big_ledger_peers=5 established_big_ledger_peers=3 active_big_ledger_peers=1 min_big_ledger_peers_for_trusted_state=15 configured=7 complete=true\n",
                "mempool_runtime timeout_soft=1.0 timeout_hard=1.5 timeout_capacity=5.0 timeouts_configured=3 timeouts_complete=true\n",
                "byron_protocol last_known_block_version=3.0.0 pbft_signature_threshold=0.6 configured=4 complete=true\n",
                "checkpoints file=checkpoints.json expected_hash=yes configured=2 complete=true\n",
                "tracing log_metrics=true logging=true dispatcher=true min_severity=Critical trace_scalar_fields=4 runtime_configured=4 runtime_complete=true\n",
                "trace_options entries=2 severity_overrides=2 detail_overrides=1 frequency_limits=1 metrics_prefix=cardano.node.metrics. resource_frequency=1000 forwarder_conn_queue=64 forwarder_disconn_queue=128 forwarder_max_reconnect_delay=30\n",
                "trace_severities silence=0 debug=0 info=1 notice=1 warning=0 error=0 critical=0 other=0\n",
                "trace_arrays trace_backends=3 default_backends=1 default_scribes=1 setup_backends=1 setup_scribes=1\n",
                "trace_backend_sinks ekg=1 forwarder=1 prometheus=1 stdout=0 katip=0 other=0 prometheus_host=127.0.0.1 prometheus_port=12798\n",
                "legacy_backend_sinks default_ekg=1 default_katip=0 default_other=0 setup_ekg=0 setup_katip=1 setup_other=0\n",
                "legacy_scribe_sinks default_stdout=1 default_file=0 default_other=0 setup_stdout=1 setup_file=0 setup_other=0\n",
                "legacy_trace_flags total=2 enabled=1 disabled=1\n",
                "legacy_telemetry ekg_port=12788 prometheus_host=127.0.0.1 prometheus_port=12798 prometheus_items=2\n",
                "legacy_trace_config verbosity=NormalVerbosity rotation_keep_files=10 rotation_log_limit_bytes=5000000 rotation_max_age_hours=24 map_backends=2 map_backend_items=2 map_subtraces=1\n",
                "network_profile=preprod network_magic=1 requires_network_magic_expected=RequiresMagic actual=RequiresMagic status=match\n",
                "genesis_files=2\n",
                "genesis_era_coverage byron=false shelley=true alonzo=false conway=false dijkstra=true expected_hashes=1 missing_hashes=1\n",
                "genesis_digest_support raw_file_hash_supported=2 raw_file_hash_eras=Shelley,Dijkstra canonical_hash_blocked=0 canonical_hash_blocked_eras=none\n",
                "genesis_hash_summary matched=0 mismatched=0 missing_expected=1 missing_actual=1 all_matched=false\n",
                "genesis era=Shelley file=shelley-genesis.json expected_hash=yes actual_digest=no status=missing-actual\n",
                "genesis era=Dijkstra file=dijkstra-genesis.json expected_hash=no actual_digest=no status=missing-expected\n",
                "topology={} local_roots=1 public_roots=1 local_peers=1 public_peers=0 bootstrap_peers=1 unique_peers=2 duplicate_peer_entries=0 local_valency=1 public_valency=0 local_warm_valency=1 public_warm_valency=0 ledger_after_slot=42 peer_snapshot=peers.json\n",
                "topology_flags advertised_local_roots=1 advertised_public_roots=0 trustable_local_roots=1 empty_local_roots=0 empty_public_roots=1 peer_snapshot_configured=true\n",
                "topology_ledger ledger_peers_enabled=true peer_snapshot_usable_by_topology=true bootstrap_peers_configured=true\n",
                "topology_trust trustable_local_peers=1 trusted_peer_source_configured=true\n",
                "peer_snapshot={} network_magic=1 expected_network_magic=1 network_magic_status=match node_to_client_version=23 point_slot=99 point_hash_present=true priority_pools=1 pools=1 relays=2 validation=valid validation_error=none paths_opened=false dials=false peer_sharing=false peer_governor=false state_mutation=false\n",
                "peer_discovery entries=4 local_roots=1 public_roots=0 seed_peers=1 snapshot_relays=2 snapshot_validation=valid paths_opened=false dials=false peer_governor=false state_mutation=false\n",
                "peer_source_status bootstrap_peers=1 bootstrap_configured=true bootstrap_blocked=true ledger_peers_enabled=true ledger_after_slot=42 peer_snapshot_configured=true peer_snapshot_usable_by_topology=true snapshot_relays=2 ledger_candidates=2 snapshot_validation=valid ledger_peers_blocked=true paths_opened=false dials=false peer_governor=false state_mutation=false\n",
                "peer_root_status local_root_peers=1 public_root_peers=0 root_candidates=1 root_target=7 root_met=false root_deficit=6 seed_peers=1 snapshot_relays=2 target_source=p2p_sync paths_opened=false dials=false peer_governor=false state_mutation=false\n",
                "peer_sharing_plan advertise_candidates=1 local_advertise_candidates=1 public_advertise_candidates=0 trustable_candidates=1 peer_sharing=false paths_opened=false dials=false peer_governor=false state_mutation=false\n",
                "peer_lifecycle added=4 promoted=4 pruned=0 target_source=p2p_sync targets_known=6 targets_established=4 targets_active=2 paths_opened=false dials=false peer_governor=false state_mutation=false\n",
                "peer_target_status known=4 warm=4 hot=2 known_met=false established_met=true active_met=true known_deficit=2 established_deficit=0 active_deficit=0 target_source=p2p_sync paths_opened=false dials=false peer_governor=false state_mutation=false\n",
                "peer_big_ledger_availability priority_relays=1 regular_relays=1 snapshot_validation=valid known_target=5 established_target=3 active_target=1 min_trusted_state=15 known_candidates_met=false established_candidates_met=false active_candidates_met=true trusted_state_candidates_met=false known_deficit=4 established_deficit=2 active_deficit=0 trusted_state_deficit=14 target_source=p2p_sync paths_opened=false dials=false peer_governor=false state_mutation=false\n",
                "peer_connection warm=4 hot=2 open_paths=false blocked=true target_source=p2p_sync targets_known=6 targets_established=4 targets_active=2 dials=false peer_governor=false state_mutation=false\n",
            ),
            config_path.display(),
            topology_path.display(),
            peer_snapshot_path.display()
        )
    );
}

#[test]
fn cardano_plan_cli_reports_peer_snapshot_network_magic_mismatch_without_dialing() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "acropolis-cardano-peer-snapshot-magic-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&dir).unwrap();
    let config_path = dir.join("config.json");
    let peer_snapshot_path = dir.join("peer-snapshot.json");
    std::fs::write(
        &config_path,
        r#"{
          "RequiresNetworkMagic": "RequiresMagic",
          "ShelleyGenesisFile": "shelley-genesis.json"
        }"#,
    )
    .unwrap();
    std::fs::write(
        &peer_snapshot_path,
        r#"{
          "NetworkMagic": 2,
          "NodeToClientVersion": 23,
          "Point": {
            "blockPointHash": "abcdef",
            "blockPointSlot": 99
          },
          "bigLedgerPools": [
            {
              "relativeStake": 1.0,
              "accumulatedStake": 1.0,
              "relays": [ { "address": "relay.local", "port": 3001 } ]
            }
          ]
        }"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args([
            "cardano-plan",
            "--network",
            "preprod",
            "--cardano-config",
            config_path.to_str().unwrap(),
            "--cardano-peer-snapshot",
            peer_snapshot_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let _ = std::fs::remove_file(&peer_snapshot_path);
    let _ = std::fs::remove_file(&config_path);
    let _ = std::fs::remove_dir(&dir);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("topology=none\n"));
    assert!(stdout.contains(
        "network_magic=2 expected_network_magic=1 network_magic_status=mismatch node_to_client_version=23 point_slot=99 point_hash_present=true priority_pools=1 pools=0 relays=1 validation=invalid validation_error=network_mismatch paths_opened=false dials=false peer_sharing=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_discovery=none topology=false snapshot_validation=invalid paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_source_status bootstrap_peers=0 bootstrap_configured=false bootstrap_blocked=false ledger_peers_enabled=false ledger_after_slot=none peer_snapshot_configured=false peer_snapshot_usable_by_topology=false snapshot_relays=0 ledger_candidates=0 snapshot_validation=invalid ledger_peers_blocked=false paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_root_status local_root_peers=0 public_root_peers=0 root_candidates=0 root_target=0 root_met=true root_deficit=0 seed_peers=0 snapshot_relays=0 target_source=default paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_sharing_plan advertise_candidates=0 local_advertise_candidates=0 public_advertise_candidates=0 trustable_candidates=0 peer_sharing=false paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_lifecycle added=0 promoted=0 pruned=0 target_source=default targets_known=100 targets_established=20 targets_active=5 paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_target_status known=0 warm=0 hot=0 known_met=false established_met=false active_met=false known_deficit=100 established_deficit=20 active_deficit=5 target_source=default paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_big_ledger_availability priority_relays=0 regular_relays=0 snapshot_validation=invalid known_target=0 established_target=0 active_target=0 min_trusted_state=0 known_candidates_met=true established_candidates_met=true active_candidates_met=true trusted_state_candidates_met=true known_deficit=0 established_deficit=0 active_deficit=0 trusted_state_deficit=0 target_source=default paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_connection warm=0 hot=0 open_paths=false blocked=false target_source=default targets_known=100 targets_established=20 targets_active=5 dials=false peer_governor=false state_mutation=false\n"
    ));
}

#[test]
fn cardano_plan_cli_reports_snapshot_only_peer_discovery_without_dialing() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "acropolis-cardano-peer-snapshot-only-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&dir).unwrap();
    let config_path = dir.join("config.json");
    let peer_snapshot_path = dir.join("peer-snapshot.json");
    std::fs::write(
        &config_path,
        r#"{
          "RequiresNetworkMagic": "RequiresMagic",
          "ShelleyGenesisFile": "shelley-genesis.json"
        }"#,
    )
    .unwrap();
    std::fs::write(
        &peer_snapshot_path,
        r#"{
          "NetworkMagic": 1,
          "NodeToClientVersion": 23,
          "Point": {
            "blockPointHash": "abcdef",
            "blockPointSlot": 99
          },
          "bigLedgerPools": [
            {
              "relativeStake": 1.0,
              "accumulatedStake": 1.0,
              "relays": [ { "address": "relay.local", "port": 3001 } ]
            }
          ]
        }"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args([
            "cardano-plan",
            "--network",
            "preprod",
            "--cardano-config",
            config_path.to_str().unwrap(),
            "--cardano-peer-snapshot",
            peer_snapshot_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    let _ = std::fs::remove_file(&peer_snapshot_path);
    let _ = std::fs::remove_file(&config_path);
    let _ = std::fs::remove_dir(&dir);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("topology=none\n"));
    assert!(stdout.contains(
        "network_magic=1 expected_network_magic=1 network_magic_status=match node_to_client_version=23 point_slot=99 point_hash_present=true priority_pools=1 pools=0 relays=1 validation=valid validation_error=none paths_opened=false dials=false peer_sharing=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_discovery entries=1 local_roots=0 public_roots=0 seed_peers=0 snapshot_relays=1 snapshot_validation=valid paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_source_status bootstrap_peers=0 bootstrap_configured=false bootstrap_blocked=false ledger_peers_enabled=false ledger_after_slot=none peer_snapshot_configured=false peer_snapshot_usable_by_topology=false snapshot_relays=1 ledger_candidates=0 snapshot_validation=valid ledger_peers_blocked=false paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_root_status local_root_peers=0 public_root_peers=0 root_candidates=0 root_target=0 root_met=true root_deficit=0 seed_peers=0 snapshot_relays=1 target_source=default paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_sharing_plan advertise_candidates=0 local_advertise_candidates=0 public_advertise_candidates=0 trustable_candidates=0 peer_sharing=false paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_lifecycle added=1 promoted=1 pruned=0 target_source=default targets_known=100 targets_established=20 targets_active=5 paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_target_status known=1 warm=1 hot=1 known_met=false established_met=false active_met=false known_deficit=99 established_deficit=19 active_deficit=4 target_source=default paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_big_ledger_availability priority_relays=1 regular_relays=0 snapshot_validation=valid known_target=0 established_target=0 active_target=0 min_trusted_state=0 known_candidates_met=true established_candidates_met=true active_candidates_met=true trusted_state_candidates_met=true known_deficit=0 established_deficit=0 active_deficit=0 trusted_state_deficit=0 target_source=default paths_opened=false dials=false peer_governor=false state_mutation=false\n"
    ));
    assert!(stdout.contains(
        "peer_connection warm=1 hot=1 open_paths=false blocked=true target_source=default targets_known=100 targets_established=20 targets_active=5 dials=false peer_governor=false state_mutation=false\n"
    ));
}

#[test]
fn cardano_plan_cli_reports_unsupported_byron_genesis_hashing() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "acropolis-cardano-byron-support-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&dir).unwrap();
    let config_path = dir.join("config.json");
    std::fs::write(
        &config_path,
        r#"{
          "ByronGenesisFile": "byron-genesis.json",
          "ByronGenesisHash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args([
            "cardano-plan",
            "--cardano-config",
            config_path.to_str().unwrap(),
            "--read-genesis-files",
        ])
        .output()
        .unwrap();

    let _ = std::fs::remove_file(&config_path);
    let _ = std::fs::remove_dir(&dir);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "genesis era=Byron file=byron-genesis.json expected_hash=yes actual_digest=no status=missing-actual\n"
    ));
    assert!(stdout.contains(
        "genesis_digest_support raw_file_hash_supported=0 raw_file_hash_eras=none canonical_hash_blocked=1 canonical_hash_blocked_eras=Byron\n"
    ));
    assert!(stdout.contains(
        "genesis_digest_support era=Byron raw_file_hash=false reason=canonical-hash-blocked\n"
    ));
}

#[test]
fn cardano_plan_cli_reports_socket_override_as_closed_preflight() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "acropolis-cardano-socket-preflight-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&dir).unwrap();
    let config_path = dir.join("config.json");
    std::fs::write(
        &config_path,
        r#"{ "ShelleyGenesisFile": "shelley-genesis.json" }"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_acropolis"))
        .args([
            "cardano-plan",
            "--cardano-config",
            config_path.to_str().unwrap(),
            "--socket",
            "cardano.socket",
        ])
        .output()
        .unwrap();

    let _ = std::fs::remove_file(&config_path);
    let _ = std::fs::remove_dir(&dir);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(
        "socket path=cardano.socket source=cli activation=false bind=false node_to_client=false stale_cleanup=false paths_opened=false\n"
    ));
    assert!(stdout.contains("topology=none\n"));
}
