//! Workflow process management.
//!
//! A process represents a running instance of a workflow. It manages
//! the execution lifecycle, including starting, aborting, and collecting outputs.

use std::sync::Arc;

use tokio::runtime::Runtime;

use crate::{
    Result,
    common::{Queue, Vars},
    dispatcher::Dispatcher,
    events::{GraphEvent, WorkflowEvent},
    model::WorkflowModel,
    runtime::{Channel, ChannelOptions, Context, channel::ChannelEvent},
    utils,
    workflow::Workflow,
};

/// Maximum number of commands that can be queued for a process.
const COMMAND_QUEUE_SIZE: usize = 100;

/// Unique identifier for a workflow process instance.
pub type ProcessId = String;

/// Commands that can be sent to control a workflow process.
#[derive(Debug, Clone)]
pub enum WorkflowCommand {
    /// Start the workflow execution.
    Start,
    /// Abort the workflow execution.
    Abort,
}

/// A running instance of a workflow.
///
/// Each process has a unique ID and maintains its own execution state.
/// Multiple processes can run the same workflow definition concurrently.
///
/// # Lifecycle
///
/// 1. Create with `Process::new()`
/// 2. Start execution with `process.start()`
/// 3. Optionally abort with `process.abort()`
/// 4. Check completion with `process.is_complete()`
/// 5. Retrieve results with `process.get_outputs()`
#[derive(Clone)]
pub struct Process {
    /// Unique process identifier.
    id: ProcessId,
    /// Workflow ID this process is running.
    wid: String,
    /// Dispatcher responsible for scheduling node execution.
    dispatcher: Arc<Dispatcher>,
    /// Queue for receiving control commands.
    command_queue: Arc<Queue<WorkflowCommand>>,
    /// Event channel for broadcasting process events.
    channel: Arc<Channel>,
}

impl Process {
    /// Creates a new process instance from a workflow model.
    ///
    /// # Arguments
    ///
    /// * `model` - Workflow definition to execute
    /// * `channel` - Event channel for broadcasting events
    /// * `runtime` - Tokio runtime for async execution
    ///
    /// # Returns
    ///
    /// Returns an `Arc<Process>` on success, or an error if creation fails.
    pub fn new(
        model: &WorkflowModel,
        channel: Arc<Channel>,
        runtime: Arc<Runtime>,
    ) -> Result<Arc<Process>> {
        let pid = utils::longid();

        let workflow = Workflow::try_from(model)?;

        let command_queue = Queue::new(COMMAND_QUEUE_SIZE);

        let ctx = Arc::new(Context::new(pid.to_owned(), channel.clone()));

        // Set environment variables from workflow model
        model.env.iter().for_each(|(k, v)| ctx.env().set(k.clone(), v.clone()));

        let dispatcher = Arc::new(Dispatcher::new(
            ctx.clone(),
            Arc::new(workflow),
            command_queue.clone(),
            runtime.clone(),
        ));

        Ok(Arc::new(Process {
            id: pid,
            wid: model.id.clone(),
            dispatcher,
            command_queue,
            channel,
        }))
    }

    /// Returns the unique process identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the workflow ID this process is running.
    pub fn wid(&self) -> &str {
        &self.wid
    }

    /// Starts the workflow execution.
    ///
    /// This method:
    /// 1. Starts the dispatcher
    /// 2. Registers event handlers for completion/failure/abort
    /// 3. Sends the start command to begin execution
    pub fn start(&self) {
        self.dispatcher.start();

        let dispatcher = self.dispatcher.clone();

        ChannelEvent::channel(self.channel.clone(), ChannelOptions::with_pid(self.id.to_owned())).on_event(move |event| {
            let dispatcher = dispatcher.clone();
            match &event.event {
                GraphEvent::Workflow(e) => match e {
                    WorkflowEvent::Succeeded | WorkflowEvent::Failed(_) | WorkflowEvent::Aborted(_) => {
                        dispatcher.stop();
                    }
                    _ => {}
                },
                _ => {}
            }
        });

        // Send start command to the command queue
        let _ = self.command_queue.send(WorkflowCommand::Start);
    }

    /// Aborts the workflow execution.
    ///
    /// Sends an abort command to gracefully terminate the running workflow.
    pub fn abort(&self) {
        let _ = self.command_queue.send(WorkflowCommand::Abort);
    }

    /// Returns the collected outputs from all executed nodes.
    pub fn get_outputs(&self) -> Vars {
        self.dispatcher.outputs()
    }

    /// Checks if the workflow execution has completed.
    ///
    /// Returns `true` if the workflow has finished (success, failure, or abort).
    pub fn is_complete(&self) -> bool {
        self.dispatcher.is_complete()
    }
}
