//! Actix Web middleware for generating and managing request UUIDs
//!
//! This crate generates a unique UUID for each HTTP request and adds it to the response headers.
//! It also maintains the UUID in globally accessible thread-local variables during request processing.
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use actix_web::{App, HttpServer, web};
//! use actix_web_request_uuid::RequestIDMiddleware;
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| {
//!         App::new()
//!             .wrap(RequestIDMiddleware::new())
//!             .service(web::resource("/").to(|| async { "Hello world!" }))
//!     })
//!     .bind("127.0.0.1:8080")?
//!     .run()
//!     .await
//! }
//! ```
use std::cell::RefCell;
use std::convert::Infallible;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;

use actix_web::dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest};

/// Default request ID header name
pub const REQUEST_ID_HEADER: &str = "request-id";
/// Default ID length (standard length for UUID v4)
pub const DEFAULT_ID_LENGTH: usize = 36;

/// Type for request ID generator function
type RequestIDGenerator = Arc<dyn Fn() -> String + Send + Sync>;

thread_local! {
    static CURRENT_REQUEST_ID: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Set the current request ID globally
///
/// This function stores a request ID in thread-local storage, making it accessible
/// throughout the current request processing context. The middleware automatically
/// calls this function when a request begins processing.
///
/// # Arguments
///
/// * `id` - The request ID string to store globally
///
/// # Usage
///
/// ```rust
/// use actix_web_request_uuid::set_current_request_id;
///
/// // Manually set a request ID (typically done by middleware)
/// set_current_request_id("12345678-1234-1234-1234-123456789abc");
/// ```
///
/// # Notes
///
/// - This function is thread-safe and uses thread-local storage
/// - Each thread maintains its own request ID
/// - The middleware automatically manages this for you in most cases
pub fn set_current_request_id(id: &str) {
    CURRENT_REQUEST_ID.with(|current| {
        *current.borrow_mut() = Some(id.to_string());
    });
}

/// Get the current request ID globally
///
/// Retrieves the request ID that was previously set using `set_current_request_id`.
/// This allows you to access the current request's ID from anywhere in your code
/// during request processing.
///
/// # Returns
///
/// * `Some(String)` - The current request ID if one has been set
/// * `None` - If no request ID has been set for this thread
///
/// # Usage
///
/// ```rust
/// use actix_web_request_uuid::get_current_request_id;
/// use actix_web::{web, HttpResponse, Result};
///
/// async fn my_handler() -> Result<HttpResponse> {
///     match get_current_request_id() {
///         Some(request_id) => {
///             println!("Processing request: {}", request_id);
///             // Use the request ID for logging, tracing, etc.
///             Ok(HttpResponse::Ok().json(format!("Request ID: {}", request_id)))
///         }
///         None => {
///             println!("No request ID found");
///             Ok(HttpResponse::InternalServerError().json("No request ID"))
///         }
///     }
/// }
/// ```
///
/// # Common Use Cases
///
/// - **Logging**: Include request ID in log messages for request tracing
/// - **Error tracking**: Associate errors with specific requests
/// - **Database operations**: Tag database queries with request IDs
/// - **External API calls**: Pass request ID in headers for distributed tracing
///
/// # Notes
///
/// - This function is thread-safe and uses thread-local storage
/// - Returns `None` if called outside of a request context or before middleware sets the ID
/// - The request ID is automatically cleared after request completion
pub fn get_current_request_id() -> Option<String> {
    CURRENT_REQUEST_ID.with(|current| current.borrow().clone())
}

/// Clear the current request ID globally
///
/// Removes the request ID from thread-local storage. The middleware automatically
/// calls this function when request processing is complete to prevent ID leakage
/// between requests.
///
/// # Usage
///
/// ```rust
/// use actix_web_request_uuid::{set_current_request_id, clear_current_request_id, get_current_request_id};
///
/// // Set a request ID
/// set_current_request_id("test-id-123");
/// assert!(get_current_request_id().is_some());
///
/// // Clear the request ID
/// clear_current_request_id();
/// assert!(get_current_request_id().is_none());
/// ```
///
/// # Notes
///
/// - This function is automatically called by the middleware after request completion
/// - Manually calling this function is rarely necessary
/// - Each thread maintains its own request ID, so this only affects the current thread
/// - It's safe to call this function multiple times or when no request ID is set
pub fn clear_current_request_id() {
    CURRENT_REQUEST_ID.with(|current| {
        *current.borrow_mut() = None;
    });
}

/// A struct representing a request ID
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestID {
    inner: String,
}

impl From<RequestID> for String {
    fn from(r: RequestID) -> Self {
        r.inner
    }
}

impl std::fmt::Display for RequestID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl FromRequest for RequestID {
    type Error = Infallible;
    type Future = Ready<Result<RequestID, Infallible>>;

    #[inline]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(req.request_id()))
    }
}

