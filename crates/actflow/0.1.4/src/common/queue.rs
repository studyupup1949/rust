//! Message queues for inter-component communication.
//!
//! Provides both point-to-point (Queue) and broadcast (BroadcastQueue) messaging.

use std::sync::Arc;

use tokio::sync::broadcast;

use crate::{ActflowError, Result};

/// Bounded MPMC (multi-producer, multi-consumer) queue.
///
/// Used for command queues where messages should be consumed by exactly one receiver.
/// Backed by flume for high-performance message passing.
#[derive(Clone)]
pub struct Queue<T> {
    receiver: Arc<flume::Receiver<T>>,
    sender: Arc<flume::Sender<T>>,
}

#[allow(unused)]
impl<T> Queue<T> {
    /// create a new queue
    pub fn new(cap: usize) -> Arc<Self> {
        let (tx, rx) = flume::bounded(cap);

        Arc::new(Self {
            receiver: Arc::new(rx),
            sender: Arc::new(tx),
        })
    }

    /// receive a message from the queue
    pub fn next(&self) -> Option<T> {
        self.receiver.recv().ok()
    }

    /// send a message to the queue
    pub fn send(
        &self,
        msg: T,
    ) -> Result<()> {
        self.sender.send(msg).map_err(|e| ActflowError::Queue(e.to_string()))
    }

    /// receive a message from the queue asynchronously
    pub async fn next_async(&self) -> Option<T> {
        self.receiver.recv_async().await.ok()
    }

    /// send a message to the queue asynchronously
    pub async fn send_async(
        &self,
        msg: T,
    ) -> Result<()> {
        self.sender.send_async(msg).await.map_err(|e| ActflowError::Queue(e.to_string()))
    }
}

/// Broadcast queue for one-to-many message distribution.
///
/// Used for event broadcasting where all subscribers receive every message.
/// Backed by tokio's broadcast channel.
#[derive(Clone)]
pub struct BroadcastQueue<T> {
    sender: Arc<broadcast::Sender<T>>,
}

impl<T: Clone> BroadcastQueue<T> {
    /// create a new broadcast queue
    pub fn new(cap: usize) -> Arc<Self> {
        let (tx, _) = broadcast::channel(cap);

        Arc::new(Self {
            sender: Arc::new(tx),
        })
    }

    /// send a message to the queue
    pub fn send(
        &self,
        msg: T,
    ) -> Result<()> {
        self.sender.send(msg).map_err(|e| ActflowError::Queue(e.to_string()))?;
        Ok(())
    }

    /// subscribe to the queue
    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sender.subscribe()
    }
}
