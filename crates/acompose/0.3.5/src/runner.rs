use std::sync::Arc;

use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager,
    tower::{StreamableHttpServerConfig, StreamableHttpService},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::compositor::Compositor;
use crate::compositor::state::State;
use crate::config::Config;
use crate::mcp_server::ComposeMcpServer;
use crate::server::Server;

/// Handles to the background services started by [`run_services`].
pub struct ServiceHandles {
    mcp: Option<(
        tokio::task::JoinHandle<anyhow::Result<()>>,
        CancellationToken,
    )>,
    server: Option<(
        tokio::task::JoinHandle<anyhow::Result<()>>,
        CancellationToken,
    )>,
}

impl ServiceHandles {
    /// Cancel and await all running services.
    pub async fn shutdown(self) {
        if let Some((handle, ct)) = self.mcp {
            ct.cancel();
            if let Err(e) = handle.await {
                error!(error = %e, "MCP server task panicked");
            }
        }

        if let Some((handle, ct)) = self.server {
            ct.cancel();
            if let Err(e) = handle.await {
                error!(error = %e, "Compose server task panicked");
            }
        }
    }
}

/// Start the MCP server and the Compose server according to `config`, spawn
/// configured sessions, and return handles for graceful shutdown.
pub async fn run_services(
    compositor: Arc<Compositor>,
    config: &Config,
    state: &State,
    cancel: CancellationToken,
) -> anyhow::Result<ServiceHandles> {
    // Start the integrated MCP server before spawning agents so they can discover
    // the acompose control-plane tools during their own initialization.
    let mcp = if config.acompose_control_mcp.enabled {
        let bind_address = config.acompose_control_mcp.bind_address.clone();
        let compositor = Arc::clone(&compositor);
        let mcp_ct = cancel.child_token();
        let axum_ct = mcp_ct.clone();
        let config_ct = mcp_ct.clone();

        let mut session_manager = LocalSessionManager::default();
        session_manager.session_config.keep_alive = None;
        let session_manager = Arc::new(session_manager);

        let service: StreamableHttpService<ComposeMcpServer, LocalSessionManager> =
            StreamableHttpService::new(
                move || Ok(ComposeMcpServer::new(Arc::clone(&compositor))),
                session_manager,
                StreamableHttpServerConfig::default()
                    .with_cancellation_token(config_ct.child_token()),
            );
        let app = axum::Router::new().nest_service("/mcp", service);

        // Bind synchronously so the port is guaranteed to be open before we spawn
        // any agents that need to connect to it.
        let listener = tokio::net::TcpListener::bind(&bind_address)
            .await
            .map_err(|e| anyhow::anyhow!("binding MCP server: {e}"))?;
        info!(%bind_address, "MCP server listening");

        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move { axum_ct.cancelled_owned().await })
                .await
                .map_err(|e| anyhow::anyhow!("running MCP server: {e}"))?;
            Ok::<(), anyhow::Error>(())
        });

        Some((handle, mcp_ct))
    } else {
        None
    };

    // Spawn sessions now that the MCP control plane is ready.
    let infos = compositor
        .spawn_sessions_from_config(config, state)
        .await
        .map_err(|e| anyhow::anyhow!("spawning sessions from config: {e}"))?;

    info!(sessions = infos.len(), "starting acompose");
    for info in &infos {
        info!(name = %info.name, session_id = %info.session_id, ?info.status, "session ready");
    }

    // Start the Compose WebSocket server if enabled.
    let server = if config.server.enabled {
        let bind_address = config
            .server
            .bind_address
            .parse()
            .map_err(|e| anyhow::anyhow!("parsing server bind address: {e}"))?;
        let server = Server::new(Arc::clone(&compositor), bind_address);
        let server_ct = cancel.child_token();
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

    Ok(ServiceHandles { mcp, server })
}
