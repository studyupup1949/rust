use std::sync::Arc;
use std::sync::atomic::Ordering;

use tonic::Code;

use crate::AdaError;
use crate::client::AuthenticatedChannel;
use crate::proto::ingest_service_client::IngestServiceClient;
use crate::proto::job_status_stream_event::Event;
use crate::proto::{
    GetIngestAccountingRequest, GetIngestAccountingResponse, GetJobStatusRequest,
    GetJobStatusResponse, GetLatestJobByDocumentRequest, JobFinished, JobProgressed, JobStarted,
    StreamJobsRequest, StreamScopeMode,
};
use crate::subscription::{
    HubControl, PrincipalListenerSet, StreamKind, Unsubscribe, local_principal_id,
};

pub(crate) struct JobHub {
    control: HubControl,
    client: IngestServiceClient<AuthenticatedChannel>,
    principal_scope: Option<String>,
    started: PrincipalListenerSet<JobStarted>,
    progressed: PrincipalListenerSet<JobProgressed>,
    finished: PrincipalListenerSet<JobFinished>,
}

impl JobHub {
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
                .connecting(StreamKind::Jobs, attempt + 1);
            let request = StreamJobsRequest {
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
                response = client.stream_jobs(request) => response,
            };
            let mut stream = match response {
                Ok(response) => {
                    self.control.lifecycle.connected(StreamKind::Jobs);
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
                        let principal_id = local_principal_id(&envelope.principal_id)
                            .unwrap_or_default()
                            .to_owned();
                        if self.dispatch(&envelope.principal_id, envelope.event) {
                            cursor = envelope.event_id;
                            self.control.lifecycle.cursor(
                                StreamKind::Jobs,
                                cursor.clone(),
                                principal_id,
                            );
                        }
                    }
                    Ok(None) => {
                        attempt += 1;
                        let error = AdaError {
                            code: Code::Unavailable,
                            message: "job stream ended".to_owned(),
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

    fn dispatch(&self, envelope_principal_id: &str, event: Option<Event>) -> bool {
        let principal_id = if self.principal_scope.is_some() {
            "*"
        } else {
            match local_principal_id(envelope_principal_id) {
                Ok(principal_id) => principal_id,
                Err(error) => {
                    self.control
                        .lifecycle
                        .protocol_error(StreamKind::Jobs, &error.message);
                    return false;
                }
            }
        };
        let Some(event) = event else {
            self.control
                .lifecycle
                .protocol_error(StreamKind::Jobs, "job envelope has an unknown variant");
            return false;
        };
        match event {
            Event::Started(value) => self.emit(&self.started, principal_id, "job.started", value),
            Event::Progressed(value) => {
                self.emit(&self.progressed, principal_id, "job.progressed", value)
            }
            Event::Finished(value) => {
                self.emit(&self.finished, principal_id, "job.finished", value)
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
                StreamKind::Jobs,
                event_name,
                principal_id,
                error,
            );
        });
    }
}

#[derive(Clone)]
pub struct PrincipalJobs {
    principal_id: Arc<str>,
    request_principal_id: Arc<str>,
    hub: Arc<JobHub>,
    client: IngestServiceClient<AuthenticatedChannel>,
}

impl PrincipalJobs {
    pub(crate) fn new(
        principal_id: Arc<str>,
        request_principal_id: Arc<str>,
        hub: Arc<JobHub>,
        client: IngestServiceClient<AuthenticatedChannel>,
    ) -> Self {
        Self {
            principal_id,
            request_principal_id,
            hub,
            client,
        }
    }

    pub fn on_started(&self, handler: impl Fn(JobStarted) + Send + Sync + 'static) -> Unsubscribe {
        self.hub.control.ensure_open();
        let unsubscribe = self.hub.started.on(&self.principal_id, handler);
        self.hub.start();
        unsubscribe
    }

    pub fn on_progressed(
        &self,
        handler: impl Fn(JobProgressed) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.hub.control.ensure_open();
        let unsubscribe = self.hub.progressed.on(&self.principal_id, handler);
        self.hub.start();
        unsubscribe
    }

    pub fn on_finished(
        &self,
        handler: impl Fn(JobFinished) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.hub.control.ensure_open();
        let unsubscribe = self.hub.finished.on(&self.principal_id, handler);
        self.hub.start();
        unsubscribe
    }

    pub async fn get_status(
        &self,
        mut request: GetJobStatusRequest,
    ) -> Result<GetJobStatusResponse, AdaError> {
        request.principal_id = self.request_principal_id.to_string();
        let mut client = self.client.clone();
        client
            .get_job_status(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn get_accounting(
        &self,
        mut request: GetIngestAccountingRequest,
    ) -> Result<GetIngestAccountingResponse, AdaError> {
        request.principal_id = self.request_principal_id.to_string();
        let mut client = self.client.clone();
        client
            .get_ingest_accounting(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn get_latest_by_document(
        &self,
        mut request: GetLatestJobByDocumentRequest,
    ) -> Result<GetJobStatusResponse, AdaError> {
        request.principal_id = self.request_principal_id.to_string();
        let mut client = self.client.clone();
        client
            .get_latest_job_by_document(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }
}

pub(crate) fn job_hub(
    control: HubControl,
    client: IngestServiceClient<AuthenticatedChannel>,
    principal_scope: Option<String>,
) -> Arc<JobHub> {
    Arc::new(JobHub {
        control,
        client,
        principal_scope,
        started: PrincipalListenerSet::default(),
        progressed: PrincipalListenerSet::default(),
        finished: PrincipalListenerSet::default(),
    })
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tokio_util::sync::CancellationToken;
    use tonic::transport::Endpoint;

    use super::*;
    use crate::client::AuthInterceptor;
    use crate::subscription::{Lifecycle, SubscriptionOptions};

    #[tokio::test]
    async fn dispatches_all_job_variants() {
        let channel = Endpoint::from_static("http://localhost").connect_lazy();
        let client = IngestServiceClient::with_interceptor(
            channel,
            AuthInterceptor::from_api_key("test").unwrap(),
        );
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let hub = job_hub(
            HubControl::new(
                cancellation,
                Arc::new(Lifecycle::default()),
                StreamKind::Jobs,
                SubscriptionOptions::default(),
            ),
            client,
            None,
        );
        let calls = Arc::new(AtomicUsize::new(0));
        let started_calls = calls.clone();
        hub.started.on("alice", move |_| {
            started_calls.fetch_add(1, Ordering::AcqRel);
        });
        let progressed_calls = calls.clone();
        hub.progressed.on("alice", move |_| {
            progressed_calls.fetch_add(1, Ordering::AcqRel);
        });
        let finished_calls = calls.clone();
        hub.finished.on("alice", move |_| {
            finished_calls.fetch_add(1, Ordering::AcqRel);
        });
        assert!(hub.dispatch(
            "namespace:alice",
            Some(Event::Started(JobStarted::default())),
        ));
        assert!(hub.dispatch(
            "namespace:alice",
            Some(Event::Progressed(JobProgressed::default())),
        ));
        assert!(hub.dispatch(
            "namespace:alice",
            Some(Event::Finished(JobFinished::default())),
        ));
        assert_eq!(calls.load(Ordering::Acquire), 3);
    }
}
