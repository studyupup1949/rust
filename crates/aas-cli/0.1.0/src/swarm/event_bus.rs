use crate::swarm::types::AgentEvent;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

pub struct EventBus {
    tx: broadcast::Sender<AgentEvent>,
    persist: bool,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        EventBus {
            tx,
            persist: false,
        }
    }

    pub fn with_persistence(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        EventBus {
            tx,
            persist: true,
        }
    }

    pub async fn emit(&self, event: AgentEvent) {
        let timestamp = match &event {
            AgentEvent::AgentStarted { timestamp, .. }
            | AgentEvent::AgentStopped { timestamp, .. }
            | AgentEvent::AgentError { timestamp, .. }
            | AgentEvent::IssueDetected { timestamp, .. }
            | AgentEvent::IssueAnalyzed { timestamp, .. }
            | AgentEvent::ActionPlanned { timestamp, .. }
            | AgentEvent::ActionStarted { timestamp, .. }
            | AgentEvent::ActionCompleted { timestamp, .. }
            | AgentEvent::ActionFailed { timestamp, .. }
            | AgentEvent::ActionRolledBack { timestamp, .. }
            | AgentEvent::EscalationNeeded { timestamp, .. }
            | AgentEvent::PredictionMade { timestamp, .. }
            | AgentEvent::Learned { timestamp, .. }
            | AgentEvent::HyperfocusRequest { timestamp, .. }
            | AgentEvent::ContextSwitch { timestamp, .. } => *timestamp,
        };

        if self.persist {
            info!("[EVENT] {:?} at {}", event.tag(), timestamp);
        }

        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.tx.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<AgentEvent> {
        self.tx.clone()
    }

    pub fn new_persisted() -> Arc<Self> {
        Arc::new(EventBus {
            tx: broadcast::channel(4096).0,
            persist: true,
        })
    }

    pub fn new_in_memory() -> Arc<Self> {
        Arc::new(EventBus {
            tx: broadcast::channel(4096).0,
            persist: false,
        })
    }
}

impl AgentEvent {
    fn tag(&self) -> &str {
        match self {
            AgentEvent::AgentStarted { .. } => "agent_started",
            AgentEvent::AgentStopped { .. } => "agent_stopped",
            AgentEvent::AgentError { .. } => "agent_error",
            AgentEvent::IssueDetected { .. } => "issue_detected",
            AgentEvent::IssueAnalyzed { .. } => "issue_analyzed",
            AgentEvent::ActionPlanned { .. } => "action_planned",
            AgentEvent::ActionStarted { .. } => "action_started",
            AgentEvent::ActionCompleted { .. } => "action_completed",
            AgentEvent::ActionFailed { .. } => "action_failed",
            AgentEvent::ActionRolledBack { .. } => "action_rolled_back",
            AgentEvent::EscalationNeeded { .. } => "escalation_needed",
            AgentEvent::PredictionMade { .. } => "prediction_made",
            AgentEvent::Learned { .. } => "learned",
            AgentEvent::HyperfocusRequest { .. } => "hyperfocus_request",
            AgentEvent::ContextSwitch { .. } => "context_switch",
        }
    }
}
