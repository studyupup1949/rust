// adminx-core/src/resource.rs
//
// The framework-neutral Resource trait. Default CRUD is expressed purely in
// terms of `ReqCtx` -> `ApiResponse` and the global `Storage`, so a single
// implementation serves Actix, Axum, or any future adapter, over SQL or Mongo.

use crate::actions::CustomAction;
use crate::authz::Action;
use crate::crud;
use crate::error::CoreError;
use crate::export::{rows_to_csv, EXPORT_CAP};
use crate::filters::parse_query;
use crate::menu::{MenuAction, MenuItem};
use crate::request::ReqCtx;
use crate::response::{ApiBody, ApiResponse};
use crate::storage::{storage, QueryOptions};
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

    /// File attachments this resource accepts. Empty (the default) means no
    /// upload widgets are shown. Each field renders an `<input type=file>` on the
    /// detail page, backed by the attach/serve/detach routes. Requires an
    /// attachment backend (`adminx-storage`) to be registered; without one the
    /// widgets are hidden.
    fn file_fields(&self) -> Vec<crate::attach::FileField> {
        Vec::new()
    }

    /// Columns included in this resource's full-text search document. Empty (the
    /// default) means the resource is not searchable — no search box, no
    /// indexing. When non-empty *and* a search backend (`adminx-search`) is
    /// registered, create/update keep the index in sync, delete removes the
    /// record, and the list page grows a `?q=` search box.
    fn search_fields(&self) -> Vec<&'static str> {
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

    /// Create. The body lives in [`crud::create`](crate::crud::create) so an
    /// overriding impl can delegate to it and keep the audit recording; see that
    /// module for why.
    async fn create(&self, ctx: &ReqCtx, body: Value) -> ApiResponse {
        crud::create(self, ctx, body).await
    }

    /// Update. Body in [`crud::update`](crate::crud::update).
    async fn update(&self, ctx: &ReqCtx, id: &str, body: Value) -> ApiResponse {
        crud::update(self, ctx, id, body).await
    }

    /// Delete. Body in [`crud::delete`](crate::crud::delete).
    async fn delete(&self, ctx: &ReqCtx, id: &str) -> ApiResponse {
        crud::delete(self, ctx, id).await
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

        // A searchable resource shows a search box. A `?q=` term switches the
        // table from the normal paginated list to full-text results: ask the
        // index for matching ids in rank order, then hydrate the rows.
        let searchable = crate::search::is_enabled() && !self.search_fields().is_empty();
        let search_term = if searchable {
            crate::search::query_term(ctx)
        } else {
            None
        };

        let page = if let Some(q) = &search_term {
            let ids = crate::search::search_ids(self.base_path(), q, opts.per_page as usize).await;
            let mut rows = Vec::new();
            for id in &ids {
                if let Ok(Some(row)) =
                    storage().get(self.table_name(), self.primary_key(), id).await
                {
                    rows.push(row);
                }
            }
            let total = rows.len() as u64;
            crate::storage::ListPage { rows, total }
        } else {
            match storage().list(self.table_name(), &opts).await {
                Ok(p) => p,
                Err(e) => return CoreError::from(e).into(),
            }
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
        c.insert("searchable", &searchable);
        c.insert("search_term", &search_term.clone().unwrap_or_default());
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
        // The History link is pointless without a log behind it.
        c.insert("audit_enabled", &crate::audit::is_enabled());

        // File fields: render an upload widget per declared field, pre-filled
        // with whatever is already attached. Only when a backend is registered —
        // otherwise the widgets would post to routes that can't store anything.
        let file_fields = self.file_fields();
        let show_files = crate::attach::is_enabled() && !file_fields.is_empty();
        if show_files {
            let attached = crate::attach::list(self.base_path(), id).await;
            let widgets: Vec<Value> = file_fields
                .iter()
                .map(|f| {
                    let current = attached.iter().find(|a| a.field == f.name);
                    json!({
                        "name": f.name,
                        "label": f.label,
                        "accept": f.accept,
                        "filename": current.map(|a| a.filename.clone()),
                        "byte_size": current.map(|a| a.byte_size),
                        "content_type": current.map(|a| a.content_type.clone()),
                    })
                })
                .collect();
            c.insert("file_fields", &widgets);
        }
        c.insert("show_files", &show_files);

        // The detail page renders a POST form per custom action.
        ui::render_with_csrf(ctx, c, "view.html")
    }

    /// Store an uploaded file against this record under `field`. Called by the
    /// web adapter after it parses the multipart body; redirects back to the
    /// detail page on success. Gated on `Update` — attaching a file changes the
    /// record.
    async fn attach_file(
        &self,
        ctx: &ReqCtx,
        id: &str,
        field: &str,
        csrf: Option<String>,
        file: crate::attach::UploadedFile,
    ) -> ApiResponse {
        if !self.authorize(ctx, Action::Update) {
            return crate::auth::login_redirect(ctx);
        }
        if let Some(reject) = csrf_guard(ctx, csrf) {
            return reject;
        }
        // Only fields the resource actually declared are attachable, so the
        // endpoint can't be used to write arbitrary keys.
        if !self.file_fields().iter().any(|f| f.name == field) {
            return CoreError::NotFound.into();
        }
        match crate::attach::store(self.base_path(), id, field, file).await {
            Ok(_) => ApiResponse::redirect(format!(
                "{}/{}/view/{}",
                ctx.mount,
                self.base_path(),
                id
            )),
            Err(resp) => resp,
        }
    }

    /// Stream a stored attachment back. Gated on `Read` — seeing the file is a
    /// read of the record it belongs to. Returns the bytes with the stored
    /// content type and a download-friendly filename.
    async fn serve_file(&self, ctx: &ReqCtx, id: &str, field: &str) -> ApiResponse {
        if !self.authorize(ctx, Action::Read) {
            return crate::auth::login_redirect(ctx);
        }
        let backend = match crate::attach::attachments() {
            Some(b) => b,
            None => return CoreError::NotFound.into(),
        };
        let meta = match backend.get(self.base_path(), id, field).await {
            Ok(Some(m)) => m,
            Ok(None) => return CoreError::NotFound.into(),
            Err(e) => return CoreError::from(e).into(),
        };
        let bytes = match backend.read(&meta.storage_key).await {
            Ok(b) => b,
            Err(e) => return CoreError::from(e).into(),
        };
        ApiResponse::new(
            200,
            crate::response::ApiBody::Bytes {
                content_type: meta.content_type,
                data: bytes,
            },
        )
        // `inline` so an image previews in-browser; the filename is used if the
        // user chooses to save it.
        .with_header(
            "Content-Disposition",
            format!("inline; filename=\"{}\"", sanitize_filename(&meta.filename)),
        )
    }

    /// Remove one field's attachment. Gated on `Update`; redirects back to the
    /// detail page.
    async fn detach_file(
        &self,
        ctx: &ReqCtx,
        id: &str,
        field: &str,
        csrf: Option<String>,
    ) -> ApiResponse {
        if !self.authorize(ctx, Action::Update) {
            return crate::auth::login_redirect(ctx);
        }
        if let Some(reject) = csrf_guard(ctx, csrf) {
            return reject;
        }
        if let Some(backend) = crate::attach::attachments() {
            if let Err(e) = backend.delete(self.base_path(), id, field).await {
                return CoreError::from(e).into();
            }
        }
        ApiResponse::redirect(format!("{}/{}/view/{}", ctx.mount, self.base_path(), id))
    }

    /// The recorded history of one record. Reads through the audit seam, so with
    /// no auditor registered it renders an empty log rather than 404 — the route
    /// exists either way and the page explains itself.
    async fn history_page(&self, ctx: &ReqCtx, id: &str) -> ApiResponse {
        // Gated on Read: seeing what a record used to be is a read of that
        // record, so anyone who may view it may see its history.
        if !self.authorize(ctx, Action::Read) {
            return crate::auth::login_redirect(ctx);
        }
        let versions = crate::audit::history(self.base_path(), id).await;

        let mut c = ui::base_context(ctx, self.resource_name());
        c.insert("resource_name", self.resource_name());
        c.insert("base_path", self.base_path());
        c.insert("item_id", &id);
        c.insert("versions", &versions);
        c.insert("audit_enabled", &crate::audit::is_enabled());
        c.insert("limit", &crate::audit::HISTORY_LIMIT);
        // No form on this page, so no CSRF token is needed.
        ui::render("history.html", &c)
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

/// Strip path separators and control characters from an uploaded filename before
/// it goes into a `Content-Disposition` header, so a crafted name can't inject a
/// header or imply a path. Keeps a plain basename.
fn sanitize_filename(name: &str) -> String {
    name.rsplit(['/', '\\'])
        .next()
        .unwrap_or(name)
        .chars()
        .filter(|c| !c.is_control() && *c != '"')
        .take(255)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sanitize_filename;

    #[test]
    fn filename_is_reduced_to_a_safe_basename() {
        assert_eq!(sanitize_filename("../../etc/passwd"), "passwd");
        assert_eq!(sanitize_filename(r"C:\Windows\evil.exe"), "evil.exe");
        assert_eq!(sanitize_filename("photo.png"), "photo.png");
    }

    #[test]
    fn header_breaking_characters_are_dropped() {
        assert_eq!(
            sanitize_filename("a\"b\r\nContent-Length: 0.png"),
            "abContent-Length: 0.png"
        );
    }
}
