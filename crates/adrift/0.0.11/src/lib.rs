pub use adrift_core as core;
pub use adrift_core::anyhow;
pub use adrift_core::clap;
pub use adrift_core::commands;
pub use adrift_core::dotenv;
pub use adrift_core::jobs;
pub use adrift_core::log;
pub use adrift_core::rocket;
pub use adrift_core::serde;
pub use adrift_core::service_providers;
pub use adrift_core::templates;
pub use adrift_core::tokio;
pub use adrift_core::tracing_subscriber;
pub use adrift_macros as macros;
pub use once_cell;

pub use adrift_core::async_trait;
pub use adrift_core::jobs::Job;
pub use adrift_core::jobs::Queue;
pub use adrift_core::service_providers::ServiceProvider;
pub use adrift_core::templates::Template;
pub use adrift_core::Container;
pub use adrift_core::RoutesConfig;
pub use adrift_core::TemplateConfig;
pub use adrift_macros::main;

#[macro_export]
macro_rules! view {
    ($template:expr, { $($key:ident),* $(,)? }) => {
        {
            adrift::Template::render($template, adrift::templates::context!{ $($key),* })
        }
    };
}