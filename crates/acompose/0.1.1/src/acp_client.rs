use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};

use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::schema::v1::{
    CloseSessionRequest, ContentBlock, InitializeRequest, InitializeResponse, NewSessionRequest,
    NewSessionResponse, PromptRequest, PromptResponse, RequestPermissionOutcome,
    RequestPermissionRequest, RequestPermissionResponse, ResumeSessionRequest,
    SelectedPermissionOutcome, SessionNotification, SessionUpdate, StopReason, TextContent,
    ToolKind,
};
use agent_client_protocol::{AcpAgent, Agent, Client, ConnectionTo, Responder};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

/// A command sent to a running ACP session task.
#[derive(Debug)]
pub enum SessionCommand {
    /// Send a prompt to the agent and return its response.
    SendPrompt {
        content: String,
        response_tx: oneshot::Sender<anyhow::Result<PromptResult>>,
    },
    /// Shut the session down gracefully.
    Shutdown,
}

/// Lifecycle status of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// The session is starting; the charter prompt is in progress.
    Initializing,
    /// The session is ready to receive follow-up prompts.
    Ready,
    /// The session encountered an error during startup.
    Error,
}

/// Result of a prompt, including the agent's streamed text response.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PromptResult {
    pub stop_reason: StopReason,
    pub text: Vec<String>,
}

/// A handle to a spawned ACP agent process and its session.
#[derive(Clone, Debug)]
pub struct SessionHandle {
    pub name: String,
    pub cwd: PathBuf,
    pub session_id: String,
    pub cmd_tx: mpsc::UnboundedSender<SessionCommand>,
    pub status: Arc<RwLock<SessionStatus>>,
}

/// Shared state used to collect streamed agent message chunks for the current prompt.
#[derive(Clone, Debug, Default)]
struct CollectorState {
    current: Option<mpsc::UnboundedSender<String>>,
}

