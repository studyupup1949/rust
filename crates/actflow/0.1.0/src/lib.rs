mod builder;
mod common;
mod config;
mod dispatcher;
mod engine;
mod error;
mod events;
mod model;
mod runtime;
mod store;
mod utils;
mod workflow;

use std::sync::{Arc, RwLock};

pub use builder::EngineBuilder;
pub use config::*;
pub use engine::Engine;
pub use error::ActflowError;
pub use model::*;
pub use runtime::{ChannelEvent, ChannelOptions};

pub type Result<T> = std::result::Result<T, ActflowError>;

pub(crate) type ShareLock<T> = Arc<RwLock<T>>;
