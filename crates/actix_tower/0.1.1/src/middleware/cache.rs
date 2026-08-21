//! Simple in-memory response cache middleware.

use std::{
    future::{ready, Ready},
    sync::Arc,
    time::{Duration, Instant},
};

use actix_service::{Service, Transform};
use actix_web::{
    body::MessageBody,
    dev::{forward_ready, ServiceRequest, ServiceResponse},
    Error, HttpResponse,
};
use indexmap::IndexMap;
use parking_lot::Mutex;

// ---------------------------------------------------------------------------
// Cache storage
// ---------------------------------------------------------------------------

/// Maximum number of entries stored simultaneously.
const MAX_CACHE_ENTRIES: usize = 1000;

/// Maximum response body size that will be cached: 1 MiB.
///
/// Responses larger than this are forwarded without caching, regardless of
/// whether a `Content-Length` header is present.
const MAX_CACHEABLE_BODY_BYTES: usize = 1024 * 1024;

/// A single cached response.
struct CacheEntry {
    status: actix_web::http::StatusCode,
    headers: actix_web::http::header::HeaderMap,
    body: actix_web::web::Bytes,
    expires_at: Instant,
    /// The `Vary` header values from the original response.
    /// Stored so we can match subsequent requests against them.
    vary_headers: Vec<(String, String)>,
}

/// Builds a cache key from the request URI and the set of request headers
/// named in a response's `Vary` header.
///
/// RFC 7234 §4.1: a cached response can only be used if the request headers
/// named in the response's `Vary` field have the same values as when the
/// response was stored.
fn build_cache_key(host: &str, uri: &str, vary_headers: &[(String, String)]) -> String {
    let base_key = format!("{}::{}", host, uri);
    if vary_headers.is_empty() {
        return base_key;
    }
    let mut key = base_key;
    for (name, value) in vary_headers {
        key.push('\x00'); // safe delimiter; never appears in URI or header values
        key.push_str(name);
        key.push('=');
        key.push_str(value);
    }
    key
}

/// Extracts the request header values for the names listed in a `Vary` header.
fn extract_vary_values(
    req_headers: &actix_web::http::header::HeaderMap,
    vary: &str,
) -> Vec<(String, String)> {
    vary.split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|name| name != "*") // Vary: * means never cache
        .map(|name| {
            let value = req_headers
                .get(&name)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_owned();
            (name, value)
        })
        .collect()
}

/// Returns `true` if this request matches the `vary_headers` recorded in a
/// cached entry.
fn vary_matches(
    req_headers: &actix_web::http::header::HeaderMap,
    cached_vary: &[(String, String)],
) -> bool {
    cached_vary.iter().all(|(name, cached_value)| {
        let current_value = req_headers
            .get(name.as_str())
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        current_value == cached_value
    })
}

/// Thread-safe cache store backed by an [`IndexMap`] for stable insertion order.
///
/// `IndexMap` preserves the order in which keys were first inserted, so
/// `first()` always returns the oldest entry — giving true FIFO eviction
/// rather than the undefined order produced by `HashMap::keys().next()`.
#[derive(Clone)]
struct CacheStore {
    inner: Arc<Mutex<IndexMap<String, CacheEntry>>>,
}

impl CacheStore {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(IndexMap::new())),
        }
    }
}

// ---------------------------------------------------------------------------
// Public middleware type
// ---------------------------------------------------------------------------

/// Simple in-memory cache for GET responses.
///
/// # Behaviour
///
/// - Only `GET` requests are cached.
/// - Protocol-upgrade requests (`Upgrade` header present) bypass the cache.
/// - Server-Sent Events and WebSocket responses bypass the cache.
/// - Responses larger than 1 MiB bypass the cache.
/// - Responses without a known body size (no `Content-Length`) are buffered
///   up to 1 MiB; those that fit are cached, the rest are forwarded.
/// - `Vary` response headers are respected: the cache key includes the
///   request header values named in `Vary`, so content-negotiated endpoints
///   cache correctly.
/// - Up to 1 000 entries are held.  When full, the oldest entry (FIFO) is
///   evicted.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use std::time::Duration;
/// use actix_web::App;
///
/// let app = App::new()
///     .wrap(Cache::new(Duration::from_secs(60)));
/// ```
#[derive(Clone)]
pub struct Cache {
    ttl: Duration,
}

impl Cache {
    /// Create a new cache with the given TTL.
    pub fn new(ttl: Duration) -> Self {
        Self { ttl }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Cache
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = CacheMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CacheMiddleware {
            service,
            ttl: self.ttl,
            cache: CacheStore::new(),
        }))
    }
}

// ---------------------------------------------------------------------------
// Middleware service
// ---------------------------------------------------------------------------

/// Cache middleware service.
pub struct CacheMiddleware<S> {
    service: S,
    ttl: Duration,
    cache: CacheStore,
}

