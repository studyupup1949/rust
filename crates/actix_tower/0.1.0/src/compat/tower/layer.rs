//! Tower Layer → Actix Transform adapter.

use actix_service::{Service as ActixService, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error,
};
use futures_util::future::Ready;
use http_body::Body as HttpBody;
use tower_layer::Layer as TowerLayerTrait;
use tower_service::Service as TowerService;

use crate::compat::tower::{
    body::ActixRequestBody,
    request::DEFAULT_MAX_BODY_BYTES,
    service::{ActixServiceWrapper, TowerMiddlewareService},
};

/// Adapter that allows a Tower [`Layer`](tower_layer::Layer) to be used
/// as an Actix [`Transform`](actix_service::Transform).
///
/// # Request Body Limit
///
/// The Tower bridge must buffer the entire request body before the inner Actix
/// service sees it, because Actix's `HttpRequest` is `!Send` and cannot be
/// held across the Tower future boundary.  To prevent OOM from large uploads,
/// a configurable body-size limit is enforced.  The default is 4 MiB.
/// Requests exceeding the limit are rejected with **413 Payload Too Large**.
///
/// ```no_run
/// use actix_tower::compat::tower::TowerLayer;
/// use actix_web::App;
///
/// // Use a 16 MiB limit for a file-upload endpoint
/// let app = App::new().wrap(
///     TowerLayer::new(tower_http::trace::TraceLayer::new_for_http())
///         .with_max_body_bytes(16 * 1024 * 1024),
/// );
/// ```
///
/// # Short-circuit Tower Middleware
///
/// Tower middleware that completely bypasses the inner service (e.g. by
/// returning an HTTP response directly without calling `inner.call(req)`)
/// will cause `TowerMiddlewareService` to return a 500 error, because the
/// original `HttpRequest` cannot be recovered.  Use Actix-native middleware
/// (`Authentication`, `Authorization`) for request rejection.
///
/// # Example
///
/// ```no_run
/// use actix_tower::compat::tower::TowerLayer;
/// use actix_web::App;
///
/// let app = App::new().wrap(TowerLayer::new(
///     tower_http::trace::TraceLayer::new_for_http()
/// ));
/// ```
pub struct TowerLayer<L> {
    /// The wrapped Tower layer.
    pub(crate) layer: L,
    /// Maximum bytes buffered from the request body before the inner service
    /// sees it.  Requests larger than this are rejected with 413.
    max_body_bytes: usize,
}

impl<L> TowerLayer<L> {
    /// Create a new `TowerLayer` with the default 4 MiB body limit.
    pub fn new(layer: L) -> Self {
        Self {
            layer,
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
        }
    }

    /// Set the maximum request body size (in bytes) that will be buffered for
    /// Tower middleware.
    ///
    /// Requests whose body exceeds this limit are rejected with **413 Payload
    /// Too Large** before any buffering beyond the limit occurs.
    pub fn with_max_body_bytes(mut self, limit: usize) -> Self {
        self.max_body_bytes = limit;
        self
    }
}

impl<L> Clone for TowerLayer<L>
where
    L: Clone,
{
    fn clone(&self) -> Self {
        Self {
            layer: self.layer.clone(),
            max_body_bytes: self.max_body_bytes,
        }
    }
}

// ===========================================================================
// Transform implementation
// ===========================================================================

impl<S, L, B, E> Transform<S, ServiceRequest> for TowerLayer<L>
where
    S: ActixService<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
    L: TowerLayerTrait<ActixServiceWrapper<S>> + 'static,
    // `+ Clone` removed: TowerMiddlewareService wraps TS in Rc<RefCell<>>,
    // so cloning the middleware shares the instance rather than cloning TS.
    L::Service: TowerService<http::Request<ActixRequestBody>, Response = http::Response<B>, Error = E>
        + 'static,
    <L::Service as TowerService<http::Request<ActixRequestBody>>>::Future: 'static,
    B: HttpBody<Data = actix_web::web::Bytes> + 'static,
    B::Error: std::fmt::Display + 'static,
    // Tightened from Display: Into<BoxError> is the standard bound used by
    // tower-http, axum, and hyper. E: Error + Send + Sync + 'static does NOT
    // work when E = Box<dyn Error + Send + Sync> due to trait-solver limitations
    // with trait-object auto-trait composition in the current Rust compiler.
    E: Into<crate::internal::common::BoxError> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = TowerMiddlewareService<L::Service>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let wrapper = ActixServiceWrapper::new(service, self.max_body_bytes);
        let tower_service = self.layer.layer(wrapper);
        let transform = TowerMiddlewareService::new(tower_service, self.max_body_bytes);
        futures_util::future::ready(Ok(transform))
    }
}

/// Compatibility alias for `TowerLayer`.
pub type TowerLayerCompat<L> = TowerLayer<L>;
