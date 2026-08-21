// adminx-rbac/src/schema.rs
//
// SQL bootstrap for the two RBAC tables. `Storage` has no portable "create
// table" primitive (Mongo makes collections on first write; SQL needs DDL), so
// — exactly like the admin-users table and the rest of adminx seeding — the SQL
// backend runs these statements at startup and Mongo needs nothing.
//
// Dialect note: auto-increment DDL is not portable across SQLite / PostgreSQL /
// MySQL, so this convenience targets **SQLite** (the demo backend), where
// `INTEGER PRIMARY KEY` aliases the auto-incrementing rowid. On PostgreSQL use
// `SERIAL PRIMARY KEY`, on MySQL `INT AUTO_INCREMENT PRIMARY KEY` — provide your
// own migration for those, the same way you own the `adminx_users` table.

/// `CREATE TABLE IF NOT EXISTS` for the RBAC tables (SQLite-flavoured; see the
/// dialect note above). Run once on a SQL backend via
/// `adminx_core::seed(adminx_rbac::migrate_sql())` before `rbac::init`.
pub const SQL: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS adminx_roles (\
        id INTEGER PRIMARY KEY, \
        name TEXT NOT NULL UNIQUE, \
        description TEXT)",
    "CREATE TABLE IF NOT EXISTS adminx_permissions (\
        id INTEGER PRIMARY KEY, \
        role TEXT NOT NULL, \
        resource TEXT NOT NULL, \
        action TEXT NOT NULL)",
];
