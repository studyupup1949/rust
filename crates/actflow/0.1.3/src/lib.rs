//! # Actflow
//!
//! Actflow is a lightweight, event-driven workflow engine written in Rust.
//! It is designed to be embedded in applications to orchestrate complex business logic.
//!
//! ## Core Features
//!
//! - **Event-Driven Architecture**: Built on a robust event bus for high decoupling and scalability
//! - **Async Execution**: Powered by `tokio` for high-concurrency workflow execution
//! - **Pluggable Storage**: Supports in-memory storage (testing) and PostgreSQL (production)
//! - **Flexible Workflow Definition**: Define workflows using JSON with various node types
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use actflow::{EngineBuilder, WorkflowModel};
//!
//! let engine = EngineBuilder::new().build().unwrap();
//! engine.launch();
//!
//! // Deploy and run workflow
//! let workflow = WorkflowModel::from_json(json_str)?;
//! engine.deploy(&workflow)?;
//! let process = engine.build_process(&workflow.id)?;
//! engine.run_process(process)?;
//! ```

mod builder;
mod common;
mod dispatcher;
mod engine;
mod error;
mod events;
mod model;
mod runtime;
mod utils;
mod workflow;

use std::sync::{Arc, RwLock};

pub use builder::EngineBuilder;
pub use engine::Engine;
pub use error::ActflowError;
pub use model::*;
pub use runtime::{ChannelEvent, ChannelOptions};

/// Result type alias for Actflow operations.
pub type Result<T> = std::result::Result<T, ActflowError>;

/// Thread-safe shared lock wrapper using Arc<RwLock<T>>.
pub(crate) type ShareLock<T> = Arc<RwLock<T>>;
