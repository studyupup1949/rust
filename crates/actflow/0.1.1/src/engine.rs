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

const PROCESS_CACHE_SIZE: usize = 2048;
const PROCESS_COMPLETE_QUEUE_SIZE: usize = 100;

pub struct Engine {
    channel: Arc<Channel>,
    store: Arc<Store>,
    monitor: Monitor,
    procs_complete_queue: Arc<Queue<ProcessId>>,
    procs: Arc<MemCache<ProcessId, Arc<Process>>>,

    running: Arc<AtomicBool>,
    runtime: Arc<Runtime>,
    shutdown: Arc<Shutdown>,
}

impl Engine {
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

    pub fn get_process(
        &self,
        process_id: &String,
    ) -> Option<Arc<Process>> {
        self.procs.get(process_id)
    }

    pub fn channel(&self) -> Arc<Channel> {
        self.channel.clone()
    }
}
