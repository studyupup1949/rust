# adminx-search

[![crates.io](https://img.shields.io/crates/v/adminx-search.svg)](https://crates.io/crates/adminx-search)
[![docs.rs](https://img.shields.io/docsrs/adminx-search)](https://docs.rs/adminx-search)
[![license: MIT](https://img.shields.io/crates/l/adminx-search.svg)](https://github.com/srotas-space/adminx/blob/main/LICENSE)

> **Full-text search for [adminx](https://crates.io/crates/adminx)**, powered by
> the standalone [`searchez`](https://crates.io/crates/searchez) crate.

A resource declares its searchable fields; adminx then keeps the index in sync
automatically — create/update index the record, delete removes it — and the list
page grows a search box. Leave it out and adminx behaves exactly as before, with
no extra queries.

---

## How it fits together

```
adminx-core        the Indexer seam + the ?q= search box on the list page
adminx-search ←──  drives a searchez::Backend behind that seam (this crate)
searchez           the standalone search-model layer (in-memory / Meilisearch)
```

`searchez` is a general-purpose crate that knows nothing about adminx. This crate
is just the glue: it maps adminx's neutral `Indexer` seam onto a `searchez::Backend`.
Swap the backend without touching your resources.

## Install

```toml
[dependencies]
adminx = { version = "3", features = ["axum", "seaorm", "search"] }
```

For the Meilisearch backend, use `search-meilisearch` instead of `search`.

## Use

```rust,ignore
// 1. storage first
adminx::set_storage(Box::new(store));

// 2. register a search backend — in-memory (BM25), no external service
adminx::search::init_memory();

// 3. the rest of your app
adminx::configure_auth(AuthConfig { /* ... */ });
adminx::register_resource(Box::new(PostResource));
```

A resource opts in by listing its searchable columns:

```rust,ignore
impl Resource for Post {
    // ...
    fn search_fields(&self) -> Vec<&'static str> {
        vec!["title", "body"]
    }
}
```

That's it. The Posts list page now shows a search box; `?q=rust` returns matches
ranked by relevance, and every create/update/delete keeps the index current.

## Backfilling

The in-memory backend is non-persistent, so seed it from the database at startup
(and after any restart):

```rust,ignore
let page = adminx::core::storage::storage().list("posts", &opts).await?;
adminx::search::reindex(&PostResource, &page.rows).await?;
```

A persistent backend (Meilisearch) keeps its index across restarts, so this is
only needed once — or after switching backends.

## Backends

### In-memory (feature `search`)

`init_memory()` uses searchez's BM25 in-memory engine. No external service —
ideal for development and small single-process datasets. Non-persistent (see
[Backfilling](#backfilling)).

### Meilisearch (feature `search-meilisearch`)

A production backend backed by a Meilisearch server:

```rust,ignore
let backend = searchez::MeilisearchBackend::new("http://localhost:7700", Some("masterKey"))?;
adminx::search::init(backend);
```

Nothing about your resources changes. (searchez's Meilisearch backend is
compile-verified but not yet exercised against a live server in CI — see the
searchez README.)

## What is and isn't indexed

- ✅ Every resource using the default CRUD that declares `search_fields()` —
  API and HTML forms alike, because the hook lives on the shared `crud`
  functions (the same seam `adminx-audit` uses).
- ✅ Only the declared fields go into the search document; other columns are not
  indexed. The record is read back after each write, so a partial update still
  indexes the full field set.
- ⚠️ Indexing is **best-effort**: a search-backend failure is logged, never
  propagated — it can't fail the write the user asked for. The index catches up
  on the next write or a `reindex`.
- ❌ Writes that bypass adminx entirely (a migration, `psql`, another service)
  don't update the index; run `reindex` to reconcile.

## License

MIT
