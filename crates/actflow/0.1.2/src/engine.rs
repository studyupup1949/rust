//! Workflow engine - the main entry point for Actflow.
//!
//! The engine manages the lifecycle of workflows and processes, including:
//! - Deploying workflow definitions
//! - Creating and running process instances
//! - Managing the event channel and storage
//! - Graceful shutdown coordination

mod monitor;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use tokio::runtime::{Builder, Runtime};

use crate::{
    ActflowError, ChannelEvent, ChannelOptions, Config, Result, StoreType,
    common::{MemCache, Queue, Shutdown},
    model::WorkflowModel,
    runtime::{Channel, Process, ProcessId},
    store::{DbStore, MemStore, PostgresStore, Store, data},
    utils,
};

use monitor::Monitor;

/// Maximum number of processes to cache in memory.
const PROCESS_CACHE_SIZE: usize = 2048;
/// Size of the queue for completed process notifications.
const PROCESS_COMPLETE_QUEUE_SIZE: usize = 100;

/// The main workflow engine.
///
/// Engine is the central coordinator for Actflow, responsible for:
/// - Managing the tokio runtime for async execution
/// - Coordinating the event channel for pub/sub messaging
/// - Storing workflow definitions and process state
/// - Creating and managing process instances
///
/// # Example
///
/// ```rust,ignore
/// let config = Config::default();
/// let engine = Engine::new_with_config(config);
/// engine.launch();
///
/// // Deploy a workflow
/// engine.deploy(&workflow_model)?;
///
/// // Create and run a process
/// let process = engine.build_process("workflow_id")?;
/// let pid = engine.run_process(process)?;
///
/// // Shutdown when done
/// engine.shutdown();
/// ```
pub struct Engine {
    /// Event channel for broadcasting workflow events.
    channel: Arc<Channel>,
    /// Persistent storage for workflows and processes.
    store: Arc<Store>,
    /// Background monitor for event persistence.
    monitor: Monitor,
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
    /// Creates a new engine with the given configuration.
    ///
    /// This initializes:
    /// - The tokio runtime with configured worker threads
    /// - The storage backend (memory or PostgreSQL)
    /// - The event channel and monitor
    pub fn new_with_config(confg: Config) -> Self {
        let runtime = Arc::new(Builder::new_multi_thread().worker_threads(confg.async_worker_thread_number.into()).enable_all().build().unwrap());

        let store = Store::new();
        let db: Box<dyn DbStore> = match confg.store.store_type {
            StoreType::Mem => {
                let mem = MemStore::new();
                Box::new(mem)
            }
            StoreType::Postgres => {
                let postgres = PostgresStore::new(
                    &confg.store.postgres.expect("Postgres configuration is required when store type is Postgres").database_url,
                    runtime.clone(),
                );
                Box::new(postgres)
            }
        };
        db.init(&store);

        let store = Arc::new(store);
        let channel = Arc::new(Channel::new(runtime.clone()));
        let monitor = Monitor::new(store.clone(), channel.clone(), runtime.clone());

        let procs_complete_queue = Queue::new(PROCESS_COMPLETE_QUEUE_SIZE);

        Self {
            channel,
            store,
            monitor,
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
    /// - Starts the event monitor for persistence
    /// - Begins listening on the event channel
    /// - Spawns a background task to clean up completed processes
    pub fn launch(&self) {
        if self.running.swap(true, Ordering::Relaxed) {
            return;
        }

        // Register handlers first, then start listening
        // This ensures no events are missed
        self.monitor.monitor();
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

    /// Deploys a workflow definition to the store.
    ///
    /// The workflow can then be instantiated as processes using `build_process`.
    pub fn deploy(
        &self,
        workflow: &WorkflowModel,
    ) -> Result<bool> {
        self.store.deploy(workflow)
    }

    /// build a process from a workflow
    pub fn build_process(
        &self,
        wid: &str,
    ) -> Result<Arc<Process>> {
        if !self.running.load(Ordering::Relaxed) {
            return Err(ActflowError::Engine("Engine is not running".to_string()));
        }
        // Find the workflow from store
        let workflow_data = self.store.workflows().find(wid)?;

        // Parse the workflow model from stored data
        let workflow_model = WorkflowModel::from_json(&workflow_data.data)?;

        // Create a new process
        let process = Process::new(&workflow_model, self.store.clone(), self.channel.clone(), self.runtime.clone())?;
        let process_id = process.id().to_string();

        // Check if process already exists in cache
        if self.procs.get(&process_id).is_some() {
            return Err(ActflowError::Process(format!("Process {} already exists in cache", process_id)));
        }

        Ok(process)
    }

    /// Run a process and return the process id
    /// Returns the pid first, then starts the process execution
    pub fn run_process(
        &self,
        process: Arc<Process>,
    ) -> Result<String> {
        let process_id = process.id().to_string();
        let workflow_id = process.wid().to_string();

        let proc_data = data::Proc {
            id: process_id.clone(),
            wid: workflow_id.to_string(),
            state: "Pending".to_string(),
            start_time: utils::time::time_millis(),
            end_time: 0,
            err: None,
            timestamp: utils::time::time_millis(),
        };
        self.store.procs().create(&proc_data)?;

        // Add process to cache first (before starting)
        self.procs.set(process_id.clone(), process.clone());

        // Start the process
        process.start();

        Ok(process_id)
    }

    /// Stops a running process by its ID.
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

    /// Gets a process by its ID from the cache.
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