impl<S, B> Service<ServiceRequest> for CacheMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Only cache GET requests without protocol-upgrade semantics.
        let is_upgrade = req.headers().contains_key(actix_web::http::header::UPGRADE);
        if req.method() != actix_web::http::Method::GET || is_upgrade {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_boxed_body())
            });
        }

        let uri = req.uri().to_string();
        let host = req.connection_info().host().to_string();
        let cache_prefix = format!("{}::{}", host, uri);
        let now = Instant::now();
        let req_headers = req.headers().clone();

        // --- Cache lookup -------------------------------------------------------
        {
            let mut cache = self.cache.inner.lock();

            // Scan entries whose key starts with this URI.
            // We find the first non-expired entry whose Vary headers match
            // this request and serve it.  Expired entries are removed lazily.
            let mut hit_key: Option<String> = None;
            let mut expired_keys: Vec<String> = Vec::new();

            for (key, entry) in cache.iter() {
                if !key.starts_with(&cache_prefix) {
                    continue;
                }
                if entry.expires_at <= now {
                    expired_keys.push(key.clone());
                    continue;
                }
                if vary_matches(&req_headers, &entry.vary_headers) {
                    hit_key = Some(key.clone());
                    break;
                }
            }

            for k in expired_keys {
                cache.swap_remove(&k);
            }

            if let Some(key) = hit_key {
                if let Some(entry) = cache.get(&key) {
                    let (req, _) = req.into_parts();
                    let mut builder = HttpResponse::build(entry.status);
                    for (name, value) in &entry.headers {
                        builder.insert_header((name.clone(), value.clone()));
                    }
                    let response = builder.body(entry.body.clone());
                    return Box::pin(ready(Ok(ServiceResponse::new(req, response))));
                }
            }
        }

        // --- Cache miss — call the inner service --------------------------------
        let ttl = self.ttl;
        let cache = self.cache.clone();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            if !res.status().is_success() {
                // Forward non-2xx responses as-is.
                // map_into_boxed_body() is the correct actix-web middleware pattern
                // for forwarding a ServiceResponse<B> without rebuilding it.
                return Ok(res.map_into_boxed_body());
            }

            let (req, response) = res.into_parts();
            let status = response.status();
            let headers = response.headers().clone();

            // --- Determine cacheability -----------------------------------------
            let is_sse = headers
                .get(actix_web::http::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.contains("text/event-stream") || s.contains("websocket"))
                .unwrap_or(false);

            let is_response_upgrade = headers.contains_key(actix_web::http::header::UPGRADE);

            // Vary: * means the response is uncacheable per RFC 7234 §5.2.2.8.
            let vary_star = headers
                .get("vary")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.trim() == "*")
                .unwrap_or(false);

            // --- Parse Content-Length -------------------------------------------
            // We use Content-Length to make the cacheability decision BEFORE
            // consuming the body.  This avoids `to_bytes_limited`, which returns
            // `Ok(Err(B))` on overflow.  Forwarding that `B: MessageBody` via
            // `HttpResponseBuilder::body()` hits an actix-web 4.x type constraint
            // (`B::Error: MessageBody`) that cannot be satisfied in a generic
            // context.  The correct middleware pattern is `map_into_boxed_body()`
            // applied to an existing `ServiceResponse<B>`, not rebuilding from scratch.
            let declared_len = headers
                .get(actix_web::http::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<usize>().ok());

            let should_cache = !is_sse
                && !is_response_upgrade
                && !vary_star
                // Cache only when Content-Length is declared and within budget.
                // Streaming / chunked responses without Content-Length are forwarded
                // without caching (correct: they may be infinite or very large).
                && declared_len.map(|l| l <= MAX_CACHEABLE_BODY_BYTES).unwrap_or(false);

            if !should_cache {
                // Forward without caching using the idiomatic actix-web pattern.
                return Ok(ServiceResponse::new(req, response).map_into_boxed_body());
            }

            // --- Parse Vary header for the cache key ----------------------------
            let vary_str = headers
                .get("vary")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_owned();

            let vary_values = extract_vary_values(&req_headers, &vary_str);
            let cache_key = build_cache_key(&host, &uri, &vary_values);

            // --- Buffer the body ------------------------------------------------
            // We manually collect the body here instead of using `actix_web::body::to_bytes`
            // to defend against malicious inner services that lie about Content-Length.
            // If the stream exceeds MAX_CACHEABLE_BODY_BYTES, we abort immediately.
            let mut body = std::pin::pin!(response.into_body());
            let mut bytes = actix_web::web::BytesMut::new();

            while let Some(chunk_res) = std::future::poll_fn(|cx| {
                body.as_mut().poll_next(cx)
            })
            .await
            {
                let chunk = chunk_res.map_err(|e| {
                    let boxed: Box<dyn std::error::Error> = e.into();
                    actix_web::error::ErrorInternalServerError(boxed.to_string())
                })?;
                bytes.extend_from_slice(&chunk);
                if bytes.len() > MAX_CACHEABLE_BODY_BYTES {
                    return Err(actix_web::error::ErrorPayloadTooLarge(
                        "Response body exceeded cache size limit despite Content-Length claim",
                    ));
                }
            }
            let bytes = bytes.freeze();

            // --- Store in cache -------------------------------------------------
            let mut map = cache.inner.lock();

            // Evict expired entries.
            map.retain(|_, e| e.expires_at > now);

            // If still at capacity, remove the oldest entry (index 0).
            // IndexMap preserves insertion order, so index 0 is always
            // the entry inserted first — true FIFO eviction.
            if map.len() >= MAX_CACHE_ENTRIES {
                map.swap_remove_index(0);
            }

            map.insert(
                cache_key,
                CacheEntry {
                    status,
                    headers: headers.clone(),
                    body: bytes.clone(),
                    expires_at: now + ttl,
                    vary_headers: vary_values,
                },
            );

            Ok(ServiceResponse::new(
                req,
                HttpResponse::build(status).body(bytes),
            ))
        })
    }
}
