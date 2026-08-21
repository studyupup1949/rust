use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;
use tonic::metadata::{Ascii, MetadataValue};
use tonic::service::interceptor::InterceptedService;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};
use tonic::{Request, Status};

use crate::errors::AdaError;
use crate::events::{EventHub, PrincipalEvents, event_hub};
use crate::jobs::{JobHub, PrincipalJobs, job_hub};
use crate::proto::browser_session_service_client::BrowserSessionServiceClient;
use crate::proto::data_service_client::DataServiceClient;
use crate::proto::ingest_service_client::IngestServiceClient;
use crate::proto::public_event_service_client::PublicEventServiceClient;
use crate::proto::recall_service_client::RecallServiceClient;
use crate::proto::signal_service_client::SignalServiceClient;
use crate::proto::summary_service_client::SummaryServiceClient;
use crate::proto::{
    ChangeRecallRequest, ChangeRecallResponse, CommunityDossierRequest, CommunityDossierResponse,
    ContradictionsRequest, ContradictionsResponse, EntityDossierRequest, EntityDossierResponse,
    EstimateIngestRequest, EstimateIngestResponse, GetDocumentStatusRequest,
    GetDocumentStatusResponse, GetMeEntityRequest, GetMeEntityResponse,
    GetPublicEventCatalogRequest, GetPublicEventCatalogResponse, GetSignalCatalogRequest,
    GetSignalCatalogResponse, IngestAcceptedResponse, IngestRequest, MintBrowserSessionRequest,
    MintBrowserSessionResponse, RecallRequest, RecallResponse, RelationshipDossierRequest,
    RelationshipDossierResponse, SetMeEntityRequest, SetMeEntityResponse, TimelineRequest,
    TimelineResponse,
};
use crate::signals::{PrincipalSignals, SignalHub, signal_hub};
use crate::subscription::{HubControl, Lifecycle, StreamConfig, StreamKind};

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub endpoint: String,
    pub api_key: String,
    pub insecure: bool,
    pub streams: StreamConfig,
}

#[derive(Clone)]
pub struct AuthInterceptor {
    authorization: MetadataValue<Ascii>,
}

impl AuthInterceptor {
    pub(crate) fn from_api_key(api_key: &str) -> Result<Self, AdaError> {
        let authorization = format!("Bearer {}", api_key.trim())
            .parse()
            .map_err(AdaError::from)?;
        Ok(Self { authorization })
    }
}

impl tonic::service::Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request
            .metadata_mut()
            .insert("authorization", self.authorization.clone());
        Ok(request)
    }
}

pub type AuthenticatedChannel = InterceptedService<Channel, AuthInterceptor>;

struct ClientInner {
    cancellation: CancellationToken,
    lifecycle: Arc<Lifecycle>,
    closed: AtomicBool,
    principals: Mutex<HashMap<String, Principal>>,
    browser_sessions: BrowserSessionServiceClient<AuthenticatedChannel>,
    ingest: IngestServiceClient<AuthenticatedChannel>,
    recall: RecallServiceClient<AuthenticatedChannel>,
    data: DataServiceClient<AuthenticatedChannel>,
    summary: SummaryServiceClient<AuthenticatedChannel>,
    public_events: PublicEventServiceClient<AuthenticatedChannel>,
    signals_service: SignalServiceClient<AuthenticatedChannel>,
    events: Arc<EventHub>,
    signals: Arc<SignalHub>,
    jobs: Arc<JobHub>,
}

#[derive(Clone)]
pub struct AdaClient {
    inner: Arc<ClientInner>,
}

