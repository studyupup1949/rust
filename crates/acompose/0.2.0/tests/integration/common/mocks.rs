//! Test helpers for acompose integration tests.
//!
//! Provides an in-memory ACP agent and a helper to connect it to a real
//! `SessionHandle` over a `tokio::io::duplex` pipe.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::schema::v1::{
    AgentCapabilities, CancelNotification, ContentBlock, ContentChunk, Implementation,
    InitializeRequest, InitializeResponse, ListSessionsRequest, ListSessionsResponse,
    LoadSessionRequest, LoadSessionResponse, McpServer as AcpMcpServer, NewSessionRequest,
    NewSessionResponse, PromptRequest, PromptResponse, RequestPermissionOutcome,
    RequestPermissionRequest, RequestPermissionResponse, SessionCapabilities,
    SessionCloseCapabilities, SessionId, SessionNotification, SessionUpdate, StopReason,
    TextContent, ToolKind,
};
use agent_client_protocol::{Agent, ByteStreams, Client, ConnectionTo, Responder};
use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};
use tokio_util::sync::CancellationToken;

use acompose::agent::session_actor::SessionActor;
use acompose::agent::session_factory::{SessionConfig, SessionFactory};
use acompose::compositor::Compositor;
use acompose::compositor::state::{PersistSession, PromptJob};
use acompose::config::McpServer;
use acompose::server::connection::ClientConnection;

/// An in-memory ACP agent for testing that connects over a `tokio::io::duplex` pair.
#[derive(Clone, Debug)]
pub struct InMemoryAgent {
    session_id: Arc<Mutex<String>>,
    recorded_prompts: Arc<Mutex<Vec<String>>>,
    recorded_cancels: Arc<Mutex<Vec<String>>>,
    live_tx: broadcast::Sender<SessionNotification>,
    response_delay: std::time::Duration,
    response_text: Arc<Mutex<String>>,
}

