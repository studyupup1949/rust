# adminx

A **framework-neutral admin-panel framework** for Rust. Define a resource once
and serve it over **Actix Web** or **Axum**, backed by **PostgreSQL / MySQL /
SQLite** (SeaORM) or **MongoDB** — with auto CRUD, a rendered HTML admin UI,
JWT-cookie auth, and RBAC.

This crate is a thin **facade**: it re-exports the neutral core plus whichever
framework and storage adapters you enable via Cargo features, so you depend on a
single crate and type `adminx::…` everywhere.

```toml
[dependencies]
# Axum + PostgreSQL/MySQL/SQLite:
adminx = { version = "2", features = ["axum", "seaorm"] }
# Actix + MongoDB:
# adminx = { version = "2", features = ["actix", "mongo"] }

tokio       = { version = "1", features = ["full"] }
axum        = "0.8"          # or actix-web = "4"
serde_json  = "1"
async-trait = "0.1"
```

Feature flags — **frameworks:** `actix`, `axum`; **storage:** `seaorm`, `mongo`.
Pick one of each. Crates you don't select are never compiled.

## Usage

```rust
use adminx::prelude::*;                 // Resource, ReqCtx, ApiResponse, auth, ...
use async_trait::async_trait;

#[derive(Clone)]
struct UserResource;

#[async_trait]
impl Resource for UserResource {
    fn resource_name(&self) -> &'static str { "Users" }
    fn base_path(&self)     -> &'static str { "users" }
    fn table_name(&self)    -> &'static str { "users" }
    fn clone_box(&self) -> Box<dyn Resource> { Box::new(self.clone()) }
    fn permit_keys(&self) -> Vec<&'static str> { vec!["name", "email"] }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    adminx::seaorm::init("postgres://localhost/mydb").await?;   // storage adapter
    register_resource(Box::new(UserResource));

    let app = axum::Router::new().nest("/adminx", adminx::axum::router());
    let l = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(l, app).await?;
    Ok(())
}
```

On Actix, the only difference is the server: mount `adminx::actix::scope()` on
your `App`. On MongoDB, call `adminx::mongo::init(uri, db)` and set
`fn primary_key(&self) -> &'static str { "_id" }` on your resources.

## What you get per resource

- **REST/JSON API** — `GET/POST /adminx/{base}/api`, `GET/PUT/DELETE /adminx/{base}/api/{id}`
- **HTML admin UI** — dashboard, list (with pagination/sort), create/edit forms, detail view
- **Auth + RBAC** — `configure_auth(..)`, JWT-in-cookie login, role-gated routes
- **Custom actions** and **CSV/JSON export**

See the [workspace README](https://github.com/srotas-space/adminx) for the full
guide, architecture, and the runnable demo.

## Versioning

`adminx` 2.x is the framework-neutral redesign. The 1.x line was a monolithic
Actix + MongoDB crate; 2.x supersedes it and is **not** API-compatible.

## License

MIT