impl AdaClient {
    pub async fn connect(config: ClientConfig) -> Result<Self, AdaError> {
        if config.endpoint.trim().is_empty() {
            return Err(AdaError {
                code: tonic::Code::InvalidArgument,
                message: "endpoint must not be empty".to_owned(),
            });
        }
        if config.api_key.trim().is_empty() {
            return Err(AdaError {
                code: tonic::Code::InvalidArgument,
                message: "api_key must not be empty".to_owned(),
            });
        }
        let mut endpoint = Endpoint::from_shared(config.endpoint).map_err(|error| AdaError {
            code: tonic::Code::InvalidArgument,
            message: error.to_string(),
        })?;
        if !config.insecure {
            endpoint = endpoint
                .tls_config(ClientTlsConfig::new().with_enabled_roots())
                .map_err(AdaError::from)?;
        }
        let channel = endpoint.connect().await.map_err(AdaError::from)?;
        let interceptor = AuthInterceptor::from_api_key(&config.api_key)?;
        let ingest = IngestServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        let public_events =
            PublicEventServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        let signal_service =
            SignalServiceClient::with_interceptor(channel.clone(), interceptor.clone());
        let cancellation = CancellationToken::new();
        let lifecycle = Arc::new(Lifecycle::default());
        let events = event_hub(
            HubControl::new(
                cancellation.clone(),
                lifecycle.clone(),
                StreamKind::Events,
                config.streams.events,
            ),
            public_events.clone(),
            None,
        );
        let signals = signal_hub(
            HubControl::new(
                cancellation.clone(),
                lifecycle.clone(),
                StreamKind::Signals,
                config.streams.signals,
            ),
            signal_service.clone(),
            None,
        );
        let jobs = job_hub(
            HubControl::new(
                cancellation.clone(),
                lifecycle.clone(),
                StreamKind::Jobs,
                config.streams.jobs,
            ),
            ingest.clone(),
            None,
        );
        Ok(Self {
            inner: Arc::new(ClientInner {
                cancellation,
                lifecycle,
                closed: AtomicBool::new(false),
                principals: Mutex::new(HashMap::new()),
                browser_sessions: BrowserSessionServiceClient::with_interceptor(
                    channel.clone(),
                    interceptor.clone(),
                ),
                ingest,
                recall: RecallServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
                data: DataServiceClient::with_interceptor(channel.clone(), interceptor.clone()),
                summary: SummaryServiceClient::with_interceptor(channel, interceptor),
                public_events,
                signals_service: signal_service,
                events,
                signals,
                jobs,
            }),
        })
    }

    pub fn principal(&self, principal_id: &str) -> Principal {
        let normalized = principal_id.trim();
        assert!(
            !normalized.is_empty() && !normalized.contains(':'),
            "principal_id must be a non-empty bare id"
        );
        let mut principals = self
            .inner
            .principals
            .lock()
            .expect("principal cache lock poisoned");
        principals
            .entry(normalized.to_owned())
            .or_insert_with(|| Principal::new(normalized, self.inner.clone()))
            .clone()
    }

    pub fn lifecycle(&self) -> &Lifecycle {
        &self.inner.lifecycle
    }

