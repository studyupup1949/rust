use std::sync::Arc;
use std::sync::atomic::Ordering;

use tonic::Code;

use crate::AdaError;
use crate::client::AuthenticatedChannel;
use crate::proto::signal_event::Signal;
use crate::proto::signal_service_client::SignalServiceClient;
use crate::proto::{
    MemoryActionItemDueSoon, MemoryActionItemOverdue, MemoryCommitmentBroken,
    MemoryCommitmentDueSoon, MemoryContradictionCreated, MemoryContradictionEscalated,
    MemoryIntentionExpired, MemoryIntentionExpiringSoon, MemoryIntentionTriggered, RoutineBroken,
    StreamScopeMode, StreamSignalsRequest, TensionEmerged, TensionEscalated,
    TrajectoryDeteriorated,
};
use crate::subscription::{
    HubControl, PrincipalListenerSet, StreamKind, Unsubscribe, local_principal_id,
};

pub(crate) struct SignalHub {
    control: HubControl,
    client: SignalServiceClient<AuthenticatedChannel>,
    principal_scope: Option<String>,
    memory_commitment_due_soon: PrincipalListenerSet<MemoryCommitmentDueSoon>,
    memory_action_item_due_soon: PrincipalListenerSet<MemoryActionItemDueSoon>,
    memory_commitment_broken: PrincipalListenerSet<MemoryCommitmentBroken>,
    memory_action_item_overdue: PrincipalListenerSet<MemoryActionItemOverdue>,
    memory_intention_triggered: PrincipalListenerSet<MemoryIntentionTriggered>,
    memory_intention_expiring_soon: PrincipalListenerSet<MemoryIntentionExpiringSoon>,
    memory_intention_expired: PrincipalListenerSet<MemoryIntentionExpired>,
    memory_contradiction_created: PrincipalListenerSet<MemoryContradictionCreated>,
    memory_contradiction_escalated: PrincipalListenerSet<MemoryContradictionEscalated>,
    tension_emerged: PrincipalListenerSet<TensionEmerged>,
    tension_escalated: PrincipalListenerSet<TensionEscalated>,
    trajectory_deteriorated: PrincipalListenerSet<TrajectoryDeteriorated>,
    routine_broken: PrincipalListenerSet<RoutineBroken>,
}