impl InMemoryAgent {
    pub fn with_response_delay(session_id: impl Into<String>, delay: std::time::Duration) -> Self {
        let (live_tx, _) = broadcast::channel(256);
        Self {
            session_id: Arc::new(Mutex::new(session_id.into())),
            recorded_prompts: Arc::new(Mutex::new(Vec::new())),
            recorded_cancels: Arc::new(Mutex::new(Vec::new())),
            live_tx,
            response_delay: delay,
            response_text: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Return the current ACP session id held by the agent.
    #[must_use]
    pub fn session_id(&self) -> String {
        self.session_id.lock().unwrap().clone()
    }

    /// Set the text that the agent will stream back as response chunks.
    pub fn set_response_text(&self, text: impl Into<String>) {
        *self.response_text.lock().unwrap() = text.into();
    }

    #[must_use]
    pub fn recorded_prompts(&self) -> Vec<String> {
        self.recorded_prompts.lock().unwrap().clone()
    }

    #[must_use]
    pub fn recorded_cancels(&self) -> Vec<String> {
        self.recorded_cancels.lock().unwrap().clone()
    }

    pub fn send_live_notification(&self, notification: SessionNotification) {
        let _ = self.live_tx.send(notification);
    }

    pub async fn run(self, channel: agent_client_protocol::Channel) {
        let session_id = self.session_id.clone();
        let session_id_for_new = session_id.clone();
        let session_id_for_prompt = session_id.clone();
        let recorded_prompts = self.recorded_prompts.clone();
        let recorded_prompts_for_new = recorded_prompts.clone();
        let recorded_cancels = self.recorded_cancels.clone();
        let recorded_cancels_for_new = recorded_cancels.clone();
        let live_tx = self.live_tx.clone();
        let response_delay = self.response_delay;
        let response_text = self.response_text.clone();

        let result = Agent
            .builder()
            .name("test-agent")
            .with_spawned(
                move |cx: ConnectionTo<agent_client_protocol::Client>| async move {
                    let mut live_rx = live_tx.subscribe();
                    loop {
                        match live_rx.recv().await {
                            Ok(notification) => {
                                if cx.send_notification(notification).is_err() {
                                    break;
                                }
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                            Err(broadcast::error::RecvError::Lagged(_)) => {}
                        }
                    }
                    Ok(())
                },
            )
            .on_receive_request(
                async move |req: InitializeRequest, responder, _cx| {
                    let resp = InitializeResponse::new(req.protocol_version)
                        .agent_info(Implementation::new("test-agent", "0.1.0"))
                        .agent_capabilities(AgentCapabilities::new().session_capabilities(
                            SessionCapabilities::new().close(SessionCloseCapabilities::new()),
                        ));
                    responder.respond(resp)
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |_req: NewSessionRequest, responder, _cx| {
                    let session_id = session_id_for_new.clone();
                    let new_session_id = uuid::Uuid::new_v4().to_string();
                    *session_id.lock().unwrap() = new_session_id.clone();
                    recorded_prompts_for_new.lock().unwrap().clear();
                    recorded_cancels_for_new.lock().unwrap().clear();
                    let resp = NewSessionResponse::new(SessionId::new(new_session_id));
                    responder.respond(resp)
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |_req: LoadSessionRequest, responder, _cx| {
                    let resp = LoadSessionResponse::new();
                    responder.respond(resp)
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_request(
                async move |req: PromptRequest, responder, cx| {
                    let recorded_prompts = recorded_prompts.clone();
                    let response_text = response_text.clone();
                    let session_id = session_id_for_prompt.lock().unwrap().clone();
                    let text: String = req
                        .prompt
                        .iter()
                        .map(|block| match block {
                            ContentBlock::Text(t) => t.text.clone(),
                            _ => String::new(),
                        })
                        .collect();
                    recorded_prompts.lock().unwrap().push(text);

                    // Spawn the delayed response so the agent event loop stays
                    // responsive to notifications (e.g. session/cancel).
                    tokio::spawn(async move {
                        tokio::time::sleep(response_delay).await;
                        let text = response_text.lock().unwrap().clone();
                        if !text.is_empty() {
                            let notification = SessionNotification::new(
                                session_id,
                                SessionUpdate::AgentMessageChunk(ContentChunk::new(
                                    ContentBlock::Text(TextContent::new(text)),
                                )),
                            );
                            let _ = cx.send_notification(notification);
                        }
                        let _ = responder.respond(PromptResponse::new(StopReason::EndTurn));
                    });
                    Ok(())
                },
                agent_client_protocol::on_receive_request!(),
            )
            .on_receive_notification(
                async move |cancel: CancelNotification, _cx| {
                    recorded_cancels
                        .lock()
                        .unwrap()
                        .push(cancel.session_id.to_string());
                    Ok(())
                },
                agent_client_protocol::on_receive_notification!(),
            )
            .connect_to(channel)
            .await;

        if let Err(e) = result {
            tracing::warn!(error = %e, "in-memory agent connection ended");
        }
    }
}

/// In-memory session factory for integration tests.
///
/// Creates [`InMemoryAgent`] instances connected to real [`SessionHandle`]s
/// over `tokio::io::duplex` pipes.
#[derive(Debug, Clone)]
pub struct MockSessionFactory {
    next_id: Arc<AtomicU64>,
    agents: Arc<Mutex<HashMap<String, InMemoryAgent>>>,
    response_delay: Duration,
}

impl Default for MockSessionFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSessionFactory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicU64::new(1)),
            agents: Arc::new(Mutex::new(HashMap::new())),
            response_delay: Duration::ZERO,
        }
    }

    /// Create a factory whose agents wait `delay` before responding to prompts.
    /// Useful for tests that need to observe an in-flight prompt.
    #[must_use]
    pub fn with_response_delay(delay: Duration) -> Self {
        Self {
            next_id: Arc::new(AtomicU64::new(1)),
            agents: Arc::new(Mutex::new(HashMap::new())),
            response_delay: delay,
        }
    }

    #[must_use]
    pub fn agent(&self, name: &str) -> Option<InMemoryAgent> {
        self.agents.lock().unwrap().get(name).cloned()
    }
}

#[async_trait]
impl SessionFactory for MockSessionFactory {
    async fn create(
        &self,
        config: SessionConfig,
        persist_tx: mpsc::UnboundedSender<PersistSession>,
        forward_tx: mpsc::UnboundedSender<(String, PromptJob)>,
        cancel_token: CancellationToken,
    ) -> anyhow::Result<SessionActor> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let session_id = format!("mock-{id}");
        let agent = InMemoryAgent::with_response_delay(&session_id, self.response_delay);
        let (session_channel, agent_channel) = agent_client_protocol::Channel::duplex();

        let agent_clone = agent.clone();
        tokio::spawn(agent_clone.run(agent_channel));

        let actor = SessionActor::connect(
            &config.name,
            config.cwd.clone(),
            config.charter,
            config.allowed_tool_kinds,
            config.load_session_id,
            config.mcp_servers,
            session_channel,
            persist_tx,
            forward_tx,
            cancel_token,
        )
        .await
        .expect("SessionActor::connect should succeed with in-memory agent");

        self.agents
            .lock()
            .unwrap()
            .insert(config.name.clone(), agent);
        Ok(actor)
    }
}

/// A typed ACP client mock that drives the proxy over a `tokio::io::duplex` pipe.
///
/// The client speaks real typed ACP directly to `ProxyConnection`.
#[derive(Clone, Debug)]
pub struct TypedClient {
    connection: ConnectionTo<Agent>,
    notifications: broadcast::Sender<SessionNotification>,
    connection_task: Arc<tokio::task::JoinHandle<()>>,
}

impl TypedClient {
    /// Initialize the connection.
    pub async fn initialize(&self) -> Result<InitializeResponse, agent_client_protocol::Error> {
        self.connection
            .send_request(InitializeRequest::new(ProtocolVersion::V1))
            .block_task()
            .await
    }

    /// List active sessions.
    pub async fn list_sessions(
        &self,
    ) -> Result<ListSessionsResponse, agent_client_protocol::Error> {
        self.connection
            .send_request(ListSessionsRequest::new())
            .block_task()
            .await
    }

    /// Create a new session through the proxy.
    pub async fn new_session(
        &self,
        name: &str,
        cwd: &Path,
        _charter: &str,
        _allowed_tool_kinds: Vec<ToolKind>,
        mcp_servers: Vec<McpServer>,
    ) -> Result<NewSessionResponse, agent_client_protocol::Error> {
        let mut meta = agent_client_protocol::schema::v1::Meta::new();
        meta.insert(
            "name".to_string(),
            serde_json::Value::String(name.to_string()),
        );
        let request = NewSessionRequest::new(cwd)
            .mcp_servers(convert_config_mcp_servers(&mcp_servers))
            .meta(meta);
        self.connection.send_request(request).block_task().await
    }

    /// Load an existing session.
    pub async fn load_session(
        &self,
        session_id: &str,
        cwd: &Path,
        mcp_servers: Vec<AcpMcpServer>,
    ) -> Result<LoadSessionResponse, agent_client_protocol::Error> {
        self.connection
            .send_request(
                LoadSessionRequest::new(SessionId::new(session_id), cwd).mcp_servers(mcp_servers),
            )
            .block_task()
            .await
    }

    /// Send a prompt to a session.
    pub async fn prompt(
        &self,
        session_id: &str,
        content: &str,
    ) -> Result<PromptResponse, agent_client_protocol::Error> {
        self.connection
            .send_request(PromptRequest::new(
                SessionId::new(session_id),
                vec![ContentBlock::Text(TextContent::new(content.to_string()))],
            ))
            .block_task()
            .await
    }

    /// Send a cancel notification for a session.
    pub async fn cancel(&self, session_id: &str) -> Result<(), agent_client_protocol::Error> {
        self.connection
            .send_notification(CancelNotification::new(SessionId::new(session_id)))
    }

    /// Subscribe to live `session/update` notifications.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<SessionNotification> {
        self.notifications.subscribe()
    }

    /// Shut the client connection down gracefully.
    pub async fn shutdown(&self) -> Result<(), agent_client_protocol::Error> {
        self.connection_task.abort();
        Ok(())
    }
}

/// Spawn `ProxyConnection` over an in-memory transport and return a [`TypedClient`] handle.
pub async fn run_proxy_client(compositor: Arc<Compositor>) -> TypedClient {
    let (client_stream, proxy_stream) = tokio::io::duplex(4096);
    let (proxy_read, proxy_write) = tokio::io::split(proxy_stream);
    let (client_read, client_write) = tokio::io::split(client_stream);

    tokio::spawn(async move {
        ClientConnection::new(compositor)
            .serve(
                ByteStreams::new(proxy_write.compat_write(), proxy_read.compat()),
                CancellationToken::new(),
            )
            .await;
    });

    let (notifications, _) = broadcast::channel::<SessionNotification>(256);
    let notifications_for_handler = notifications.clone();
    let (connection_tx, connection_rx) = oneshot::channel();

    let task = tokio::spawn(async move {
        let result = Client
            .builder()
            .name("typed-test-client")
            .on_receive_notification(
                move |notification: SessionNotification, _cx: ConnectionTo<Agent>| {
                    let tx = notifications_for_handler.clone();
                    async move {
                        let _ = tx.send(notification);
                        Ok(())
                    }
                },
                agent_client_protocol::on_receive_notification!(),
            )
            .on_receive_request(
                move |_request: RequestPermissionRequest,
                      responder: Responder<RequestPermissionResponse>,
                      _cx: ConnectionTo<Agent>| {
                    async move {
                        responder.respond(RequestPermissionResponse::new(
                            RequestPermissionOutcome::Cancelled,
                        ))
                    }
                },
                agent_client_protocol::on_receive_request!(),
            )
            .connect_with(
                ByteStreams::new(client_write.compat_write(), client_read.compat()),
                |connection: ConnectionTo<Agent>| async move {
                    let _ = connection_tx.send(connection);
                    futures::future::pending::<Result<(), agent_client_protocol::Error>>().await
                },
            )
            .await;
        if let Err(e) = result {
            tracing::warn!(error = %e, "typed test client connection ended");
        }
    });

    let connection = connection_rx
        .await
        .expect("connection should be handed back from connect_with callback");

    TypedClient {
        connection,
        notifications,
        connection_task: Arc::new(task),
    }
}

fn convert_config_mcp_servers(servers: &[McpServer]) -> Vec<AcpMcpServer> {
    servers
        .iter()
        .map(|s| match s.transport {
            acompose::config::McpServerTransport::Http => {
                AcpMcpServer::Http(AcpMcpServerHttp::new(s.name.clone(), s.url.clone()))
            }
            acompose::config::McpServerTransport::Sse => {
                AcpMcpServer::Sse(AcpMcpServerSse::new(s.name.clone(), s.url.clone()))
            }
            acompose::config::McpServerTransport::Stdio => AcpMcpServer::Stdio(
                AcpMcpServerStdio::new(
                    s.name.clone(),
                    std::path::PathBuf::from(s.command.clone().unwrap_or_default()),
                )
                .args(s.args.clone()),
            ),
        })
        .collect()
}

// Bring the typed MCP server constructors into scope for the helper above.
use agent_client_protocol::schema::v1::{
    McpServerHttp as AcpMcpServerHttp, McpServerSse as AcpMcpServerSse,
    McpServerStdio as AcpMcpServerStdio,
};
