use dashmap::DashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use agent_client_protocol::schema::v1::{
    CancelNotification, ContentBlock, InitializeRequest, InitializeResponse, ListSessionsRequest,
    ListSessionsResponse, LoadSessionRequest, LoadSessionResponse, McpServer as AcpMcpServer,
    NewSessionRequest, NewSessionResponse, PromptRequest, PromptResponse, RequestPermissionOutcome,
    RequestPermissionRequest, RequestPermissionResponse, SessionId, SessionNotification,
    SessionUpdate, StopReason,
};
use agent_client_protocol::{Agent, Client, ConnectionTo, Responder};
use anyhow::Context;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::compositor::Compositor;
use crate::config::{McpServer, McpServerTransport};

/// Compose client connection handler.
///
/// Speaks typed ACP as an [`Agent`] on one side and forwards calls to the
/// [`Compositor`] on the other side. Live session notifications are forwarded
/// through per-session tasks spawned when a client calls `session/load`.
pub struct ClientConnection {
    compositor: Arc<Compositor>,
    suppressed_message_ids: Arc<DashSet<String>>,
    initialized: Arc<AtomicBool>,
    /// Unique identifier for this connection, used in logs to distinguish
    /// concurrent client connections.
    connection_id: String,
    /// Sessions already loaded on this connection. Used to prevent multiple
    /// concurrent forward tasks for the same session, which would deliver every
    /// `session/update` notification twice.
    loaded_sessions: Arc<DashSet<String>>,
}

impl ClientConnection {
    #[must_use]
    pub fn new(compositor: Arc<Compositor>) -> Self {
        Self {
            compositor,
            suppressed_message_ids: Arc::new(DashSet::new()),
            initialized: Arc::new(AtomicBool::new(false)),
            connection_id: uuid::Uuid::new_v4().to_string(),
            loaded_sessions: Arc::new(DashSet::new()),
        }
    }

