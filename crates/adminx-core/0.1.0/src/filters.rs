// adminx-core/src/filters.rs
use crate::storage::{FilterClause, FilterOp, QueryOptions};
use serde::Serialize;
use std::collections::HashMap;

/// The kind of input rendered for a filter, and how its value is matched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterKind {
    /// Free-text box; matched as a case-insensitive substring.
    Text,
    /// Dropdown of `options`; matched exactly.
    Select,
    /// Yes/No dropdown; matched exactly.
    Boolean,
    /// A pair of date inputs (`{field}_from` / `{field}_to`); matched as a
    /// `>= from AND <= to` range. Ideal for timestamp columns like `created_at`.
    DateRange,
}

/// One option in a `Select` filter.
#[derive(Debug, Clone, Serialize)]
pub struct FilterOption {
    pub value: String,
    pub label: String,
}

impl FilterOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self { value: value.into(), label: label.into() }
    }
}

/// A filter a resource exposes on its list page. Declare these from
/// `Resource::filterable_fields`; the list UI and query builder do the rest.
#[derive(Debug, Clone, Serialize)]
pub struct FilterField {
    pub field: String,
    pub label: String,
    pub kind: FilterKind,
    pub options: Vec<FilterOption>,
}

impl FilterField {
    pub fn text(field: impl Into<String>, label: impl Into<String>) -> Self {
        Self { field: field.into(), label: label.into(), kind: FilterKind::Text, options: Vec::new() }
    }

    pub fn boolean(field: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            label: label.into(),
            kind: FilterKind::Boolean,
            options: vec![FilterOption::new("true", "Yes"), FilterOption::new("false", "No")],
        }
    }

    pub fn select(field: impl Into<String>, label: impl Into<String>, options: Vec<FilterOption>) -> Self {
        Self { field: field.into(), label: label.into(), kind: FilterKind::Select, options }
    }

    pub fn date_range(field: impl Into<String>, label: impl Into<String>) -> Self {
        Self { field: field.into(), label: label.into(), kind: FilterKind::DateRange, options: Vec::new() }
    }
}

/// Extract active filter clauses from a query string, limited to the resource's
/// declared `fields` (so arbitrary columns can't be filtered on). Text fields
/// match as substrings; select/boolean match exactly; date-range fields read
/// `{field}_from` / `{field}_to` into `>=` / `<=` clauses.
pub fn parse_filters(query: &str, fields: &[FilterField]) -> Vec<FilterClause> {
    if fields.is_empty() {
        return Vec::new();
    }
    let params: HashMap<String, String> = serde_urlencoded::from_str(query).unwrap_or_default();
    let get = |k: &str| params.get(k).map(|s| s.trim()).filter(|s| !s.is_empty());

    let mut out = Vec::new();
    for f in fields {
        match f.kind {
            FilterKind::DateRange => {
                if let Some(from) = get(&format!("{}_from", f.field)) {
                    out.push(FilterClause { field: f.field.clone(), op: FilterOp::Gte, value: from.to_string() });
                }
                if let Some(to) = get(&format!("{}_to", f.field)) {
                    // Bare `YYYY-MM-DD` "to" should cover the whole day.
                    let value = if to.len() == 10 { format!("{to}T23:59:59") } else { to.to_string() };
                    out.push(FilterClause { field: f.field.clone(), op: FilterOp::Lte, value });
                }
            }
            kind => {
                if let Some(v) = get(&f.field) {
                    let op = if kind == FilterKind::Text { FilterOp::Contains } else { FilterOp::Eq };
                    out.push(FilterClause { field: f.field.clone(), op, value: v.to_string() });
                }
            }
        }
    }
    out
}

/// Raw filter input values for repopulating the form, keyed by input name
/// (`field` for simple filters, `field_from`/`field_to` for date ranges).
pub fn filter_values(query: &str, fields: &[FilterField]) -> HashMap<String, String> {
    let params: HashMap<String, String> = serde_urlencoded::from_str(query).unwrap_or_default();
    let mut out = HashMap::new();
    let mut take = |key: &str| {
        if let Some(v) = params.get(key).map(|s| s.trim()).filter(|s| !s.is_empty()) {
            out.insert(key.to_string(), v.to_string());
        }
    };
    for f in fields {
        match f.kind {
            FilterKind::DateRange => {
                take(&format!("{}_from", f.field));
                take(&format!("{}_to", f.field));
            }
            _ => take(&f.field),
        }
    }
    out
}

/// Default page size when the client doesn't specify one.
pub const DEFAULT_PER_PAGE: u64 = 25;
/// Hard upper bound on client-requested page size.
pub const MAX_PER_PAGE: u64 = 200;

/// Clamp pagination to safe bounds (`page >= 1`, `1 <= per_page <= MAX`).
pub fn clamp_pagination(page: u64, per_page: u64) -> (u64, u64) {
    (page.max(1), per_page.clamp(1, MAX_PER_PAGE))
}

/// Parse a raw query string into pagination + ordering.
/// `sort=name` ascending, `sort=-name` descending.
pub fn parse_query(query: &str) -> QueryOptions {
    let params: HashMap<String, String> = serde_urlencoded::from_str(query).unwrap_or_default();

    let page = params
        .get("page")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(1);
    let per_page = params
        .get("per_page")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_PER_PAGE);
    let (page, per_page) = clamp_pagination(page, per_page);

    let (sort_by, sort_desc) = match params.get("sort") {
        Some(raw) if raw.starts_with('-') => (Some(raw[1..].to_string()), true),
        Some(raw) if !raw.is_empty() => (Some(raw.clone()), false),
        _ => (None, false),
    };

    QueryOptions {
        page,
        per_page,
        sort_by,
        sort_desc,
        filters: Vec::new(),
    }
}
