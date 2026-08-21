use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager,
    tower::{StreamableHttpServerConfig, StreamableHttpService},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use acompose::compositor::Compositor;
use acompose::mcp_server::ComposeMcpServer;
use acompose::server::Server;

#[derive(Parser, Debug)]
#[command(name = "acompose")]
#[command(about = "Spawn persistent agents via ACP and expose them through an MCP server.")]
struct Cli {
    /// Path to the TOML configuration file.
    #[arg(short, long, default_value = "acompose.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let main_cancel = CancellationToken::new();
    let (compositor, config, state) =
        Compositor::from_config_file(&cli.config, None, None, Some(main_cancel.clone()))
            .await
            .context("loading config")?;
    let compositor = Arc::new(compositor);

    // Start the integrated MCP server before spawning agents so they can discover
    // the acompose control-plane tools during their own initialization.
    let mcp_handle = if config.acompose_control_mcp.enabled {
        let bind_address = config.acompose_control_mcp.bind_address.clone();
        let compositor = Arc::clone(&compositor);
        let mcp_ct = main_cancel.child_token();
        let axum_ct = mcp_ct.clone();
        let config_ct = mcp_ct.clone();

        let service: StreamableHttpService<ComposeMcpServer, LocalSessionManager> =
            StreamableHttpService::new(
                move || Ok(ComposeMcpServer::new(Arc::clone(&compositor))),
                Arc::default(),
                StreamableHttpServerConfig::default()
                    .with_cancellation_token(config_ct.child_token()),
            );
        let app = axum::Router::new().nest_service("/mcp", service);

        // Bind synchronously so the port is guaranteed to be open before we spawn
        // any agents that need to connect to it.
        let listener = tokio::net::TcpListener::bind(&bind_address)
            .await
            .context("binding MCP server")?;
        info!(%bind_address, "MCP server listening");

        let handle = tokio::spawn(async move {
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

    // Spawn sessions now that the MCP control plane is ready.
    let infos = compositor
        .spawn_sessions_from_config(&config, &state)
        .await
        .context("spawning sessions from config")?;

    info!(sessions = infos.len(), "starting acompose");
    for info in &infos {
        info!(name = %info.name, session_id = %info.session_id, ?info.status, "session ready");
    }

    // Start the Compose WebSocket server if enabled.
    let server_handle = if config.server.enabled {
        let bind_address = config
            .server
            .bind_address
            .parse()
            .context("parsing server bind address")?;
        let server = Server::new(Arc::clone(&compositor), bind_address);
        let server_ct = main_cancel.child_token();
        let server_ct_clone = server_ct.clone();
        let handle = tokio::spawn(async move {
            info!(%bind_address, "starting Compose server");
            tokio::select! {
                result = server.run() => {
                    if let Err(e) = result {
                        error!(error = %e, "Compose server error");
                    }
                }
                () = server_ct_clone.cancelled_owned() => {
                    info!("Compose server cancelled");
                }
            }
            Ok::<(), anyhow::Error>(())
        });
        Some((handle, server_ct))
    } else {
        None
    };

    // Wait for shutdown signal.
    tokio::signal::ctrl_c()
        .await
        .context("waiting for shutdown signal")?;
    info!("shutdown signal received, stopping acompose");

    main_cancel.cancel();

    if let Some((handle, ct)) = mcp_handle {
        ct.cancel();
        if let Err(e) = handle.await {
            error!(error = %e, "MCP server task panicked");
        }
    }

    if let Some((handle, ct)) = server_handle {
        ct.cancel();
        if let Err(e) = handle.await {
            error!(error = %e, "Compose server task panicked");
        }
    }

    compositor.shutdown().await;
    info!("acompose finished");
    Ok(())
}
