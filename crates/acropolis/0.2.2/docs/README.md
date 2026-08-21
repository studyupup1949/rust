# UNDER CONSTRUCTION

Acropolis is an under-construction Rust Cardano node project. It is not production-ready and must not be used to manage funds, produce blocks, submit transactions, or operate a blockchain node.

## Safety Status

- Default commands are local and non-mutating.
- Mainnet contact is blocked.
- Production ledger, sync, forging, storage, and peer-to-peer behavior are incomplete.
- Live testnet behavior is limited to explicitly enabled, bounded probe commands.
- No credentials, keys, tokens, user sessions, or local machine paths are included in the published package.

## Package Identity

- Name: Acropolis
- Purpose: under-construction Cardano node and blockchain tooling
- Authors: Trevor Knott and Knott Dynamics
- Rust crate: `acropolis`

## Construction Boundaries

- Local planning and reporting are available now.
- Production consensus and ledger guarantees are not available.
- Mainnet connections are disabled.
- Public-testnet probes require explicit authorization and strict limits.
- The handshake probe writes one proposal frame and reads one bounded response frame.
- No background network crawl or unbounded synchronization is enabled.

## Published Archive

The crate archive is limited to Cargo metadata, licenses, public documentation, implementation files, examples, and tests. Development logs, machine paths, temporary files, credentials, keys, tokens, and user sessions are excluded.

Review `README.md` for safe local commands and current limitations.
