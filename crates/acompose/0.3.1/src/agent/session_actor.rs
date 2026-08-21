use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::schema::v1::{
    CancelNotification, CloseSessionRequest, ContentBlock, ContentChunk, InitializeRequest,
    InitializeResponse, LoadSessionRequest, McpServerHttp, McpServerSse, McpServerStdio,
    NewSessionRequest, NewSessionResponse, PromptRequest, RequestPermissionOutcome,
    RequestPermissionRequest, RequestPermissionResponse, SelectedPermissionOutcome,
    SessionNotification, SessionUpdate, StopReason, TextContent, ToolKind,
};
use agent_client_protocol::{Agent, Client, ConnectionTo, Responder};
use chrono::{DateTime, Utc};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use crate::agent::notification_history::NotificationHistory;
use crate::agent::session_handle::SessionHandle;
use crate::compositor::state::{PersistSession, PromptJob, PromptStatus, SessionState};
use crate::config::{McpServer, McpServerTransport};
use crate::cron::worker::{CronCommand, CronWorker};

/// A command sent to a running session actor.
#[derive(Debug)]
pub enum SessionCommand {
    /// Queue a prompt for the agent and return its response.
    Prompt {
        prompt_id: String,
        content: String,
        cron_job_name: Option<String>,
        send_result_to: Option<String>,
    },
    /// Cancel a queued or in-flight prompt.
    Cancel {
        prompt_id: String,
    },
    /// Cancel whatever prompt is currently in flight, regardless of id.
    CancelCurrent,
    Cron(CronCommand),
    /// Return the full notification history.
    GetHistory {
        response_tx: oneshot::Sender<Vec<SessionNotification>>,
    },
    /// Recreate the underlying ACP session (new session id) while keeping the
    /// same actor, job queue, cron worker, and command channel.
    Recreate {
        extra_charter: Option<String>,
        respond_to: oneshot::Sender<anyhow::Result<(SessionHandle, Option<String>)>>,
    },
    /// Shut the session down gracefully.
    Shutdown {
        done_tx: Option<oneshot::Sender<()>>,
    },
    /// Internal: persist the current actor state.
    Persist,
}

/// A forwarded permission request that an external handler can answer.
#[derive(Debug)]
pub struct PermissionJob {
    pub request: RequestPermissionRequest,
    pub response_tx: oneshot::Sender<RequestPermissionResponse>,
}

/// Result of a prompt, including the agent's streamed text response.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptResult {
    pub stop_reason: StopReason,
    pub text: String,
}

/// A queued prompt job inside the actor.
#[derive(Debug)]
struct Job {
    prompt_id: String,
    content: String,
    send_result_to: Option<String>,
    cron_job_name: Option<String>,
    created_at: DateTime<Utc>,
}

impl Job {
    fn to_prompt_job(
        &self,
        status: PromptStatus,
        result: Option<PromptResult>,
        error: Option<String>,
    ) -> PromptJob {
        PromptJob {
            target: String::new(), // filled by the actor from session name
            content: self.content.clone(),
            status,
            send_result_to: self.send_result_to.clone(),
            cron_job_name: self.cron_job_name.clone(),
            result,
            error,
            created_at: self.created_at,
        }
    }
}

/// Shared state used to collect streamed agent message chunks for the current prompt.
#[derive(Clone, Debug, Default)]
struct CollectorState {
    current_text: String,
}

/// State shared between the notification handler and the actor task.
#[derive(Clone)]
struct SharedState {
    name: String,
    collector: Arc<Mutex<CollectorState>>,
    history: Arc<tokio::sync::RwLock<NotificationHistory>>,
    broadcast_tx: broadcast::Sender<SessionNotification>,
}

impl SharedState {
    fn new(name: String, broadcast_tx: broadcast::Sender<SessionNotification>) -> Self {
        Self {
            name,
            collector: Arc::new(Mutex::new(CollectorState::default())),
            history: Arc::new(tokio::sync::RwLock::new(NotificationHistory::new())),
            broadcast_tx,
        }
    }

