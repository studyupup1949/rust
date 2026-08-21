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

use adminx_core::request::ReqCtx;
use adminx_core::resource::Resource;
use adminx_core::response::ApiResponse;
use async_trait::async_trait;
use serde_json::Value;

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

    // create / update / delete delegate to the shared default bodies, then
    // reload the cache on success. Delegating rather than copying is what keeps
    // permission edits in the audit log — a hand-copied body would silently miss
    // any invariant the default gains later. The form handlers route through
    // these, so both the API and the panel UI trigger a reload.

    async fn create(&self, ctx: &ReqCtx, body: Value) -> ApiResponse {
        let resp = adminx_core::crud::create(self, ctx, body).await;
        if resp.status < 300 {
            reload_cache().await;
        }
        resp
    }

    async fn update(&self, ctx: &ReqCtx, id: &str, body: Value) -> ApiResponse {
        let resp = adminx_core::crud::update(self, ctx, id, body).await;
        if resp.status < 300 {
            reload_cache().await;
        }
        resp
    }

    async fn delete(&self, ctx: &ReqCtx, id: &str) -> ApiResponse {
        let resp = adminx_core::crud::delete(self, ctx, id).await;
        if resp.status < 300 {
            reload_cache().await;
        }
        resp
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
