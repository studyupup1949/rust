mod client;
mod server;
mod types;

use client::GitHubClient;
use rmcp::{ServiceExt, transport::stdio};
use server::GitHubServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = std::env::var("GITHUB_TOKEN")
        .or_else(|_| std::env::var("GITHUB_PERSONAL_ACCESS_TOKEN"))
        .unwrap_or_default();
    let client = Arc::new(GitHubClient::new(token));
    let server = GitHubServer { client };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