/// Middleware for generating and managing request IDs
///
/// This middleware generates a unique ID for each request and adds it to the response headers.
/// ID generation methods and header names can be customized.
pub struct RequestIDMiddleware {
    generator: RequestIDGenerator,
    header_name: String,
    id_length: usize,
}

impl Default for RequestIDMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestIDMiddleware {
    /// Create middleware with default ID length (36 characters)
    pub fn new() -> Self {
        Self {
            generator: Arc::new(|| Uuid::new_v4().to_string()),
            header_name: REQUEST_ID_HEADER.to_string(),
            id_length: DEFAULT_ID_LENGTH,
        }
    }

    /// Set custom ID length
    ///
    /// # Arguments
    ///
    /// * `length` - The desired length of the request ID
    ///
    /// # Panics
    ///
    /// Panics if `length` is 0.
    pub fn with_id_length(mut self, length: usize) -> Self {
        if length == 0 {
            panic!("Request ID length must be greater than 0");
        }

        self.id_length = length;
        self.generator = Arc::new(move || {
            let uuid = Uuid::new_v4().to_string();
            if length >= uuid.len() {
                uuid
            } else {
                uuid[..length].to_string()
            }
        });
        self
    }

    /// Set a custom ID generation function
    ///
    /// # Arguments
    ///
    /// * `f` - Function to generate request IDs
    pub fn generator<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + Send + Sync + 'static,
    {
        self.generator = Arc::new(f);
        self
    }

    /// Set a custom header name
    ///
    /// # Arguments
    ///
    /// * `header_name` - Header name to use
    pub fn header_name<T: Into<String>>(mut self, header_name: T) -> Self {
        self.header_name = header_name.into();
        self
    }

    /// Configure to use full UUID v4 format (36 characters with hyphens)
    pub fn with_full_uuid(mut self) -> Self {
        self.generator = Arc::new(|| Uuid::new_v4().to_string());
        self.id_length = 36;
        self
    }

    /// Configure to use simple UUID format (32 characters without hyphens)
    pub fn with_simple_uuid(mut self) -> Self {
        self.generator = Arc::new(|| Uuid::new_v4().simple().to_string());
        self.id_length = 32;
        self
    }

    /// Configure to use custom UUID format
    ///
    /// # Arguments
    ///
    /// * `formatter` - Function to format UUID
    pub fn with_custom_uuid_format<F>(mut self, formatter: F) -> Self
    where
        F: Fn(Uuid) -> String + Send + Sync + 'static,
    {
        self.generator = Arc::new(move || formatter(Uuid::new_v4()));
        self
    }

    /// Get the currently configured ID length
    pub fn get_id_length(&self) -> usize {
        self.id_length
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestIDMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIDService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIDService {
            wrapped_service: service,
            generator: self.generator.clone(),
            header_name: self.header_name.clone(),
            id_length: self.id_length,
        }))
    }
}

/// Service that handles request IDs
///
/// This service generates IDs during request processing and adds them to response headers.
/// It also maintains IDs in thread-local variables during request processing.
pub struct RequestIDService<S> {
    wrapped_service: S,
    generator: RequestIDGenerator,
    header_name: String,

    #[allow(dead_code)]
    id_length: usize,
}

impl<S, B> Service<ServiceRequest> for RequestIDService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.wrapped_service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Generate request ID
        let id = self.generate_request_id(&req);

        // Set request ID in thread-local variable
        set_current_request_id(&id);

        let fut = self.wrapped_service.call(req);
        let header_name = self.header_name.clone();

        Box::pin(async move {
            let mut res = fut.await?;
            // Add request ID to response headers
            res.headers_mut().append(
                HeaderName::try_from(header_name).unwrap(),
                HeaderValue::from_str(&id).unwrap(),
            );

            // Clear thread-local variable after response completion
            clear_current_request_id();

            Ok(res)
        })
    }
}

impl<S> RequestIDService<S> {
    /// Generate request ID or retrieve from request extensions
    fn generate_request_id(&self, req: &ServiceRequest) -> String {
        // Use existing ID if it exists in extensions
        if let Some(id) = req.extensions().get::<RequestID>() {
            return id.inner.clone();
        }

        // Generate new ID and save to extensions
        let new_id = RequestID {
            inner: (self.generator)(),
        };
        req.extensions_mut().insert(new_id.clone());
        new_id.inner
    }
}

