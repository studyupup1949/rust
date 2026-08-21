# adminx-rbac

**Pluggable, DB-backed RBAC** for the [adminx](https://crates.io/crates/adminx)
admin-panel framework: gate every resource route by **`(role, action, resource)`**
permissions read from your database and editable at runtime — in the spirit of
ActiveAdmin's authorization adapter.

Without this crate, adminx uses its built-in per-resource `allowed_roles()` list
(the same check for every operation). Add it for **per-action** control —
*"editors may update posts but not delete them"* — with no redeploy to change a
grant.

## You probably want `adminx`

Don't depend on this crate directly — depend on the single
**[`adminx`](https://crates.io/crates/adminx)** facade and enable the `rbac`
feature:

```toml
adminx = { version = "2", features = ["axum", "seaorm", "rbac"] }
```

It then appears as `adminx::rbac`.

## How it works

Grants live in an **`adminx_permissions`** table as `(role, resource, action)`
rows. Actions are `list` · `read` · `create` · `update` · `delete` · `export`
and each custom action **by name**; `*` is any resource and `manage` is any
action. Declare a starting policy in code — it **seeds the DB on first boot**,
then the database is authoritative and admins edit it in the panel.

```rust
use adminx::rbac::{self, Ability};

adminx::seaorm::init(&db_url).await?;         // 1. storage
adminx::seed(rbac::migrate_sql()).await?;     // 2. tables (SQL only; Mongo skips this)
rbac::init(vec![                              // 3. seed-if-empty, load cache, register
    Ability::role("admin").can_manage_all(),
    Ability::role("editor")
        .can("update", "posts")
        .can("publish", "posts"),             // a custom action, by name
    Ability::role("viewer").can_read_all(),
]).await?;
// configure_auth(...) then register_resource(...);
rbac::register_resources();                   // optional: role/permission editors in the panel
```

- **Additive & opt-in.** Implements an `Authorizer` trait in
  [`adminx-core`](https://crates.io/crates/adminx-core) and registers it. Leave
  the crate out and behaviour is exactly the built-in role check.
- **Storage-agnostic.** Grants flow through the same `Storage` trait as
  everything else, so it works over **SeaORM** (SQL) or **MongoDB**. The one
  asymmetry is table creation: SQL runs `rbac::migrate_sql()`
  (SQLite-flavoured — adapt the DDL for Postgres/MySQL), Mongo needs nothing.
- **Fast checks.** Grants load into an in-memory cache once; the per-request
  decision does no I/O. A permission edit in the panel reloads the cache
  (single-writer per process in this version).

Full documentation and usage: **[docs.rs/adminx](https://docs.rs/adminx)** ·
the [`adminx` README](https://crates.io/crates/adminx). MIT licensed.
