// adminx-search/src/lib.rs
//
// Full-text search for adminx, powered by the standalone `searchez` crate.
// Register it and any resource that declares `search_fields()` is indexed
// automatically — create/update sync the index, delete removes the record — and
// its list page grows a search box. Leave it out and adminx behaves exactly as
// before, issuing no extra queries.
//
// The design keeps `searchez` independent: adminx-core exposes a neutral
// `Indexer` seam, and this crate is the thin adapter that drives a
// `searchez::Backend` behind it. searchez has no idea adminx exists; swap its
// backend (in-memory, Meilisearch, ...) without touching adminx.
//
// ## Startup order
//
// ```ignore
// adminx_seaorm::init(&db_url).await?;         // 1. storage
// adminx_search::init_memory();                // 2. register a backend (dev/small)
// configure_auth(AuthConfig { /* ... */ });    // 3. auth
// register_resource(Box::new(PostResource));   // declares search_fields()
// adminx_search::reindex(&PostResource, rows).await?;  // 4. optional backfill
// ```
//
// A resource opts in by returning its searchable columns:
//
// ```ignore
// fn search_fields(&self) -> Vec<&'static str> { vec!["title", "body"] }
// ```
//
// ## Backends
//
// [`init_memory`] uses searchez's BM25 in-memory engine — no external service,
// great for development and small datasets. For production, enable the
// `meilisearch` feature and register [`searchez::MeilisearchBackend`] via
// [`init`]. Nothing about your resources changes.

use adminx_core::search::Indexer;
use adminx_core::storage::StorageError;
use async_trait::async_trait;
use searchez::{Backend, Document, IndexDoc, Query};
use serde_json::{Map, Value};

/// Drives a [`searchez::Backend`] behind adminx-core's [`Indexer`] seam. The
/// whole integration is these three methods mapping one neutral vocabulary onto
/// the other.
pub struct SearchezIndexer {
    backend: Box<dyn Backend>,
}

impl SearchezIndexer {
    pub fn new(backend: impl Backend + 'static) -> Self {
        Self {
            backend: Box::new(backend),
        }
    }
}

#[async_trait]
impl Indexer for SearchezIndexer {
    async fn index(
        &self,
        index: &str,
        id: &str,
        document: Map<String, Value>,
    ) -> Result<(), StorageError> {
        self.backend
            .upsert(
                index,
                vec![IndexDoc {
                    id: id.to_string(),
                    document: Document::from(document),
                }],
            )
            .await
            .map_err(to_storage_err)
    }

    async fn remove(&self, index: &str, id: &str) -> Result<(), StorageError> {
        self.backend
            .delete(index, &[id.to_string()])
            .await
            .map_err(to_storage_err)
    }

    async fn search(
        &self,
        index: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<String>, StorageError> {
        let hits = self
            .backend
            .search(index, &Query::text(query).limit(limit))
            .await
            .map_err(to_storage_err)?;
        Ok(hits.into_iter().map(|h| h.id).collect())
    }
}

/// Register a search backend with adminx-core. Use [`init_memory`] for the
/// batteries-included path, or this to supply a configured backend (e.g.
/// `searchez::MeilisearchBackend`).
pub fn init(backend: impl Backend + 'static) {
    adminx_core::set_indexer(Box::new(SearchezIndexer::new(backend)));
    tracing::info!("adminx-search: full-text search enabled");
}

/// Register searchez's in-memory BM25 backend. No external service — ideal for
/// development and small single-process datasets. Non-persistent, so run
/// [`reindex`] at startup (or after a restart) to repopulate it from the
/// database.
pub fn init_memory() {
    init(searchez::MemoryBackend::new());
}

/// Backfill a resource's index from a set of already-loaded rows. Each row is
/// mapped through the resource's `search_fields()` and its primary key. Run this
/// to seed a new index, after switching backends, or on startup for the
/// non-persistent in-memory backend.
///
/// Returns the number of rows indexed.
pub async fn reindex<R: adminx_core::resource::Resource + ?Sized>(
    resource: &R,
    rows: &[Value],
) -> Result<usize, StorageError> {
    let Some(indexer) = adminx_core::search::indexer() else {
        return Ok(0);
    };
    let fields = resource.search_fields();
    let pk = resource.primary_key();
    let mut n = 0;
    for row in rows {
        let Some(id) = row.get(pk).map(value_to_id) else {
            continue;
        };
        let doc = adminx_core::search::document_for(row, &fields);
        indexer.index(resource.base_path(), &id, doc).await?;
        n += 1;
    }
    tracing::info!("adminx-search: reindexed {n} row(s) into `{}`", resource.base_path());
    Ok(n)
}

/// A primary-key value as a string, however the backend typed it.
fn value_to_id(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn to_storage_err(e: searchez::SearchError) -> StorageError {
    StorageError::Backend(format!("search: {e}"))
}
