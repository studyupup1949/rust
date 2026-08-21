// adminx-core/src/crud.rs
//
// The default create / update / delete, as free functions.
//
// Rust won't let an overriding impl call the trait's default method, so a
// resource that needs to *extend* the default behaviour (reload a cache, send a
// notification) historically had to copy the body — and any invariant later
// added to the default, like audit recording, silently skipped those copies.
//
// Keeping the implementation here instead means `Resource`'s defaults and any
// override delegate to the same code, so the invariants hold in both:
//
// ```ignore
// async fn create(&self, ctx: &ReqCtx, body: Value) -> ApiResponse {
//     let resp = adminx_core::crud::create(self, ctx, body).await;
//     if resp.status < 300 {
//         self.reload_cache().await;
//     }
//     resp
// }
// ```

use crate::audit;
use crate::authz::Action;
use crate::error::CoreError;
use crate::request::ReqCtx;
use crate::resource::Resource;
use crate::response::ApiResponse;
use crate::storage::{storage, CreateOutcome};
use serde_json::{json, Value};

/// Authorize, filter to writable columns, insert, and record the create.
pub async fn create<R: Resource + ?Sized>(res: &R, ctx: &ReqCtx, body: Value) -> ApiResponse {
    if !res.authorize(ctx, Action::Create) {
        return CoreError::Unauthorized.into();
    }
    let data = match res.filter_writable(body) {
        Ok(d) => d,
        Err(resp) => return resp,
    };

    // Only pay for the copy when something is listening.
    let snapshot = audit::is_enabled().then(|| data.clone());
    match storage().create(res.table_name(), data).await {
        Ok(CreateOutcome { last_insert_id }) => {
            if let Some(written) = snapshot {
                let entry = audit::AuditEntry::new(
                    ctx,
                    res.base_path(),
                    last_insert_id.clone().unwrap_or_default(),
                    audit::Event::Create,
                    audit::diff(None, &written),
                );
                if let Some(reject) = audit::emit(entry).await {
                    return reject;
                }
            }
            // Keep the search index in sync. Best-effort and gated on both a
            // registered backend and the resource opting in, so an app without
            // search pays nothing.
            reindex_after_write(res, last_insert_id.as_deref()).await;

            ApiResponse::created(json!({
                "success": true,
                "message": format!("{} created successfully", res.resource_name()),
                "last_insert_id": last_insert_id,
            }))
        }
        Err(e) => CoreError::from(e).into(),
    }
}

/// After a create or update, index the record's current searchable document.
/// Reads the row back so a partial update still indexes the full field set;
/// issued only when search is on and the resource declares `search_fields`.
async fn reindex_after_write<R: Resource + ?Sized>(res: &R, id: Option<&str>) {
    let fields = res.search_fields();
    if fields.is_empty() || !crate::search::is_enabled() {
        return;
    }
    let Some(id) = id else { return };
    if let Ok(Some(row)) = storage().get(res.table_name(), res.primary_key(), id).await {
        let doc = crate::search::document_for(&row, &fields);
        crate::search::index_record(res.base_path(), id, doc).await;
    }
}

/// Authorize, filter to writable columns, update, and record the diff against
/// the row it replaced.
pub async fn update<R: Resource + ?Sized>(
    res: &R,
    ctx: &ReqCtx,
    id: &str,
    body: Value,
) -> ApiResponse {
    if !res.authorize(ctx, Action::Update) {
        return CoreError::Unauthorized.into();
    }
    let data = match res.filter_writable(body) {
        Ok(d) => d,
        Err(resp) => return resp,
    };

    // The row as it stands, read only when auditing is on — this is the extra
    // query, and an unaudited app never issues it. A read failure degrades the
    // diff to "everything is new" rather than blocking the write.
    let (before, snapshot) = if audit::is_enabled() {
        let before = storage()
            .get(res.table_name(), res.primary_key(), id)
            .await
            .unwrap_or(None);
        (before, Some(data.clone()))
    } else {
        (None, None)
    };

    match storage()
        .update(res.table_name(), res.primary_key(), id, data)
        .await
    {
        Ok(n) if n > 0 => {
            if let Some(written) = snapshot {
                let changes = audit::diff(before.as_ref(), &written);
                // A save that changed nothing is noise, not history.
                if !changes.is_empty() {
                    let entry = audit::AuditEntry::new(
                        ctx,
                        res.base_path(),
                        id,
                        audit::Event::Update,
                        changes,
                    );
                    if let Some(reject) = audit::emit(entry).await {
                        return reject;
                    }
                }
            }
            // Re-index the updated record (reads it back, so a partial update
            // still indexes the full search document).
            reindex_after_write(res, Some(id)).await;
            ApiResponse::ok(json!({
                "success": true,
                "message": format!("{} updated successfully", res.resource_name()),
                "modified_count": n,
            }))
        }
        Ok(_) => CoreError::NotFound.into(),
        Err(e) => CoreError::from(e).into(),
    }
}

/// Authorize, delete (soft or hard per the resource), and record the record as
/// it last existed.
pub async fn delete<R: Resource + ?Sized>(res: &R, ctx: &ReqCtx, id: &str) -> ApiResponse {
    if !res.authorize(ctx, Action::Delete) {
        return CoreError::Unauthorized.into();
    }
    let soft = res.soft_delete();

    // Capture the record before it goes: for a hard delete this is the only
    // surviving copy, which is the whole point of auditing a destroy.
    let before = if audit::is_enabled() {
        storage()
            .get(res.table_name(), res.primary_key(), id)
            .await
            .unwrap_or(None)
    } else {
        None
    };

    match storage()
        .delete(res.table_name(), res.primary_key(), id, soft)
        .await
    {
        Ok(n) if n > 0 => {
            if audit::is_enabled() {
                let changes = before.as_ref().map(audit::diff_removed).unwrap_or_default();
                let entry =
                    audit::AuditEntry::new(ctx, res.base_path(), id, audit::Event::Delete, changes);
                if let Some(reject) = audit::emit(entry).await {
                    return reject;
                }
            }
            // Don't let blobs outlive the record they belonged to. Best-effort:
            // a purge failure is logged, not surfaced — the delete already
            // happened and blocking on cleanup would be worse than an orphan.
            // Skipped on a soft delete, where the record still exists.
            if !soft && crate::attach::is_enabled() {
                crate::attach::purge(res.base_path(), id).await;
            }
            // Drop it from the search index too. A soft-deleted record still
            // exists, but should no longer surface in search, so remove either way.
            if crate::search::is_enabled() && !res.search_fields().is_empty() {
                crate::search::remove_record(res.base_path(), id).await;
            }
            ApiResponse::ok(json!({
                "success": true,
                "message": format!("{} deleted successfully", res.resource_name()),
                "soft_delete": soft,
                "affected": n,
            }))
        }
        Ok(_) => CoreError::NotFound.into(),
        Err(e) => CoreError::from(e).into(),
    }
}
