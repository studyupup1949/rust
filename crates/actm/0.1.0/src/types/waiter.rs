//! Utility type for waiting on an event to happen, possibly across a sync/async boundary.

use flume::{Receiver, Sender};

/// Utility type for waiting on an event to happen, possibly across a sync/async boundary.
pub struct Waiter {
    /// Internal channel
    recv: Receiver<()>,
}

/// Utility type for signaling an event has happend, possibly across a sync/async boundary.
pub struct Trigger {
    /// Internal channel
    send: Sender<()>,
}

impl Waiter {
    /// Create a new `Waiter`/`Trigger` pair
    pub fn new() -> (Waiter, Trigger) {
        let (send, recv) = flume::bounded(1);
        (Waiter { recv }, Trigger { send })
    }

    /// Asynchronously wait for the event, returning `true` if the event happened, or `false` if the
    /// `Trigger` was dropped before completion
    pub async fn wait(self) -> bool {
        self.recv.recv_async().await.is_ok()
    }

    /// Synchronously wait for the event, returning `true` if the event happened, or `false` if the
    /// `Trigger` was dropped before completion
    pub fn wait_sync(self) -> bool {
        self.recv.recv().is_ok()
    }
}

impl Trigger {
    /// Consume the trigger and signal the waiter
    pub fn trigger(self) {
        let _ = self.send.send(());
    }
}