/// Spawns a `kimi acp` process in `cwd`, completes initialization, creates or resumes a
/// session, sends the charter prompt when creating a new session, and returns a handle.
pub async fn spawn_session(
    kimi_binary: &str,
    name: &str,
    cwd: PathBuf,
    charter: Option<&str>,
    allowed_tool_kinds: Vec<ToolKind>,
    resume_session_id: Option<String>,
) -> anyhow::Result<SessionHandle> {
    let command = format!("{} acp", kimi_binary);
    info!(name, cwd = %cwd.display(), ?resume_session_id, command, "spawning agent");

    let agent = AcpAgent::from_str(&command)
        .map_err(|e| anyhow::anyhow!("failed to parse agent command '{}': {}", command, e))?;

    let charter = charter.map(|s| s.to_string());
    let name = name.to_string();
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<SessionCommand>();
    let (ready_tx, ready_rx) = oneshot::channel::<anyhow::Result<SessionHandle>>();

    let cmd_tx_for_handle = cmd_tx.clone();
    let collector: Arc<Mutex<CollectorState>> = Arc::new(Mutex::new(CollectorState::default()));
    let collector_for_notification = Arc::clone(&collector);
    let allowed_for_permission = allowed_tool_kinds.clone();

    tokio::spawn(async move {
        let name_for_notification = name.clone();
        let name_for_permission = name.clone();
        let name_for_prompt = name.clone();

        let result = Client
            .builder()
            .on_receive_notification(
                move |notification: SessionNotification, _cx| {
                    let name = name_for_notification.clone();
                    let collector = Arc::clone(&collector_for_notification);
                    async move {
                        debug!(%name, ?notification.update, "received session notification");
                        if let SessionUpdate::AgentMessageChunk(chunk) = notification.update {
                            if let ContentBlock::Text(text_content) = chunk.content {
                                if let Ok(state) = collector.lock() {
                                    if let Some(tx) = state.current.as_ref() {
                                        let _ = tx.send(text_content.text);
                                    }
                                }
                            }
                        }
                        Ok(())
                    }
                },
                agent_client_protocol::on_receive_notification!(),
            )
            .on_receive_request(
                move |request: RequestPermissionRequest,
                      responder: Responder<RequestPermissionResponse>,
                      _connection| {
                    let name = name_for_permission.clone();
                    let allowed = allowed_for_permission.clone();
                    async move {
                        let kind = request.tool_call.fields.kind.unwrap_or(ToolKind::Other);
                        let permitted = allowed.is_empty() || allowed.iter().any(|k| *k == kind);
                        let outcome = if permitted {
                            let option_id =
                                request.options.first().map(|opt| opt.option_id.clone());
                            option_id.map_or(RequestPermissionOutcome::Cancelled, |id| {
                                info!(%name, ?kind, "approving permission request");
                                RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                                    id,
                                ))
                            })
                        } else {
                            warn!(%name, ?kind, "denying permission request");
                            RequestPermissionOutcome::Cancelled
                        };
                        let _ = responder.respond(RequestPermissionResponse::new(outcome));
                        Ok(())
                    }
                },
                agent_client_protocol::on_receive_request!(),
            )
            .connect_with(agent, move |connection: ConnectionTo<Agent>| {
                let cwd = cwd.clone();
                let charter = charter.clone();
                let name = name_for_prompt.clone();
                let collector = Arc::clone(&collector);
                let mut ready_tx = Some(ready_tx);
                async move {
                    let mut send_ready = |res: anyhow::Result<SessionHandle>| {
                        if let Some(tx) = ready_tx.take() {
                            let _ = tx.send(res);
                        }
                    };

                    debug!(%name, "initializing agent");
                    let init_response: InitializeResponse = connection
                        .send_request(InitializeRequest::new(ProtocolVersion::V1))
                        .block_task()
                        .await
                        .map_err(|e| {
                            send_ready(Err(anyhow::anyhow!("initialize failed: {}", e)));
                            e
                        })?;
                    info!(%name, ?init_response.agent_info, "agent initialized");

                    let close_supported = init_response
                        .agent_capabilities
                        .session_capabilities
                        .close
                        .is_some();

                    let (session_id, is_resumed) = if let Some(resume_id) = resume_session_id {
                        debug!(%name, %resume_id, "resuming session");
                        connection
                            .send_request(ResumeSessionRequest::new(resume_id.clone(), cwd.clone()))
                            .block_task()
                            .await
                            .map_err(|e| {
                                send_ready(Err(anyhow::anyhow!("resume failed: {}", e)));
                                e
                            })?;
                        info!(%name, %resume_id, "session resumed");
                        (resume_id, true)
                    } else {
                        debug!(%name, "creating session");
                        let new_session: NewSessionResponse = connection
                            .send_request(NewSessionRequest::new(cwd.clone()))
                            .block_task()
                            .await
                            .map_err(|e| {
                                send_ready(Err(anyhow::anyhow!("session creation failed: {}", e)));
                                e
                            })?;
                        let session_id = new_session.session_id.to_string();
                        info!(%name, %session_id, "session created");
                        (session_id, false)
                    };

                    let status = Arc::new(RwLock::new(SessionStatus::Initializing));
                    let handle = SessionHandle {
                        name: name.clone(),
                        cwd: cwd.clone(),
                        session_id: session_id.clone(),
                        cmd_tx: cmd_tx.clone(),
                        status: Arc::clone(&status),
                    };
                    send_ready(Ok(handle));

                    if !is_resumed {
                        if let Some(charter) = charter {
                            debug!(%name, "sending charter");
                            let prompt_response: PromptResponse = connection
                                .send_request(PromptRequest::new(
                                    session_id.clone(),
                                    vec![ContentBlock::Text(TextContent::new(charter))],
                                ))
                                .block_task()
                                .await
                                .map_err(|e| {
                                    error!(%name, error = %e, "charter prompt failed");
                                    if let Ok(mut s) = status.write() {
                                        *s = SessionStatus::Error;
                                    }
                                    e
                                })?;
                            info!(%name, ?prompt_response.stop_reason, "charter prompt completed");
                        }
                    }

                    if let Ok(mut s) = status.write() {
                        *s = SessionStatus::Ready;
                    }
                    info!(%name, "session command loop started");
                    loop {
                        match cmd_rx.recv().await {
                            Some(SessionCommand::SendPrompt {
                                content,
                                response_tx,
                            }) => {
                                debug!(%name, "sending prompt");

                                let (chunk_tx, mut chunk_rx) = mpsc::unbounded_channel::<String>();
                                {
                                    let mut state = collector.lock().map_err(|e| {
                                        anyhow::anyhow!("collector lock poisoned: {}", e)
                                    })?;
                                    state.current = Some(chunk_tx);
                                }

                                let prompt_result = connection
                                    .send_request(PromptRequest::new(
                                        session_id.clone(),
                                        vec![ContentBlock::Text(TextContent::new(content))],
                                    ))
                                    .block_task()
                                    .await
                                    .map_err(|e| anyhow::anyhow!("prompt failed: {}", e));

                                let mut text = Vec::new();
                                loop {
                                    match chunk_rx.try_recv() {
                                        Ok(t) => text.push(t),
                                        Err(mpsc::error::TryRecvError::Empty) => break,
                                        Err(mpsc::error::TryRecvError::Disconnected) => break,
                                    }
                                }
                                {
                                    let mut state = collector.lock().map_err(|e| {
                                        anyhow::anyhow!("collector lock poisoned: {}", e)
                                    })?;
                                    state.current = None;
                                }

                                let result = prompt_result.map(|response| PromptResult {
                                    stop_reason: response.stop_reason,
                                    text,
                                });

                                if response_tx.send(result).is_err() {
                                    warn!(%name, "prompt response receiver dropped");
                                }
                            }
                            Some(SessionCommand::Shutdown) | None => {
                                if close_supported {
                                    debug!(%name, "closing session before shutdown");
                                    let _ = connection
                                        .send_request(CloseSessionRequest::new(session_id.clone()))
                                        .block_task()
                                        .await;
                                }
                                info!(%name, "shutting down session");
                                break;
                            }
                        }
                    }

                    Ok(())
                }
            })
            .await
            .map_err(|e| anyhow::anyhow!("ACP session failed: {}", e));

        if let Err(e) = result {
            error!(error = %e, "ACP session task ended with error");
            // The ready sender was moved into the connect_with closure; if the closure never
            // completed, the receiver will observe a cancelled channel and surface the failure.
        }
    });

    let handle = ready_rx
        .await
        .map_err(|_| anyhow::anyhow!("session task closed before ready signal"))??;

    Ok(SessionHandle {
        name: handle.name,
        cwd: handle.cwd,
        session_id: handle.session_id,
        cmd_tx: cmd_tx_for_handle,
        status: handle.status,
    })
}

/// Send a prompt to a session and await its response.
pub async fn send_prompt(handle: &SessionHandle, content: &str) -> anyhow::Result<PromptResult> {
    let (tx, rx) = oneshot::channel();
    handle
        .cmd_tx
        .send(SessionCommand::SendPrompt {
            content: content.to_string(),
            response_tx: tx,
        })
        .map_err(|e| anyhow::anyhow!("failed to send prompt command: {}", e))?;

    rx.await
        .map_err(|_| anyhow::anyhow!("session task closed while waiting for prompt response"))?
}

/// Shut down a session gracefully.
pub async fn shutdown_session(handle: &SessionHandle) {
    let _ = handle.cmd_tx.send(SessionCommand::Shutdown);
}