    async fn handle_notification(&self, notification: SessionNotification) {
        debug!(%self.name, ?notification.update, "received session notification");
        if let SessionUpdate::AgentMessageChunk(ref chunk) = notification.update
            && let ContentBlock::Text(text_content) = &chunk.content
            && let Ok(mut state) = self.collector.lock()
        {
            state.current_text.push_str(&text_content.text);
        }
        let mut h = self.history.write().await;
        h.push(notification.clone());
        let history_len = h.len();
        drop(h);
        debug!(%self.name, history_len, "session notification saved to history");
        let _ = self.broadcast_tx.send(notification);
    }

    fn take_collected_text(&self) -> anyhow::Result<String> {
        let mut state = self
            .collector
            .lock()
            .map_err(|e| anyhow::anyhow!("collector lock poisoned: {}", e))?;
        Ok(std::mem::take(&mut state.current_text))
    }

    fn clear_collected_text(&self) {
        if let Ok(mut state) = self.collector.lock() {
            state.current_text.clear();
        }
    }
}

/// A running ACP session actor.
pub struct SessionActor {
    name: String,
    cwd: PathBuf,
    session_id: String,
    charter: String,
    connection: ConnectionTo<Agent>,
    cmd_rx: mpsc::UnboundedReceiver<SessionCommand>,
    cmd_tx: mpsc::UnboundedSender<SessionCommand>,
    shared: SharedState,
    persist_tx: mpsc::UnboundedSender<PersistSession>,
    forward_tx: mpsc::UnboundedSender<(String, PromptJob)>,

    allowed_tool_kinds: Vec<ToolKind>,
    mcp_servers: Vec<McpServer>,
    acp_mcp_servers: Vec<agent_client_protocol::schema::v1::McpServer>,
    close_supported: bool,

    job_queue: VecDeque<Job>,
    current_prompt: Option<(Job, oneshot::Receiver<anyhow::Result<PromptResult>>)>,

    cron_worker: Option<CronWorker>,
    cron_tx: mpsc::UnboundedSender<CronCommand>,
    cron_watch_rx: watch::Receiver<HashMap<String, crate::compositor::state::CronJobState>>,
    pub cancel_token: CancellationToken,
}

