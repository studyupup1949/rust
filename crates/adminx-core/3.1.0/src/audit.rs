// adminx-core/src/audit.rs
//
// The audit seam. Default CRUD records who changed what, through a pluggable
// backend — the same pattern `storage` and `authz` use. With no auditor
// registered nothing is recorded and no extra query is issued, so the cost is
// zero until a crate like `adminx-audit` opts in.
//
// Why this lives on `Resource` and not `Storage`: only the resource layer holds
// the `ReqCtx`, and without it there is no *whodunnit*. The trade-off is that a
// resource which overrides `create`/`update`/`delete` outright bypasses
// auditing — such an override owns the recording itself. See `Resource`.

use crate::error::CoreError;
use crate::request::ReqCtx;
use crate::response::ApiResponse;
use crate::storage::StorageError;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use serde_json::{json, Map, Value};

/// What happened to a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Create,
    Update,
    Delete,
}

impl Event {
    /// The token stored in the `event` column.
    pub fn as_str(&self) -> &'static str {
        match self {
            Event::Create => "create",
            Event::Update => "update",
            Event::Delete => "delete",
        }
    }
}

/// One recorded change to one record.
///
/// `changes` is PaperTrail's `object_changes` shape — a map of column name to a
/// `[old, new]` pair, holding *only* the columns that actually changed. A create
/// has `null` on the left, a delete `null` on the right.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// The resource's `base_path()`, identifying what kind of record changed.
    pub item_type: String,
    /// Primary key of the affected record. Empty when a create didn't report one.
    pub item_id: String,
    pub event: Event,
    /// `claims.sub` of the acting principal; `None` when auth is unconfigured.
    pub whodunnit: Option<String>,
    /// `claims.email`, denormalized so the log stays readable after the admin
    /// user is renamed or removed.
    pub whodunnit_email: Option<String>,
    pub changes: Map<String, Value>,
}

impl AuditEntry {
    /// Build an entry, lifting the actor out of the request context.
    pub fn new(
        ctx: &ReqCtx,
        item_type: impl Into<String>,
        item_id: impl Into<String>,
        event: Event,
        changes: Map<String, Value>,
    ) -> Self {
        let (whodunnit, whodunnit_email) = match &ctx.claims {
            Some(c) => (Some(c.sub.clone()), Some(c.email.clone())),
            None => (None, None),
        };
        Self {
            item_type: item_type.into(),
            item_id: item_id.into(),
            event,
            whodunnit,
            whodunnit_email,
            changes,
        }
    }
}

/// A pluggable audit sink. Unlike [`Authorizer`](crate::authz::Authorizer) this
/// *is* async and does I/O — it runs once per mutation, not several times per
/// request.
#[async_trait]
pub trait Auditor: Send + Sync {
    async fn record(&self, entry: AuditEntry) -> Result<(), StorageError>;

    /// Recorded entries for one record, newest first, capped at `limit`.
    ///
    /// Rows come back in whatever shape the backend stored them; the history
    /// page reads the `event` / `whodunnit_email` / `changes` / `created_at`
    /// keys and tolerates their absence. The default returns nothing, so a
    /// write-only sink (shipping to syslog, say) needn't implement reading —
    /// the history page then simply shows an empty log.
    async fn history(
        &self,
        _item_type: &str,
        _item_id: &str,
        _limit: u64,
    ) -> Result<Vec<Value>, StorageError> {
        Ok(Vec::new())
    }

    /// Whether a failed audit write should fail the operation that triggered it.
    ///
    /// The default is best-effort: the error is logged and the write stands, so
    /// a sick audit table cannot take the panel down. Returning `true` surfaces
    /// the failure to the caller as a 500 instead — see [`emit`] for exactly
    /// what that does and does not guarantee.
    fn strict(&self) -> bool {
        false
    }
}

static AUDITOR: OnceCell<Box<dyn Auditor>> = OnceCell::new();

