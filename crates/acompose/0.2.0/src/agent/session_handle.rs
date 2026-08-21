use crate::agent::session_actor::SessionCommand;
use agent_client_protocol::schema::v1::SessionNotification;
use std::path::PathBuf;
use tokio::sync::{broadcast, mpsc, oneshot};

/// A handle to a spawned ACP session actor.
#[derive(Clone, Debug)]
pub struct SessionHandle {
    pub name: String,
    pub cwd: PathBuf,
    pub session_id: String,
    pub cmd_tx: mpsc::UnboundedSender<SessionCommand>,
    pub broadcast_tx: broadcast::Sender<SessionNotification>,
}

impl SessionHandle {
    /// Queue a prompt and return a receiver for its result.
    pub fn send_prompt(&self, prompt_id: &str, content: &str, send_result_to: Option<String>) {
        let _ = self.cmd_tx.send(SessionCommand::Prompt {
            prompt_id: prompt_id.to_string(),
            content: content.to_string(),
            cron_job_name: None,
            send_result_to,
        });
    }

    /// Cancel a queued or in-flight prompt.
    pub fn cancel(&self, prompt_id: &str) -> anyhow::Result<()> {
        self.cmd_tx
            .send(SessionCommand::Cancel {
                prompt_id: prompt_id.to_string(),
            })
            .map_err(|e| anyhow::anyhow!("failed to send cancel command: {}", e))
    }

    /// Cancel whatever prompt is currently in flight, regardless of id.
    pub fn cancel_current(&self) -> anyhow::Result<()> {
        self.cmd_tx
            .send(SessionCommand::CancelCurrent)
            .map_err(|e| anyhow::anyhow!("failed to send cancel current command: {}", e))
    }

    /// Subscribe to live session notifications.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<SessionNotification> {
        self.broadcast_tx.subscribe()
    }

    /// Return a receiver that resolves with the full notification history.
    #[must_use]
    pub fn history(&self) -> oneshot::Receiver<Vec<SessionNotification>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .cmd_tx
            .send(SessionCommand::GetHistory { response_tx: tx });
        rx
    }

    /// Shut the session down gracefully.
    #[must_use]
    pub fn shutdown(&self) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .cmd_tx
            .send(SessionCommand::Shutdown { done_tx: Some(tx) });
        rx
    }
}
