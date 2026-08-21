# Acropolis

> **UNDER CONSTRUCTION:** Acropolis is not production-ready. Do not use it to manage funds, produce blocks, submit transactions, or operate a blockchain node.

Acropolis is a safe local-first Rust scaffold for Cardano node and `cardano-node` development. The project is authored by Trevor Knott and Knott Dynamics.

## Safety

- The default command prints a local startup plan.
- Mainnet contact is blocked.
- Default paths do not dial peers, fetch remote data, serve interfaces, or mutate node state.
- Production ledger, sync, forging, storage, and peer-to-peer behavior are incomplete.
- Live testnet behavior requires an explicit opt-in and remains strictly bounded.
- Published package contents are restricted by an explicit Cargo allowlist.

## Local Capabilities

- Startup and shutdown planning.
- Offline network profile reporting.
- Bounded local Cardano configuration, topology, and peer snapshot inspection.
- Deterministic Cardano network protocol vectors and in-memory conformance checks.
- Local observability and safety reports.
- An optional `tui` dashboard.

## Safe Commands

```bash
cargo fmt --check
cargo check --offline
cargo test --offline
cargo run --offline -- plan
cargo run --offline -- status
cargo run --offline -- networks
cargo run --offline -- testnet-plan --network testnet
cargo run --offline -- testnet-conformance
cargo run --offline -- testnet-readiness
cargo check --offline --features tui --examples
cargo test --offline --features tui
```

## Construction Preview

```bash
cargo install acropolis
acropolis status
```

This installs the local reporting scaffold only. It does not install a production Cardano node.

The `testnet-probe` and `testnet-handshake-probe` commands can open one explicitly authorized public-testnet connection. The handshake probe writes one Cardano proposal frame and reads one bounded response frame. Both commands reject mainnet, DNS hostnames, non-default ports, retries, and timeouts above five seconds.

## Publication

The crate is published as `acropolis` with these discovery terms: Acropolis, Cardano, Cardano node, `cardano-node`, blockchain, and node.

See `docs/README.md` for the construction and safety status.
