// adminx-core/src/search.rs
//
// The search seam. A resource can declare `search_fields()`; default CRUD then
// keeps a search index in sync — index on create/update, remove on delete —
// through a pluggable backend registered once. With no indexer registered
// nothing is indexed and no extra query is issued, so the cost is zero until a
// crate like `adminx-search` opts in.
//
// Core never names a search engine. `adminx-search` implements this trait over
// the standalone `searchez` crate (in-memory, Meilisearch, ...); nothing here
// depends on it.

use crate::request::ReqCtx;
use crate::storage::StorageError;
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use serde_json::{Map, Value};

/// A pluggable search index. Keyed by `index` (a resource's `base_path()`), so
/// one backend serves every searchable resource. Async — it does I/O, once per
/// mutation, not per request.
#[async_trait]
pub trait Indexer: Send + Sync {
    /// Insert or replace the document for one record.
    async fn index(
        &self,
        index: &str,
        id: &str,
        document: Map<String, Value>,
    ) -> Result<(), StorageError>;

    /// Remove a record from the index.
    async fn remove(&self, index: &str, id: &str) -> Result<(), StorageError>;

    /// Full-text search, returning matching record ids in rank order (best
    /// first). The caller hydrates the rows from storage.
    async fn search(
        &self,
        index: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<String>, StorageError>;
}

static INDEXER: OnceCell<Box<dyn Indexer>> = OnceCell::new();

/// Register the global search backend. Set-once, matching the other seams.
pub fn set_indexer(indexer: Box<dyn Indexer>) {
    if INDEXER.set(indexer).is_err() {
        tracing::warn!("adminx indexer already initialized; ignoring reset");
    }
}

/// The registered indexer, if any.
pub fn indexer() -> Option<&'static dyn Indexer> {
    INDEXER.get().map(|b| b.as_ref())
}

/// Whether search is on. Default CRUD checks this before spending a read to
/// build a document, so an unindexed app issues exactly the queries it did
/// before this module existed.
pub fn is_enabled() -> bool {
    INDEXER.get().is_some()
}

/// Project a stored row to the document to index: just the resource's declared
/// `search_fields` (plus nothing else — the id is carried separately). Missing
/// fields are simply absent.
pub fn document_for(row: &Value, fields: &[&str]) -> Map<String, Value> {
    let mut doc = Map::new();
    if let Value::Object(map) = row {
        for f in fields {
            if let Some(v) = map.get(*f) {
                doc.insert((*f).to_string(), v.clone());
            }
        }
    }
    doc
}

/// Index (or re-index) a record after a write. Best-effort: an indexing failure
/// is logged, never propagated — a search backend hiccup must not fail the write
/// the user asked for. The index catches up on the next write or a reindex.
pub async fn index_record(index: &str, id: &str, document: Map<String, Value>) {
    if let Some(indexer) = indexer() {
        if let Err(e) = indexer.index(index, id, document).await {
            tracing::error!("adminx: failed to index {index}/{id}: {e}");
        }
    }
}

/// Remove a record from the index after a delete. Best-effort, same rationale as
/// [`index_record`].
pub async fn remove_record(index: &str, id: &str) {
    if let Some(indexer) = indexer() {
        if let Err(e) = indexer.remove(index, id).await {
            tracing::error!("adminx: failed to de-index {index}/{id}: {e}");
        }
    }
}

/// Search an index, returning matching ids in rank order. Returns empty (never
/// errors to the caller) so a list page renders even when the backend is down.
pub async fn search_ids(index: &str, query: &str, limit: usize) -> Vec<String> {
    let Some(indexer) = indexer() else {
        return Vec::new();
    };
    match indexer.search(index, query, limit).await {
        Ok(ids) => ids,
        Err(e) => {
            tracing::error!("adminx: search on {index} failed: {e}");
            Vec::new()
        }
    }
}

/// The `q` full-text term from a request query string, trimmed; `None` when
/// absent or blank. The list page uses this to decide between a search and the
/// normal paginated listing.
pub fn query_term(ctx: &ReqCtx) -> Option<String> {
    let params: std::collections::HashMap<String, String> =
        serde_urlencoded::from_str(&ctx.query).unwrap_or_default();
    params
        .get("q")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
