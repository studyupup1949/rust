//! Prelude module — brings the most commonly used items into scope.
//!
//! ```no_run
//! use actix_tower::prelude::*;
//! ```

// ---- Actix Web re-exports -------------------------------------------------
pub use actix_web::{
    self, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result as ActixResult,
};

// ---- Extractors -----------------------------------------------------------
#[cfg(feature = "extract")]
pub use crate::extract::{
    AutoData, AutoForm, AutoJson, AutoPath, AutoQuery, AutoState, ValidatedForm, ValidatedJson,
    ValidatedQuery,
};

// ---- Middleware -----------------------------------------------------------
#[cfg(feature = "middleware")]
pub use crate::middleware::{
    auth::{AuthExtractor, AuthMiddleware, Authentication, Authorization},
    cache::Cache,
    compression::Compression,
    metrics::{Metrics, MetricsData, MetricsStore},
    rate_limit::{RateLimit, RateLimitConfig},
    request_id::RequestId,
    timeout::Timeout,
    tracing::{TracingConfig, TracingMiddleware},
};

// ---- Tower compatibility --------------------------------------------------
#[cfg(feature = "tower")]
pub use crate::{
    compat::tower::{apply_tower, TowerLayer, TowerLayerCompat},
    tower_layer,
};

// ---- Utils ----------------------------------------------------------------
#[cfg(feature = "utils")]
pub use crate::utils::{
    builder::{AppBuilder, ServiceConfigBuilder},
    error::{ApiError, ApiErrorResponse, ErrorCode},
    response::{ApiResponse, TypedResponse},
    validation::{in_range, is_email, not_empty, Validator},
    RequestExt,
};

// ---- Macros ---------------------------------------------------------------
#[cfg(feature = "macros")]
pub use crate::macros::{extract, handler, route};

// ---- Internal re-exports (commonly used types) ----------------------------
pub use bytes::Bytes;
pub use serde::{Deserialize, Serialize};
pub use serde_json::Value as JsonValue;
