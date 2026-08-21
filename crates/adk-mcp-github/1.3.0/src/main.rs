mod client;
mod git_local;
mod server;
mod types;

use client::GitHubClient;
use rmcp::{ServiceExt, transport::stdio};
use server::GitHubServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse().unwrap())).init();
    let token = std::env::var("GITHUB_TOKEN")
        .or_else(|_| std::env::var("GITHUB_PERSONAL_ACCESS_TOKEN"))
        .unwrap_or_default();
    let client = Arc::new(GitHubClient::new(token));
    let server = GitHubServer { client };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
