# adminx-actix

[![crates.io](https://img.shields.io/crates/v/adminx-actix.svg)](https://crates.io/crates/adminx-actix)
[![docs.rs](https://img.shields.io/docsrs/adminx-actix)](https://docs.rs/adminx-actix)
[![license: MIT](https://img.shields.io/crates/l/adminx-actix.svg)](https://github.com/srotas-space/adminx/blob/main/LICENSE)

**Actix Web adapter** for the [adminx](https://crates.io/crates/adminx) admin-panel framework.

## You probably want `adminx`

Don't depend on this crate directly — depend on the single
**[`adminx`](https://crates.io/crates/adminx)** facade and enable it via features:

```toml
adminx = { version = "3", features = ["actix", "seaorm"] }
```

Full documentation and usage: **[docs.rs/adminx](https://docs.rs/adminx)**. MIT licensed.
