use std::sync::Arc;

use tokio::runtime::Runtime;

use crate::{
    events::{GraphEvent, NodeEvent, WorkflowEvent},
    runtime::Channel,
    store::{Store, data},
    utils,
};

pub struct Monitor {
    store: Arc<Store>,
    channel: Arc<Channel>,

    runtime: Arc<Runtime>,
}

impl Monitor {
    pub fn new(
        store: Arc<Store>,
        channel: Arc<Channel>,
        runtime: Arc<Runtime>,
    ) -> Self {
        Self {
            store,
            channel,
            runtime,
        }
    }

    pub fn monitor(&self) {
        let store = self.store.clone();
        let channel = self.channel.clone();

        self.runtime.spawn(async move {
            let mut event_queue = channel.event_queue().subscribe();
            while let Ok(event_msg) = event_queue.recv().await {
                let event = &event_msg;
                // 1. Persist raw event
                let _ = store.events().create(&data::Event {
                    id: utils::longid(),
                    pid: event.pid.clone(),
                    nid: event.nid.clone(),
                    name: match &event.event {
                        GraphEvent::Workflow(_) => "Workflow".to_string(),
                        GraphEvent::Node(n) => format!("{:?}", n),
                    },
                    message: format!("{:?}", event.event),
                    timestamp: utils::time::time_millis(),
                });

                // 2. Update Entity State (Proc / Node)
                match &event.event {
                    GraphEvent::Workflow(e) => {
                        // Handle workflow start - batch create all nodes with Pending state
                        if let WorkflowEvent::Start(start_event) = e {
                            let now = utils::time::time_millis();
                            for nid in &start_event.node_ids {
                                let node_data = data::Node {
                                    id: format!("{}-{}", event.pid, nid),
                                    pid: event.pid.clone(),
                                    nid: nid.clone(),
                                    state: "Pending".to_string(),
                                    err: None,
                                    start_time: 0,
                                    end_time: 0,
                                    timestamp: now,
                                };
                                let _ = store.nodes().create(&node_data);
                            }
                        }

                        // Update proc state
                        if let Ok(mut proc_data) = store.procs().find(&event.pid) {
                            proc_data.state = e.str().to_string();
                            proc_data.timestamp = utils::time::time_millis();

                            match e {
                                WorkflowEvent::Succeeded | WorkflowEvent::Failed(_) | WorkflowEvent::Aborted(_) => {
                                    proc_data.end_time = utils::time::time_millis();
                                }
                                _ => {}
                            }

                            if let WorkflowEvent::Failed(f) = e {
                                proc_data.err = Some(f.error.clone());
                            }
                            if let WorkflowEvent::Aborted(a) = e {
                                proc_data.err = Some(a.reason.clone());
                            }

                            let _ = store.procs().update(&proc_data);
                        }
                    }
                    GraphEvent::Node(n) => {
                        let node_id = format!("{}-{}", event.pid, event.nid);
                        let now = utils::time::time_millis();

                        // Get or create node record (handles race condition where Running event
                        // arrives before Start event has been processed)
                        let mut node_data = match store.nodes().find(&node_id) {
                            Ok(data) => data,
                            Err(_) => {
                                // Node doesn't exist yet, create it
                                let new_node = data::Node {
                                    id: node_id.clone(),
                                    pid: event.pid.clone(),
                                    nid: event.nid.clone(),
                                    state: "Pending".to_string(),
                                    err: None,
                                    start_time: 0,
                                    end_time: 0,
                                    timestamp: now,
                                };
                                let _ = store.nodes().create(&new_node);
                                new_node
                            }
                        };

                        node_data.state = n.str().to_string();
                        node_data.timestamp = now;

                        // Update start_time when node starts running
                        if let NodeEvent::Running(timestamp) = n {
                            node_data.start_time = *timestamp;
                        }

                        // Update end_time and use actual times from NodeSuccess
                        if let NodeEvent::Succeeded(timestamp) = n {
                            node_data.end_time = *timestamp;
                        }

                        // Update end_time for stopped states
                        if let NodeEvent::Stopped(timestamp) = n {
                            node_data.end_time = *timestamp;
                        }

                        // Set error message if present
                        if let NodeEvent::Error(e) = n {
                            node_data.err = Some(e.to_string());
                        }

                        let _ = store.nodes().update(&node_data);
                    }
                }
            }
        });

        let store = self.store.clone();
        let channel = self.channel.clone();

        self.runtime.spawn(async move {
            let mut log_queue = channel.log_queue().subscribe();
            while let Ok(log_msg) = log_queue.recv().await {
                let log = &log_msg;
                let _ = store.logs().create(&data::Log {
                    id: utils::longid(),
                    pid: log.pid.clone(),
                    nid: log.nid.clone(),
                    content: log.content.clone(),
                    timestamp: log.timestamp,
                });
            }
        });
    }
}
