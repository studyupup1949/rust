// adminx-storage/src/lib.rs
//
// File attachments for adminx. Register it and any resource can declare
// `file_fields()`; the panel then shows an upload widget on the detail page and
// the web adapters expose attach / serve / detach routes. Leave it out and
// adminx behaves exactly as before.
//
// Two layers, both swappable:
//   - `BlobStore` holds the bytes. The MVP ships `LocalFsStore` (local disk); an
//     S3 / object-store backend can slot in behind it later.
//   - the `adminx_attachments` table holds the metadata, written through
//     adminx-core's `Storage`, so it works over SeaORM (SQL) or MongoDB.
//
// ## Startup order
//
// ```ignore
// adminx_seaorm::init(&db_url).await?;                      // 1. storage
// adminx_core::seed(adminx_storage::migrate_sql()).await?;  // 2. SQL table (SQL backends only)
// adminx_storage::init_local("./adminx_uploads");           // 3. register a local-disk backend
// configure_auth(AuthConfig { /* ... */ });                 // 4. auth
// register_resource(Box::new(MyResource));                  // declares file_fields()
// ```
//
// A resource opts in by returning fields from `file_fields()`:
//
// ```ignore
// fn file_fields(&self) -> Vec<adminx_core::attach::FileField> {
//     vec![adminx_core::attach::FileField::new("avatar", "Avatar").images()]
// }
// ```
//
// ## What is stored where
//
// The bytes go to the `BlobStore` under an opaque key; the row in
// `adminx_attachments` records the original filename, content type, size and
// that key. Serving reads the row, then the bytes. Deleting a record purges its
// attachments (see `adminx_core::crud::delete`).

mod blobstore;
mod schema;
mod store;

pub use blobstore::{BlobStore, LocalFsStore};
pub use store::{AttachmentStore, TABLE};

/// Register a backend built from any [`BlobStore`]. Use this to supply a custom
/// store; most apps want [`init_local`] instead.
pub fn init(blobs: Box<dyn BlobStore>) {
    adminx_core::set_attachments(Box::new(AttachmentStore::new(blobs)));
    tracing::info!("adminx-storage: attachments enabled (metadata in `{TABLE}`)");
}

/// Register a local-filesystem backend rooted at `dir` (created on first write).
/// The simplest way to turn attachments on; swap for an object-store backend in
/// production.
pub fn init_local(dir: impl Into<std::path::PathBuf>) {
    let dir = dir.into();
    tracing::info!("adminx-storage: storing blobs under {}", dir.display());
    init(Box::new(LocalFsStore::new(dir)));
}

/// SQL `CREATE TABLE IF NOT EXISTS` + index for the attachment table. Run once on
/// a SQL backend via `adminx_core::seed(adminx_storage::migrate_sql())`. Mongo
/// needs nothing.
pub fn migrate_sql() -> &'static [&'static str] {
    schema::SQL
}
