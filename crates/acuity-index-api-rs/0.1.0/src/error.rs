use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionTerminated {
    pub reason: String,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum IndexerApiError {
    #[error("invalid indexer url: {0}")]
    Url(#[from] url::ParseError),
    #[error("websocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("request {request_id} was cancelled")]
    RequestCancelled { request_id: u64 },
    #[error("request {request_id} response channel closed before a response arrived")]
    ResponseChannelClosed { request_id: u64 },
    #[error("indexer returned error {code}: {message}")]
    Server { code: String, message: String },
    #[error("status subscription terminated ({reason}): {message}")]
    StatusSubscriptionTerminated { reason: String, message: String },
    #[error("event subscription terminated ({reason}): {message}")]
    EventSubscriptionTerminated { reason: String, message: String },
    #[error("received unexpected response for request {request_id}: {message_type}")]
    UnexpectedResponseType {
        request_id: u64,
        message_type: String,
    },
    #[error("received websocket binary frame that was not utf-8")]
    NonUtf8Binary,
    #[error("indexer closed the websocket connection")]
    ConnectionClosed,
    #[error("background reader task ended")]
    BackgroundTaskEnded,
}

impl From<ServerError> for IndexerApiError {
    fn from(value: ServerError) -> Self {
        Self::Server {
            code: value.code,
            message: value.message,
        }
    }
}

impl From<SubscriptionTerminated> for IndexerApiError {
    fn from(value: SubscriptionTerminated) -> Self {
        Self::StatusSubscriptionTerminated {
            reason: value.reason,
            message: value.message,
        }
    }
}
