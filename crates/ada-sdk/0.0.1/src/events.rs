use std::sync::Arc;
use std::sync::atomic::Ordering;

use tonic::Code;

use crate::AdaError;
use crate::client::AuthenticatedChannel;
use crate::proto::public_event::Event;
use crate::proto::public_event_service_client::PublicEventServiceClient;
use crate::proto::{
    MemoryIngestFinished, MemoryIngestStarted, MemoryRecallFinished, MemoryRecallStarted,
    StreamPublicEventsRequest, StreamScopeMode,
};
use crate::subscription::{
    HubControl, PrincipalListenerSet, StreamKind, Unsubscribe, local_principal_id,
};

pub(crate) struct EventHub {
    control: HubControl,
    client: PublicEventServiceClient<AuthenticatedChannel>,
    principal_scope: Option<String>,
    memory_ingest_started: PrincipalListenerSet<MemoryIngestStarted>,
    memory_ingest_finished: PrincipalListenerSet<MemoryIngestFinished>,
    memory_recall_started: PrincipalListenerSet<MemoryRecallStarted>,
    memory_recall_finished: PrincipalListenerSet<MemoryRecallFinished>,
}

impl EventHub {
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
                .connecting(StreamKind::Events, attempt + 1);
            let request = StreamPublicEventsRequest {
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
                response = client.stream_public_events(request) => response,
            };
            let mut stream = match response {
                Ok(response) => {
                    self.control.lifecycle.connected(StreamKind::Events);
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
                            .event
                            .as_ref()
                            .and_then(|event| local_principal_id(&event.principal_id).ok())
                            .unwrap_or_default()
                            .to_owned();
                        if self.dispatch(envelope.event) {
                            cursor = envelope.cursor;
                            self.control.lifecycle.cursor(
                                StreamKind::Events,
                                cursor.clone(),
                                principal_id,
                            );
                        }
                    }
                    Ok(None) => {
                        attempt += 1;
                        let error = AdaError {
                            code: Code::Unavailable,
                            message: "public event stream ended".to_owned(),
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

    fn dispatch(&self, event: Option<crate::proto::PublicEvent>) -> bool {
        let Some(event) = event else {
            self.control
                .lifecycle
                .protocol_error(StreamKind::Events, "public event envelope is missing event");
            return false;
        };
        let principal_id = if self.principal_scope.is_some() {
            "*"
        } else {
            match local_principal_id(&event.principal_id) {
                Ok(principal_id) => principal_id,
                Err(error) => {
                    self.control
                        .lifecycle
                        .protocol_error(StreamKind::Events, &error.message);
                    return false;
                }
            }
        };
        let Some(payload) = event.event else {
            self.control.lifecycle.protocol_error(
                StreamKind::Events,
                "public event envelope has an unknown variant",
            );
            return false;
        };
        match payload {
            Event::MemoryIngestStarted(value) => self.emit(
                &self.memory_ingest_started,
                principal_id,
                "memory.ingest.started",
                value,
            ),
            Event::MemoryIngestFinished(value) => self.emit(
                &self.memory_ingest_finished,
                principal_id,
                "memory.ingest.finished",
                value,
            ),
            Event::MemoryRecallStarted(value) => self.emit(
                &self.memory_recall_started,
                principal_id,
                "memory.recall.started",
                value,
            ),
            Event::MemoryRecallFinished(value) => self.emit(
                &self.memory_recall_finished,
                principal_id,
                "memory.recall.finished",
                value,
            ),
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
                StreamKind::Events,
                event_name,
                principal_id,
                error,
            );
        });
    }
}

#[derive(Clone)]
pub struct PrincipalEvents {
    principal_id: Arc<str>,
    hub: Arc<EventHub>,
}

impl PrincipalEvents {
    pub(crate) fn new(principal_id: Arc<str>, hub: Arc<EventHub>) -> Self {
        Self { principal_id, hub }
    }

