# adminx-audit

[![crates.io](https://img.shields.io/crates/v/adminx-audit.svg)](https://crates.io/crates/adminx-audit)
[![docs.rs](https://img.shields.io/docsrs/adminx-audit)](https://docs.rs/adminx-audit)
[![license: MIT](https://img.shields.io/crates/l/adminx-audit.svg)](https://github.com/srotas-space/adminx/blob/main/LICENSE)

> **Who changed what, and what it used to be.** Optional audit logging for
> [adminx](https://crates.io/crates/adminx).

Register it and every create / update / delete that goes through adminx records
the acting admin and a per-column before/after diff. Leave it out and adminx
behaves exactly as it did, issuing **not one extra query**.

Works over **SeaORM** (PostgreSQL / MySQL / SQLite) or **MongoDB** — rows are
written through adminx's `Storage` trait, so this crate names neither.

---

## What a record looks like

```json
{"body": [null, "first body"], "title": [null, "Audit test"]}          // create
{"published": [false, true], "title": ["Audit test", "Audit test EDITED"]}  // update
{"id": [3, null], "title": ["Audit test EDITED", null]}                // delete
```

`{"column": [old, new]}` — PaperTrail's `object_changes` shape. Only columns
that **actually changed** are stored: resubmitting a field unchanged does not
record it, and a save that changes nothing records no entry at all.

---

## Install

```toml
[dependencies]
adminx = { version = "3", features = ["axum", "seaorm", "audit"] }
```

Or depend on the crate directly:

```toml
[dependencies]
adminx-audit = "3"
```

## Use

```rust,ignore
// 1. storage first
let store = adminx::seaorm::connect(&database_url).await?;

// 2. the audit table + its indexes (SQL backends only; Mongo auto-creates)
for stmt in adminx::audit::migrate_sql() {
    store.execute_sql(stmt).await?;
}
adminx::set_storage(Box::new(store));

// 3. register the auditor — before auth, so the first mutation is captured
adminx::audit::init(adminx::audit::AuditConfig::default());

// 4. the rest of your app
adminx::configure_auth(AuthConfig { /* ... */ });
adminx::register_resource(Box::new(PostResource));

// 5. optional: the in-panel log viewer
adminx::audit::register_resources();
```

That's it. Two views come with it:

- **Per record** — a **History** button on every detail page, at
  `/{resource}/history/{id}`, showing each change newest-first as a
  field / before / after table.
- **Everything** — `/adminx/adminx-audit-versions/list`, filterable by resource,
  record id, event, who, and date range.

The History route is always mounted. With no auditor registered it renders a
short "audit logging is not enabled" notice instead of 404ing, and the button is
hidden from detail pages.

---

## The log is append-only

The viewer refuses `create` / `update` / `delete` with **405** even for an admin,
and declares no writable columns. A log the panel can quietly rewrite is not
worth keeping, so rows are written only through the `Auditor` seam.

## Failure policy

By default an audit write that fails is logged and stepped over — a sick audit
table cannot take your panel down.

```rust,ignore
adminx::audit::init(AuditConfig::strict());  // fail the request instead
```

**What `strict` does not buy you.** The mutation has already committed by the
time the entry is written, and `Storage` exposes no transaction spanning both.
So a strict failure returns a 500 for a change that *did* land. It makes a hole
in the log loud rather than silent — it is not atomicity. Recording *before* the
mutation would be worse: it would log changes that never happened.

## What is and isn't captured

The hook lives on the `Resource` trait's default `create` / `update` / `delete`,
because only that layer holds the request context identifying the actor.

- ✅ Every resource using the default CRUD — API and HTML forms alike.
- ✅ A resource that *extends* the default CRUD, as long as it delegates to
  `adminx_core::crud::{create, update, delete}` rather than copying the body.
  `adminx-rbac`'s `PermissionResource` does exactly this, so permission edits
  are captured.
- ⚠️ A resource that replaces those methods with a wholly custom body records
  nothing unless it emits its own entry via `adminx_core::audit::emit`.
- ❌ Writes that bypass adminx entirely — a migration, `psql`, another service.
  If you need those too, the answer is database triggers, not an
  application-level log.

## Schema

`migrate_sql()` is **SQLite-flavoured**, matching the adminx demo. On PostgreSQL
use `SERIAL PRIMARY KEY` (and `JSONB` for `changes`), on MySQL
`INT AUTO_INCREMENT PRIMARY KEY` — supply your own migration, the same way you
own the `adminx_users` table.

| Column | Notes |
| --- | --- |
| `item_type` | the resource's `base_path()` |
| `item_id` | primary key of the affected record |
| `event` | `create` / `update` / `delete` |
| `whodunnit` | acting admin's id; `NULL` when auth is unconfigured |
| `whodunnit_email` | denormalized, so the log stays readable after the user is renamed or deleted |
| `changes` | the `{"column": [old, new]}` document, as JSON text |
| `created_at` | RFC 3339, stamped by the app so it doesn't depend on DB clock config |

Indexed on `(item_type, item_id)` and `created_at`. An audit table only ever
grows, so an unindexed scan degrades steadily in production.

## Cost

One extra `SELECT` per update and delete, to capture the before-state — issued
**only** when an auditor is registered.

## License

MIT
