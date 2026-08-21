# adminx-mongo

**MongoDB storage backend** for the [adminx](https://crates.io/crates/adminx) admin-panel framework.

## You probably want `adminx`

Don't depend on this crate directly — depend on the single
**[`adminx`](https://crates.io/crates/adminx)** facade and enable it via features:

```toml
adminx = { version = "2", features = ["mongo", "axum"] }
```

Full documentation and usage: **[docs.rs/adminx](https://docs.rs/adminx)**. MIT licensed.