impl SignalHub {
    fn start(self: &Arc<Self>) {
        if self
            .control
            .started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            let hub = self.clone();
            tokio::spawn(async move {
                hub.run().await;
            });
        }
    }

    async fn run(self: Arc<Self>) {
        let mut client = self.client.clone();
        let mut cursor = self.control.options.after_event_id.clone();
        let mut attempt = 0;
        loop {
            if self.control.cancellation.is_cancelled() {
                return;
            }
            self.control
                .lifecycle
                .connecting(StreamKind::Signals, attempt + 1);
            let request = StreamSignalsRequest {
                event_types: Vec::new(),
                after_event_id: cursor.clone(),
                replay_limit: self.control.options.replay_limit,
                principal_id: self.principal_scope.clone().unwrap_or_default(),
                scope_mode: if self.principal_scope.is_some() {
                    StreamScopeMode::Principal as i32
                } else {
                    StreamScopeMode::Visible as i32
                },
            };
            let response = tokio::select! {
                () = self.control.cancellation.cancelled() => return,
                response = client.stream_signals(request) => response,
            };
            let mut stream = match response {
                Ok(response) => {
                    self.control.lifecycle.connected(StreamKind::Signals);
                    response.into_inner()
                }
                Err(status) => {
                    attempt += 1;
                    if !self.control.failure(attempt, AdaError::from(status)).await {
                        return;
                    }
                    continue;
                }
            };
            loop {
                let message = tokio::select! {
                    () = self.control.cancellation.cancelled() => return,
                    message = stream.message() => message,
                };
                match message {
                    Ok(Some(envelope)) => {
                        attempt = 0;
                        let principal_id = envelope
                            .signal
                            .as_ref()
                            .and_then(|signal| local_principal_id(&signal.principal_id).ok())
                            .unwrap_or_default()
                            .to_owned();
                        if self.dispatch(envelope.signal) {
                            cursor = envelope.cursor;
                            self.control.lifecycle.cursor(
                                StreamKind::Signals,
                                cursor.clone(),
                                principal_id,
                            );
                        }
                    }
                    Ok(None) => {
                        attempt += 1;
                        let error = AdaError {
                            code: Code::Unavailable,
                            message: "signal stream ended".to_owned(),
                        };
                        if !self.control.failure(attempt, error).await {
                            return;
                        }
                        break;
                    }
                    Err(status) => {
                        attempt += 1;
                        if !self.control.failure(attempt, AdaError::from(status)).await {
                            return;
                        }
                        break;
                    }
                }
            }
        }
    }

    fn dispatch(&self, signal: Option<crate::proto::SignalEvent>) -> bool {
        let Some(signal) = signal else {
            self.control
                .lifecycle
                .protocol_error(StreamKind::Signals, "signal envelope is missing signal");
            return false;
        };
        let principal_id = if self.principal_scope.is_some() {
            "*"
        } else {
            match local_principal_id(&signal.principal_id) {
                Ok(principal_id) => principal_id,
                Err(error) => {
                    self.control
                        .lifecycle
                        .protocol_error(StreamKind::Signals, &error.message);
                    return false;
                }
            }
        };
        let Some(payload) = signal.signal else {
            self.control.lifecycle.protocol_error(
                StreamKind::Signals,
                "signal envelope has an unknown variant",
            );
            return false;
        };
        match payload {
            Signal::MemoryCommitmentDueSoon(value) => self.emit(
                &self.memory_commitment_due_soon,
                principal_id,
                "memory.commitment.due_soon",
                value,
            ),
            Signal::MemoryActionItemDueSoon(value) => self.emit(
                &self.memory_action_item_due_soon,
                principal_id,
                "memory.action_item.due_soon",
                value,
            ),
            Signal::MemoryCommitmentBroken(value) => self.emit(
                &self.memory_commitment_broken,
                principal_id,
                "memory.commitment.broken",
                value,
            ),
            Signal::MemoryActionItemOverdue(value) => self.emit(
                &self.memory_action_item_overdue,
                principal_id,
                "memory.action_item.overdue",
                value,
            ),
            Signal::MemoryIntentionTriggered(value) => self.emit(
                &self.memory_intention_triggered,
                principal_id,
                "memory.intention.triggered",
                value,
            ),
            Signal::MemoryIntentionExpiringSoon(value) => self.emit(
                &self.memory_intention_expiring_soon,
                principal_id,
                "memory.intention.expiring_soon",
                value,
            ),
            Signal::MemoryIntentionExpired(value) => self.emit(
                &self.memory_intention_expired,
                principal_id,
                "memory.intention.expired",
                value,
            ),
            Signal::MemoryContradictionCreated(value) => self.emit(
                &self.memory_contradiction_created,
                principal_id,
                "memory.contradiction.created",
                value,
            ),
            Signal::MemoryContradictionEscalated(value) => self.emit(
                &self.memory_contradiction_escalated,
                principal_id,
                "memory.contradiction.escalated",
                value,
            ),
            Signal::TensionEmerged(value) => self.emit(
                &self.tension_emerged,
                principal_id,
                "tension.emerged",
                value,
            ),
            Signal::TensionEscalated(value) => self.emit(
                &self.tension_escalated,
                principal_id,
                "tension.escalated",
                value,
            ),
            Signal::TrajectoryDeteriorated(value) => self.emit(
                &self.trajectory_deteriorated,
                principal_id,
                "trajectory.deteriorated",
                value,
            ),
            Signal::RoutineBroken(value) => {
                self.emit(&self.routine_broken, principal_id, "routine.broken", value)
            }
        }
        true
    }

    fn emit<T: Clone + Send + 'static>(
        &self,
        listeners: &PrincipalListenerSet<T>,
        principal_id: &str,
        event_name: &str,
        value: T,
    ) {
        listeners.emit_with(principal_id, value, |error| {
            self.control.lifecycle.listener_error(
                StreamKind::Signals,
                event_name,
                principal_id,
                error,
            );
        });
    }
}

#[derive(Clone)]
pub struct PrincipalSignals {
    principal_id: Arc<str>,
    hub: Arc<SignalHub>,
}

macro_rules! signal_handler {
    ($name:ident, $field:ident, $payload:ty) => {
        pub fn $name(&self, handler: impl Fn($payload) + Send + Sync + 'static) -> Unsubscribe {
            self.hub.control.ensure_open();
            let unsubscribe = self.hub.$field.on(&self.principal_id, handler);
            self.hub.start();
            unsubscribe
        }
    };
}

impl PrincipalSignals {
    pub(crate) fn new(principal_id: Arc<str>, hub: Arc<SignalHub>) -> Self {
        Self { principal_id, hub }
    }

    signal_handler!(
        on_memory_commitment_due_soon,
        memory_commitment_due_soon,
        MemoryCommitmentDueSoon
    );
    signal_handler!(
        on_memory_action_item_due_soon,
        memory_action_item_due_soon,
        MemoryActionItemDueSoon
    );
    signal_handler!(
        on_memory_commitment_broken,
        memory_commitment_broken,
        MemoryCommitmentBroken
    );
    signal_handler!(
        on_memory_action_item_overdue,
        memory_action_item_overdue,
        MemoryActionItemOverdue
    );
    signal_handler!(
        on_memory_intention_triggered,
        memory_intention_triggered,
        MemoryIntentionTriggered
    );
    signal_handler!(
        on_memory_intention_expiring_soon,
        memory_intention_expiring_soon,
        MemoryIntentionExpiringSoon
    );
    signal_handler!(
        on_memory_intention_expired,
        memory_intention_expired,
        MemoryIntentionExpired
    );
    signal_handler!(
        on_memory_contradiction_created,
        memory_contradiction_created,
        MemoryContradictionCreated
    );
    signal_handler!(
        on_memory_contradiction_escalated,
        memory_contradiction_escalated,
        MemoryContradictionEscalated
    );
    signal_handler!(on_tension_emerged, tension_emerged, TensionEmerged);
    signal_handler!(on_tension_escalated, tension_escalated, TensionEscalated);
    signal_handler!(
        on_trajectory_deteriorated,
        trajectory_deteriorated,
        TrajectoryDeteriorated
    );
    signal_handler!(on_routine_broken, routine_broken, RoutineBroken);
}