    pub fn on_memory_ingest_started(
        &self,
        handler: impl Fn(MemoryIngestStarted) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.hub.control.ensure_open();
        let unsubscribe = self
            .hub
            .memory_ingest_started
            .on(&self.principal_id, handler);
        self.hub.start();
        unsubscribe
    }

    pub fn on_memory_ingest_finished(
        &self,
        handler: impl Fn(MemoryIngestFinished) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.hub.control.ensure_open();
        let unsubscribe = self
            .hub
            .memory_ingest_finished
            .on(&self.principal_id, handler);
        self.hub.start();
        unsubscribe
    }

    pub fn on_memory_recall_started(
        &self,
        handler: impl Fn(MemoryRecallStarted) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.hub.control.ensure_open();
        let unsubscribe = self
            .hub
            .memory_recall_started
            .on(&self.principal_id, handler);
        self.hub.start();
        unsubscribe
    }

    pub fn on_memory_recall_finished(
        &self,
        handler: impl Fn(MemoryRecallFinished) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.hub.control.ensure_open();
        let unsubscribe = self
            .hub
            .memory_recall_finished
            .on(&self.principal_id, handler);
        self.hub.start();
        unsubscribe
    }
}

pub(crate) fn event_hub(
    control: HubControl,
    client: PublicEventServiceClient<AuthenticatedChannel>,
    principal_scope: Option<String>,
) -> Arc<EventHub> {
    Arc::new(EventHub {
        control,
        client,
        principal_scope,
        memory_ingest_started: PrincipalListenerSet::default(),
        memory_ingest_finished: PrincipalListenerSet::default(),
        memory_recall_started: PrincipalListenerSet::default(),
        memory_recall_finished: PrincipalListenerSet::default(),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tokio_util::sync::CancellationToken;
    use tonic::transport::Endpoint;

    use super::*;
    use crate::client::AuthInterceptor;
    use crate::proto::PublicEvent;
    use crate::subscription::{Lifecycle, SubscriptionOptions};

    #[tokio::test]
    async fn all_variants_route_to_only_the_matching_principal() {
        let channel = Endpoint::from_static("http://localhost").connect_lazy();
        let client = PublicEventServiceClient::with_interceptor(
            channel,
            AuthInterceptor::from_api_key("test").unwrap(),
        );
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let hub = event_hub(
            HubControl::new(
                cancellation,
                Arc::new(Lifecycle::default()),
                StreamKind::Events,
                SubscriptionOptions::default(),
            ),
            client,
            None,
        );
        let alice = Arc::new(AtomicUsize::new(0));
        let bob = Arc::new(AtomicUsize::new(0));
        let alice_calls = alice.clone();
        hub.memory_ingest_started.on("alice", move |_| {
            alice_calls.fetch_add(1, Ordering::AcqRel);
        });
        let bob_calls = bob.clone();
        hub.memory_ingest_started.on("bob", move |_| {
            bob_calls.fetch_add(1, Ordering::AcqRel);
        });
        let calls = alice.clone();
        hub.memory_ingest_finished.on("alice", move |_| {
            calls.fetch_add(1, Ordering::AcqRel);
        });
        let calls = alice.clone();
        hub.memory_recall_started.on("alice", move |_| {
            calls.fetch_add(1, Ordering::AcqRel);
        });
        let calls = alice.clone();
        hub.memory_recall_finished.on("alice", move |_| {
            calls.fetch_add(1, Ordering::AcqRel);
        });
        let variants = [
            Event::MemoryIngestStarted(MemoryIngestStarted::default()),
            Event::MemoryIngestFinished(MemoryIngestFinished::default()),
            Event::MemoryRecallStarted(MemoryRecallStarted::default()),
            Event::MemoryRecallFinished(MemoryRecallFinished::default()),
        ];
        for event in variants {
            assert!(hub.dispatch(Some(PublicEvent {
                principal_id: "namespace:alice".to_owned(),
                event: Some(event),
                ..Default::default()
            })));
        }
        assert_eq!(alice.load(Ordering::Acquire), 4);
        assert_eq!(bob.load(Ordering::Acquire), 0);
    }
}