/// Register the global audit sink. Set-once, matching `set_storage` and
/// `set_authorizer`.
pub fn set_auditor(auditor: Box<dyn Auditor>) {
    if AUDITOR.set(auditor).is_err() {
        tracing::warn!("adminx auditor already initialized; ignoring reset");
    }
}

/// The registered auditor, if any.
pub fn auditor() -> Option<&'static dyn Auditor> {
    AUDITOR.get().map(|b| b.as_ref())
}

/// Whether auditing is on. Default CRUD checks this before spending a read to
/// capture before-state, so an unaudited app issues exactly the queries it did
/// before this module existed.
pub fn is_enabled() -> bool {
    AUDITOR.get().is_some()
}

/// Hand an entry to the registered auditor.
///
/// Returns `Some(response)` only when the auditor is [`strict`](Auditor::strict)
/// *and* the write failed — the caller returns it in place of its success
/// response.
///
/// **What strict mode does not do:** the mutation has already been committed by
/// the time this runs, and [`Storage`](crate::storage::Storage) exposes no
/// transaction spanning both writes. So a strict failure reports a 500 for a
/// change that *did* land. It converts a silent hole in the audit log into a
/// loud one; it is not atomicity. Recording before the mutation would be worse —
/// it would log changes that never happened.
pub async fn emit(entry: AuditEntry) -> Option<ApiResponse> {
    let auditor = auditor()?;
    match auditor.record(entry).await {
        Ok(()) => None,
        Err(e) => {
            tracing::error!("adminx: failed to record audit entry: {e}");
            if auditor.strict() {
                Some(
                    CoreError::Internal(format!(
                        "the change was applied but could not be recorded to the audit log: {e}"
                    ))
                    .into(),
                )
            } else {
                None
            }
        }
    }
}

/// How many versions the history page shows. An audit table only grows, so the
/// per-record view is capped rather than paginated for now.
pub const HISTORY_LIMIT: u64 = 100;

/// Entries for one record, shaped for the history template: the stored
/// `changes` document is expanded into a per-field `[old, new]` list so the
/// view layer doesn't have to parse JSON.
///
/// Returns an empty list when auditing is off, so callers can render the page
/// unconditionally.
pub async fn history(item_type: &str, item_id: &str) -> Vec<Value> {
    let Some(auditor) = auditor() else {
        return Vec::new();
    };
    let rows = match auditor.history(item_type, item_id, HISTORY_LIMIT).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("adminx: failed to read audit history: {e}");
            return Vec::new();
        }
    };
    rows.iter().map(present).collect()
}

/// One stored row -> the shape the template renders.
fn present(row: &Value) -> Value {
    // `changes` is stored as a JSON string (the one representation every backend
    // keeps identically); a backend that stored it natively works here too.
    let changes = match row.get("changes") {
        Some(Value::String(s)) => serde_json::from_str(s).unwrap_or(Value::Null),
        Some(other) => other.clone(),
        None => Value::Null,
    };

    let mut fields = Vec::new();
    if let Value::Object(map) = &changes {
        for (name, pair) in map {
            let (old, new) = match pair {
                Value::Array(a) if a.len() == 2 => (display(&a[0]), display(&a[1])),
                other => (String::new(), display(other)),
            };
            fields.push(json!({ "name": name, "old": old, "new": new }));
        }
    }

    json!({
        "id": row.get("id").cloned().unwrap_or(Value::Null),
        "event": row.get("event").and_then(|v| v.as_str()).unwrap_or(""),
        "whodunnit_email": row
            .get("whodunnit_email")
            .and_then(|v| v.as_str())
            .unwrap_or("—"),
        "created_at": row.get("created_at").and_then(|v| v.as_str()).unwrap_or(""),
        "fields": fields,
    })
}

