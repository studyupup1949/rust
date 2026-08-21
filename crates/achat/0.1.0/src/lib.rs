// SPDX-License-Identifier: Apache-2.0

//! `achat` — minimal LAN agent-to-agent messaging.
//!
//! Core building blocks for the `achat` CLI: protocol types, daemon logic,
//! peer discovery, transport, IPC, and storage.

pub mod daemon;
pub mod discovery;
pub mod ipc;
pub mod protocol;
pub mod storage;
pub mod transport;
pub mod util;
