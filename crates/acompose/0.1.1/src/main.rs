use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager,
    tower::{StreamableHttpServerConfig, StreamableHttpService},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use acompose::config::Config;
use acompose::mcp_server::ComposeMcpServer;
use acompose::orchestrator::Orchestrator;

#[derive(Parser, Debug)]
#[command(name = "acompose")]
#[command(about = "Spawn persistent Kimi agents via ACP and expose them through an MCP server.")]
struct Cli {
    /// Path to the TOML configuration file.
    #[arg(short, long, default_value = "acompose.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let config = Config::from_file(cli.config.clone()).context("loading config")?;
    let state_path = cli
        .config
        .parent()
        .map(|p| p.join("state.json"))
        .unwrap_or_else(|| PathBuf::from("state.json"));

    info!(sessions = config.sessions.len(), "starting acompose");

    let orchestrator = Arc::new(Orchestrator::new(config.kimi_binary.clone(), state_path)?);

    // Collect config session names to avoid duplicates.
    let config_names: HashSet<String> = config.sessions.iter().map(|s| s.name.clone()).collect();

    // Spawn configured sessions through the shared orchestrator.
    for session in &config.sessions {
        let orchestrator = Arc::clone(&orchestrator);
        let name = session.name.clone();
        let cwd = session.cwd.clone();
        let charter = session.charter.clone();
        let allowed_tool_kinds = session.allowed_tool_kinds.clone();

        tokio::spawn(async move {
            match orchestrator
                .create_session(&name, cwd, &charter, allowed_tool_kinds)
                .await
            {
                Ok(info) => {
                    info!(name = %info.name, session_id = %info.session_id, ?info.status, "session ready");
                }
                Err(e) => {
                    error!(name, error = %e, "failed to start session");
                }
            }
        });
    }

    // Resume any additional sessions that were persisted in state but not in config.
    match orchestrator.persisted_sessions() {
        Ok(persisted) => {
            for (name, session_state) in persisted {
                if config_names.contains(&name) {
                    continue;
                }
                let Some(cwd) = session_state.cwd else {
                    warn!(name, "skipping persisted session with missing cwd");
                    continue;
                };
                let charter = session_state.charter.unwrap_or_default();
                let orchestrator = Arc::clone(&orchestrator);
                tokio::spawn(async move {
                    match orchestrator.create_session(&name, cwd, &charter, vec![]).await {
                        Ok(info) => {
                            info!(name = %info.name, session_id = %info.session_id, ?info.status, "state session ready");
                        }
                        Err(e) => {
                            error!(name, error = %e, "failed to resume state session");
                        }
                    }
                });
            }
        }
        Err(e) => {
            warn!(error = %e, "failed to read persisted sessions");
        }
    }

    // Start the integrated MCP server if enabled.
    let mcp_handle = if config.mcp_server.enabled {
        let bind_address = config.mcp_server.bind_address.clone();
        let orchestrator = Arc::clone(&orchestrator);
        let mcp_ct = CancellationToken::new();
        let axum_ct = mcp_ct.clone();
        let config_ct = mcp_ct.clone();

        let handle = tokio::spawn(async move {
            info!(%bind_address, "starting MCP server");

            let service: StreamableHttpService<ComposeMcpServer, LocalSessionManager> =
                StreamableHttpService::new(
                    move || Ok(ComposeMcpServer::new(Arc::clone(&orchestrator))),
                    Default::default(),
                    StreamableHttpServerConfig::default()
                        .with_cancellation_token(config_ct.child_token()),
                );

            let app = axum::Router::new().nest_service("/mcp", service);
            let listener = tokio::net::TcpListener::bind(&bind_address)
                .await
                .context("binding MCP server")?;

            axum::serve(listener, app)
                .with_graceful_shutdown(async move { axum_ct.cancelled_owned().await })
                .await
                .context("running MCP server")?;

            Ok::<(), anyhow::Error>(())
        });

        Some((handle, mcp_ct))
    } else {
        None
    };

    // Wait for shutdown signal.
    tokio::signal::ctrl_c()
        .await
        .context("waiting for shutdown signal")?;
    info!("shutdown signal received, stopping acompose");

    if let Some((handle, ct)) = mcp_handle {
        ct.cancel();
        if let Err(e) = handle.await {
            error!(error = %e, "MCP server task panicked");
        }
    }

    orchestrator.shutdown().await;
    info!("acompose finished");
    Ok(())
}
