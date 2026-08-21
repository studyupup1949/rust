# adminx-mongo

[![crates.io](https://img.shields.io/crates/v/adminx-mongo.svg)](https://crates.io/crates/adminx-mongo)
[![docs.rs](https://img.shields.io/docsrs/adminx-mongo)](https://docs.rs/adminx-mongo)
[![license: MIT](https://img.shields.io/crates/l/adminx-mongo.svg)](https://github.com/srotas-space/adminx/blob/main/LICENSE)

**MongoDB storage backend** for the [adminx](https://crates.io/crates/adminx) admin-panel framework.

## You probably want `adminx`

Don't depend on this crate directly — depend on the single
**[`adminx`](https://crates.io/crates/adminx)** facade and enable it via features:

```toml
adminx = { version = "3", features = ["mongo", "axum"] }
```

Full documentation and usage: **[docs.rs/adminx](https://docs.rs/adminx)**. MIT licensed.
