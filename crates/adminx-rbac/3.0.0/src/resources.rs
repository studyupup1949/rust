// adminx-rbac/src/resources.rs
//
// Two adminx resources that make roles and permissions editable inside the
// panel — the whole point of a DB-backed model (change access without a
// redeploy). Both are admin-only.
//
// `PermissionResource` writes are what change authorization, so its create /
// update / delete refresh the in-memory cache on success (the reload-on-write
// invalidation strategy). `RoleResource` is metadata the `can` check never
// reads, so it needs no reload.
//
// A `PermissionResource` mutation legitimately stores the wildcard grants `"*"`
// (any resource) and `"manage"` (any action) — those are not rejected here; they
// are how an admin grants broad access from the panel.

use adminx_core::authz::Action;
use adminx_core::error::CoreError;
use adminx_core::request::ReqCtx;
use adminx_core::resource::Resource;
use adminx_core::response::ApiResponse;
use adminx_core::storage::storage;
use async_trait::async_trait;
use serde_json::{json, Value};

/// Refresh the authorizer cache after a permission write; log but don't fail the
/// request if the reload itself errors (the write already succeeded).
async fn reload_cache() {
    if let Err(e) = crate::reload().await {
        tracing::error!("adminx-rbac: cache reload after a permission write failed: {e:?}");
    }
}

/// Editable `adminx_permissions` rows: one grant = `(role, resource, action)`.
#[derive(Clone)]
pub struct PermissionResource;

#[async_trait]
impl Resource for PermissionResource {
    fn resource_name(&self) -> &'static str {
        "Permissions"
    }
    fn base_path(&self) -> &'static str {
        "adminx-permissions"
    }
    fn table_name(&self) -> &'static str {
        "adminx_permissions"
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    fn permit_keys(&self) -> Vec<&'static str> {
        vec!["role", "resource", "action"]
    }
    fn menu(&self) -> &'static str {
        "Permissions"
    }

    // create / update / delete mirror the trait defaults, then reload the cache
    // on success. (A default trait method can't be called from an override in
    // Rust, so the small bodies are reproduced.) The form handlers delegate to
    // these, so both the API and the panel UI trigger a reload.

    async fn create(&self, ctx: &ReqCtx, body: Value) -> ApiResponse {
        if !self.authorize(ctx, Action::Create) {
            return CoreError::Unauthorized.into();
        }
        let data = match self.filter_writable(body) {
            Ok(d) => d,
            Err(resp) => return resp,
        };
        match storage().create(self.table_name(), data).await {
            Ok(_) => {
                reload_cache().await;
                ApiResponse::created(json!({ "success": true, "message": "Permission created" }))
            }
            Err(e) => CoreError::from(e).into(),
        }
    }

    async fn update(&self, ctx: &ReqCtx, id: &str, body: Value) -> ApiResponse {
        if !self.authorize(ctx, Action::Update) {
            return CoreError::Unauthorized.into();
        }
        let data = match self.filter_writable(body) {
            Ok(d) => d,
            Err(resp) => return resp,
        };
        match storage()
            .update(self.table_name(), self.primary_key(), id, data)
            .await
        {
            Ok(n) if n > 0 => {
                reload_cache().await;
                ApiResponse::ok(json!({ "success": true, "message": "Permission updated", "modified_count": n }))
            }
            Ok(_) => CoreError::NotFound.into(),
            Err(e) => CoreError::from(e).into(),
        }
    }

    async fn delete(&self, ctx: &ReqCtx, id: &str) -> ApiResponse {
        if !self.authorize(ctx, Action::Delete) {
            return CoreError::Unauthorized.into();
        }
        match storage()
            .delete(self.table_name(), self.primary_key(), id, false)
            .await
        {
            Ok(n) if n > 0 => {
                reload_cache().await;
                ApiResponse::ok(json!({ "success": true, "message": "Permission deleted", "affected": n }))
            }
            Ok(_) => CoreError::NotFound.into(),
            Err(e) => CoreError::from(e).into(),
        }
    }
}

/// Editable `adminx_roles` metadata (name + description). Not consulted by the
/// `can` check, so no cache reload is needed on write.
#[derive(Clone)]
pub struct RoleResource;

#[async_trait]
impl Resource for RoleResource {
    fn resource_name(&self) -> &'static str {
        "Roles"
    }
    fn base_path(&self) -> &'static str {
        "adminx-roles"
    }
    fn table_name(&self) -> &'static str {
        "adminx_roles"
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    fn permit_keys(&self) -> Vec<&'static str> {
        vec!["name", "description"]
    }
    fn menu(&self) -> &'static str {
        "Roles"
    }
}
