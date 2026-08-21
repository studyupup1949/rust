# adminx-axum

[![crates.io](https://img.shields.io/crates/v/adminx-axum.svg)](https://crates.io/crates/adminx-axum)
[![docs.rs](https://img.shields.io/docsrs/adminx-axum)](https://docs.rs/adminx-axum)
[![license: MIT](https://img.shields.io/crates/l/adminx-axum.svg)](https://github.com/srotas-space/adminx/blob/main/LICENSE)

**Axum web adapter** for the [adminx](https://crates.io/crates/adminx) admin-panel framework.

## You probably want `adminx`

Don't depend on this crate directly — depend on the single
**[`adminx`](https://crates.io/crates/adminx)** facade and enable it via features:

```toml
adminx = { version = "3", features = ["axum", "seaorm"] }
```

Full documentation and usage: **[docs.rs/adminx](https://docs.rs/adminx)**. MIT licensed.
