//! Workflow engine - the main entry point for Actflow.
//!
//! The engine manages the lifecycle of workflows and processes, including:
//! - Creating and running process instances
//! - Graceful shutdown coordination

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::runtime::Runtime;

use crate::{
    ActflowError, ChannelEvent, ChannelOptions, Result,
    common::{MemCache, Queue, Shutdown},
    model::WorkflowModel,
    runtime::{Channel, Process, ProcessId},
};

/// Maximum number of processes to cache in memory.
const PROCESS_CACHE_SIZE: usize = 2048;
/// Size of the queue for completed process notifications.
const PROCESS_COMPLETE_QUEUE_SIZE: usize = 100;

/// The main workflow engine.
///
/// Engine is the central coordinator for Actflow, responsible for:
/// - Managing the tokio runtime for async execution
/// - Coordinating the event channel for pub/sub messaging
pub struct Engine {
    /// Event channel for broadcasting workflow events.
    channel: Arc<Channel>,
    /// Queue for receiving process completion notifications.
    procs_complete_queue: Arc<Queue<ProcessId>>,
    /// In-memory cache of active processes.
    procs: Arc<MemCache<ProcessId, Arc<Process>>>,

    /// Flag indicating if the engine is running.
    running: Arc<AtomicBool>,
    /// Tokio runtime for async task execution.
    runtime: Arc<Runtime>,
    /// Shutdown coordinator for graceful termination.
    shutdown: Arc<Shutdown>,
}

impl Engine {
    /// Creates a new engine.
    pub fn new(runtime: Arc<Runtime>) -> Self {
        let channel = Arc::new(Channel::new(runtime.clone()));

        let procs_complete_queue = Queue::new(PROCESS_COMPLETE_QUEUE_SIZE);

        Self {
            channel,
            procs_complete_queue,
            procs: Arc::new(MemCache::new(PROCESS_CACHE_SIZE)),
            running: Arc::new(AtomicBool::new(false)),
            runtime,
            shutdown: Arc::new(Shutdown::new()),
        }
    }

    /// Starts the engine and begins processing events.
    ///
    /// This method:
    /// - Begins listening on the event channel
    /// - Spawns a background task to clean up completed processes
    pub fn launch(&self) {
        if self.running.swap(true, Ordering::Relaxed) {
            return;
        }

        self.channel.listen();

        // Process complete queue
        let procs_complete_queue = self.procs_complete_queue.clone();
        ChannelEvent::channel(self.channel.clone(), ChannelOptions::default()).on_complete(move |pid| {
            let _ = procs_complete_queue.send(pid);
        });

        let procs_complete_queue = self.procs_complete_queue.clone();
        let shutdown = self.shutdown.clone();
        let procs = self.procs.clone();
        self.runtime.spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.wait() => break,
                    Some(pid) = procs_complete_queue.next_async() => {
                        procs.remove(&pid);
                    }
                }
            }
        });
    }

    /// Gracefully shuts down the engine.
    ///
    /// This method:
    /// - Signals all components to stop
    /// - Aborts all running processes
    /// - Shuts down the event channel
    pub fn shutdown(&self) {
        if self.running.swap(false, Ordering::Relaxed) {
            return;
        }

        self.shutdown.shutdown();
        for (_, proc) in self.procs.iter() {
            proc.abort();
        }
        self.channel.shutdown();
    }

    /// Creates a new process instance from a workflow model.
    pub fn build_workflow_process(
        &self,
        workflow: &WorkflowModel,
    ) -> Result<Arc<Process>> {
        if !self.running.load(Ordering::Relaxed) {
            return Err(ActflowError::Engine("Engine is not running".to_string()));
        }
        // Create a new process
        let process = Process::new(workflow, self.channel.clone(), self.runtime.clone())?;
        let process_id = process.id().to_string();

        // Check if process already exists in cache
        if self.procs.get(&process_id).is_some() {
            return Err(ActflowError::Process(format!("Process {} already exists in cache", process_id)));
        }
        // Add process to cache first (before starting)
        self.procs.set(process_id.clone(), process.clone());

        Ok(process)
    }

    /// Stops a running process by its id.
    pub fn stop(
        &self,
        process_id: &str,
    ) -> Result<()> {
        let process_id_string = process_id.to_string();
        if let Some(process) = self.procs.get(&process_id_string) {
            process.abort();
            Ok(())
        } else {
            Err(ActflowError::Process(format!("Process {} not found", process_id)))
        }
    }

    /// Gets a process by its id from the cache.
    pub fn get_process(
        &self,
        process_id: &String,
    ) -> Option<Arc<Process>> {
        self.procs.get(process_id)
    }

    /// Returns a reference to the event channel.
    pub fn channel(&self) -> Arc<Channel> {
        self.channel.clone()
    }
}