pub(crate) fn signal_hub(
    control: HubControl,
    client: SignalServiceClient<AuthenticatedChannel>,
    principal_scope: Option<String>,
) -> Arc<SignalHub> {
    Arc::new(SignalHub {
        control,
        client,
        principal_scope,
        memory_commitment_due_soon: PrincipalListenerSet::default(),
        memory_action_item_due_soon: PrincipalListenerSet::default(),
        memory_commitment_broken: PrincipalListenerSet::default(),
        memory_action_item_overdue: PrincipalListenerSet::default(),
        memory_intention_triggered: PrincipalListenerSet::default(),
        memory_intention_expiring_soon: PrincipalListenerSet::default(),
        memory_intention_expired: PrincipalListenerSet::default(),
        memory_contradiction_created: PrincipalListenerSet::default(),
        memory_contradiction_escalated: PrincipalListenerSet::default(),
        tension_emerged: PrincipalListenerSet::default(),
        tension_escalated: PrincipalListenerSet::default(),
        trajectory_deteriorated: PrincipalListenerSet::default(),
        routine_broken: PrincipalListenerSet::default(),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tokio_util::sync::CancellationToken;
    use tonic::transport::Endpoint;

    use super::*;
    use crate::client::AuthInterceptor;
    use crate::proto::SignalEvent;
    use crate::subscription::{Lifecycle, SubscriptionOptions};

    macro_rules! register {
        ($hub:expr, $field:ident, $calls:expr) => {{
            let calls = $calls.clone();
            $hub.$field.on("alice", move |_| {
                calls.fetch_add(1, Ordering::AcqRel);
            });
        }};
    }

    #[tokio::test]
    async fn dispatches_all_signal_variants() {
        let channel = Endpoint::from_static("http://localhost").connect_lazy();
        let client = SignalServiceClient::with_interceptor(
            channel,
            AuthInterceptor::from_api_key("test").unwrap(),
        );
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let hub = signal_hub(
            HubControl::new(
                cancellation,
                Arc::new(Lifecycle::default()),
                StreamKind::Signals,
                SubscriptionOptions::default(),
            ),
            client,
            None,
        );
        let calls = Arc::new(AtomicUsize::new(0));
        register!(hub, memory_commitment_due_soon, calls);
        register!(hub, memory_action_item_due_soon, calls);
        register!(hub, memory_commitment_broken, calls);
        register!(hub, memory_action_item_overdue, calls);
        register!(hub, memory_intention_triggered, calls);
        register!(hub, memory_intention_expiring_soon, calls);
        register!(hub, memory_intention_expired, calls);
        register!(hub, memory_contradiction_created, calls);
        register!(hub, memory_contradiction_escalated, calls);
        register!(hub, tension_emerged, calls);
        register!(hub, tension_escalated, calls);
        register!(hub, trajectory_deteriorated, calls);
        register!(hub, routine_broken, calls);
        let variants = [
            Signal::MemoryCommitmentDueSoon(MemoryCommitmentDueSoon::default()),
            Signal::MemoryActionItemDueSoon(MemoryActionItemDueSoon::default()),
            Signal::MemoryCommitmentBroken(MemoryCommitmentBroken::default()),
            Signal::MemoryActionItemOverdue(MemoryActionItemOverdue::default()),
            Signal::MemoryIntentionTriggered(MemoryIntentionTriggered::default()),
            Signal::MemoryIntentionExpiringSoon(MemoryIntentionExpiringSoon::default()),
            Signal::MemoryIntentionExpired(MemoryIntentionExpired::default()),
            Signal::MemoryContradictionCreated(MemoryContradictionCreated::default()),
            Signal::MemoryContradictionEscalated(MemoryContradictionEscalated::default()),
            Signal::TensionEmerged(TensionEmerged::default()),
            Signal::TensionEscalated(TensionEscalated::default()),
            Signal::TrajectoryDeteriorated(TrajectoryDeteriorated::default()),
            Signal::RoutineBroken(RoutineBroken::default()),
        ];
        for signal in variants {
            assert!(hub.dispatch(Some(SignalEvent {
                principal_id: "namespace:alice".to_owned(),
                signal: Some(signal),
                ..Default::default()
            })));
        }
        assert_eq!(calls.load(Ordering::Acquire), 13);
    }
}
