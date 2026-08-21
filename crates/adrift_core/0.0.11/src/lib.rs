use std::{collections::HashMap, sync::Arc};

use templates::tera::Function;
pub use tokio;
pub mod commands;
pub mod jobs;
pub mod service_providers;
pub use anyhow;
pub use async_trait::async_trait;
pub use commands::Command;
pub use tracing_subscriber;
pub mod serde {
    pub use serde::*;
    pub use serde_json as json;
}
pub use clap;
pub use dotenv;
pub use log;
pub use rocket;
pub use rocket_dyn_templates as templates;
pub use silhouette::facade::Container;

#[derive(Debug, Clone)]
pub struct RoutesConfig {
    pub inner: Vec<rocket::Route>,
}

#[derive(Clone)]
pub struct TemplateConfig {
    pub functions: HashMap<String, Arc<dyn Function>>,
}
