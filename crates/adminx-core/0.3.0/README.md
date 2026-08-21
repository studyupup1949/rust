# adminx-core

Framework- and database-neutral **core** of the [adminx](https://crates.io/crates/adminx)
admin-panel framework: the `Resource` trait, the `Storage` abstraction, the
registry, neutral `ReqCtx`/`ApiResponse` types, the Tera-rendered UI, JWT auth +
RBAC, MFA, CSRF-protected forms, per-account login/MFA rate limiting, a pluggable
`Authorizer` seam (implemented by [`adminx-rbac`](https://crates.io/crates/adminx-rbac)),
and list filters. It contains **no web framework and no database**.

## You probably want `adminx`

Depend on the single **[`adminx`](https://crates.io/crates/adminx)** facade
instead — it re-exports this core plus your chosen framework (Actix/Axum) and
storage (SeaORM/Mongo) adapters behind Cargo features:

```toml
adminx = { version = "2", features = ["axum", "seaorm"] }
```

Full documentation and usage: **[docs.rs/adminx](https://docs.rs/adminx)** and the
[`adminx` README](https://crates.io/crates/adminx). MIT licensed.
