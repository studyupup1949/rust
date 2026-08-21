//! Body type adapters between Actix Web and the `http-body` ecosystem.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use actix_web::{
    body::{BoxBody, MessageBody},
    dev::Payload,
    web::Bytes,
};
use futures_util::Stream;
use http_body::{Body as HttpBody, Frame};
use pin_project_lite::pin_project;

use crate::internal::common::{BoxError, StringError};

// ---------------------------------------------------------------------------
// Request Body:  Actix Payload  →  http_body::Body
// ---------------------------------------------------------------------------

/// Wraps an Actix `Payload` as an `http_body::Body`.
///
/// This is used when converting a `ServiceRequest` into an `http::Request`
/// for Tower middleware consumption.
pub struct ActixRequestBody {
    /// The Actix payload stream.
    payload: Payload,
}

impl ActixRequestBody {
    /// Create a new `ActixRequestBody` from an Actix `Payload`.
    pub fn new(payload: Payload) -> Self {
        Self { payload }
    }
}

impl HttpBody for ActixRequestBody {
    type Data = Bytes;
    // Changed from Infallible: Payload::poll_next yields Result<Bytes, PayloadError>.
    // A dropped TCP connection (or body size limit violation upstream) returns Err.
    // Panicking on that error crashes the Actix worker; propagating it is correct.
    type Error = BoxError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match Pin::new(&mut self.payload).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Ok(bytes))) => Poll::Ready(Some(Ok(Frame::data(bytes)))),
            Poll::Ready(Some(Err(e))) => {
                let boxed: BoxError = Box::new(StringError(format!("payload read error: {e}")));
                Poll::Ready(Some(Err(boxed)))
            }
        }
    }
}

impl Unpin for ActixRequestBody {}

// ---------------------------------------------------------------------------
// Response Body:  Actix MessageBody  →  http_body::Body
// ---------------------------------------------------------------------------

pin_project! {
    /// Wraps an Actix `MessageBody` as an `http_body::Body`.
    ///
    /// Used when converting a `ServiceResponse` into an `http::Response`
    /// for Tower middleware to process.
    pub struct ActixResponseBody<B> {
        #[pin]
        body: B,
    }
}

impl ActixResponseBody<BoxBody> {
    /// Create from a `BoxBody` (the default Actix response body type).
    pub fn from_box_body(body: BoxBody) -> Self {
        Self { body }
    }
}

impl Default for ActixResponseBody<BoxBody> {
    fn default() -> Self {
        Self {
            body: BoxBody::new(()),
        }
    }
}

impl Default for ActixResponseBody<Bytes> {
    fn default() -> Self {
        Self { body: Bytes::new() }
    }
}

impl Default for ActixResponseBody<String> {
    fn default() -> Self {
        Self {
            body: String::new(),
        }
    }
}

impl Default for ActixResponseBody<Vec<u8>> {
    fn default() -> Self {
        Self { body: Vec::new() }
    }
}

impl Default for ActixResponseBody<()> {
    fn default() -> Self {
        Self { body: () }
    }
}

impl<B: MessageBody> HttpBody for ActixResponseBody<B> {
    type Data = Bytes;
    type Error = crate::internal::common::BoxError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        match this.body.poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Ok(bytes))) => Poll::Ready(Some(Ok(Frame::data(bytes)))),
            Poll::Ready(Some(Err(e))) => {
                let err_box: Box<dyn std::error::Error + 'static> = e.into();
                let boxed = Box::new(crate::internal::common::StringError(err_box.to_string()));
                Poll::Ready(Some(Err(boxed)))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Body Stream:  http_body::Body  →  Stream (for Actix BodyStream)
// ---------------------------------------------------------------------------

pin_project! {
    /// Converts an `http_body::Body` into a `Stream<Item = Result<Bytes, E>>`.
    ///
    /// This is used to convert a Tower response body back into an Actix body
    /// via `actix_web::body::BodyStream`.
    pub struct TowerBodyStream<B> {
        #[pin]
        body: B,
    }
}

impl<B> TowerBodyStream<B> {
    /// Create a new `TowerBodyStream` from an `http_body::Body`.
    pub fn new(body: B) -> Self {
        Self { body }
    }
}

impl<B: HttpBody<Data = Bytes>> Stream for TowerBodyStream<B> {
    type Item = Result<Bytes, TowerBodyError<B::Error>>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, TowerBodyError<B::Error>>>> {
        let mut this = self.project();
        loop {
            match this.body.as_mut().poll_frame(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(Ok(frame))) => {
                    if let Ok(data) = frame.into_data() {
                        return Poll::Ready(Some(Ok(data)));
                    }
                    // Skip non-data frames (trailers, etc.)
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(TowerBodyError(e))));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Error wrapper for body stream errors
// ---------------------------------------------------------------------------

/// Error produced when reading a Tower response body.
#[derive(Debug)]
pub struct TowerBodyError<E>(pub E);

impl<E: std::fmt::Display> std::fmt::Display for TowerBodyError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tower body error: {}", self.0)
    }
}

impl<E: std::error::Error + 'static> std::error::Error for TowerBodyError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl<E: std::fmt::Display + 'static> From<TowerBodyError<E>> for actix_web::Error {
    fn from(e: TowerBodyError<E>) -> Self {
        let boxed: BoxError = Box::new(crate::internal::common::StringError(e.0.to_string()));
        actix_web::Error::from(crate::internal::common::TowerError(boxed))
    }
}

// ---------------------------------------------------------------------------
// Collect body into Bytes
// ---------------------------------------------------------------------------

/// Collects an `http_body::Body` fully into `Bytes`.
///
/// Used for server-generated response bodies where size is controlled.
/// For request bodies from untrusted clients, use [`collect_body_limited`].
pub async fn collect_body<B>(body: B) -> Result<Bytes, B::Error>
where
    B: HttpBody + Unpin,
{
    use http_body_util::BodyExt;
    let collected = body.collect().await?;
    Ok(collected.to_bytes())
}

/// Collects an `http_body::Body` into `Bytes`, enforcing a byte-count limit.
///
/// Returns `Err` if the body exceeds `limit` bytes or if a read error occurs.
/// No more than `limit + 1` bytes of body data are ever buffered; the stream
/// is aborted as soon as the limit is breached.
///
/// # Errors
///
/// - `Err("request body too large")` when the body exceeds the limit.
/// - `Err(body_error)` when the underlying body stream returns an error.
pub async fn collect_body_limited<B>(body: B, limit: usize) -> Result<Bytes, BoxError>
where
    B: HttpBody<Data = Bytes> + Unpin,
    B::Error: Into<BoxError>,
{
    use http_body_util::{BodyExt, Limited};

    let limited = Limited::new(body, limit);
    match limited.collect().await {
        Ok(collected) => Ok(collected.to_bytes()),
        Err(e) => {
            // http_body_util::Limited wraps the inner error OR emits a
            // LengthLimitError.  Both are surfaced via the same error type.
            Err(Box::new(StringError(format!(
                "request body too large or read error: {e}"
            ))))
        }
    }
}