/// Render a stored value for the diff table. `null` becomes an em dash so an
/// absent value reads differently from an empty string — the distinction the
/// diff deliberately preserves.
fn display(v: &Value) -> String {
    match v {
        Value::Null => "—".to_string(),
        Value::String(s) if s.is_empty() => "(empty)".to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Columns never worth recording: they change on every write and say nothing
/// about intent.
const NOISE: [&str; 2] = ["updated_at", "created_at"];

/// Diff a submitted write against the row it replaced, keeping only the columns
/// that actually changed.
///
/// `before` is the stored row (`None` for a create). Returns `[old, new]` pairs.
pub fn diff(before: Option<&Value>, after: &Map<String, Value>) -> Map<String, Value> {
    let mut out = Map::new();
    for (key, new) in after {
        if NOISE.contains(&key.as_str()) {
            continue;
        }
        let old = before
            .and_then(|b| b.get(key))
            .cloned()
            .unwrap_or(Value::Null);
        if !same(&old, new) {
            out.insert(key.clone(), json!([old, new]));
        }
    }
    out
}

/// The inverse shape, for a delete: every stored column moves from its value to
/// `null`, so the log holds the whole record as it last existed.
pub fn diff_removed(before: &Value) -> Map<String, Value> {
    let mut out = Map::new();
    if let Value::Object(map) = before {
        for (key, old) in map {
            out.insert(key.clone(), json!([old, Value::Null]));
        }
    }
    out
}

/// Loose scalar equality.
///
/// Everything posted through an HTML form arrives as a string, so a stored
/// integer `5` would compare unequal to a submitted `"5"` and strict JSON
/// equality would report every field as modified on every save. Compare scalars
/// by their text; anything structural falls back to exact equality.
fn same(a: &Value, b: &Value) -> bool {
    if a == b {
        return true;
    }
    match (scalar_text(a), scalar_text(b)) {
        (Some(x), Some(y)) => x == y,
        // Null vs a value, or two differing objects/arrays: a real change.
        _ => false,
    }
}

fn scalar_text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(v: Value) -> Map<String, Value> {
        match v {
            Value::Object(m) => m,
            _ => panic!("expected object"),
        }
    }

    #[test]
    fn create_diff_has_null_on_the_left() {
        let changes = diff(None, &map(json!({"title": "Hello"})));
        assert_eq!(changes["title"], json!([Value::Null, "Hello"]));
    }

    #[test]
    fn only_changed_columns_are_recorded() {
        let before = json!({"title": "Old", "body": "Same"});
        let changes = diff(Some(&before), &map(json!({"title": "New", "body": "Same"})));
        assert_eq!(changes.len(), 1);
        assert_eq!(changes["title"], json!(["Old", "New"]));
    }

    #[test]
    fn form_strings_do_not_read_as_changes_against_typed_columns() {
        // The exact false positive that would otherwise mark every row dirty.
        let before = json!({"views": 5, "published": true});
        let changes = diff(Some(&before), &map(json!({"views": "5", "published": "true"})));
        assert!(changes.is_empty(), "expected no changes, got {changes:?}");
    }

    #[test]
    fn a_real_numeric_change_is_still_caught() {
        let before = json!({"views": 5});
        let changes = diff(Some(&before), &map(json!({"views": "6"})));
        assert_eq!(changes["views"], json!([5, "6"]));
    }

    #[test]
    fn null_and_empty_string_are_distinct() {
        let before = json!({"nickname": Value::Null});
        let changes = diff(Some(&before), &map(json!({"nickname": ""})));
        assert_eq!(changes["nickname"], json!([Value::Null, ""]));
    }

    #[test]
    fn timestamps_are_not_recorded_as_changes() {
        let before = json!({"title": "A", "updated_at": "2026-01-01"});
        let changes = diff(
            Some(&before),
            &map(json!({"title": "A", "updated_at": "2026-07-22"})),
        );
        assert!(changes.is_empty());
    }

    #[test]
    fn delete_captures_the_whole_row() {
        let changes = diff_removed(&json!({"id": 1, "title": "Gone"}));
        assert_eq!(changes["title"], json!(["Gone", Value::Null]));
        assert_eq!(changes["id"], json!([1, Value::Null]));
    }
}
