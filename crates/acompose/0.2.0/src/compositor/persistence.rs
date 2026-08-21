use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::error;

use crate::compositor::state::{PersistSession, State, StateStore};

/// Actor that handles asynchronous persistence of [`State`].
pub struct PersistenceActor {
    state_store: Arc<dyn StateStore>,
    rx: mpsc::UnboundedReceiver<PersistSession>,
    snapshot: State,
}

impl PersistenceActor {
    /// Create a new persistence actor and return the sender half of its channel.
    pub fn new(state_store: Arc<dyn StateStore>) -> (Self, mpsc::UnboundedSender<PersistSession>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self {
                state_store,
                rx,
                snapshot: State::default(),
            },
            tx,
        )
    }

    /// Run the actor, saving every received state until cancelled.
    pub async fn run(mut self, cancel: CancellationToken) {
        self.snapshot = self.state_store.load().await.unwrap_or_default();
        loop {
            tokio::select! {
                Some(msg) = self.rx.recv() => {
                    match msg.state {
                        Some(state) => {
                            self.snapshot.sessions.insert(msg.name, state);
                        }
                        None => {
                            self.snapshot.sessions.remove(&msg.name);
                        }
                    }
                    if let Err(e) = self.state_store.save(&self.snapshot).await {
                        error!(error = %e, "failed to save state");
                    }
                }
                () = cancel.cancelled() => {
                    break;
                }
            }
        }
    }
}
