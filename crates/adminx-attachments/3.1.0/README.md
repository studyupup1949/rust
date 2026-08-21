# adminx-attachments

[![crates.io](https://img.shields.io/crates/v/adminx-attachments.svg)](https://crates.io/crates/adminx-attachments)
[![docs.rs](https://img.shields.io/docsrs/adminx-attachments)](https://docs.rs/adminx-attachments)
[![license: MIT](https://img.shields.io/crates/l/adminx-attachments.svg)](https://github.com/srotas-space/adminx/blob/main/LICENSE)

> **Attach files to any resource.** Optional file uploads for
> [adminx](https://crates.io/crates/adminx) — an ActiveStorage-shaped layer for
> the Rust admin panel.

A resource declares which file fields it accepts; the panel then shows an upload
widget on each record's detail page, and the web adapter exposes attach / serve /
detach routes. Bytes live behind a pluggable backend (local disk today,
object-store/S3 to follow); metadata lives in your own database.

Leave it out and adminx behaves exactly as before — no widgets, no routes, no
new tables.

---

## Two swappable layers

| Layer | Responsibility | Backend |
| --- | --- | --- |
| `BlobStore` | holds the raw bytes | `LocalFsStore` (local disk), or **S3 / GCS / Azure** via [`adminx-cloud`](https://crates.io/crates/adminx-cloud) |
| `adminx_attachments` table | filename, type, size, storage key | via adminx's `Storage` — SeaORM **or** MongoDB |

The bytes go to the blob store under an opaque key; the metadata row records the
original filename, content type, size, and that key. Serving reads the row, then
the bytes.

## Install

```toml
[dependencies]
adminx = { version = "3", features = ["axum", "seaorm", "attachments"] }
```

## Use

```rust,ignore
// 1. storage first
let store = adminx::seaorm::connect(&database_url).await?;

// 2. the metadata table (SQL backends only; Mongo auto-creates)
for stmt in adminx::attachments::migrate_sql() {
    store.execute_sql(stmt).await?;
}
adminx::set_storage(Box::new(store));

// 3. register a backend — local disk here; swap for object-store in production
adminx::attachments::init_local("./adminx_uploads");
```

Then a resource opts in by declaring file fields:

```rust,ignore
impl Resource for Post {
    // ...
    fn file_fields(&self) -> Vec<adminx::attach::FileField> {
        vec![adminx::attach::FileField::new("cover", "Cover image").images()]
    }
}
```

An upload widget for **Cover image** now appears on every post's detail page,
pre-filled with the current file. Images preview inline.

## Routes

Each resource with file fields gains three routes under its base path:

| Method | Path | Gated on | Purpose |
| --- | --- | --- | --- |
| `POST` | `/{resource}/{id}/attach/{field}` | `Update` | upload (multipart), replacing any existing file |
| `GET` | `/{resource}/{id}/blob/{field}` | `Read` | stream the file back (inline) |
| `POST` | `/{resource}/{id}/detach/{field}` | `Update` | remove the file |

Uploads ride a **dedicated multipart endpoint**, not the create/edit form, so the
existing URL-encoded form pipeline is untouched: attach a file to a record that
already exists. All three are CSRF-protected and authorized like any other
mutating route.

## Lifecycle guarantees

- **One file per field.** Re-uploading replaces — the old bytes are deleted, no
  accumulation.
- **Purge on delete.** Deleting a record removes all its attachments (bytes and
  rows), so blobs never outlive their owner. Skipped on a soft delete, where the
  record still exists.
- **No orphans on partial failure.** If the metadata write fails after the bytes
  land, the bytes are cleaned up.

## Security notes

- **Filenames are sanitized** before they reach the `Content-Disposition` header
  — path separators and control characters are stripped, so a crafted name can't
  imply a path or inject a header.
- **Blob keys are validated** at the filesystem boundary: a key containing `..`
  or an absolute component is refused, so nothing can be written or read outside
  the store root.
- **Only declared fields are attachable.** The attach endpoint rejects any field
  the resource didn't list in `file_fields()`.
- Files are served **auth-gated** (the resource's `Read` check), not via public
  URLs. Signed public URLs are a possible future addition.

## Backend (`BlobStore`)

`init_local(dir)` is the batteries-included path. For cloud storage, add
[`adminx-cloud`](https://crates.io/crates/adminx-cloud) and swap one line:

```rust,ignore
adminx::attachments::init(adminx::cloud::s3_from_env("my-bucket")?);   // or gcs_/azure_
```

To store bytes anywhere else, implement `BlobStore` (three methods — `put` /
`get` / `delete`) and register it with `init(Box::new(your_store))`.

```rust,ignore
#[async_trait]
impl BlobStore for MyS3Store {
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StorageError> { /* ... */ }
    async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> { /* ... */ }
    async fn delete(&self, key: &str) -> Result<(), StorageError> { /* ... */ }
}
```

## Current limits

- **Whole file held in memory** on upload (fine for admin assets — logos,
  avatars, small docs). Streaming can come later.
- **No image variants / thumbnails**, no signed URLs, one file per field —
  files are served auth-gated at full size.

Shipped since the initial cut: **Actix and Axum** adapters both, and **S3 / GCS /
Azure** backends via [`adminx-cloud`](https://crates.io/crates/adminx-cloud).

## Schema

`migrate_sql()` is **SQLite-flavoured**. On PostgreSQL use `SERIAL PRIMARY KEY`,
on MySQL `INT AUTO_INCREMENT PRIMARY KEY` — supply your own migration, as with
the other adminx tables.

| Column | Notes |
| --- | --- |
| `owner_type` | the resource's `base_path()` |
| `owner_id` | primary key of the owning record |
| `field` | the file field name (e.g. `cover`) |
| `filename` / `content_type` / `byte_size` | original upload metadata |
| `storage_key` | opaque key handed to the `BlobStore` |
| `created_at` | RFC 3339 |

Indexed on `(owner_type, owner_id)`.

## License

MIT
