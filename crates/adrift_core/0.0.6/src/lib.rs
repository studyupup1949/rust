pub use tokio;
pub mod commands;
pub use async_trait::async_trait;
pub use commands::Command;
pub use anyhow;
pub mod serde {
    pub use serde::*;
    pub use serde_json as json;
}
pub use clap;
pub use silhouette::facade::Container;
pub use rocket;
pub use rocket_dyn_templates as templates;
pub fn get_commands() -> Vec<Box<dyn Command>> {
    vec![
        Box::new(commands::inspire::Inspire),
        Box::new(commands::serve::Serve),
        Box::new(commands::make_command::MakeCommand),
        Box::new(commands::clean::Clean),
    ]
}
#[derive(Debug, Clone)]
pub struct Routes {
    pub inner: Vec<rocket::Route>,
}