    pub async fn mint_browser_session(
        &self,
        request: MintBrowserSessionRequest,
    ) -> Result<MintBrowserSessionResponse, AdaError> {
        let mut client = self.inner.browser_sessions.clone();
        client
            .mint_browser_session(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn get_public_event_catalog(
        &self,
    ) -> Result<GetPublicEventCatalogResponse, AdaError> {
        let mut client = self.inner.public_events.clone();
        client
            .get_public_event_catalog(GetPublicEventCatalogRequest {})
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn get_signal_catalog(&self) -> Result<GetSignalCatalogResponse, AdaError> {
        let mut client = self.inner.signals_service.clone();
        client
            .get_signal_catalog(GetSignalCatalogRequest {})
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn close(&self) {
        if !self.inner.closed.swap(true, Ordering::AcqRel) {
            self.inner.lifecycle.closing();
            self.inner.cancellation.cancel();
            tokio::task::yield_now().await;
            self.inner.lifecycle.closed();
        }
    }
}

#[derive(Clone)]
pub struct Principal {
    id: Arc<str>,
    client: Arc<ClientInner>,
    events: PrincipalEvents,
    signals: PrincipalSignals,
    jobs: PrincipalJobs,
    documents: PrincipalDocuments,
    data: PrincipalData,
    summary: PrincipalSummary,
}

impl Principal {
    fn new(principal_id: &str, client: Arc<ClientInner>) -> Self {
        let id: Arc<str> = Arc::from(principal_id);
        Self {
            id: id.clone(),
            events: PrincipalEvents::new(id.clone(), client.events.clone()),
            signals: PrincipalSignals::new(id.clone(), client.signals.clone()),
            jobs: PrincipalJobs::new(
                id.clone(),
                id.clone(),
                client.jobs.clone(),
                client.ingest.clone(),
            ),
            documents: PrincipalDocuments {
                principal_id: id.clone(),
                client: client.ingest.clone(),
            },
            data: PrincipalData {
                principal_id: id.clone(),
                client: client.data.clone(),
            },
            summary: PrincipalSummary {
                principal_id: id.clone(),
                client: client.summary.clone(),
            },
            client,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn events(&self) -> &PrincipalEvents {
        &self.events
    }

    pub fn signals(&self) -> &PrincipalSignals {
        &self.signals
    }

    pub fn jobs(&self) -> &PrincipalJobs {
        &self.jobs
    }

    pub fn documents(&self) -> &PrincipalDocuments {
        &self.documents
    }

    pub fn data(&self) -> &PrincipalData {
        &self.data
    }

    pub fn summary(&self) -> &PrincipalSummary {
        &self.summary
    }

    pub async fn ingest(
        &self,
        mut request: IngestRequest,
    ) -> Result<IngestAcceptedResponse, AdaError> {
        request.principal_id = self.id.to_string();
        let mut client = self.client.ingest.clone();
        client
            .ingest(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn estimate_ingest(
        &self,
        mut request: EstimateIngestRequest,
    ) -> Result<EstimateIngestResponse, AdaError> {
        request.principal_id = self.id.to_string();
        let mut client = self.client.ingest.clone();
        client
            .estimate_ingest(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn set_me_entity(
        &self,
        mut request: SetMeEntityRequest,
    ) -> Result<SetMeEntityResponse, AdaError> {
        request.principal_id = self.id.to_string();
        let mut client = self.client.ingest.clone();
        client
            .set_me_entity(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn get_me_entity(
        &self,
        mut request: GetMeEntityRequest,
    ) -> Result<GetMeEntityResponse, AdaError> {
        request.principal_id = self.id.to_string();
        let mut client = self.client.ingest.clone();
        client
            .get_me_entity(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn recall(&self, mut request: RecallRequest) -> Result<RecallResponse, AdaError> {
        request.principal_id = self.id.to_string();
        let mut client = self.client.recall.clone();
        client
            .recall(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn entity_dossier(
        &self,
        mut request: EntityDossierRequest,
    ) -> Result<EntityDossierResponse, AdaError> {
        request.principal_id = self.id.to_string();
        let mut client = self.client.recall.clone();
        client
            .entity_dossier(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn timeline(
        &self,
        mut request: TimelineRequest,
    ) -> Result<TimelineResponse, AdaError> {
        request.principal_id = self.id.to_string();
        let mut client = self.client.recall.clone();
        client
            .timeline(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn contradictions(
        &self,
        mut request: ContradictionsRequest,
    ) -> Result<ContradictionsResponse, AdaError> {
        request.principal_id = self.id.to_string();
        let mut client = self.client.recall.clone();
        client
            .contradictions(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn community_dossier(
        &self,
        mut request: CommunityDossierRequest,
    ) -> Result<CommunityDossierResponse, AdaError> {
        request.principal_id = self.id.to_string();
        let mut client = self.client.recall.clone();
        client
            .community_dossier(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn change_recall(
        &self,
        mut request: ChangeRecallRequest,
    ) -> Result<ChangeRecallResponse, AdaError> {
        request.principal_id = self.id.to_string();
        let mut client = self.client.recall.clone();
        client
            .change_recall(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }

    pub async fn relationship_dossier(
        &self,
        mut request: RelationshipDossierRequest,
    ) -> Result<RelationshipDossierResponse, AdaError> {
        request.principal_id = self.id.to_string();
        let mut client = self.client.recall.clone();
        client
            .relationship_dossier(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }
}

#[derive(Clone)]
pub struct PrincipalDocuments {
    principal_id: Arc<str>,
    client: IngestServiceClient<AuthenticatedChannel>,
}

impl PrincipalDocuments {
    pub async fn get_status(
        &self,
        mut request: GetDocumentStatusRequest,
    ) -> Result<GetDocumentStatusResponse, AdaError> {
        request.principal_id = self.principal_id.to_string();
        let mut client = self.client.clone();
        client
            .get_document_status(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(AdaError::from)
    }
}

#[derive(Clone)]
pub struct PrincipalData {
    principal_id: Arc<str>,
    client: DataServiceClient<AuthenticatedChannel>,
}

macro_rules! data_method {
    ($name:ident, $request:ident, $response:ident, $rpc:ident) => {
        pub async fn $name(
            &self,
            mut request: crate::proto::$request,
        ) -> Result<crate::proto::$response, AdaError> {
            request.principal_id = self.principal_id.to_string();
            let mut client = self.client.clone();
            client
                .$rpc(request)
                .await
                .map(tonic::Response::into_inner)
                .map_err(AdaError::from)
        }
    };
}

impl PrincipalData {
    data_method!(
        list_entities,
        ListEntitiesRequest,
        ListEntitiesResponse,
        list_entities
    );
    data_method!(list_atoms, ListAtomsRequest, ListAtomsResponse, list_atoms);
    data_method!(
        list_relations,
        ListRelationsRequest,
        ListRelationsResponse,
        list_relations
    );
    data_method!(
        list_commitments,
        ListCommitmentsRequest,
        ListCommitmentsResponse,
        list_commitments
    );
    data_method!(
        list_decisions,
        ListDecisionsRequest,
        ListDecisionsResponse,
        list_decisions
    );
    data_method!(
        list_action_items,
        ListActionItemsRequest,
        ListActionItemsResponse,
        list_action_items
    );
    data_method!(
        list_contradictions,
        ListContradictionsRequest,
        ListContradictionsResponse,
        list_contradictions
    );
    data_method!(
        list_observations,
        ListObservationsRequest,
        ListObservationsResponse,
        list_observations
    );
    data_method!(
        list_documents,
        ListDocumentsRequest,
        ListDocumentsResponse,
        list_documents
    );
    data_method!(
        list_events,
        ListEventsRequest,
        ListEventsResponse,
        list_events
    );
    data_method!(
        list_meetings,
        ListMeetingsRequest,
        ListMeetingsResponse,
        list_meetings
    );
    data_method!(
        list_intentions,
        ListIntentionsRequest,
        ListIntentionsResponse,
        list_intentions
    );
    data_method!(
        list_priorities,
        ListPrioritiesRequest,
        ListPrioritiesResponse,
        list_priorities
    );
    data_method!(
        list_ingest_jobs,
        ListIngestJobsRequest,
        ListIngestJobsResponse,
        list_ingest_jobs
    );
    data_method!(
        get_usage_report,
        GetUsageReportRequest,
        GetUsageReportResponse,
        get_usage_report
    );
    data_method!(
        list_usage_reports,
        ListUsageReportsRequest,
        ListUsageReportsResponse,
        list_usage_reports
    );
    data_method!(
        get_usage_event,
        GetUsageEventRequest,
        GetUsageEventResponse,
        get_usage_event
    );
    data_method!(
        list_usage_events,
        ListUsageEventsRequest,
        ListUsageEventsResponse,
        list_usage_events
    );
    data_method!(
        list_communities,
        ListCommunitiesRequest,
        ListCommunitiesResponse,
        list_communities
    );
    data_method!(
        list_recall_runs,
        ListRecallRunsRequest,
        ListRecallRunsResponse,
        list_recall_runs
    );
    data_method!(
        get_projection_status,
        GetProjectionStatusRequest,
        GetProjectionStatusResponse,
        get_projection_status
    );
    data_method!(delete, DeleteDataRequest, DeleteDataResponse, delete_data);
}

#[derive(Clone)]
pub struct PrincipalSummary {
    principal_id: Arc<str>,
    client: SummaryServiceClient<AuthenticatedChannel>,
}

macro_rules! summary_method {
    ($name:ident, $request:ident, $response:ident, $rpc:ident) => {
        pub async fn $name(
            &self,
            mut request: crate::proto::$request,
        ) -> Result<crate::proto::$response, AdaError> {
            request.principal_id = self.principal_id.to_string();
            let mut client = self.client.clone();
            client
                .$rpc(request)
                .await
                .map(tonic::Response::into_inner)
                .map_err(AdaError::from)
        }
    };
}

impl PrincipalSummary {
    summary_method!(
        get_overview,
        GetSummaryOverviewRequest,
        GetSummaryOverviewResponse,
        get_summary_overview
    );
    summary_method!(
        list_principals,
        ListSummaryPrincipalsRequest,
        ListSummaryPrincipalsResponse,
        list_summary_principals
    );
    summary_method!(
        list_activity,
        ListSummaryActivityRequest,
        ListSummaryActivityResponse,
        list_summary_activity
    );
    summary_method!(
        get_principal_overview,
        GetPrincipalOverviewRequest,
        GetPrincipalOverviewResponse,
        get_principal_overview
    );
    summary_method!(
        get_knowledge,
        GetPrincipalKnowledgeSummaryRequest,
        GetPrincipalKnowledgeSummaryResponse,
        get_principal_knowledge_summary
    );
    summary_method!(
        get_sources,
        GetPrincipalSourcesSummaryRequest,
        GetPrincipalSourcesSummaryResponse,
        get_principal_sources_summary
    );
    summary_method!(
        get_now,
        GetPrincipalNowRequest,
        GetPrincipalNowResponse,
        get_principal_now
    );
}
