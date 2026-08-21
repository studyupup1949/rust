// adminx-core/src/actions.rs
//
// Framework-neutral custom actions: extra id-scoped operations a resource
// exposes beyond CRUD (e.g. "publish", "approve", "toggle-status"). The handler
// is a plain fn pointer returning a neutral `ApiResponse`, so it works under any
// web adapter. Wired by adapters at `POST /{base}/{id}/action/{name}`.

use crate::request::ReqCtx;
use crate::response::ApiResponse;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

/// Boxed future returned by an action handler.
pub type ActionFuture = Pin<Box<dyn Future<Output = ApiResponse> + Send>>;

/// An action handler receives the (cloned) request context, the target record
/// id, and the JSON request body (`{}` when none was sent).
pub type ActionHandler = fn(ReqCtx, String, Value) -> ActionFuture;

/// Describes a custom action exposed by a resource.
pub struct CustomAction {
    /// URL/segment name, e.g. `"publish"`.
    pub name: &'static str,
    /// Human label for the UI button (defaults to `name`).
    pub label: Option<&'static str>,
    /// Server-side handler.
    pub handler: ActionHandler,
}

impl CustomAction {
    pub fn new(name: &'static str, handler: ActionHandler) -> Self {
        Self {
            name,
            label: None,
            handler,
        }
    }

    pub fn labeled(name: &'static str, label: &'static str, handler: ActionHandler) -> Self {
        Self {
            name,
            label: Some(label),
            handler,
        }
    }

    pub fn display_label(&self) -> &'static str {
        self.label.unwrap_or(self.name)
    }
}
