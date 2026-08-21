// adminx-storage/src/schema.rs
//
// SQL bootstrap for the attachment-metadata table. As with the RBAC and audit
// tables, `Storage` has no portable "create table" primitive, so a SQL backend
// runs this at startup and Mongo needs nothing.
//
// Dialect note: SQLite-flavoured (the demo backend). On PostgreSQL use
// `SERIAL PRIMARY KEY`, on MySQL `INT AUTO_INCREMENT PRIMARY KEY` — supply your
// own migration.

/// `CREATE TABLE IF NOT EXISTS` + index for the attachment table. Run once on a
/// SQL backend via `adminx_core::seed(adminx_storage::migrate_sql())`.
///
/// One row per attached file. `(owner_type, owner_id, field)` is unique — a
/// field holds at most one file, so re-uploading replaces. `storage_key` is the
/// opaque key handed to the `BlobStore`.
pub const SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS adminx_attachments (\
        id INTEGER PRIMARY KEY, \
        owner_type TEXT NOT NULL, \
        owner_id TEXT NOT NULL, \
        field TEXT NOT NULL, \
        filename TEXT NOT NULL, \
        content_type TEXT NOT NULL, \
        byte_size INTEGER NOT NULL, \
        storage_key TEXT NOT NULL, \
        created_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS adminx_attachments_owner_idx \
        ON adminx_attachments (owner_type, owner_id)",
];
