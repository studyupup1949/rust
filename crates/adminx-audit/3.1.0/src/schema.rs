// adminx-audit/src/schema.rs
//
// SQL bootstrap for the audit table. `Storage` has no portable "create table"
// primitive (Mongo makes collections on first write; SQL needs DDL), so — like
// the RBAC tables and the admin-users table — the SQL backend runs this at
// startup and Mongo needs nothing.
//
// Dialect note: auto-increment DDL is not portable across SQLite / PostgreSQL /
// MySQL, so this convenience targets **SQLite** (the demo backend), where
// `INTEGER PRIMARY KEY` aliases the auto-incrementing rowid. On PostgreSQL use
// `SERIAL PRIMARY KEY` (and `JSONB` for `changes`), on MySQL
// `INT AUTO_INCREMENT PRIMARY KEY` — provide your own migration for those, the
// same way you own the `adminx_users` table.

/// `CREATE TABLE IF NOT EXISTS` for the audit table (SQLite-flavoured; see the
/// dialect note above). Run once on a SQL backend via
/// `adminx_core::seed(adminx_audit::migrate_sql())` before [`init`](crate::init).
///
/// `changes` holds the `{"column": [old, new]}` JSON document as text, which is
/// the one shape every backend stores identically. On PostgreSQL you may prefer
/// `JSONB` so the column is queryable.
///
/// The index matters: the panel's per-record history view filters on
/// `(item_type, item_id)`, and an audit table is append-only — it only ever
/// grows, so an unindexed scan degrades steadily in production.
pub const SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS adminx_audit_versions (\
        id INTEGER PRIMARY KEY, \
        item_type TEXT NOT NULL, \
        item_id TEXT NOT NULL, \
        event TEXT NOT NULL, \
        whodunnit TEXT, \
        whodunnit_email TEXT, \
        changes TEXT NOT NULL, \
        created_at TEXT NOT NULL)",
    "CREATE INDEX IF NOT EXISTS adminx_audit_versions_item_idx \
        ON adminx_audit_versions (item_type, item_id)",
    "CREATE INDEX IF NOT EXISTS adminx_audit_versions_created_idx \
        ON adminx_audit_versions (created_at)",
];
