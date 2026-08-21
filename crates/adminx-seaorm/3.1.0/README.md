# adminx-seaorm

[![crates.io](https://img.shields.io/crates/v/adminx-seaorm.svg)](https://crates.io/crates/adminx-seaorm)
[![docs.rs](https://img.shields.io/docsrs/adminx-seaorm)](https://docs.rs/adminx-seaorm)
[![license: MIT](https://img.shields.io/crates/l/adminx-seaorm.svg)](https://github.com/srotas-space/adminx/blob/main/LICENSE)

**SeaORM storage backend** (PostgreSQL / MySQL / SQLite) for the [adminx](https://crates.io/crates/adminx) admin-panel framework.

## You probably want `adminx`

Don't depend on this crate directly — depend on the single
**[`adminx`](https://crates.io/crates/adminx)** facade and enable it via features:

```toml
adminx = { version = "3", features = ["seaorm", "axum"] }
```

Full documentation and usage: **[docs.rs/adminx](https://docs.rs/adminx)**. MIT licensed.
