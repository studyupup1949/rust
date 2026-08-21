// adminx-audit/src/resource.rs
//
// The in-panel viewer for the audit log. Registered like any other resource, so
// it inherits the existing list/view routes and templates from every web adapter
// — no route or template changes in adminx-axum / adminx-actix.
//
// It is deliberately **append-only**: the log's value is that it cannot be
// quietly rewritten by the same panel that produces it, so create / update /
// delete are refused here even for an admin. Rows are written only by the
// `Auditor` seam, never through this resource.

use adminx_core::authz::Action;
use adminx_core::error::CoreError;
use adminx_core::filters::{FilterField, FilterOption};
use adminx_core::request::ReqCtx;
use adminx_core::resource::Resource;
use adminx_core::response::ApiResponse;
use async_trait::async_trait;
use serde_json::Value;

/// Read-only view over `adminx_audit_versions`.
#[derive(Clone)]
pub struct AuditVersionResource;

/// The refusal returned by every mutating entry point. 405: the route exists,
/// the verb does not apply to it.
fn append_only() -> ApiResponse {
    ApiResponse::json(
        405,
        serde_json::json!({
            "success": false,
            "message": "The audit log is append-only; entries cannot be created, \
                        edited or deleted from the panel.",
        }),
    )
}

#[async_trait]
impl Resource for AuditVersionResource {
    fn resource_name(&self) -> &'static str {
        "Audit Log"
    }
    fn base_path(&self) -> &'static str {
        "adminx-audit-versions"
    }
    fn table_name(&self) -> &'static str {
        crate::store::TABLE
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    fn menu(&self) -> &'static str {
        "Audit Log"
    }

    /// Nothing is writable. This also means `filter_writable` can never admit a
    /// column, so even if a mutating override were removed the defaults would
    /// reject the body rather than insert a forged entry.
    fn permit_keys(&self) -> Vec<&'static str> {
        vec![]
    }

    fn filterable_fields(&self) -> Vec<FilterField> {
        vec![
            FilterField::text("item_type", "Resource"),
            FilterField::text("item_id", "Record ID"),
            FilterField::select(
                "event",
                "Event",
                vec![
                    FilterOption::new("create", "Created"),
                    FilterOption::new("update", "Updated"),
                    FilterOption::new("delete", "Deleted"),
                ],
            ),
            FilterField::text("whodunnit_email", "Changed by"),
            FilterField::date_range("created_at", "When"),
        ]
    }

    // ===== APPEND-ONLY =====
    //
    // Both the JSON API and the HTML form handlers funnel through these, so the
    // panel offers no path to a write. `new_page` / `edit_page` are refused too,
    // so the UI never renders a form that could not be submitted.

    async fn create(&self, _ctx: &ReqCtx, _body: Value) -> ApiResponse {
        append_only()
    }

    async fn update(&self, _ctx: &ReqCtx, _id: &str, _body: Value) -> ApiResponse {
        append_only()
    }

    async fn delete(&self, _ctx: &ReqCtx, _id: &str) -> ApiResponse {
        append_only()
    }

    async fn new_page(&self, ctx: &ReqCtx) -> ApiResponse {
        if !self.authorize(ctx, Action::Read) {
            return CoreError::Unauthorized.into();
        }
        append_only()
    }

    async fn edit_page(&self, ctx: &ReqCtx, _id: &str) -> ApiResponse {
        if !self.authorize(ctx, Action::Read) {
            return CoreError::Unauthorized.into();
        }
        append_only()
    }
}
