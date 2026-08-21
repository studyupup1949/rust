//! Workflow dispatcher for scheduling and executing nodes.
//!
//! The dispatcher is responsible for:
//! - Traversing the workflow graph
//! - Scheduling nodes for execution when dependencies are met
//! - Handling node results and determining next steps
//! - Managing retries and timeouts

use std::{sync::Arc, time::Duration};

use tokio::{runtime::Runtime, sync::mpsc};

use crate::{
    common::{Queue, Shutdown, Vars},
    events::{ErrorReason, Event, GraphEvent, Message, NodeEvent, WorkflowAbortedEvent, WorkflowEvent, WorkflowFailedEvent, WorkflowStartEvent},
    runtime::{Context, WorkflowCommand},
    utils,
    workflow::{
        Workflow,
        actions::{ActionOutput, ActionType},
        consts::{IF_ELSE_FALSE, IF_ELSE_SELECTED, IF_ELSE_TRUE},
        edge::{EdgeSelectOptions, FixedHandle, SourceHandle},
        node::{NodeExecutionStatus, NodeId, NodeResult},
    },
};

/// Workflow execution dispatcher.
///
/// The dispatcher manages the execution of a workflow by:
/// - Processing commands (Start, Abort)
/// - Spawning node execution tasks
/// - Handling node completion and scheduling successors
/// - Managing conditional branching (if_else nodes)
pub struct Dispatcher {
    /// Execution context with environment and outputs.
    ctx: Arc<Context>,
    /// The workflow graph to execute.
    workflow: Arc<Workflow>,
    /// Queue for receiving workflow commands.
    command_queue: Arc<Queue<WorkflowCommand>>,
    /// Tokio runtime for spawning tasks.
    runtime: Arc<Runtime>,
    /// Shutdown coordinator.
    shutdown: Arc<Shutdown>,
}

impl Dispatcher {
    /// Creates a new dispatcher for the given workflow.
    pub fn new(
        ctx: Arc<Context>,
        workflow: Arc<Workflow>,
        command_queue: Arc<Queue<WorkflowCommand>>,
        runtime: Arc<Runtime>,
    ) -> Self {
        Self {
            ctx,
            workflow,
            command_queue,
            runtime,
            shutdown: Arc::new(Shutdown::new()),
        }
    }

