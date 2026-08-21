#![forbid(unsafe_code)]
#![doc = "# UNDER CONSTRUCTION\n\nAcropolis is a non-production Rust Cardano node scaffold."]

pub mod agentic;
pub mod app;
pub mod block_forging;
pub mod block_production;
pub mod bootstrap;
pub mod chain;
pub mod config;
pub mod events;
pub mod interfaces;
pub mod ledger;
pub mod mempool;
pub mod network;
pub mod observability;
pub mod peers;
pub mod scheduler;
pub mod storage;
pub mod sync;
pub mod topology;

#[cfg(feature = "tui")]
pub mod dashboard;

pub use app::{Node, NodeError, StartupPlan, StartupStep};
pub use config::NodeConfig;
