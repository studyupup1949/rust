use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::stream::StreamExt;

use crate::types::{SendMessageRequest, StreamResponse, SubscribeToTaskRequest};

use super::handler::A2AHandler;
use super::rest::RestErrorResponse;

const SSE_KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(15);

pub(super) async fn send_message<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Json(request): Json<SendMessageRequest>,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, RestErrorResponse>
where
    H: A2AHandler,
{
    handler.validate_protocol_headers(&headers).await?;
    request.validate()?;
    let stream = handler.send_streaming_message(request).await?;
    Ok(sse_response(stream))
}

pub(super) async fn tenant_send_message<H>(
    State(handler): State<Arc<H>>,
    headers: HeaderMap,
    Path(tenant): Path<String>,
    Json(mut request): Json<SendMessageRequest>,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, RestErrorResponse>
where
    H: A2AHandler,
{
    request.tenant = Some(tenant);
    send_message(State(handler), headers, Json(request)).await
}

pub(super) async fn subscribe_to_task_response<H>(
    handler: Arc<H>,
    request: SubscribeToTaskRequest,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, RestErrorResponse>
where
    H: A2AHandler,
{
    let stream = handler.subscribe_to_task(request).await?;
    Ok(sse_response(stream))
}

fn sse_response(
    stream: super::A2AStream,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    Sse::new(stream_to_sse(stream)).keep_alive(
        KeepAlive::new()
            .interval(SSE_KEEP_ALIVE_INTERVAL)
            .text("keep-alive"),
    )
}

fn stream_to_sse(
    stream: super::A2AStream,
) -> impl futures_core::Stream<Item = Result<Event, Infallible>> {
    stream.map(|item| Ok(Event::default().data(serialize_stream_response(&item))))
}

fn serialize_stream_response(item: &StreamResponse) -> String {
    match serde_json::to_string(item) {
        Ok(json) => json,
        // These protocol types should serialize deterministically. If a future
        // change violates that assumption, emit a diagnostic payload rather than
        // panic inside the response stream.
        Err(error) => serde_json::json!({
            "error": {
                "code": crate::jsonrpc::INTERNAL_ERROR,
                "message": error.to_string(),
            }
        })
        .to_string(),
    }
}
