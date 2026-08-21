// adminx-core/src/lib.rs
//
// Framework-neutral core of the adminx admin-panel framework. Contains no web
// framework and no database dependency: it defines the `Resource` trait, the
// `Storage` abstraction, the registry, and neutral request/response types.
//
// Web adapters (adminx-actix, adminx-axum) translate their framework's
// request/response to/from `ReqCtx`/`ApiResponse`. Storage adapters
// (adminx-seaorm, adminx-mongo) implement `Storage`.

pub mod actions;
pub mod auth;
pub mod authz;
pub mod csrf;
pub mod error;
pub mod export;
pub mod filters;
pub mod menu;
pub mod mfa;
pub mod ratelimit;
pub mod registry;
pub mod request;
pub mod resource;
pub mod response;
pub mod storage;
pub mod ui;

pub use actions::{ActionFuture, ActionHandler, CustomAction};
pub use auth::AuthConfig;
pub use authz::{authorizer, set_authorizer, Action, Authorizer};
pub use error::CoreError;
pub use filters::{FilterField, FilterKind, FilterOption};
pub use menu::{MenuAction, MenuItem};
pub use registry::{all_resources, get_registered_menus, register_resource};
pub use request::{Claims, ReqCtx};
pub use resource::Resource;
pub use response::{ApiBody, ApiResponse};
pub use storage::{
    seed, set_storage, storage, CreateOutcome, ListPage, QueryOptions, Storage, StorageError,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Common imports for defining resources and wiring an app.
pub mod prelude {
    pub use crate::actions::{ActionFuture, CustomAction};
    pub use crate::auth::{configure as configure_auth, create_admin, AuthConfig};
    pub use crate::authz::{set_authorizer, Action, Authorizer};
    pub use crate::error::CoreError;
    pub use crate::filters::{FilterField, FilterKind, FilterOption};
    pub use crate::registry::{all_resources, register_resource};
    pub use crate::request::{Claims, ReqCtx};
    pub use crate::resource::Resource;
    pub use crate::response::{ApiBody, ApiResponse};
    pub use crate::storage::{seed, set_storage, storage, QueryOptions, Storage};
}
