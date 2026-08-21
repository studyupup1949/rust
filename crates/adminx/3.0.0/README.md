# adminx

[![crates.io](https://img.shields.io/crates/v/adminx.svg)](https://crates.io/crates/adminx)
[![docs.rs](https://img.shields.io/docsrs/adminx)](https://docs.rs/adminx)
[![license: MIT](https://img.shields.io/crates/l/adminx.svg)](./LICENSE)
![Adminx](https://srotasspace.s3.ap-south-1.amazonaws.com/srotas.svg)



**One `Resource` definition → a complete admin panel.** Any web framework, any database.

A **framework-neutral admin-panel framework** for Rust. Write a resource once and
serve it over **Actix Web** or **Axum**, backed by **PostgreSQL · MySQL · SQLite**
(SeaORM) or **MongoDB**. The logic lives in a neutral core; the frameworks and
databases are thin, swappable adapters — switching either is a one-line change and
your resource code never moves.

**Per resource, with almost no boilerplate, you get:**

- 🧩 &nbsp;Auto **CRUD** — a REST/JSON API *and* a rendered HTML admin UI
- 🔍 &nbsp;**List filters** — text, select, boolean, and date-range (collapsible sidebar)
- 🔐 &nbsp;**Auth + RBAC** — JWT-in-cookie login and role-gated routes, plus **opt-in per-action RBAC** (`adminx-rbac`): DB-backed permissions, editable right in the panel
- 🛡️ &nbsp;**Hardened by default** — CSRF tokens on every form, per-account rate limiting on login & MFA
- 🔒 &nbsp;**MFA** — TOTP (authenticator apps) with one-time backup codes
- ⚡ &nbsp;**Custom actions**, **CSV/JSON export**, pagination, sorting, soft-delete
- 🌱 &nbsp;**Seeding** and admin-user creation from a **CLI** or from code
- 🔄 &nbsp;The *same* resource code on **Actix or Axum**, over **SQL or Mongo**

---

## Contents

- [Install](#install)
- [How to use](#how-to-use)
- [Examples](#examples)
- [How to seed](#how-to-seed)
- [Implement on every stack](#implement-on-every-stack)
  - [Axum + SeaORM](#axum--seaorm)
  - [Actix + SeaORM](#actix--seaorm)
  - [Axum + MongoDB](#axum--mongodb)
  - [Actix + MongoDB](#actix--mongodb)
- [Usage examples](#usage-examples)
- [Full example (single file)](#full-example-single-file)
- [Reference](#reference)
  - [The `Resource` trait](#the-resource-trait)
  - [Customizing the form](#customizing-the-form)
  - [Filters](#filters)
  - [Custom actions](#custom-actions)
  - [Authentication & RBAC](#authentication--rbac)
  - [CSRF protection](#csrf-protection)
  - [Rate limiting](#rate-limiting)
  - [Fine-grained RBAC (`adminx-rbac`)](#fine-grained-rbac-adminx-rbac)
  - [Protecting the panel with HTTP Basic Auth](#protecting-the-panel-with-http-basic-auth)
  - [Multi-factor auth (MFA)](#multi-factor-auth-mfa)
  - [The `adminx` CLI](#the-adminx-cli)
  - [Admin-users table](#admin-users-table)
  - [Managing admin users](#managing-admin-users)
  - [CSV / JSON export](#csv--json-export)
  - [REST API surface](#rest-api-surface)
  - [HTML admin UI](#html-admin-ui)
  - [Environment variables](#environment-variables)
  - [Deployment](#deployment)
  - [Troubleshooting](#troubleshooting)
  - [Status](#status)

---

## Install

Depend on the single **`adminx`** facade and pick **one framework** (`actix` or
`axum`) and **one storage** (`seaorm` or `mongo`) via features. Crates you don't
select are never compiled.

```toml
[dependencies]
adminx      = { version = "3", features = ["axum", "seaorm"] }
tokio       = { version = "1", features = ["full"] }
axum        = "0.8"          # the framework you chose
serde_json  = "1"
async-trait = "0.1"
```

| You want | `features = [...]` | web dep |
|---|---|---|
| Axum + Postgres/MySQL/SQLite | `["axum", "seaorm"]` | `axum = "0.8"` |
| Actix + Postgres/MySQL/SQLite | `["actix", "seaorm"]` | `actix-web = "4"` |
| Axum + MongoDB | `["axum", "mongo"]` | `axum = "0.8"` |
| Actix + MongoDB | `["actix", "mongo"]` | `actix-web = "4"` |

> You do **not** add `sea-orm` or `mongodb` yourself — the storage adapter wraps
> the driver. During local development use a path dep:
> `adminx = { path = "../crates/adminx-suite/adminx", features = [...] }`.

---

## How to use

Three steps: **define a resource**, **wire a `main`**, **open the panel**.

### 1. Define a resource

A resource maps a table/collection to an admin screen. Four methods are required;
everything else has a default.

```rust
use adminx::prelude::*;
use async_trait::async_trait;

#[derive(Clone)]
pub struct PostResource;

#[async_trait]
impl Resource for PostResource {
    fn resource_name(&self) -> &'static str { "Posts" }     // display name
    fn base_path(&self)     -> &'static str { "posts" }     // URL segment
    fn table_name(&self)    -> &'static str { "posts" }     // SQL table / Mongo collection
    fn clone_box(&self) -> Box<dyn Resource> { Box::new(self.clone()) }

    // Columns editable on create/edit forms:
    fn permit_keys(&self) -> Vec<&'static str> { vec!["title", "body", "published"] }
}
```

That already gives you a list view, detail page, create/edit forms, a JSON API,
CSV/JSON export, and role-gated auth — all generated.

### 2. Wire `main` (Axum + SQLite here)

Put this **in the same `src/main.rs`** as the `PostResource` above. Cargo deps:
`adminx = { version = "3", features = ["axum", "seaorm"] }`, plus `tokio`
(`features = ["full"]`), `axum = "0.8"`, `async-trait`, `serde_json`. SQLite needs
no database server, so this runs as-is with `cargo run`.

```rust
use adminx::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Storage: connect + create tables (SQLite needs no server).
    let store = adminx::seaorm::connect("sqlite://admin.db?mode=rwc").await?;
    store.execute_sql("CREATE TABLE IF NOT EXISTS posts (\
        id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, body TEXT, \
        published BOOLEAN NOT NULL DEFAULT 0)").await?;
    store.execute_sql("CREATE TABLE IF NOT EXISTS adminx_users (\
        id INTEGER PRIMARY KEY AUTOINCREMENT, email TEXT NOT NULL UNIQUE, \
        encrypted_password TEXT NOT NULL, role TEXT NOT NULL DEFAULT 'admin', \
        mfa_enabled BOOLEAN NOT NULL DEFAULT 0, mfa_secret TEXT, mfa_backup_codes TEXT)").await?;
    set_storage(Box::new(store));

    // Auth: sign cookies, seed an admin.
    configure_auth(AuthConfig {
        jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".into()),
        token_ttl_secs: 86_400,
        admin_table: "adminx_users".into(),
        secure_cookie: false,          // true behind HTTPS
    });
    let _ = create_admin("admin@example.com", "changeme", "admin").await;

    // Register resources, mount the panel at /adminx.
    register_resource(Box::new(PostResource));

    let app = axum::Router::new().nest("/adminx", adminx::axum::router());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

### 3. Open the panel

Visit **http://localhost:8080/adminx** and sign in with `admin@example.com` /
`changeme`. First login prompts the (skippable) [MFA setup](#multi-factor-auth-mfa).

> Auth is optional: until `configure_auth(..)` is called, every page is public —
> handy for a quick look.

---

## Examples

The same `Resource` trait scales from one line to a fully customized screen.

**Minimal** — one text input per permitted column:

```rust
#[async_trait]
impl Resource for Categories {
    fn resource_name(&self) -> &'static str { "Categories" }
    fn base_path(&self)     -> &'static str { "categories" }
    fn table_name(&self)    -> &'static str { "categories" }
    fn clone_box(&self) -> Box<dyn Resource> { Box::new(self.clone()) }
    fn permit_keys(&self) -> Vec<&'static str> { vec!["name", "slug"] }
}
```

**Custom form + sidebar grouping**:

```rust
#[async_trait]
impl Resource for Products {
    fn resource_name(&self) -> &'static str { "Products" }
    fn base_path(&self)     -> &'static str { "products" }
    fn table_name(&self)    -> &'static str { "products" }
    fn clone_box(&self) -> Box<dyn Resource> { Box::new(self.clone()) }
    fn menu_group(&self)    -> Option<&'static str> { Some("Shop") }   // sidebar section
    fn permit_keys(&self) -> Vec<&'static str> { vec!["name","sku","price_cents","active"] }

    fn form_structure(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({ "groups": [{ "title": "Product", "fields": [
            { "name": "name",        "field_type": "text",     "label": "Name" },
            { "name": "sku",         "field_type": "text",     "label": "SKU" },
            { "name": "price_cents", "field_type": "number",   "label": "Price (cents)" },
            { "name": "active",      "field_type": "checkbox", "label": "Active" }
        ]}]}))
    }
}
```

**With list filters** (adds the collapsible filter sidebar — see [Filters](#filters)):

```rust
fn filterable_fields(&self) -> Vec<FilterField> {
    vec![
        FilterField::text("name", "Name"),          // case-insensitive contains
        FilterField::boolean("active", "Active"),    // Yes / No
        FilterField::date_range("created_at", "Created"),
    ]
}
```

**Role-gated** — only these roles can reach it:

```rust
fn allowed_roles(&self) -> Vec<String> { vec!["admin".into(), "editor".into()] }
```

**Mongo** — collections are schemaless, and the key is `_id`:

```rust
fn table_name(&self)  -> &'static str { "posts" }   // collection
fn primary_key(&self) -> &'static str { "_id" }     // <-- Mongo only
```

---

## How to seed

Populate tables/collections with starter data. Three ways, all idempotent when
you write your statements that way (`ON CONFLICT DO NOTHING`, etc.).

### From the CLI (recommended)

Install the CLI once, then seed by pointing `DATABASE_URL` (SQL) or `MONGO_URL`
(Mongo) at your database. One statement per line; blank lines and `--` / `#`
comments are ignored.

```sh
cargo install adminx --features cli
```

**SQL** (`seeds.sql`):

```sql
INSERT INTO categories (name, slug) VALUES ('Books','books') ON CONFLICT (slug) DO NOTHING
INSERT INTO products (name, sku, price_cents, active) VALUES ('Rust Book','SKU-RB',3999,true) ON CONFLICT (sku) DO NOTHING
```

```sh
DATABASE_URL=postgres://user:pass@127.0.0.1:5432/mydb adminx seed --file seeds.sql
```

**Mongo** — each line is a JSON command document (`seeds.json`):

```json
{"insert":"categories","documents":[{"name":"Books","slug":"books"}]}
{"insert":"products","documents":[{"name":"Rust Book","sku":"SKU-RB","active":true}]}
```

```sh
MONGO_URL=mongodb://127.0.0.1:27017 MONGO_DB=mydb adminx seed --file seeds.json
# or pipe it
echo '{"insert":"products","documents":[{"name":"Widget"}]}' | MONGO_URL=... adminx seed
```

### From code, per backend

```rust
// SeaORM — SQL statements. Connects and runs them; returns rows affected.
adminx::seaorm::seed("postgres://user:pass@localhost/mydb", &[
    "INSERT INTO categories (name, slug) VALUES ('Books','books') ON CONFLICT DO NOTHING",
]).await?;

// Mongo — JSON command documents.
adminx::mongo::seed("mongodb://localhost:27017", "mydb", &[
    r#"{"insert":"categories","documents":[{"name":"Books","slug":"books"}]}"#,
]).await?;
```

### From code, after `set_storage` (backend-neutral)

Once a backend is registered, `adminx::seed` runs against whichever one is active
— SQL strings on SeaORM, JSON command docs on Mongo:

```rust
adminx::seed(&[
    "INSERT INTO categories (name, slug) VALUES ('Books','books') ON CONFLICT DO NOTHING",
]).await?;
```

> `adminx create-admin` (below) seeds the **admin user** the same way — from the
> CLI or `create_admin(email, password, role)` in code.

---

## Implement on every stack

The `Resource` is **identical** across all four combos — only `main` changes (and,
for Mongo, `primary_key() -> "_id"`). Select the stack with Cargo features from
[Install](#install).

### Axum + SeaORM

`features = ["axum", "seaorm"]`, dep `axum = "0.8"`.

```rust
use adminx::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = adminx::seaorm::connect("postgres://user:pass@localhost/mydb").await?;
    // ... execute_sql(CREATE TABLE ...) as needed, or skip if tables exist ...
    set_storage(Box::new(store));

    configure_auth(AuthConfig {
        jwt_secret: std::env::var("JWT_SECRET").expect("set JWT_SECRET"),
        token_ttl_secs: 86_400,
        admin_table: "adminx_users".into(),
        secure_cookie: true,
    });
    let _ = create_admin("admin@example.com", "changeme", "admin").await;
    register_resource(Box::new(PostResource));

    let app = axum::Router::new().nest("/adminx", adminx::axum::router());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

Swap the URL for another SQL dialect — `postgres://…`, `mysql://…`, or
`sqlite://file.db?mode=rwc`. If your tables already exist, skip `execute_sql` and
use `adminx::seaorm::init(url).await?` (connect + register in one call).

### Actix + SeaORM

`features = ["actix", "seaorm"]`, dep `actix-web = "4"`.

```rust
use adminx::prelude::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    adminx::seaorm::init("postgres://user:pass@localhost/mydb").await.unwrap();

    configure_auth(AuthConfig {
        jwt_secret: std::env::var("JWT_SECRET").expect("set JWT_SECRET"),
        token_ttl_secs: 86_400,
        admin_table: "adminx_users".into(),
        secure_cookie: true,
    });
    let _ = create_admin("admin@example.com", "changeme", "admin").await;
    register_resource(Box::new(PostResource));

    actix_web::HttpServer::new(|| {
        actix_web::App::new().service(adminx::actix::scope())   // mounts at /adminx
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
```

### Axum + MongoDB

`features = ["axum", "mongo"]`. Mongo is schemaless (no DDL), key is `_id`:

```rust
use adminx::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    adminx::mongo::init("mongodb://localhost:27017", "mydb").await?;   // connect + register

    configure_auth(AuthConfig {
        jwt_secret: std::env::var("JWT_SECRET").expect("set JWT_SECRET"),
        token_ttl_secs: 86_400,
        admin_table: "adminx_users".into(),   // a Mongo collection
        secure_cookie: true,
    });
    let _ = create_admin("admin@example.com", "changeme", "admin").await;
    register_resource(Box::new(PostResource));   // its primary_key() returns "_id"

    let app = axum::Router::new().nest("/adminx", adminx::axum::router());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

### Actix + MongoDB

`features = ["actix", "mongo"]`. Combine the Actix `main` (above) with Mongo setup:

```rust
use adminx::prelude::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    adminx::mongo::init("mongodb://localhost:27017", "mydb").await.unwrap();

    configure_auth(AuthConfig {
        jwt_secret: std::env::var("JWT_SECRET").expect("set JWT_SECRET"),
        token_ttl_secs: 86_400,
        admin_table: "adminx_users".into(),
        secure_cookie: true,
    });
    let _ = create_admin("admin@example.com", "changeme", "admin").await;
    register_resource(Box::new(PostResource));   // primary_key() -> "_id"

    actix_web::HttpServer::new(|| {
        actix_web::App::new().service(adminx::actix::scope())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
```

> Runnable references live in `projects/demos/axumtestsql` and
> `projects/demos/actixtestsql` (SQL-only, self-contained), plus `adminx-demo`
> (Axum + SQLite).

---

## Usage examples

**Create an admin user** (CLI — backend chosen from the environment):

```sh
DATABASE_URL=postgres://user:pass@127.0.0.1:5432/mydb \
  EMAIL=admin@example.com PASSWORD=changeme adminx create-admin

MONGO_URL=mongodb://127.0.0.1:27017 MONGO_DB=mydb \
  EMAIL=admin@example.com PASSWORD=changeme adminx create-admin
```

**Filter the list** (query string mirrors the UI sidebar):

```
/adminx/products/list?name=rust                 # text contains
/adminx/products/list?active=false              # boolean
/adminx/products/list?created_at_from=2024-01-01&created_at_to=2024-12-31
```

**Export** the current (optionally filtered) list:

```
/adminx/products/list?download=csv
/adminx/products/list?download=json&active=true
```

**Use the JSON API** (relative to `/adminx`):

```sh
curl /adminx/products/api?page=1&per_page=25&sort=-created_at   # list
curl -X POST /adminx/products/api  -d '{"name":"Widget","sku":"W1"}'
curl -X PUT  /adminx/products/api/5 -d '{"active":false}'
curl -X DELETE /adminx/products/api/5
```

**Run a custom action** on a record:

```
POST /adminx/orders/{id}/action/refund
```

---

## Full example (single file)

A complete, copy-paste project — **Axum + SQLite**, with a filtered resource,
seeded rows, and auth. No database server needed: `cargo run`, then open
http://localhost:8080/adminx and sign in with `admin@example.com` / `changeme`.

`Cargo.toml`:

```toml
[package]
name = "adminx-quickstart"
version = "0.1.0"
edition = "2021"

[dependencies]
adminx      = { version = "3", features = ["axum", "seaorm"] }
tokio       = { version = "1", features = ["full"] }
axum        = "0.8"
async-trait = "0.1"
serde_json  = "1"
```

`src/main.rs`:

```rust
use adminx::prelude::*;
use async_trait::async_trait;

#[derive(Clone)]
struct Products;

#[async_trait]
impl Resource for Products {
    fn resource_name(&self) -> &'static str { "Products" }
    fn base_path(&self)     -> &'static str { "products" }
    fn table_name(&self)    -> &'static str { "products" }
    fn clone_box(&self) -> Box<dyn Resource> { Box::new(self.clone()) }
    fn menu_group(&self)    -> Option<&'static str> { Some("Shop") }
    fn permit_keys(&self) -> Vec<&'static str> { vec!["name", "sku", "active"] }

    fn filterable_fields(&self) -> Vec<FilterField> {
        vec![
            FilterField::text("name", "Name"),
            FilterField::boolean("active", "Active"),
            FilterField::date_range("created_at", "Created"),
        ]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Storage + tables (SQLite: no server needed).
    let store = adminx::seaorm::connect("sqlite://quickstart.db?mode=rwc").await?;
    store.execute_sql("CREATE TABLE IF NOT EXISTS products (\
        id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, sku TEXT UNIQUE, \
        active BOOLEAN NOT NULL DEFAULT 1, \
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP)").await?;
    store.execute_sql("CREATE TABLE IF NOT EXISTS adminx_users (\
        id INTEGER PRIMARY KEY AUTOINCREMENT, email TEXT NOT NULL UNIQUE, \
        encrypted_password TEXT NOT NULL, role TEXT NOT NULL DEFAULT 'admin', \
        mfa_enabled BOOLEAN NOT NULL DEFAULT 0, mfa_secret TEXT, mfa_backup_codes TEXT)").await?;
    set_storage(Box::new(store));

    // 2. Seed a couple of rows (idempotent via the UNIQUE sku).
    adminx::seed(&[
        "INSERT INTO products (name, sku) VALUES ('Rust Book','SKU-RB') ON CONFLICT DO NOTHING",
        "INSERT INTO products (name, sku) VALUES ('Desk Lamp','SKU-LAMP') ON CONFLICT DO NOTHING",
    ]).await?;

    // 3. Auth + seed an admin.
    configure_auth(AuthConfig {
        jwt_secret: "dev-secret-change-me".into(),
        token_ttl_secs: 86_400,
        admin_table: "adminx_users".into(),
        secure_cookie: false,           // local http
    });
    let _ = create_admin("admin@example.com", "changeme", "admin").await;

    // 4. Register resources + serve.
    register_resource(Box::new(Products));
    let app = axum::Router::new().nest("/adminx", adminx::axum::router());
    println!("adminx → http://localhost:8080/adminx  (admin@example.com / changeme)");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

Swap two lines to move to production: change the connect URL to
`postgres://…` (and the `CREATE TABLE` to Postgres DDL, e.g. `SERIAL PRIMARY KEY`),
or to Mongo with `adminx::mongo::init(uri, db)` and `primary_key() -> "_id"`.

---

## Reference

### The `Resource` trait

Four methods are required; the rest have defaults you override to customize.

| Method | Required | Default / purpose |
|---|---|---|
| `resource_name()` | ✅ | display name (e.g. `"Posts"`) |
| `base_path()` | ✅ | URL segment (e.g. `"posts"`) |
| `table_name()` | ✅ | SQL table / Mongo collection |
| `clone_box()` | ✅ | `Box::new(self.clone())` |
| `primary_key()` | | `"id"` (use `"_id"` for Mongo) |
| `permit_keys()` | | `[]` — columns settable on create/update |
| `readonly_keys()` | | `["id","created_at","updated_at"]` |
| `allowed_roles()` | | `["admin"]` — RBAC gate |
| `menu_group()` / `menu()` | | sidebar grouping / label |
| `form_structure()` | | custom form (else derived from `permit_keys`) |
| `filterable_fields()` | | `[]` — list filters (see below) |
| `custom_actions()` | | `[]` — id-scoped actions |
| `soft_delete()` | | `true` when `"deleted"` is permitted |
| `list/get/create/update/delete` | | full default CRUD via `Storage` |
| `list_page/new_page/edit_page/view_page` | | full default HTML pages |

Overrides return the neutral `ApiResponse`, so they keep working on both frameworks.

### Customizing the form

Without `form_structure()`, the create/edit form is one text input per
`permit_keys()`. Provide one to control labels and field types:

```rust
fn form_structure(&self) -> Option<serde_json::Value> {
    Some(serde_json::json!({ "groups": [{ "title": "Post", "fields": [
        { "name": "title",     "field_type": "text",     "label": "Title" },
        { "name": "body",      "field_type": "textarea", "label": "Body" },
        { "name": "published", "field_type": "checkbox", "label": "Published" }
    ]}]}))
}
```

Field types: `text`, `number`, `email`, `password`, `textarea`, `checkbox` (any
HTML input type works for the plain case).

### Filters

Declare `filterable_fields()` and adminx renders a **collapsible filter sidebar**
on the list page (hidden by default; a "Filters" button toggles it, and it opens
automatically when a filter is active). Filters apply on both storage backends and
also constrain CSV/JSON export.

```rust
fn filterable_fields(&self) -> Vec<FilterField> {
    vec![
        FilterField::text("name", "Name"),                 // case-insensitive contains
        FilterField::boolean("active", "Active"),           // Yes / No (exact)
        FilterField::select("status", "Status", vec![       // dropdown (exact)
            FilterOption::new("paid", "Paid"),
            FilterOption::new("pending", "Pending"),
        ]),
        FilterField::date_range("created_at", "Created"),   // from / to (>= and <=)
    ]
}
```

| Kind | Match | Query params |
|---|---|---|
| `text` | case-insensitive substring | `?field=value` |
| `select` / `boolean` | exact | `?field=value` |
| `date_range` | `>= from AND <= to` | `?field_from=YYYY-MM-DD&field_to=YYYY-MM-DD` |

A bare `to` date (`YYYY-MM-DD`) covers the **whole** day (extended to `23:59:59`).
`FilterField` / `FilterKind` / `FilterOption` come from the prelude. On Mongo,
`contains` becomes a case-insensitive `$regex` and a date range becomes
`{$gte,$lte}`.

### Custom actions

Id-scoped buttons on the detail page that POST to `/{base}/{id}/action/{name}`.
Declare them by returning `CustomAction`s from `custom_actions()`, each with its
own async handler (`CustomAction` / `ActionFuture` are in the prelude); see
`adminx-core/src/actions.rs`.

### Authentication & RBAC

adminx has built-in login: a **signed JWT (HS256) in an HttpOnly cookie** — no
server-side session, so it behaves identically on Actix and Axum. Set it up in
four steps.

**1. Create the admin table** (see [Admin-users table](#admin-users-table)), or
let `adminx create-admin` create it for you on SeaORM.

**2. Configure auth** once at startup:

```rust
configure_auth(AuthConfig {
    jwt_secret: std::env::var("JWT_SECRET").unwrap(),  // openssl rand -hex 32
    token_ttl_secs: 86_400,               // cookie/JWT lifetime (24h)
    admin_table: "adminx_users".into(),   // table (SQL) or collection (Mongo)
    secure_cookie: true,                  // HTTPS only — false for local http
});
```

| Field | Meaning |
|---|---|
| `jwt_secret` | HS256 signing key — keep it secret; rotating it invalidates all sessions |
| `token_ttl_secs` | how long a login stays valid, in seconds |
| `admin_table` | where admin users live |
| `secure_cookie` | `true` in production (HTTPS); `false` for local `http://` |

**3. Seed an admin** (once) — in code or via the [CLI](#the-adminx-cli):

```rust
create_admin("admin@example.com", "changeme", "admin").await?;   // bcrypt-hashed
```

**4. Log in.** adminx adds `GET/POST /adminx/login` and `GET /adminx/logout`, and
enforces access automatically:

- Unauthenticated **UI** page → `303` redirect to `/adminx/login`.
- Unauthenticated **API** request → `401`.
- A resource is reachable only by principals holding one of its `allowed_roles()`.

> **Sessions & revocation.** The session is a **stateless** signed JWT — there's no
> server-side session store, so `/adminx/logout` clears the *browser's* cookie but
> can't invalidate a token that was already copied elsewhere; that token stays
> valid until it expires. Two levers control the blast radius:
> - **`token_ttl_secs`** bounds how long any leaked token lives — set it short
>   (e.g. `3_600`) if that matters more than staying logged in.
> - **Rotating `jwt_secret`** invalidates **every** session immediately — the
>   built-in "revoke all" / "log everyone out" switch. (Per-user revocation would
>   need a stateful session store; not built in.)

> **Don't leave auth off in production.** Until `configure_auth(..)` is called
> every route is public *and RBAC is bypassed* — adminx logs a loud one-time
> warning the first time a request hits an access check with auth unconfigured.

**Roles (RBAC).** Each resource declares who may open it; the role comes from the
admin user's `role` column and travels in the JWT:

```rust
fn allowed_roles(&self) -> Vec<String> { vec!["admin".into(), "editor".into()] }
```

> **Auth is opt-in.** Until `configure_auth(..)` is called, every page is public —
> handy while prototyping. Call it (and seed an admin) to lock things down.

### CSRF protection

Every built-in form post is protected by a double-submit cookie: a
random token is set in an `adminx_csrf` cookie and mirrored in a hidden `_csrf`
field, and the two must match or the post is rejected with a `403`. This covers
the auth forms (`/login`, `/mfa/enable`, `/mfa/verify`) **and** the resource
forms (create, update, delete, and custom-action buttons). It's automatic — the
built-in templates already include the field.

Resource-form CSRF follows the same opt-in rule as the rest of auth: it's
enforced once `configure_auth(..)` is called, and skipped while auth is
unconfigured (the prototype mode where every page is public anyway). The auth
forms are always checked.

The auth cookie is already `SameSite=Strict`, which by itself blocks forged posts
to *authenticated* endpoints. The token matters most for `POST /login`, which
carries no prior cookie and so gets no SameSite protection: without it an
attacker can force a victim's browser to log into the *attacker's* account. On
the already-authenticated resource forms the token is defence in depth — legacy
browsers, and same-site-but-not-same-origin attackers (a hostile subdomain is
"same-site").

If you render your **own** form against any of these routes, add the token:

```html
<input type="hidden" name="_csrf" value="{{ csrf_token }}">
```

> **Custom-action bodies (behaviour change).** The `/{id}/action/{name}` route now
> reads a url-encoded **form** body, matching what the action button always sent
> (previously the button's body was parsed as JSON, so it silently arrived empty).
> A `custom_actions()` handler now receives its submitted fields as the JSON
> `body` — an added `<input name="reason">` arrives as `{"reason": "..."}`. If you
> were POSTing to this endpoint directly with a JSON body, switch to form
> encoding.

### Rate limiting

Failed password and second-factor attempts are throttled per account, **on by
default** — a 6-digit TOTP is only a million combinations, so the throttle is
what makes the second factor worth having. Exceeding a limit returns `429` until
the window lapses; a successful login clears the count.

| | Default | Meaning |
|---|---|---|
| `login` | 10 per 15 min | failed password attempts per account |
| `mfa` | 5 per 15 min | failed TOTP/backup-code attempts per account |

Tune it (before serving — the first attempt locks the defaults in):

```rust
use adminx::ratelimit::{self, Limit, RateLimitConfig};

ratelimit::configure(RateLimitConfig {
    login: Some(Limit::new(5, 600)),   // 5 attempts per 10 minutes
    mfa: Some(Limit::new(3, 600)),
    // `None` disables a throttle entirely — only sensible in tests.
});
```

Two things to know about the design:

- **Counters are per-process**, held in memory. N replicas behind a load balancer
  therefore allow N× the attempts. That still bounds the attack, but a hard
  global limit needs a shared store (see [ROADMAP](ROADMAP.md)).
- **Keyed by account, not by client address.** This stops a targeted brute force
  against one admin, but not credential stuffing spread across many accounts. The
  flip side is that an attacker can deliberately burn an admin's attempts to keep
  them out — which is why this is a short self-clearing window rather than a
  lockout needing manual intervention.

### Fine-grained RBAC (`adminx-rbac`)

The built-in check is coarse: a resource lists `allowed_roles()` and every
operation on it shares that list. For **per-action** control — "editors may update
posts but not delete them" — add the optional `adminx-rbac` crate. It's DB-backed
and editable at runtime, in the spirit of ActiveAdmin's authorization adapter.

```toml
adminx = { version = "3", features = ["axum", "seaorm", "rbac"] }
```

Grants are `(role, action, resource)` rows in an `adminx_permissions` table.
Actions are `list` / `read` / `create` / `update` / `delete` / `export` and each
custom action by name; `*` is any resource and `manage` is any action. Declare a
starting policy in code — it **seeds the DB on first boot**, then the database is
authoritative and admins edit it in the panel (no redeploy):

```rust
use adminx::rbac::{self, Ability};

adminx::seaorm::init(&db_url).await?;                  // 1. storage
adminx::seed(rbac::migrate_sql()).await?;              // 2. tables (SQL backends only; Mongo skips this)
rbac::init(vec![                                       // 3. seed-if-empty + load cache + register
    Ability::role("admin").can_manage_all(),
    Ability::role("editor")
        .can("update", "posts")
        .can("publish", "posts"),                      // a custom action, by name
    Ability::role("viewer").can_read_all(),
]).await?;
configure_auth(AuthConfig { /* ... */ });              // 4. turn auth on
register_resource(Box::new(PostResource));
rbac::register_resources();                            // 5. in-panel role/permission editors (optional)
```

How it fits together:

- **Pluggable, no core dependency.** `adminx-rbac` implements an `Authorizer`
  trait in `adminx-core` and registers it. Without the crate, adminx uses the
  built-in `allowed_roles()` list exactly as before — RBAC is purely additive.
- **Storage-agnostic.** Grants are read/written through the same `Storage` trait
  as everything else, so it works over SeaORM or Mongo. The one asymmetry is
  table creation: SQL needs `rbac::migrate_sql()` (SQLite-flavoured — adapt the
  DDL for Postgres/MySQL), Mongo needs nothing.
- **Fast checks.** Grants load into an in-memory cache once; the per-request
  decision does no I/O. A permission edit in the panel reloads the cache.
- **Single-writer caching (v1).** The cache reloads on writes made through *this*
  process, so a multi-instance deployment editing permissions on one node won't
  invalidate its peers until they reload. A shared-store/TTL option is on the
  [ROADMAP](ROADMAP.md).

### Protecting the panel with HTTP Basic Auth

The JWT login above is the main gate. If you *also* want a coarse **HTTP Basic**
prompt in front of the whole panel — e.g. to hide a staging deployment behind a
browser username/password — wrap the mounted routes with framework middleware and
read the credentials from an env var so it's easy to toggle.

**Axum** (add `base64 = "0.22"`):

```rust
use axum::{extract::Request, http::{header, StatusCode},
           middleware::{self, Next}, response::{IntoResponse, Response}};
use base64::{engine::general_purpose::STANDARD, Engine};

async fn basic_gate(req: Request, next: Next) -> Response {
    let want = std::env::var("PANEL_BASIC_AUTH").ok();   // "user:pass"; unset = off
    let ok = match &want {
        None => true,
        Some(creds) => req.headers().get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|h| h.strip_prefix("Basic "))
            .and_then(|b| STANDARD.decode(b).ok())
            .and_then(|d| String::from_utf8(d).ok())
            .is_some_and(|got| &got == creds),
    };
    if ok {
        next.run(req).await
    } else {
        (StatusCode::UNAUTHORIZED,
         [(header::WWW_AUTHENTICATE, r#"Basic realm="adminx""#)],
         "Unauthorized").into_response()
    }
}

// Gate only the panel:
let panel = adminx::axum::router().route_layer(middleware::from_fn(basic_gate));
let app = axum::Router::new().nest("/adminx", panel);
```

**Actix** — do the same header check in a `Transform` middleware and wrap the scope:
`App::new().service(adminx::actix::scope().wrap(YourBasicAuth))`.

### Multi-factor auth (MFA)

adminx ships **TOTP** two-factor auth (Google Authenticator, Authy, 1Password, …)
on top of the password login. No extra config — just the three `mfa_*` columns in
the [admin-users table](#admin-users-table).

The JWT carries an MFA step — `ok` or `pending` (a pending session can reach only
the MFA pages):

```
                        ┌─ mfa_enabled = false ─→  /adminx/mfa/setup   (skippable prompt)
POST /adminx/login  ───►┤
  (password ok)         └─ mfa_enabled = true  ─→  /adminx/mfa/verify  (enforced)
```

- **Not enabled** → logged in, but nudged to `/adminx/mfa/setup` (QR + secret).
  Confirming a code enables MFA and shows **10 one-time backup codes** once. The
  prompt is **skippable**.
- **Enabled** → login yields a `pending` session that must submit a TOTP **or a
  backup code** at `/adminx/mfa/verify`; a used backup code is consumed.

- **Details**: SHA-1 / 6 digits / 30 s step / ±1 skew; backup codes stored
  bcrypt-hashed; tokens issued before MFA existed decode as `ok` (backward
  compatible).

### The `adminx` CLI

```sh
cargo install adminx --features cli
```

| Command | Purpose |
|---|---|
| `adminx create-admin` | Create an admin user (idempotent). Flags `-e/--email`, `-p/--password`, `-r/--role`, or env `EMAIL`/`PASSWORD`/`ROLE`. |
| `adminx seed --file <path>` | Run seed statements (SQL for SeaORM, JSON commands for Mongo). Reads stdin if `--file` is omitted. |

Backend is chosen from the environment: `DATABASE_URL` → SeaORM, or
`MONGO_URL` + `MONGO_DB` → Mongo. SeaORM `create-admin` auto-creates the
`adminx_users` table (with MFA columns) if missing.

### Admin-users table

The admin table/collection needs `id`, `email`, `encrypted_password` (bcrypt),
`role`, plus three columns for MFA:

```sql
CREATE TABLE adminx_users (
    id                 SERIAL PRIMARY KEY,        -- INTEGER AUTOINCREMENT on SQLite
    email              TEXT NOT NULL UNIQUE,
    encrypted_password TEXT NOT NULL,             -- bcrypt hash
    role               TEXT NOT NULL DEFAULT 'admin',
    mfa_enabled        BOOLEAN NOT NULL DEFAULT false,  -- 0 on SQLite
    mfa_secret         TEXT,                      -- base32 TOTP secret
    mfa_backup_codes   TEXT                       -- JSON array of bcrypt-hashed codes
);
```

Already have the table? Add the MFA columns without recreating it:

```sql
ALTER TABLE adminx_users
    ADD COLUMN IF NOT EXISTS mfa_enabled BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS mfa_secret TEXT,
    ADD COLUMN IF NOT EXISTS mfa_backup_codes TEXT;
```

On Mongo the collection is schemaless — no DDL needed.

### Managing admin users

`create_admin` (and `adminx create-admin`) is **idempotent** — it skips an email
that already exists. To change something afterwards, run raw statements against the
live database with `adminx seed` (SQL, or Mongo command docs).

**Reset a password** — delete the user, then recreate (passwords are bcrypt-hashed,
so a plain SQL `UPDATE` can't set one):

```sh
# SQL
echo "DELETE FROM adminx_users WHERE email='admin@example.com'" \
  | DATABASE_URL=postgres://user:pass@host/db adminx seed
DATABASE_URL=postgres://user:pass@host/db \
  EMAIL=admin@example.com PASSWORD=newpass adminx create-admin

# Mongo
echo '{"delete":"adminx_users","deletes":[{"q":{"email":"admin@example.com"},"limit":1}]}' \
  | MONGO_URL=mongodb://host:27017 MONGO_DB=db adminx seed
MONGO_URL=mongodb://host:27017 MONGO_DB=db \
  EMAIL=admin@example.com PASSWORD=newpass adminx create-admin
```

**Change a role** (plain column, no hashing):

```sh
echo "UPDATE adminx_users SET role='editor' WHERE email='admin@example.com'" \
  | DATABASE_URL=... adminx seed
```

**Locked out by MFA?** Clear the user's MFA to force the setup prompt again:

```sh
# SQL
echo "UPDATE adminx_users SET mfa_enabled=false, mfa_secret=NULL, mfa_backup_codes=NULL \
  WHERE email='admin@example.com'" | DATABASE_URL=... adminx seed

# Mongo
echo '{"update":"adminx_users","updates":[{"q":{"email":"admin@example.com"},"u":{"$set":{"mfa_enabled":false,"mfa_secret":null,"mfa_backup_codes":null}}}]}' \
  | MONGO_URL=... MONGO_DB=... adminx seed
```

### CSV / JSON export

Every list exports without extra code (and honours active [filters](#filters)):

- `GET /adminx/{base}/list?download=csv`
- `GET /adminx/{base}/list?download=json`

Also available as buttons on the list page. Capped at 10,000 rows; CSV values are
RFC-escaped.

### REST API surface

Per resource, relative to the mount (`/adminx`):

| Route | Method | Purpose |
|---|---|---|
| `/{base}/api` | GET | List — `?page=`, `?per_page=` (≤200), `?sort=col` / `?sort=-col`, plus filters |
| `/{base}/api` | POST | Create (JSON body) |
| `/{base}/api/{id}` | GET / PUT / DELETE | Get / update / delete |
| `/{base}/{id}/action/{name}` | POST | Custom action |
| `/health` | GET | DB connectivity probe |

### HTML admin UI

Every resource also gets a Tera-rendered UI (dark-mode aware, TailwindCSS), served
identically by both adapters. Record data is autoescaped (XSS-safe).

| Route | Purpose |
|---|---|
| `/` | Dashboard (auto menu of registered resources) |
| `/{base}/list` | Table + pagination + filters + View/Edit/Delete + Export |
| `/{base}/new`, `/{base}/edit/{id}` | Create / edit form |
| `/{base}/view/{id}` | Record detail + custom-action buttons |
| `/login`, `/logout`, `/mfa/setup`, `/mfa/verify` | Auth + MFA pages |

### Environment variables

Conventions used by the demos and the CLI; your app decides what to read.

| Var | Purpose |
|---|---|
| `DATABASE_URL` | SeaORM URL: `postgres://…`, `mysql://…`, `sqlite://file.db?mode=rwc` |
| `MONGO_URL` + `MONGO_DB` | MongoDB connection + database |
| `JWT_SECRET` | HS256 signing key (`openssl rand -hex 32`) |
| `PORT` | listen port |
| `EMAIL` / `PASSWORD` / `ROLE` | inputs for `adminx create-admin` |
| `ADMINX_EMAIL` / `ADMINX_PASSWORD` | seeded admin (demo convention) |
| `ADMINX_SECURE_COOKIE` | `1` when served over HTTPS |
| `ADMINX_TAILWIND_SRC` | override the UI stylesheet source (see below) |

**Self-hosting the UI stylesheet.** The admin UI loads Tailwind from
`cdn.tailwindcss.com` by default — convenient, but a dev-mode build that needs
network access and is blocked by a strict Content-Security-Policy. Point it at a
self-hosted build instead, either with the `ADMINX_TAILWIND_SRC` env var or in
code:

```rust
adminx::set_tailwind_src("/static/tailwind.min.css");   // served by your app
```

### Deployment

adminx compiles into your app's single binary. Ship that binary, run it under
**systemd**, and put **nginx** in front for TLS. Set `secure_cookie: true` and a
strong `JWT_SECRET` in production.

**1. Build the release binary** (locally or on the server):

```sh
cargo build --release        # → target/release/myapp
```

**2. Place the binary + environment** on the server:

```sh
sudo mkdir -p /opt/myapp
sudo cp target/release/myapp /opt/myapp/
sudo tee /opt/myapp/app.env >/dev/null <<'ENV'
JWT_SECRET=REPLACE_WITH_openssl_rand_-hex_32
DATABASE_URL=postgres://user:pass@127.0.0.1:5432/mydb
PORT=8080
ADMINX_SECURE_COOKIE=1
ENV
sudo chmod 600 /opt/myapp/app.env
```

> Your `main` reads these — e.g. set `AuthConfig { secure_cookie: true, .. }`
> in production so the session cookie is HTTPS-only.

**3. systemd unit** — `/etc/systemd/system/myapp.service`:

```ini
[Unit]
Description=My adminx app
After=network.target postgresql.service

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/myapp
EnvironmentFile=/opt/myapp/app.env
ExecStart=/opt/myapp/myapp
Restart=on-failure
RestartSec=3

[Install]
WantedBy=multi-user.target
```

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now myapp
sudo systemctl status myapp        # check it's listening on 127.0.0.1:8080
```

**4. nginx reverse proxy + TLS** — `/etc/nginx/sites-available/myapp`:

```nginx
server {
    listen 80;
    server_name admin.example.com;
    return 301 https://$host$request_uri;      # force HTTPS
}

server {
    listen 443 ssl;
    server_name admin.example.com;

    ssl_certificate     /etc/letsencrypt/live/admin.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/admin.example.com/privkey.pem;

    location / {
        proxy_pass         http://127.0.0.1:8080;
        proxy_set_header   Host              $host;
        proxy_set_header   X-Real-IP         $remote_addr;
        proxy_set_header   X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header   X-Forwarded-Proto $scheme;   # tells the app it's on TLS
    }
}
```

```sh
sudo ln -s /etc/nginx/sites-available/myapp /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
sudo certbot --nginx -d admin.example.com     # obtain/renew the TLS cert
```

Your panel is now at `https://admin.example.com/adminx`. Update after a new build
with `cargo build --release`, copy the binary, and `sudo systemctl restart myapp`.

**Hardening tips:** keep the app bound to `127.0.0.1` (only nginx faces the
internet), add an [HTTP Basic gate](#protecting-the-panel-with-http-basic-auth) or
an nginx `allow`/`deny` IP allow-list for staging, and rotate `JWT_SECRET` to
invalidate all sessions. `adminx-demo` has a fuller **AWS EC2** walkthrough in
[`adminx-demo/README.md`](https://github.com/srotas-space/adminx/blob/main/adminx-demo/README.md).

### Troubleshooting

| Symptom | Cause / fix |
|---|---|
| **"Invalid email or password"** with the right password | The admin was created in a *different* database than the app reads. Match `DATABASE_URL` / `MONGO_DB` to the running app. |
| Login **loops** or lands on `/adminx/mfa/verify` and you're stuck | MFA is on and the authenticator is lost — clear it via SQL/Mongo (see [Managing admin users](#managing-admin-users)). |
| Every page is **public**, never asks to log in | `configure_auth(..)` wasn't called — auth is opt-in. |
| Login "succeeds" but bounces back to `/login` | `secure_cookie: true` while on plain `http://` — the browser drops the cookie. Use `secure_cookie: false` for local http. |
| Login form returns **403** "Your session expired" | The `adminx_csrf` cookie didn't reach the POST. Same `secure_cookie` mismatch as above, or the form was posted from a page served on a different site. Reload `/adminx/login` and retry. |
| Login returns **429** "Too many attempts" | The per-account throttle tripped (default 10 failures / 15 min). Wait for the window to lapse, or loosen it via `ratelimit::configure` — see [Rate limiting](#rate-limiting). |
| Mongo: View/Edit links or updates target the wrong record | Add `fn primary_key(&self) -> &'static str { "_id" }` to the resource. |
| SeaORM: **"relation … does not exist"** | The table wasn't created — run your `CREATE TABLE`/seed. adminx doesn't migrate your app tables. |
| Publish: **"no matching package named `adminx-core`"** | Publish dependencies first: `adminx-core` → adapters → `adminx`. Each must be on crates.io before the next resolves. |

### Status

Complete and tested: neutral core, SeaORM (PostgreSQL/MySQL/SQLite) + MongoDB,
Actix + Axum (Axum 0.8), dynamic JSON CRUD, Tera HTML UI, JWT-cookie auth + RBAC,
**TOTP MFA with backup codes**, **list filters**, custom actions, CSV/JSON export,
**seeding + admin CLI**, pagination/sort, health, and the single-name **`adminx`**
facade.

Roadmap: a switch to make MFA mandatory (today it's a skippable prompt), and
backup-code regeneration from an account page.

---

## 🌟 Community

[![GitHub Discussions](https://img.shields.io/github/discussions/srotas-space/adminx)](https://github.com/srotas-space/adminx/discussions)

Join our growing community of Rust developers building admin panels with AdminX!

- 📖 [Documentation](https://docs.rs/adminx)
- 💬 [Discussions](https://github.com/srotas-space/adminx/discussions)
- 🐛 [Issues](https://github.com/srotas-space/adminx/issues)
- 📧 Email: info@srotas.space
- 📧 Email: snmmaurya@gmail.com
- 📧 Email: deepxmaurya@gmail.com

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](https://github.com/srotas-space/adminx/blob/main/LICENSE) file for details.

## 🙏 Acknowledgments

- Web frameworks: [Actix Web](https://actix.rs/) and [Axum](https://github.com/tokio-rs/axum)
- Storage: [SeaORM](https://www.sea-ql.org/SeaORM/) (PostgreSQL / MySQL / SQLite) and [MongoDB](https://www.mongodb.com/)
- UI: [TailwindCSS](https://tailwindcss.com/) styling with [Tera](https://keats.github.io/tera/) templates
- Auth: JWT via [jsonwebtoken](https://crates.io/crates/jsonwebtoken), TOTP via [totp-rs](https://crates.io/crates/totp-rs)


## 🗺️ Roadmap

We are actively building AdminX step by step.  
The roadmap includes phases like core CRUD foundation, extended resource features, authentication & RBAC, export/import, custom pages, UI themes, and optional extensions.

👉 See the full roadmap here: [ROADMAP.md](https://github.com/srotas-space/adminx/blob/main/ROADMAP.md)

[![Project Status](https://img.shields.io/badge/status-actively--developed-brightgreen.svg)](https://github.com/srotas-space/adminx)
[![Contributions Welcome](https://img.shields.io/badge/contributions-welcome-blue.svg)](https://github.com/srotas-space/adminx/issues)

📦 [Sample starter template](https://github.com/srotas-space/adminx-examples)
---


Made with ❤️ by [Srotas Space](https://open-source.srotas.space)

---


## 👥 Contributors

- **[Snm Maurya](https://github.com/srotas-space)** - Creator & Lead Developer
  <img src="https://srotasspace.s3.ap-south-1.amazonaws.com/snm.png" alt="Snm Maurya" width="80" height="80" style="border-radius: 50%;">
  [LinkedIn](https://www.linkedin.com/in/snmmaurya/)

- **[Deepak Maurya](https://github.com/deepxmaurya)** - Core Developer & Contributor
  <img src="https://srotasspace.s3.ap-south-1.amazonaws.com/deepx.png" alt="Deepak Maurya" width="80" height="80" style="border-radius: 50%;"> 
  [LinkedIn](https://www.linkedin.com/in/deepxmaurya/)

---

[![GitHub stars](https://img.shields.io/github/stars/srotas-space/adminx?style=social)](https://github.com/srotas-space/adminx)