    /// Starts the dispatcher's main event loop.
    ///
    /// The loop processes:
    /// - Workflow commands (Start, Abort)
    /// - Node execution results
    pub fn start(&self) {
        // Internal channel for worker task completion events
        let (tx, mut rx) = mpsc::channel::<(NodeId, NodeEvent)>(1024);

        let ctx = self.ctx.clone();
        let workflow = self.workflow.clone();
        let command_queue = self.command_queue.clone();
        let runtime = self.runtime.clone();
        let shutdown = self.shutdown.clone();

        self.runtime.spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.wait() => break,

                    // Handle node execution results
                    Some((nid, event)) = rx.recv() => {
                        // Publish node event to external channel
                        let _ = ctx.channel().event_queue().send(Event::new(&Message {
                            pid: ctx.pid(),
                            nid: nid.clone(),
                            event: GraphEvent::Node(event.clone()),
                        })).unwrap();

                        match event {
                            NodeEvent::Succeeded(_) => {
                                Self::handle_node_success(&ctx, &workflow, &runtime, &tx, nid).await;
                            }
                            NodeEvent::Error(err) => {
                                // Send workflow failed event
                                let _ = ctx.channel().event_queue().send(Event::new(&Message {
                                    pid: ctx.pid(),
                                    nid: nid.clone(),
                                    event: GraphEvent::Workflow(WorkflowEvent::Failed(WorkflowFailedEvent {
                                        error: err.to_string(),
                                    })),
                                }));
                                // Stop the workflow
                                shutdown.shutdown();
                            }
                            _ => {}
                        }
                    }

                    // Handle workflow commands
                    cmd_opt = command_queue.next_async() => {
                        if let Some(cmd) = cmd_opt {
                            match cmd {
                                WorkflowCommand::Start => {
                                    if let Some(root_node) = workflow.get_root_node() {
                                        // Get all node IDs for batch initialization
                                        let node_ids = workflow.get_all_node_ids();

                                        let _ = ctx.channel().event_queue().send(Event::new(&Message {
                                            pid: ctx.pid(),
                                            nid: "".to_string(),
                                            event: GraphEvent::Workflow(WorkflowEvent::Start(WorkflowStartEvent {
                                                node_ids,
                                            })),
                                        }));

                                        Self::spawn_node(&ctx, &workflow, &runtime, &tx, root_node.id);
                                    }
                                }
                                WorkflowCommand::Abort => {
                                     let _ = ctx.channel().event_queue().send(Event::new(&Message {
                                         pid: ctx.pid(),
                                         nid: "".to_string(),
                                         event: GraphEvent::Workflow(WorkflowEvent::Aborted(WorkflowAbortedEvent {
                                             reason: "Aborted by command".to_string(),
                                             outputs: std::collections::HashMap::new(),
                                         })),
                                     }));
                                     shutdown.shutdown();
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    /// Stops the dispatcher.
    pub fn stop(&self) {
        self.shutdown.shutdown();
    }

    /// Returns all node outputs collected during execution.
    pub fn outputs(&self) -> Vars {
        let mut result = Vars::new();
        for (nid, vars) in self.ctx.outputs().iter() {
            result.set(nid.as_str(), vars.clone());
        }
        result
    }

    /// Checks if the dispatcher has completed execution.
    pub fn is_complete(&self) -> bool {
        self.shutdown.is_terminated()
    }

    /// Spawns a node for execution in a separate task.
    fn spawn_node(
        ctx: &Arc<Context>,
        workflow: &Arc<Workflow>,
        runtime: &Arc<Runtime>,
        tx: &mpsc::Sender<(NodeId, NodeEvent)>,
        nid: NodeId,
    ) {
        let ctx = ctx.clone();
        let workflow = workflow.clone();
        let tx = tx.clone();

        workflow.mark_node_taken(&nid);

        runtime.spawn(async move {
            let result = Self::execute_node(ctx, workflow, nid.clone()).await;
            let _ = tx.send((nid, result)).await;
        });
    }

    async fn handle_node_success(
        ctx: &Arc<Context>,
        workflow: &Arc<Workflow>,
        runtime: &Arc<Runtime>,
        tx: &mpsc::Sender<(NodeId, NodeEvent)>,
        nid: NodeId,
    ) {
        workflow.mark_node_executed(&nid);

        let mut edge_select_options = EdgeSelectOptions::default();

        // Handle if-else node: determine selected branch and skip others
        let node = workflow.get_node(&nid).unwrap(); // should be safe
        if node.action.action_type() == ActionType::IfElse {
            if let Some(outputs) = ctx.outputs().get(&nid) {
                if let Some(source_handle) = outputs.get::<String>(IF_ELSE_SELECTED) {
                    // Determine the selected source handle
                    let selected_handle = if source_handle == IF_ELSE_TRUE {
                        SourceHandle::Fixed(FixedHandle::True)
                    } else if source_handle == IF_ELSE_FALSE {
                        SourceHandle::Fixed(FixedHandle::False)
                    } else {
                        SourceHandle::Node(source_handle)
                    };

                    edge_select_options.source_handle = selected_handle.clone();

                    // Skip unselected branches and send events
                    let skipped = workflow.skip_unselected_branches(&nid, &selected_handle);
                    for (skipped_nid, _) in skipped {
                        let _ = ctx.channel().event_queue().send(Event::new(&Message {
                            pid: ctx.pid(),
                            nid: skipped_nid,
                            event: GraphEvent::Node(NodeEvent::Skipped),
                        }));
                    }
                }
            }
        }

        let next_nodes = workflow.get_next_ready_node(&nid, edge_select_options);
        let all_executed = workflow.is_all_node_executed();

        if next_nodes.is_empty() && all_executed {
            let _ = ctx.channel().event_queue().send(Event::new(&Message {
                pid: ctx.pid(),
                nid: nid.clone(),
                event: GraphEvent::Workflow(WorkflowEvent::Succeeded),
            }));
            ctx.done();
        }

        for next_nid in next_nodes {
            Self::spawn_node(ctx, workflow, runtime, tx, next_nid);
        }
    }

    /// Executes a single node logic, including retries and timeout handling.
    /// This function is intended to be spawned as a separate task by the dispatcher.
    async fn execute_node(
        ctx: Arc<Context>,
        workflow: Arc<Workflow>,
        nid: NodeId,
    ) -> NodeEvent {
        let event_queue = ctx.channel().event_queue();

        let node = match workflow.get_node(&nid) {
            Some(n) => Arc::new(n),
            None => {
                return NodeEvent::Error(ErrorReason::Exception(format!("Node {} not found", nid)));
            }
        };

        let mut retry_times = node.retry.as_ref().map(|r| r.times).unwrap_or(0);
        let retry_interval = node.retry.as_ref().map(|r| r.interval).unwrap_or(0);

        // Track start time before action execution (as timestamp)
        let start_time = utils::time::time_millis();

        // Emit Running event
        let _ = event_queue.send(Event::new(&Message {
            pid: ctx.pid(),
            nid: nid.clone(),
            event: GraphEvent::Node(NodeEvent::Running(start_time)),
        }));

        loop {
            let action_ctx = ctx.clone();
            let action_node = node.clone();
            let action_nid = nid.clone();

            let run_future = async move {
                if let Some(timeout) = action_node.timeout {
                    tokio::time::timeout(timeout, async move {
                        action_node.action.run(action_ctx, action_node.id.clone()).await
                    })
                    .await
                } else {
                    Ok(action_node.action.run(action_ctx, action_nid).await)
                }
            };

            let ret = tokio::select! {
                _ = ctx.wait_shutdown() => return NodeEvent::Stopped(utils::time::time_millis()),
                res = run_future => res,
            };

            // Track end time after action execution (as timestamp)
            let end_time = utils::time::time_millis();

            // Convert ActionOutput to NodeResult
            let node_result = match &ret {
                Ok(output) => NodeResult::from_result_output(output.clone()),
                Err(_) => NodeResult::from_output(ActionOutput::failed("Timeout".to_string())),
            };

            let should_retry = match node_result.status {
                NodeExecutionStatus::Failed => true,
                _ => false,
            };

            if should_retry && retry_times > 0 {
                retry_times -= 1;
                if retry_interval > 0 {
                    tokio::select! {
                        _ = ctx.wait_shutdown() => return NodeEvent::Stopped(utils::time::time_millis()),
                        _ = tokio::time::sleep(Duration::from_millis(retry_interval)) => {}
                    }
                }
                let _ = event_queue.send(Event::new(&Message {
                    pid: ctx.pid(),
                    nid: nid.clone(),
                    event: GraphEvent::Node(NodeEvent::Retry),
                }));
                continue;
            }

            return match node_result.status {
                NodeExecutionStatus::Pending => unreachable!(),
                NodeExecutionStatus::Succeeded => {
                    ctx.add_output(nid.clone(), node_result.outputs);
                    NodeEvent::Succeeded(end_time)
                }
                NodeExecutionStatus::Failed => NodeEvent::Error(ErrorReason::Failed(node_result.error.unwrap_or_default())),
                NodeExecutionStatus::Exception => NodeEvent::Error(ErrorReason::Exception(node_result.exception.unwrap_or_default())),
                NodeExecutionStatus::Stopped => NodeEvent::Stopped(end_time),
                NodeExecutionStatus::Paused => NodeEvent::Paused(end_time),
            };
        }
    }
}
