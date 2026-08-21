// adminx-core/src/ui.rs
//
// HTML rendering for the admin UI. The core renders full HTML pages (via Tera)
// and returns them as `ApiResponse` byte bodies, so both the Actix and Axum
// adapters serve the identical UI without templating logic of their own.

use crate::error::CoreError;
use crate::registry::get_registered_menus;
use crate::request::ReqCtx;
use crate::response::ApiResponse;
use lazy_static::lazy_static;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use tera::{Context, Tera};

lazy_static! {
    static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("layout.html", include_str!("templates/layout.html.tera")),
            ("header.html", include_str!("templates/header.html.tera")),
            ("footer.html", include_str!("templates/footer.html.tera")),
            ("dashboard.html", include_str!("templates/dashboard.html.tera")),
            ("list.html", include_str!("templates/list.html.tera")),
            ("form.html", include_str!("templates/form.html.tera")),
            ("view.html", include_str!("templates/view.html.tera")),
            ("login.html", include_str!("templates/login.html.tera")),
            ("mfa_setup.html", include_str!("templates/mfa_setup.html.tera")),
            ("mfa_backup.html", include_str!("templates/mfa_backup.html.tera")),
            ("mfa_verify.html", include_str!("templates/mfa_verify.html.tera")),
        ])
        .expect("adminx: failed to parse embedded templates");
        tera.autoescape_on(vec![".html"]);
        tera
    };
}

/// Render a named template into an HTML `ApiResponse`.
pub fn render(name: &str, ctx: &Context) -> ApiResponse {
    match TEMPLATES.render(name, ctx) {
        Ok(html) => ApiResponse::html(200, html),
        Err(e) => {
            tracing::error!("adminx template render error [{name}]: {e}");
            ApiResponse::error(CoreError::Internal(format!("template error: {e}")))
        }
    }
}

/// Base template context: title, mount prefix, and the navigation menus.
pub fn base_context(ctx: &ReqCtx, title: &str) -> Context {
    let mut c = Context::new();
    c.insert("title", title);
    c.insert("mount", &ctx.mount);
    c.insert("menus", &get_registered_menus());
    c.insert("is_authenticated", &ctx.claims.is_some());
    c
}

/// Render the top-level dashboard.
pub fn dashboard(ctx: &ReqCtx) -> ApiResponse {
    let c = base_context(ctx, "Dashboard");
    render("dashboard.html", &c)
}

/// Derive table column headers from a set of rows: the primary key first, then
/// the remaining keys of the first row in their natural order.
pub fn derive_headers(rows: &[Value], pk: &str) -> Vec<String> {
    let mut headers: Vec<String> = Vec::new();
    if let Some(Value::Object(first)) = rows.first() {
        if first.contains_key(pk) {
            headers.push(pk.to_string());
        }
        for k in first.keys() {
            if k != pk {
                headers.push(k.clone());
            }
        }
    }
    headers
}

/// Build a default field list (for create/edit forms) from a set of column
/// names, when a resource provides no explicit `form_structure`.
pub fn default_fields(columns: &[&str]) -> Vec<Value> {
    columns
        .iter()
        .map(|name| {
            let field_type = if *name == "deleted" { "checkbox" } else { "text" };
            json!({
                "name": name,
                "label": humanize(name),
                "field_type": field_type,
            })
        })
        .collect()
}

/// Extract a flat `fields` array from an explicit `form_structure` value.
/// Supports `{ "groups": [ { "fields": [...] } ] }` and `{ "fields": [...] }`.
pub fn fields_from_structure(structure: &Value) -> Vec<Value> {
    if let Some(groups) = structure.get("groups").and_then(|g| g.as_array()) {
        return groups
            .iter()
            .filter_map(|g| g.get("fields").and_then(|f| f.as_array()))
            .flatten()
            .cloned()
            .collect();
    }
    if let Some(fields) = structure.get("fields").and_then(|f| f.as_array()) {
        return fields.clone();
    }
    Vec::new()
}

/// Turn a submitted HTML form (all string values) into a typed JSON object:
/// `"true"/"false"` → bool, integer/float text → number, everything else stays
/// a string.
pub fn form_to_json(form: HashMap<String, String>) -> Value {
    let mut map = Map::new();
    for (k, v) in form {
        let value = if v == "true" {
            Value::Bool(true)
        } else if v == "false" {
            Value::Bool(false)
        } else if let Ok(i) = v.parse::<i64>() {
            Value::from(i)
        } else if let Ok(f) = v.parse::<f64>() {
            Value::from(f)
        } else {
            Value::String(v)
        };
        map.insert(k, value);
    }
    Value::Object(map)
}

/// `created_at` → `Created At`.
pub fn humanize(name: &str) -> String {
    name.split('_')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