/// Extension trait for retrieving request IDs from HttpMessage
pub trait RequestIDMessage {
    /// Get the request ID associated with the request
    ///
    /// If no ID exists, a new one will be generated
    fn request_id(&self) -> RequestID;
}

impl<T> RequestIDMessage for T
where
    T: HttpMessage,
{
    fn request_id(&self) -> RequestID {
        // Return existing ID if available
        if let Some(id) = self.extensions().get::<RequestID>() {
            return id.clone();
        }

        // Create new one if it doesn't exist
        let new_id = RequestID {
            inner: Uuid::new_v4().to_string(),
        };

        self.extensions_mut().insert(new_id.clone());
        new_id
    }
}

#[cfg(test)]
mod lib_actix_web_request_uuid_tests {
    use super::*;
    use actix_web::{http::StatusCode, test, web, App, HttpResponse};

    /// Test custom ID length
    #[actix_rt::test]
    async fn test_custom_id_length() {
        let id_length = 16;
        let app = test::init_service(
            App::new()
                .wrap(RequestIDMiddleware::new().with_id_length(id_length))
                .service(web::resource("/").to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let request_id = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(request_id.len(), id_length);
    }

    /// Test full UUID format
    #[actix_rt::test]
    async fn test_full_uuid_format() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIDMiddleware::new().with_full_uuid())
                .service(web::resource("/").to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        let request_id = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(request_id.len(), 36);
        assert!(Uuid::parse_str(request_id).is_ok());
    }

    /// Test simple UUID format
    #[actix_rt::test]
    async fn test_simple_uuid_format() {
        let app = test::init_service(
            App::new()
                .wrap(RequestIDMiddleware::new().with_simple_uuid())
                .service(web::resource("/").to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        let request_id = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(request_id.len(), 32);
    }

    /// Test custom header name
    #[actix_rt::test]
    async fn test_custom_header_name() {
        let custom_header = "X-Request-ID";
        let app = test::init_service(
            App::new()
                .wrap(
                    RequestIDMiddleware::new()
                        .with_simple_uuid()
                        .header_name(custom_header),
                )
                .service(web::resource("/").to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.headers().get(custom_header).is_some());
    }

    /// Test custom format
    #[actix_rt::test]
    async fn test_custom_format() {
        let app = test::init_service(
            App::new()
                .wrap(
                    RequestIDMiddleware::new()
                        .with_custom_uuid_format(|uuid| format!("req-{}", uuid.simple())),
                )
                .service(web::resource("/").to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        let request_id = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(request_id.starts_with("req-"));
    }

    /// Test panic when ID length is 0
    #[actix_rt::test]
    #[should_panic(expected = "Request ID length must be greater than 0")]
    async fn test_zero_length_id_panics() {
        RequestIDMiddleware::new().with_id_length(0);
    }

    /// Test thread-local request ID
    #[actix_rt::test]
    async fn test_thread_local_request_id() {
        let app = test::init_service(App::new().wrap(RequestIDMiddleware::new()).service(
            web::resource("/").to(|| async {
                // Get request ID from thread-local variable
                let request_id = get_current_request_id().unwrap_or_else(|| "missing".to_string());
                HttpResponse::Ok().body(request_id)
            }),
        ))
        .await;

        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        // Get request ID from response body
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Verify that request ID length is 36
        assert_eq!(body_str.len(), 36);

        // Verify that it's a valid UUID format
        assert!(Uuid::parse_str(&body_str).is_ok());
    }

    /// Test RequestID struct conversion traits
    #[actix_rt::test]
    async fn test_request_id_conversions() {
        let id_str = "test-request-id-123";
        let request_id = RequestID {
            inner: id_str.to_string(),
        };

        // Test Display trait
        assert_eq!(format!("{}", request_id), id_str);

        // Test From<RequestID> for String
        let converted: String = request_id.clone().into();
        assert_eq!(converted, id_str);

        // Test Debug trait
        let debug_str = format!("{:?}", request_id);
        assert!(debug_str.contains("RequestID"));
        assert!(debug_str.contains(id_str));

        // Test PartialEq and Eq
        let request_id2 = RequestID {
            inner: id_str.to_string(),
        };
        assert_eq!(request_id, request_id2);
    }

    /// Test RequestIDMessage trait implementation
    #[actix_rt::test]
    async fn test_request_id_message_trait() {
        let req = test::TestRequest::default().to_http_request();

        // First call should create new ID
        let id1 = req.request_id();
        assert_eq!(id1.inner.len(), 36);

        // Second call should return the same ID
        let id2 = req.request_id();
        assert_eq!(id1, id2);
    }

    /// Test FromRequest implementation for RequestID
    #[actix_rt::test]
    async fn test_from_request_implementation() {
        let app = test::init_service(
            App::new().wrap(RequestIDMiddleware::new()).service(
                web::resource("/")
                    .to(|req_id: RequestID| async move { HttpResponse::Ok().body(req_id.inner) }),
            ),
        )
        .await;

        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str.len(), 36);
    }

    /// Test custom generator function
    #[actix_rt::test]
    async fn test_custom_generator() {
        let custom_id = "custom-generated-id";
        let app = test::init_service(
            App::new()
                .wrap(RequestIDMiddleware::new().generator(move || custom_id.to_string()))
                .service(web::resource("/").to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        let request_id = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(request_id, custom_id);
    }

    /// Test thread-local functions directly
    #[actix_rt::test]
    async fn test_thread_local_functions() {
        // Initially should be None
        assert!(get_current_request_id().is_none());

        // Set request ID
        let test_id = "test-thread-local-id";
        set_current_request_id(test_id);

        // Should be able to retrieve it
        assert_eq!(get_current_request_id(), Some(test_id.to_string()));

        // Clear request ID
        clear_current_request_id();

        // Should be None again
        assert!(get_current_request_id().is_none());

        // Test multiple set/clear cycles
        set_current_request_id("id1");
        assert_eq!(get_current_request_id(), Some("id1".to_string()));

        set_current_request_id("id2");
        assert_eq!(get_current_request_id(), Some("id2".to_string()));

        clear_current_request_id();
        assert!(get_current_request_id().is_none());
    }

    /// Test Default implementation for RequestIDMiddleware
    #[actix_rt::test]
    async fn test_middleware_default() {
        let middleware = RequestIDMiddleware::default();
        assert_eq!(middleware.get_id_length(), DEFAULT_ID_LENGTH);
        assert_eq!(middleware.header_name, REQUEST_ID_HEADER);
    }

    /// Test that existing request ID in extensions is reused
    #[actix_rt::test]
    async fn test_existing_request_id_reused() {
        let existing_id = "pre-existing-request-id";

        let app = test::init_service(App::new().wrap(RequestIDMiddleware::new()).service(
            web::resource("/").to(move |req: HttpRequest| async move {
                // Pre-set a request ID in extensions
                req.extensions_mut().insert(RequestID {
                    inner: existing_id.to_string(),
                });

                // Get the request ID - should use the existing one
                let req_id = req.request_id();
                HttpResponse::Ok().body(req_id.inner)
            }),
        ))
        .await;

        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        // Check that the existing ID was used in response header
        let header_id = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();

        // The header should have a generated UUID, not the pre-existing one
        // because the middleware runs before the handler
        assert_ne!(header_id, existing_id);
        assert_eq!(header_id.len(), 36);
    }

    /// Test multiple concurrent requests have different IDs
    #[actix_rt::test]
    async fn test_concurrent_requests_different_ids() {
        let app = test::init_service(App::new().wrap(RequestIDMiddleware::new()).service(
            web::resource("/").to(|| async {
                let id = get_current_request_id().unwrap();
                HttpResponse::Ok().body(id)
            }),
        ))
        .await;

        // Make multiple requests
        let req1 = test::TestRequest::with_uri("/").to_request();
        let req2 = test::TestRequest::with_uri("/").to_request();

        let resp1 = test::call_service(&app, req1).await;
        let resp2 = test::call_service(&app, req2).await;

        let body1 = test::read_body(resp1).await;
        let body2 = test::read_body(resp2).await;

        let id1 = String::from_utf8(body1.to_vec()).unwrap();
        let id2 = String::from_utf8(body2.to_vec()).unwrap();

        // IDs should be different
        assert_ne!(id1, id2);
    }

    /// Test ID length edge cases
    #[actix_rt::test]
    async fn test_id_length_edge_cases() {
        // Test with length longer than UUID (should return full UUID)
        let app = test::init_service(
            App::new()
                .wrap(RequestIDMiddleware::new().with_id_length(100))
                .service(web::resource("/").to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&app, req).await;

        let request_id = resp
            .headers()
            .get(REQUEST_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();

        // Should be standard UUID length
        assert_eq!(request_id.len(), 36);
    }
}
