// adminx-core/src/resource.rs
//
// The framework-neutral Resource trait. Default CRUD is expressed purely in
// terms of `ReqCtx` -> `ApiResponse` and the global `Storage`, so a single
// implementation serves Actix, Axum, or any future adapter, over SQL or Mongo.

use crate::actions::CustomAction;
use crate::authz::Action;
use crate::error::CoreError;
use crate::export::{rows_to_csv, EXPORT_CAP};
use crate::filters::parse_query;
use crate::menu::{MenuAction, MenuItem};
use crate::request::ReqCtx;
use crate::response::{ApiBody, ApiResponse};
use crate::storage::{storage, CreateOutcome, QueryOptions};
use crate::ui;
use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::collections::HashSet;

#[async_trait]
pub trait Resource: Send + Sync {
    // ===== REQUIRED =====
    fn resource_name(&self) -> &'static str;
    fn base_path(&self) -> &'static str;
    /// Backing table (SQL) or collection (Mongo) name.
    fn table_name(&self) -> &'static str;
    fn clone_box(&self) -> Box<dyn Resource>;

    // ===== CONFIG (defaults) =====
    fn primary_key(&self) -> &'static str {
        "id"
    }
    fn menu_group(&self) -> Option<&'static str> {
        None
    }
    fn menu(&self) -> &'static str {
        self.resource_name()
    }
    fn allowed_roles(&self) -> Vec<String> {
        vec!["admin".to_string()]
    }
    fn allowed_actions(&self) -> Option<Vec<MenuAction>> {
        None
    }

    /// Extra id-scoped operations beyond CRUD, exposed at
    /// `POST /{base}/{id}/action/{name}` and as buttons on the detail page.
    fn custom_actions(&self) -> Vec<CustomAction> {
        vec![]
    }
    /// Mass-assignment allow-list for create/update.
    fn permit_keys(&self) -> Vec<&'static str> {
        vec![]
    }
    /// Columns that must never be client-set.
    fn readonly_keys(&self) -> Vec<&'static str> {
        vec!["id", "created_at", "updated_at"]
    }
    /// Whether delete should soft-delete (set `deleted = true`).
    fn soft_delete(&self) -> bool {
        self.permit_keys().contains(&"deleted")
    }

    /// Custom form layout for create/edit pages. Return `None` to derive fields
    /// from `permit_keys()`. Shape: `{ "groups": [{ "fields": [...] }] }`.
    fn form_structure(&self) -> Option<Value> {
        None
    }

    /// Columns exposed as filters on the list page. Empty (the default) means no
    /// filter bar is shown. Build entries with `FilterField::text/select/boolean`.
    fn filterable_fields(&self) -> Vec<crate::filters::FilterField> {
        Vec::new()
    }

    // ===== DEFAULT CRUD =====

    async fn list(&self, ctx: &ReqCtx) -> ApiResponse {
        if !self.authorize(ctx, Action::List) {
            return CoreError::Unauthorized.into();
        }
        let opts = parse_query(&ctx.query);
        match storage().list(self.table_name(), &opts).await {
            Ok(page) => ApiResponse::ok(json!({
                "data": page.rows,
                "total": page.total,
                "page": opts.page,
                "per_page": opts.per_page,
            })),
            Err(e) => CoreError::from(e).into(),
        }
    }

    async fn get(&self, ctx: &ReqCtx, id: &str) -> ApiResponse {
        if !self.authorize(ctx, Action::Read) {
            return CoreError::Unauthorized.into();
        }
        match storage().get(self.table_name(), self.primary_key(), id).await {
            Ok(Some(row)) => ApiResponse::ok(row),
            Ok(None) => CoreError::NotFound.into(),
            Err(e) => CoreError::from(e).into(),
        }
    }

    async fn create(&self, ctx: &ReqCtx, body: Value) -> ApiResponse {
        if !self.authorize(ctx, Action::Create) {
            return CoreError::Unauthorized.into();
        }
        let data = match self.filter_writable(body) {
            Ok(d) => d,
            Err(resp) => return resp,
        };
        match storage().create(self.table_name(), data).await {
            Ok(CreateOutcome { last_insert_id }) => ApiResponse::created(json!({
                "success": true,
                "message": format!("{} created successfully", self.resource_name()),
                "last_insert_id": last_insert_id,
            })),
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
            Ok(n) if n > 0 => ApiResponse::ok(json!({
                "success": true,
                "message": format!("{} updated successfully", self.resource_name()),
                "modified_count": n,
            })),
            Ok(_) => CoreError::NotFound.into(),
            Err(e) => CoreError::from(e).into(),
        }
    }

    async fn delete(&self, ctx: &ReqCtx, id: &str) -> ApiResponse {
        if !self.authorize(ctx, Action::Delete) {
            return CoreError::Unauthorized.into();
        }
        let soft = self.soft_delete();
        match storage()
            .delete(self.table_name(), self.primary_key(), id, soft)
            .await
        {
            Ok(n) if n > 0 => ApiResponse::ok(json!({
                "success": true,
                "message": format!("{} deleted successfully", self.resource_name()),
                "soft_delete": soft,
                "affected": n,
            })),
            Ok(_) => CoreError::NotFound.into(),
            Err(e) => CoreError::from(e).into(),
        }
    }

    // ===== HTML UI PAGES (served identically by every web adapter) =====

    /// The form fields to render on create/edit. Derived from an explicit
    /// `form_structure()` when present, otherwise from `permit_keys()`.
    fn form_fields(&self) -> Vec<Value> {
        match self.form_structure() {
            Some(structure) => ui::fields_from_structure(&structure),
            None => ui::default_fields(&self.permit_keys()),
        }
    }

    async fn list_page(&self, ctx: &ReqCtx) -> ApiResponse {
        if !self.authorize(ctx, Action::List) {
            return crate::auth::login_redirect(ctx);
        }

        // `?download=json|csv` exports instead of rendering the table.
        let params: HashMap<String, String> =
            serde_urlencoded::from_str(&ctx.query).unwrap_or_default();
        if let Some(format) = params.get("download") {
            return self.export(ctx, format).await;
        }

        let mut opts = parse_query(&ctx.query);
        let filter_fields = self.filterable_fields();
        opts.filters = crate::filters::parse_filters(&ctx.query, &filter_fields);

        // Raw input values for repopulating the form (handles date-range
        // from/to keys, which the clause list can't represent one-to-one).
        let current_filters = crate::filters::filter_values(&ctx.query, &filter_fields);

        let page = match storage().list(self.table_name(), &opts).await {
            Ok(p) => p,
            Err(e) => return CoreError::from(e).into(),
        };
        let headers = ui::derive_headers(&page.rows, self.primary_key());

        let mut c = ui::base_context(ctx, self.resource_name());
        c.insert("resource_name", self.resource_name());
        c.insert("base_path", self.base_path());
        c.insert("pk", self.primary_key());
        c.insert("headers", &headers);
        c.insert("rows", &page.rows);
        c.insert("total", &page.total);
        c.insert("page", &opts.page);
        c.insert("per_page", &opts.per_page);
        c.insert("filter_fields", &filter_fields);
        c.insert("current_filters", &current_filters);
        c.insert("has_filters", &(!filter_fields.is_empty()));
        c.insert("has_active_filters", &(!opts.filters.is_empty()));
        // Each row carries a delete form, so the page needs a CSRF token.
        ui::render_with_csrf(ctx, c, "list.html")
    }

    async fn new_page(&self, ctx: &ReqCtx) -> ApiResponse {
        if !self.authorize(ctx, Action::Create) {
            return crate::auth::login_redirect(ctx);
        }
        let mut c = ui::base_context(ctx, self.resource_name());
        c.insert("resource_name", self.resource_name());
        c.insert("base_path", self.base_path());
        c.insert("fields", &self.form_fields());
        c.insert("is_edit", &false);
        c.insert("record", &json!({}));
        ui::render_with_csrf(ctx, c, "form.html")
    }

    async fn edit_page(&self, ctx: &ReqCtx, id: &str) -> ApiResponse {
        if !self.authorize(ctx, Action::Update) {
            return crate::auth::login_redirect(ctx);
        }
        let record = match storage().get(self.table_name(), self.primary_key(), id).await {
            Ok(Some(r)) => r,
            Ok(None) => return CoreError::NotFound.into(),
            Err(e) => return CoreError::from(e).into(),
        };
        let mut c = ui::base_context(ctx, self.resource_name());
        c.insert("resource_name", self.resource_name());
        c.insert("base_path", self.base_path());
        c.insert("fields", &self.form_fields());
        c.insert("is_edit", &true);
        c.insert("item_id", &id);
        c.insert("record", &record);
        ui::render_with_csrf(ctx, c, "form.html")
    }

    async fn view_page(&self, ctx: &ReqCtx, id: &str) -> ApiResponse {
        if !self.authorize(ctx, Action::Read) {
            return crate::auth::login_redirect(ctx);
        }
        let record = match storage().get(self.table_name(), self.primary_key(), id).await {
            Ok(Some(r)) => r,
            Ok(None) => return CoreError::NotFound.into(),
            Err(e) => return CoreError::from(e).into(),
        };
        let headers = ui::derive_headers(std::slice::from_ref(&record), self.primary_key());
        let actions: Vec<Value> = self
            .custom_actions()
            .iter()
            .map(|a| json!({ "name": a.name, "label": a.display_label() }))
            .collect();

        let mut c = ui::base_context(ctx, self.resource_name());
        c.insert("resource_name", self.resource_name());
        c.insert("base_path", self.base_path());
        c.insert("item_id", &id);
        c.insert("headers", &headers);
        c.insert("record", &record);
        c.insert("actions", &actions);
        // The detail page renders a POST form per custom action.
        ui::render_with_csrf(ctx, c, "view.html")
    }

    /// Handle a submitted create form; redirects to the list on success.
    async fn create_form(&self, ctx: &ReqCtx, mut form: HashMap<String, String>) -> ApiResponse {
        if !self.authorize(ctx, Action::Create) {
            return crate::auth::login_redirect(ctx);
        }
        if let Some(reject) = csrf_guard(ctx, form.remove(crate::csrf::FIELD_NAME)) {
            return reject;
        }
        let body = ui::form_to_json(form);
        let resp = self.create(ctx, body).await;
        if resp.status < 300 {
            ApiResponse::redirect(format!("{}/{}/list", ctx.mount, self.base_path()))
        } else {
            resp
        }
    }

    /// Handle a submitted edit form; redirects to the item view on success.
    async fn update_form(
        &self,
        ctx: &ReqCtx,
        id: &str,
        mut form: HashMap<String, String>,
    ) -> ApiResponse {
        if !self.authorize(ctx, Action::Update) {
            return crate::auth::login_redirect(ctx);
        }
        if let Some(reject) = csrf_guard(ctx, form.remove(crate::csrf::FIELD_NAME)) {
            return reject;
        }
        let body = ui::form_to_json(form);
        let resp = self.update(ctx, id, body).await;
        if resp.status < 300 {
            ApiResponse::redirect(format!("{}/{}/view/{}", ctx.mount, self.base_path(), id))
        } else {
            resp
        }
    }

    /// Handle a delete from the list UI; redirects back to the list. `csrf` is
    /// the submitted hidden field, checked against the cookie before anything
    /// is removed.
    async fn delete_form(&self, ctx: &ReqCtx, id: &str, csrf: Option<String>) -> ApiResponse {
        if !self.authorize(ctx, Action::Delete) {
            return crate::auth::login_redirect(ctx);
        }
        if let Some(reject) = csrf_guard(ctx, csrf) {
            return reject;
        }
        let resp = self.delete(ctx, id).await;
        if resp.status < 300 {
            ApiResponse::redirect(format!("{}/{}/list", ctx.mount, self.base_path()))
        } else {
            resp
        }
    }

    // ===== CUSTOM ACTIONS =====

    /// Look up a custom action by name and run it (after auth + CSRF checks).
    /// `csrf` is the submitted hidden field; the action button posts a form, so
    /// it's guarded like the other mutating form handlers.
    async fn run_action(
        &self,
        ctx: &ReqCtx,
        name: &str,
        id: String,
        body: Value,
        csrf: Option<String>,
    ) -> ApiResponse {
        if !self.authorize(ctx, Action::Custom(name)) {
            return CoreError::Unauthorized.into();
        }
        if let Some(reject) = csrf_guard(ctx, csrf) {
            return reject;
        }
        for action in self.custom_actions() {
            if action.name == name {
                return (action.handler)(ctx.clone(), id, body).await;
            }
        }
        CoreError::NotFound.into()
    }

    // ===== EXPORT =====

    /// Export the resource's rows as `json` or `csv` (used by `?download=`).
    async fn export(&self, ctx: &ReqCtx, format: &str) -> ApiResponse {
        if !self.authorize(ctx, Action::Export) {
            return crate::auth::login_redirect(ctx);
        }

        let opts = QueryOptions {
            page: 1,
            per_page: EXPORT_CAP,
            sort_by: None,
            sort_desc: false,
            // Export honours the active filters from the list query.
            filters: crate::filters::parse_filters(&ctx.query, &self.filterable_fields()),
        };
        let page = match storage().list(self.table_name(), &opts).await {
            Ok(p) => p,
            Err(e) => return CoreError::from(e).into(),
        };

        match format {
            "json" => {
                let data = serde_json::to_vec_pretty(&page.rows).unwrap_or_default();
                ApiResponse::new(
                    200,
                    ApiBody::Bytes {
                        content_type: "application/json".to_string(),
                        data,
                    },
                )
                .with_header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}.json\"", self.base_path()),
                )
            }
            "csv" => {
                let headers = ui::derive_headers(&page.rows, self.primary_key());
                let data = rows_to_csv(&headers, &page.rows).into_bytes();
                ApiResponse::new(
                    200,
                    ApiBody::Bytes {
                        content_type: "text/csv".to_string(),
                        data,
                    },
                )
                .with_header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}.csv\"", self.base_path()),
                )
            }
            other => {
                CoreError::BadRequest(format!("unsupported export format: {other}")).into()
            }
        }
    }

    // ===== HELPERS =====

    /// Whether the principal in `ctx` may perform `action` on this resource.
    /// Delegates to the authorization seam: always allowed when auth is not
    /// configured; a registered [`Authorizer`](crate::authz::Authorizer) decides
    /// per action; otherwise the principal must hold one of `allowed_roles()`.
    fn authorize(&self, ctx: &ReqCtx, action: Action<'_>) -> bool {
        crate::authz::authorize(ctx, &self.allowed_roles(), self.base_path(), action)
    }

    /// Apply the permit/readonly/primary-key rules to an incoming JSON body,
    /// returning the writable column map or a ready-made error response.
    fn filter_writable(&self, body: Value) -> Result<Map<String, Value>, ApiResponse> {
        let permitted: HashSet<&str> = self.permit_keys().into_iter().collect();
        let readonly: HashSet<&str> = self.readonly_keys().into_iter().collect();
        let pk = self.primary_key();

        let mut out = Map::new();
        if let Value::Object(map) = body {
            for (k, v) in map {
                // Deny-list wins over allow-list; the primary key is never client-set.
                if permitted.contains(k.as_str()) && !readonly.contains(k.as_str()) && k != pk {
                    out.insert(k, v);
                }
            }
        }

        if out.is_empty() {
            return Err(ApiResponse::error(CoreError::BadRequest(
                "No permitted fields in payload".into(),
            )));
        }
        Ok(out)
    }

    // ===== MENU =====
    fn generate_menu(&self) -> Option<MenuItem> {
        Some(MenuItem {
            title: self.menu().to_string(),
            path: self.base_path().to_string(),
            icon: Some("table".to_string()),
            order: Some(10),
            children: None,
        })
    }
}

impl Clone for Box<dyn Resource> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// CSRF check shared by every mutating form handler. Returns `Some(reject)` when
/// the submitted `_csrf` field is missing or doesn't match the cookie, and
/// `None` when the post may proceed. Taking the token by value lets callers hand
/// over the value they lifted out of the form map with `remove`.
fn csrf_guard(ctx: &ReqCtx, submitted: Option<String>) -> Option<ApiResponse> {
    // Mirrors `is_authorized`: with auth unconfigured the whole panel is public
    // by design, so form posts stay frictionless too. Once auth is on, so is this.
    if !crate::auth::is_configured() {
        return None;
    }
    if crate::csrf::verify(ctx, submitted.as_deref()) {
        None
    } else {
        // A 403 the browser can read. These posts are already SameSite-protected,
        // so this fires mainly on a token that lapsed with the browser session —
        // reloading the page mints a fresh one.
        Some(ApiResponse::html(
            403,
            "<h1>403 Forbidden</h1><p>Your session expired or the request could \
             not be verified. Please reload the page and try again.</p>"
                .to_string(),
        ))
    }
}