    /// Run the client connection on the provided transport until the connection closes
    /// or `cancel` is triggered.
    pub async fn serve<T>(self, transport: T, cancel: CancellationToken)
    where
        T: agent_client_protocol::ConnectTo<Agent> + Send + 'static,
    {
        let state = Arc::new(self);

        let result = Agent
            .builder()
            .name("acompose-server")
            .on_receive_request(
                {
                    let state = Arc::clone(&state);
                    move |req: InitializeRequest,
                          responder: Responder<InitializeResponse>,
                          _cx: ConnectionTo<Client>| {
                        let state = Arc::clone(&state);
                        async move {
                            let response = InitializeResponse::new(req.protocol_version);
                            state.initialized.store(true, Ordering::SeqCst);
                            let _ = responder.respond(response);
                            Ok(())
                        }
                    }
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                {
                    let state = Arc::clone(&state);
                    move |req: ListSessionsRequest,
                          responder: Responder<ListSessionsResponse>,
                          _cx: ConnectionTo<Client>| {
                        let state = Arc::clone(&state);
                        async move {
                            match state.handle_list_sessions(req).await {
                                Ok(response) => {
                                    let _ = responder.respond(response);
                                }
                                Err(e) => {
                                    let _ = responder.respond_with_internal_error(e.to_string());
                                }
                            }
                            Ok(())
                        }
                    }
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                {
                    let state = Arc::clone(&state);
                    move |req: NewSessionRequest,
                          responder: Responder<NewSessionResponse>,
                          _cx: ConnectionTo<Client>| {
                        let state = Arc::clone(&state);
                        async move {
                            match state.handle_new_session(req).await {
                                Ok(response) => {
                                    let _ = responder.respond(response);
                                }
                                Err(e) => {
                                    let _ = responder.respond_with_internal_error(e.to_string());
                                }
                            }
                            Ok(())
                        }
                    }
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                {
                    let state = Arc::clone(&state);
                    let cancel = cancel.clone();
                    move |req: LoadSessionRequest,
                          responder: Responder<LoadSessionResponse>,
                          cx: ConnectionTo<Client>| {
                        let state = Arc::clone(&state);
                        let cancel = cancel.child_token();
                        async move {
                            match state.handle_load_session(cx, req, cancel).await {
                                Ok(response) => {
                                    let _ = responder.respond(response);
                                }
                                Err(e) => {
                                    let _ = responder.respond_with_internal_error(e.to_string());
                                }
                            }
                            Ok(())
                        }
                    }
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                {
                    let state = Arc::clone(&state);
                    async move |req: PromptRequest,
                                responder: Responder<PromptResponse>,
                                _cx: ConnectionTo<Client>| {
                        let state = Arc::clone(&state);
                        match state.handle_prompt(req).await {
                            Ok(response) => {
                                let _ = responder.respond(response);
                            }
                            Err(e) => {
                                let _ = responder.respond_with_internal_error(e.to_string());
                            }
                        }
                        Ok(())
                    }
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                |_req: RequestPermissionRequest,
                 responder: Responder<RequestPermissionResponse>,
                 _cx: ConnectionTo<Client>| async move {
                    // Skip for MVP — permission forwarding needs a pending-job registry.
                    let _ = responder.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Cancelled,
                    ));
                    Ok(())
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_notification(
                {
                    let state = Arc::clone(&state);
                    move |notif: CancelNotification, _cx: ConnectionTo<Client>| {
                        let state = Arc::clone(&state);
                        async move {
                            if let Some(handle) = state
                                .compositor
                                .get_session_handle(&notif.session_id.to_string())
                            {
                                let _ = handle.cancel_current();
                            }
                            Ok(())
                        }
                    }
                },
                agent_client_protocol::on_receive_notification!(),
            )
            .connect_to(transport)
            .await;

        if let Err(e) = result {
            warn!(error = %e, "typed client connection ended");
        }
    }

    async fn handle_list_sessions(
        &self,
        _req: ListSessionsRequest,
    ) -> anyhow::Result<ListSessionsResponse> {
        self.check_initialized()?;
        let sessions = self.compositor.list_sessions().await?;
        let acp_sessions: Vec<agent_client_protocol::schema::v1::SessionInfo> = sessions
            .into_iter()
            .map(|s| {
                agent_client_protocol::schema::v1::SessionInfo::new(s.session_id, s.cwd)
                    .title(s.name)
            })
            .collect();
        Ok(ListSessionsResponse::new(acp_sessions))
    }

    async fn handle_new_session(
        &self,
        req: NewSessionRequest,
    ) -> anyhow::Result<NewSessionResponse> {
        self.check_initialized()?;
        let mcp_servers = convert_mcp_servers(&req.mcp_servers);
        // ACP session/new has no name/charter; we accept an optional session name via
        // the reserved `_meta` field so the compositor can identify the created session.
        let name = req
            .meta
            .as_ref()
            .and_then(|m| m.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed");
        let (info, _charter_prompt_id) = self
            .compositor
            .create_session(name, req.cwd, "", vec![], mcp_servers)
            .await?;
        Ok(NewSessionResponse::new(SessionId::new(info.session_id)))
    }

    async fn handle_load_session(
        &self,
        connection: ConnectionTo<Client>,
        req: LoadSessionRequest,
        cancel: CancellationToken,
    ) -> anyhow::Result<LoadSessionResponse> {
        self.check_initialized()?;
        let session_id = req.session_id.to_string();
        info!(
            connection_id = %self.connection_id,
            %session_id,
            "client session/load started"
        );

        let handle = self
            .compositor
            .get_session_handle(&session_id)
            .context("session not found")?;

        // A single connection cannot meaningfully load the same session twice:
        // live updates are not tagged by subscription, so the client would
        // receive every notification duplicated.
        if self.loaded_sessions.contains(session_id.as_str()) {
            warn!(
                connection_id = %self.connection_id,
                %session_id,
                "client session/load rejected: session already loaded on this connection"
            );
            anyhow::bail!(
                "session '{}' is already loaded on this connection",
                session_id
            );
        }

        // Replay existing history to the client.
        let history = handle.history().await.unwrap_or_default();
        info!(
            connection_id = %self.connection_id,
            %session_id,
            history_len = history.len(),
            "client session/load replaying history"
        );
        for notification in history {
            let _ = connection.send_notification(notification);
        }

        // Subscribe to live updates and spawn a forward task for this session.
        let mut rx = handle.subscribe();
        let suppressed = Arc::clone(&self.suppressed_message_ids);
        let loaded_sessions = Arc::clone(&self.loaded_sessions);
        let session_id_for_task = session_id.clone();
        let connection_id = self.connection_id.clone();
        let cancel = cancel.child_token();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    notification = rx.recv() => {
                        match notification {
                            Ok(notification) => {
                                if should_forward(&notification, &suppressed).await
                                    && connection.send_notification(notification).is_err()
                                {
                                    break;
                                }
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                            Err(broadcast::error::RecvError::Lagged(_)) => {}
                        }
                    }
                    () = cancel.cancelled() => break,
                }
            }
            loaded_sessions.remove(&session_id_for_task);
            info!(
                %connection_id,
                %session_id_for_task,
                "session/load forward task ended"
            );
        });
        self.loaded_sessions.insert(session_id.clone());

        info!(
            connection_id = %self.connection_id,
            %session_id,
            "client session/load subscribed to live updates"
        );

        Ok(LoadSessionResponse::new())
    }

    async fn handle_prompt(&self, req: PromptRequest) -> anyhow::Result<PromptResponse> {
        self.check_initialized()?;
        let session_id = req.session_id.to_string();

        // Verify the session exists before accepting the prompt.
        if self.compositor.get_session_handle(&session_id).is_none() {
            anyhow::bail!("session not found");
        }

        let text: String = req
            .prompt
            .iter()
            .map(|block| match block {
                ContentBlock::Text(t) => t.text.clone(),
                _ => String::new(),
            })
            .collect();

        let prompt_id = self
            .compositor
            .send_message_async(&session_id, &text, None, false)
            .await?;

        self.suppressed_message_ids.insert(prompt_id);

        Ok(PromptResponse::new(StopReason::EndTurn))
    }
    fn check_initialized(&self) -> anyhow::Result<()> {
        if !self.initialized.load(Ordering::SeqCst) {
            anyhow::bail!("not initialized");
        }
        Ok(())
    }
}

async fn should_forward(
    notification: &SessionNotification,
    suppressed_message_ids: &DashSet<String>,
) -> bool {
    if let SessionUpdate::UserMessageChunk(chunk) = &notification.update
        && let Some(message_id) = &chunk.message_id
    {
        return suppressed_message_ids
            .remove(&message_id.to_string())
            .is_none();
    }
    true
}

fn convert_mcp_servers(servers: &[AcpMcpServer]) -> Vec<McpServer> {
    servers
        .iter()
        .map(|s| match s {
            AcpMcpServer::Http(http) => McpServer {
                name: http.name.clone(),
                url: http.url.clone(),
                transport: McpServerTransport::Http,
                command: None,
                args: vec![],
            },
            AcpMcpServer::Sse(sse) => McpServer {
                name: sse.name.clone(),
                url: sse.url.clone(),
                transport: McpServerTransport::Sse,
                command: None,
                args: vec![],
            },
            AcpMcpServer::Stdio(stdio) => McpServer {
                name: stdio.name.clone(),
                url: String::new(),
                transport: McpServerTransport::Stdio,
                command: Some(stdio.command.to_string_lossy().to_string()),
                args: stdio.args.clone(),
            },
            _ => McpServer {
                name: String::new(),
                url: String::new(),
                transport: McpServerTransport::Http,
                command: None,
                args: vec![],
            },
        })
        .collect()
}