impl SessionActor {
    /// Connect a session actor over the provided ACP transport.
    ///
    /// The returned handle is immediately usable, but the session will not
    /// restore persisted state or start cron scheduling until it receives a
    /// `Start` command.
    #[allow(clippy::too_many_arguments)]
    pub async fn connect<T>(
        name: &str,
        cwd: PathBuf,
        charter: String,
        allowed_tool_kinds: Vec<ToolKind>,
        load_session_id: Option<String>,
        mcp_servers: Vec<McpServer>,
        transport: T,
        persist_tx: mpsc::UnboundedSender<PersistSession>,
        forward_tx: mpsc::UnboundedSender<(String, PromptJob)>,
        cancel_token: CancellationToken,
    ) -> anyhow::Result<Self>
    where
        T: agent_client_protocol::ConnectTo<Client> + Send + 'static,
    {
        let name = name.to_string();
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<SessionCommand>();
        let (ready_tx, ready_rx) = oneshot::channel::<Self>();

        let acp_mcp_servers = convert_mcp_servers(&mcp_servers);
        let (broadcast_tx, _) = broadcast::channel::<SessionNotification>(256);
        let shared = SharedState::new(name.clone(), broadcast_tx.clone());
        let shared_for_notification = shared.clone();
        let allowed_for_permission = allowed_tool_kinds.clone();

        let client = Client
            .builder()
            .on_receive_notification(async move |notification: SessionNotification, _cx| {
                let shared = shared_for_notification.clone();
                shared.handle_notification(notification).await;
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
            )
            .on_receive_request(async move |
                request: RequestPermissionRequest,
                responder: Responder<RequestPermissionResponse>,
                _connection
            | {
                let name = name.clone();
                let allowed = allowed_for_permission.clone();
                let outcome = handle_permission_request(&name, &allowed, &request);
                let _ = responder.respond(RequestPermissionResponse::new(outcome));
                Ok(())
            },
            agent_client_protocol::on_receive_request!(),
            ).connect_with(transport, async move |connection: ConnectionTo<Agent>| {
                let shared = shared.clone();
                let persist_tx = persist_tx.clone();
                let forward_tx = forward_tx.clone();
                let cwd = cwd.clone();
                let charter = charter.clone();
                let name = shared.name.clone();
                let allowed_tool_kinds = allowed_tool_kinds.clone();
                let mcp_servers = mcp_servers.clone();
                let acp_mcp_servers = acp_mcp_servers.clone();

                debug!(%name, "initializing agent");
                let init_response: InitializeResponse = connection
                    .send_request(InitializeRequest::new(ProtocolVersion::V1))
                    .block_task()
                    .await?;
                info!(%name, ?init_response.agent_info, "agent initialized");
                info!(%name, ?init_response.agent_capabilities.mcp_capabilities, "agent mcp capabilities");

                let close_supported = init_response
                    .agent_capabilities
                    .session_capabilities
                    .close
                    .is_some();

                let session_id = if let Some(load_id) = load_session_id {
                    debug!(%name, %load_id, "loading session");
                    connection.send_request(
                        LoadSessionRequest::new(load_id.clone(), cwd.clone())
                        .mcp_servers(acp_mcp_servers.clone()),
                    ).block_task().await?;
                    info!(%name, %load_id, "session loaded");
                    load_id
                } else {
                    debug!(%name, "creating session");
                    let new_session: NewSessionResponse = connection.send_request(
                        NewSessionRequest::new(cwd.clone())
                        .mcp_servers(acp_mcp_servers.clone()),
                    ).block_task().await?;
                    let session_id = new_session.session_id.to_string();
                    info!(%name, %session_id, "session created");
                    session_id
                };

                let (worker, cron_tx, cron_watch_rx) = CronWorker::new(
                    name.clone(),
                    HashMap::new(),
                    cmd_tx.clone(),
                );
                let connection_cancel = cancel_token.child_token();

                let _ = ready_tx.send(Self {
                    name,
                    cwd,
                    session_id,
                    charter,
                    connection,
                    cmd_rx,
                    cmd_tx,
                    shared,
                    persist_tx,
                    forward_tx,
                    allowed_tool_kinds,
                    mcp_servers,
                    acp_mcp_servers,
                    close_supported,
                    job_queue: VecDeque::new(),
                    current_prompt: None,

                    cron_worker: Some(worker),
                    cron_tx,
                    cron_watch_rx,
                    cancel_token,
                });

                // Keep the ACP connection handler alive until the actor is
                // dropped or explicitly cancelled.
                connection_cancel.cancelled().await;
                Ok(())
            },
        );

        tokio::spawn(async move {
            let result = client
                .await
                .map_err(|e| anyhow::anyhow!("ACP session failed: {}", e));

            if let Err(e) = result {
                error!(error = %e, "ACP session task ended with error");
            }
        });

        ready_rx
            .await
            .map_err(|_| anyhow::anyhow!("session task closed before ready signal"))
    }

    pub async fn spawn(mut self, initial_state: Option<&SessionState>) -> SessionHandle {
        let handle = self.get_handle();

        if let Some(state) = initial_state {
            self.restore_jobs(state.jobs.clone()).await;
        }
        self.persist();

        let initial_jobs: HashMap<String, crate::compositor::state::CronJobState> = initial_state
            .as_ref()
            .map(|s| {
                s.cron_jobs
                    .iter()
                    .map(|(name, state)| (name.clone(), state.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let Some(worker) = self.cron_worker.take() else {
            error!(%self.name, "cron worker missing during spawn");
            return handle;
        };
        let cron_cancel = self.cancel_token.child_token();
        tokio::spawn(async move { self.run().await });
        tokio::spawn(worker.run(initial_jobs, cron_cancel));
        handle
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                cmd = self.cmd_rx.recv() => {
                    if let Some(cmd) = cmd {
                        if self.handle_command(cmd).await {
                            break;
                        }
                    } else {
                        info!(%self.name, "command channel closed, shutting down session");
                        break;
                    }
                }
                prompt_result = await_current_prompt(&mut self.current_prompt
                ), if self.current_prompt.is_some() => {
                    self.handle_prompt_result(prompt_result).await;
                }
                Ok(()) = self.cron_watch_rx.changed() => {
                    self.persist();
                }
            }
        }
        info!(%self.name, "session actor loop ended");
        self.cancel_token.cancel();
    }

    /// Handle a command. Returns `true` if the actor should stop.
    async fn handle_command(&mut self, cmd: SessionCommand) -> bool {
        match cmd {
            SessionCommand::Prompt {
                prompt_id,
                content,
                cron_job_name,
                send_result_to,
            } => {
                self.handle_prompt(prompt_id, content, cron_job_name, send_result_to)
                    .await;
            }
            SessionCommand::Cancel { prompt_id } => self.handle_cancel(&prompt_id).await,
            SessionCommand::CancelCurrent => self.handle_cancel_current().await,
            SessionCommand::Cron(cmd) => {
                self.handle_cron(cmd);
            }
            SessionCommand::GetHistory { response_tx } => {
                let guard = self.shared.history.read().await;
                let _ = response_tx.send(guard.to_vec());
            }
            SessionCommand::Recreate {
                extra_charter,
                respond_to,
            } => {
                self.handle_recreate(extra_charter, respond_to).await;
            }
            SessionCommand::Persist => {
                self.persist();
            }
            SessionCommand::Shutdown { done_tx } => {
                self.shutdown(done_tx).await;
                return true;
            }
        }
        false
    }

    async fn handle_prompt(
        &mut self,
        prompt_id: String,
        content: String,
        cron_job_name: Option<String>,
        send_result_to: Option<String>,
    ) {
        self.job_queue.push_back(Job {
            prompt_id,
            content,
            send_result_to,
            cron_job_name,
            created_at: Utc::now(),
        });
        self.persist();
        self.start_next_job().await;
    }

    async fn handle_cancel(&mut self, prompt_id: &str) {
        info!(%self.name, %prompt_id, "cancelling prompt");
        let was_current = self
            .current_prompt
            .as_ref()
            .is_some_and(|(job, _)| job.prompt_id == prompt_id);
        if was_current {
            self.cancel_current_prompt().await;
        } else if let Some(pos) = self.job_queue.iter().position(|j| j.prompt_id == prompt_id) {
            self.job_queue.remove(pos);
        }
    }

    async fn handle_cancel_current(&mut self) {
        info!(%self.name, "cancelling current prompt");
        self.cancel_current_prompt().await;
    }

    async fn cancel_current_prompt(&mut self) {
        if self.current_prompt.take().is_some() {
            if let Err(e) = self
                .connection
                .send_notification(CancelNotification::new(self.session_id.clone()))
            {
                warn!(%self.name, error = %e, "failed to send cancel notification");
            }
            self.start_next_job().await;
        }
    }

    async fn handle_recreate(
        &mut self,
        extra_charter: Option<String>,
        respond_to: oneshot::Sender<anyhow::Result<(SessionHandle, Option<String>)>>,
    ) {
        self.job_queue.clear();
        let _ = self.current_prompt.take();

        if self.close_supported {
            debug!(%self.name, "closing session before recreate");
            let _ = self
                .connection
                .send_request(CloseSessionRequest::new(self.session_id.clone()))
                .block_task()
                .await;
        }

        debug!(%self.name, "recreating session");
        let new_session: NewSessionResponse = match self
            .connection
            .send_request(
                NewSessionRequest::new(self.cwd.clone()).mcp_servers(self.acp_mcp_servers.clone()),
            )
            .block_task()
            .await
        {
            Ok(s) => s,
            Err(e) => {
                let _ = respond_to.send(Err(anyhow::anyhow!("recreate failed: {}", e)));
                return;
            }
        };

        self.session_id = new_session.session_id.to_string();
        info!(%self.name, session_id = %self.session_id, "session recreated");

        // The new ACP session has no history, so re-send the charter just like
        // create_session does. If extra_charter is provided, append it to a
        // temporary copy so the stored base charter remains unchanged.
        let prompt_content = match extra_charter.filter(|e| !e.is_empty()) {
            Some(extra) => format!("{}\n\n{}", self.charter, extra),
            None => self.charter.clone(),
        };

        let prompt_id = format!("recreate-{}", uuid_for_prompt_id());
        let charter_prompt_id = Some(prompt_id.clone());
        self.job_queue.push_back(Job {
            prompt_id,
            content: prompt_content,
            send_result_to: None,
            cron_job_name: None,
            created_at: Utc::now(),
        });

        let _ = respond_to.send(Ok((self.get_handle(), charter_prompt_id)));
        self.persist();
        self.start_next_job().await;
    }

    fn get_handle(&self) -> SessionHandle {
        SessionHandle {
            name: self.name.clone(),
            cwd: self.cwd.clone(),
            session_id: self.session_id.clone(),
            cmd_tx: self.cmd_tx.clone(),
            broadcast_tx: self.shared.broadcast_tx.clone(),
        }
    }

    async fn shutdown(&mut self, done_tx: Option<oneshot::Sender<()>>) {
        self.job_queue.clear();
        if self.close_supported {
            debug!(%self.name, "closing session before shutdown");
            let _ = self
                .connection
                .send_request(CloseSessionRequest::new(self.session_id.clone()))
                .block_task()
                .await;
        }
        info!(%self.name, "shutting down session");
        self.cancel_token.cancel();
        if let Some(tx) = done_tx {
            let _ = tx.send(());
        }
    }

    async fn handle_prompt_result(
        &mut self,
        prompt_result: anyhow::Result<anyhow::Result<PromptResult>>,
    ) {
        info!(%self.name, ?prompt_result, "prompt result returned");

        let Some((job, _)) = self.current_prompt.take() else {
            return;
        };

        let collected_text = match self.shared.take_collected_text() {
            Ok(text) => text,
            Err(e) => {
                warn!(%self.name, error = %e, "failed to take collected text");
                self.start_next_job().await;
                return;
            }
        };

        let result: anyhow::Result<PromptResult> =
            prompt_result.and_then(|inner| inner).map(|mut pr| {
                pr.text = collected_text;
                pr
            });

        if let Some(target) = job.send_result_to.as_ref().filter(|_| result.is_ok())
            && let Err(e) = self.forward_result(&result, target, job.cron_job_name.as_deref())
        {
            warn!(%self.name, error = %e, "failed to forward prompt result");
        }

        self.persist();
        self.start_next_job().await;
    }

    async fn start_next_job(&mut self) {
        if self.current_prompt.is_some() || self.job_queue.is_empty() {
            return;
        }
        let Some(job) = self.job_queue.pop_front() else {
            return;
        };
        info!(
            %self.name,
            prompt_id = %job.prompt_id,
            content = %job.content,
            "sending prompt"
        );

        let synthetic = SessionNotification::new(
            self.session_id.clone(),
            SessionUpdate::UserMessageChunk(
                ContentChunk::new(ContentBlock::Text(TextContent::new(job.content.clone())))
                    .message_id(job.prompt_id.as_str()),
            ),
        );
        {
            let mut h = self.shared.history.write().await;
            h.push(synthetic.clone());
            info!(
                %self.name,
                history_len = h.len(),
                "synthetic user_message_chunk saved"
            );
        }
        let _ = self.shared.broadcast_tx.send(synthetic);

        self.shared.clear_collected_text();

        let session_id = self.session_id.clone();
        let content = job.content.clone();
        let connection = self.connection.clone();
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            let result = connection
                .send_request(PromptRequest::new(
                    session_id,
                    vec![ContentBlock::Text(TextContent::new(content))],
                ))
                .block_task()
                .await
                .map(|response| PromptResult {
                    stop_reason: response.stop_reason,
                    text: String::new(), // filled in from the collector once the prompt finishes
                })
                .map_err(|e| anyhow::anyhow!("prompt failed: {}", e));
            let _ = tx.send(result);
        });

        self.current_prompt = Some((job, rx));
        self.persist();
    }

    async fn restore_jobs(&mut self, jobs: Vec<PromptJob>) {
        const CONTINUE_MESSAGE: &str = "сессия была перезапущена, продолжай";

        let mut pending: Vec<PromptJob> = Vec::new();
        let mut queued: Vec<PromptJob> = Vec::new();
        for job in jobs {
            match job.status {
                PromptStatus::Pending => pending.push(job),
                PromptStatus::Queued => queued.push(job),
                PromptStatus::Completed | PromptStatus::Error => {}
            }
        }

        // Keep only the earliest pending job as pending; convert extras to queued.
        // Use the creation timestamp to preserve the original prompt order.
        pending.sort_by_key(|j| j.created_at);
        if pending.len() > 1 {
            for mut job in pending.split_off(1) {
                job.status = PromptStatus::Queued;
                queued.push(job);
            }
        }

        for job in pending {
            self.job_queue.push_back(Job {
                prompt_id: format!("restored-{}", uuid_for_prompt_id()),
                content: CONTINUE_MESSAGE.to_string(),
                send_result_to: job.send_result_to,
                cron_job_name: job.cron_job_name,
                created_at: job.created_at,
            });
        }
        for job in queued {
            self.job_queue.push_back(Job {
                prompt_id: format!("restored-{}", uuid_for_prompt_id()),
                content: job.content,
                send_result_to: job.send_result_to,
                cron_job_name: job.cron_job_name,
                created_at: job.created_at,
            });
        }

        self.start_next_job().await;
    }

    fn persist(&self) {
        let cron_jobs = self.cron_watch_rx.borrow().clone();
        let mut jobs: Vec<PromptJob> = self
            .job_queue
            .iter()
            .map(|j| {
                let mut pj = j.to_prompt_job(PromptStatus::Queued, None, None);
                pj.target.clone_from(&self.name);
                pj
            })
            .collect();
        if let Some((job, _)) = &self.current_prompt {
            let mut pj = job.to_prompt_job(PromptStatus::Pending, None, None);
            pj.target.clone_from(&self.name);
            jobs.push(pj);
        }
        let state = SessionState {
            session_id: self.session_id.clone(),
            cwd: self.cwd.clone(),
            charter: Some(self.charter.clone()),
            allowed_tool_kinds: self.allowed_tool_kinds.clone(),
            mcp_servers: self.mcp_servers.clone(),
            cron_jobs,
            jobs,
        };
        let _ = self.persist_tx.send(PersistSession {
            name: self.name.clone(),
            state: Some(state),
        });
    }

    fn forward_result(
        &self,
        result: &anyhow::Result<PromptResult>,
        target: &str,
        cron_job_name: Option<&str>,
    ) -> anyhow::Result<()> {
        let text = match result {
            Ok(pr) => pr.text.clone(),
            Err(_) => return Ok(()),
        };
        let pj = PromptJob {
            target: target.to_string(),
            content: format!("Message from agent '{}':\n\n{}", self.name, text),
            status: PromptStatus::Queued,
            send_result_to: None,
            cron_job_name: cron_job_name.map(ToString::to_string),
            result: None,
            error: None,
            created_at: Utc::now(),
        };
        self.forward_tx
            .send((target.to_string(), pj))
            .map_err(|e| anyhow::anyhow!("forward channel closed: {}", e))
    }

    fn handle_cron(&self, cmd: CronCommand) {
        let _ = self.cron_tx.send(cmd);
    }
}

async fn await_current_prompt(
    current: &mut Option<(Job, oneshot::Receiver<anyhow::Result<PromptResult>>)>,
) -> anyhow::Result<anyhow::Result<PromptResult>> {
    if let Some((_, rx)) = current.as_mut() {
        rx.await
            .map_err(|e| anyhow::anyhow!("prompt result receiver dropped: {}", e))
    } else {
        Ok(Err(anyhow::anyhow!("no current prompt")))
    }
}

fn convert_mcp_servers(servers: &[McpServer]) -> Vec<agent_client_protocol::schema::v1::McpServer> {
    servers
        .iter()
        .cloned()
        .map(|s| match s.transport {
            McpServerTransport::Http => agent_client_protocol::schema::v1::McpServer::Http(
                McpServerHttp::new(s.name, s.url),
            ),
            McpServerTransport::Sse => {
                agent_client_protocol::schema::v1::McpServer::Sse(McpServerSse::new(s.name, s.url))
            }
            McpServerTransport::Stdio => {
                let cmd = PathBuf::from(s.command.clone().unwrap_or_default());
                let args = s.args.clone();
                agent_client_protocol::schema::v1::McpServer::Stdio(
                    McpServerStdio::new(s.name, cmd).args(args),
                )
            }
        })
        .collect()
}

fn uuid_for_prompt_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn handle_permission_request(
    name: &str,
    allowed: &[ToolKind],
    request: &RequestPermissionRequest,
) -> RequestPermissionOutcome {
    let kind = request.tool_call.fields.kind.unwrap_or(ToolKind::Other);
    let permitted = allowed.is_empty() || allowed.contains(&kind);
    if permitted {
        let option_id = request.options.first().map(|opt| opt.option_id.clone());
        option_id.map_or_else(
            || {
                warn!(%name, ?kind, "permission request has no options; cancelling");
                RequestPermissionOutcome::Cancelled
            },
            |id| {
                info!(%name, ?kind, "approving permission request");
                RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(id))
            },
        )
    } else {
        warn!(%name, ?kind, "denying permission request");
        RequestPermissionOutcome::Cancelled
    }
}
