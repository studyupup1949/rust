# addrezz

Generic, RFC-compliant address parsing with confident defaults.

`addrezz` parses anything that names a network endpoint (URLs, bare domains,
IP literals, IPv6, SSH/SCP-style remotes) into one normalized [`Addr`] type.
It fills in sensible defaults (scheme for bare hosts, IANA default ports) and
exposes the parts you actually reach for: scheme, host, port, userinfo, path,
query, and fragment.

```toml
[dependencies]
addrezz = "0.1"
```

Optional integrations are behind feature flags (see [Feature flags](#feature-flags)).

## Basic usage

One entry point, many input shapes:

```rust
use addrezz::{Addr, Scheme, Host};

// Full URL.
let a = Addr::parse("https://user@example.com:8443/v1/items?q=rust#top").unwrap();
assert_eq!(a.scheme, Scheme::Https);
assert_eq!(a.port, Some(8443));
assert_eq!(a.path, "/v1/items");

// Bare host: the scheme is filled in (https by default).
let a = Addr::parse("example.com").unwrap();
assert_eq!(a.scheme, Scheme::Https);

// SCP-style git remote becomes ssh.
let a = Addr::parse("git@github.com:rust-lang/rust").unwrap();
assert_eq!(a.scheme, Scheme::Ssh);
assert_eq!(a.path, "/rust-lang/rust");

// IP literals, v4 and v6.
let a = Addr::parse("http://127.0.0.1:8080/").unwrap();
assert!(matches!(a.host, Host::Ipv4(_)));
let a = Addr::parse("https://[::1]:9200").unwrap();
assert!(matches!(a.host, Host::Ipv6(_)));
```

Ports and schemes carry their IANA semantics:

```rust
use addrezz::Addr;

let a = Addr::parse("postgres://db.internal/app").unwrap();
assert_eq!(a.port, None);                 // none was written
assert_eq!(a.effective_port(), Some(5432)); // scheme default

let a = Addr::parse("wss://gateway.example.com").unwrap();
assert!(a.scheme.is_secure());
```

Pull apart the path and query without re-parsing:

```rust
use addrezz::Addr;

let a = Addr::parse("https://example.com/foo/bar%20baz?key=val&a=b%20c").unwrap();

let segs: Vec<_> = a.path_segments().collect();
assert_eq!(segs, vec!["foo", "bar baz"]); // percent-decoded

let pairs: Vec<_> = a.query_pairs().collect();
assert_eq!(pairs[0], ("key".into(), "val".into()));
```

Compare origins (RFC 6454) and spot local addresses:

```rust
use addrezz::Addr;

let a = Addr::parse("https://example.com/").unwrap();
let b = Addr::parse("https://example.com:443/other").unwrap();
assert!(a.origin().same_origin(&b.origin())); // default port is implied

let a = Addr::parse("http://192.168.1.10:3000/").unwrap();
assert!(a.is_local());
```

Build values at compile time or run time with the macros:

```rust
use addrezz::{addr, addrezz};

let a = addr!("https://example.com/");        // runtime parse, panics on bad input
let b = addrezz!("ssh://git@example.com/repo"); // compile-time, errors at the call site
```

See [docs/MACROS.md](docs/MACROS.md) for the full set (`addr!`, `try_addr!`,
`addrezz!`, `addrezz_vec!`).

## serde

Every public type derives `Serialize` / `Deserialize` behind the `serde`
feature. `Addr` serializes as a struct, so the components stay addressable in
the serialized form.

```toml
addrezz = { version = "0.1", features = ["serde"] }
```

```rust
use addrezz::Addr;

let a = Addr::parse("https://user:secret@api.example.com:8443/v1?q=1").unwrap();
let json = serde_json::to_string_pretty(&a).unwrap();
// {
//   "scheme": "Https",
//   "userinfo": { "username": "user", "password": "secret" },
//   "host": { "Domain": "api.example.com" },
//   "port": 8443,
//   "path": "/v1",
//   "query": "q=1",
//   "fragment": null
// }

let back: Addr = serde_json::from_str(&json).unwrap();
assert_eq!(a, back);
```

If you want `Addr` stored as a single string instead of a struct, serialize
its `Display` form (`a.to_string()`) and `Addr::parse` it back.

## url

With the `url` feature, `Addr` converts to and from [`url::Url`]. Conversions
are fallible because `url::Url` is stricter about some shapes.

```toml
addrezz = { version = "0.1", features = ["url"] }
```

```rust
use addrezz::Addr;
use url::Url;

let a = Addr::parse("https://example.com/a/b?x=1").unwrap();

let u: Url = a.clone().try_into().unwrap();
assert_eq!(u.as_str(), "https://example.com/a/b?x=1");

let back: Addr = (&u).try_into().unwrap();
assert_eq!(a, back);
```

## http

The `http` feature converts `Addr` into the `http` crate's types: `Uri`,
`Authority`, and a `Host`-header `HeaderValue`. Useful when building requests
by hand.

```toml
addrezz = { version = "0.1", features = ["http"] }
```

```rust
use addrezz::Addr;
use http::{Uri, HeaderValue, uri::Authority};

let a = Addr::parse("https://user@example.com:8443/v1/items").unwrap();

let uri: Uri = (&a).try_into().unwrap();
let auth: Authority = (&a).try_into().unwrap();    // user@example.com:8443

// Ready for the `Host` request header. The port is elided when it
// matches the scheme default, kept otherwise.
let host: HeaderValue = (&a).try_into().unwrap();
assert_eq!(host.to_str().unwrap(), "example.com:8443");
```

## reqwest

The `reqwest` feature adds conversions to `reqwest::Url` (and implies `http`).
Because `reqwest::Url` re-exports `url::Url`, enabling `url` already covers
these conversions; the separate feature is for crates that depend on `reqwest`
without naming `url` directly.

```toml
addrezz = { version = "0.1", features = ["reqwest"] }
```

```rust
use addrezz::Addr;

let a = Addr::parse("https://api.example.com/search?q=rust").unwrap();

// Hand the URL straight to a reqwest client.
let url: reqwest::Url = a.try_into().unwrap();
let resp = reqwest::Client::new().get(url).send().await?;
# Ok::<(), reqwest::Error>(())
```

## sqlx

With the `sqlx` feature, `Addr` implements `Type` / `Encode` / `Decode`,
stored as `TEXT` / `VARCHAR`. Decoding runs the stored string back through
`Addr::parse`. Works on any sqlx backend with a `String` type (Postgres,
MySQL, SQLite).

```toml
addrezz = { version = "0.1", features = ["sqlx"] }
```

```rust
use addrezz::Addr;

# async fn run(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
let endpoint = Addr::parse("https://hooks.example.com/incoming").unwrap();

sqlx::query("INSERT INTO webhooks (url) VALUES ($1)")
    .bind(&endpoint)
    .execute(&pool)
    .await?;

let row: (Addr,) = sqlx::query_as("SELECT url FROM webhooks LIMIT 1")
    .fetch_one(&pool)
    .await?;
assert_eq!(row.0.host.to_string(), "hooks.example.com");
# Ok(())
# }
```

## Feature flags

| Feature | Default | What it adds |
|---|---|---|
| `default_http` | | bare hosts default to `http` (mutually exclusive with above) |
| `macros` | yes | the compile-time `addrezz!` / `addrezz_vec!` proc macros |
| `redact` | yes | `Debug` for `Userinfo` hides the password ([docs](docs/REDACT.md)) |
| `serde` | | `Serialize` / `Deserialize` on all public types |
| `url` | | conversions to/from `url::Url` |
| `http` | | conversions to `http::Uri`, `Authority`, `HeaderValue` |
| `reqwest` | | conversions to `reqwest::Url` (implies `http`) |
| `resolve` | | blocking DNS via `getaddrinfo` ([docs](docs/RESOLVE.md)) |
| `resolve_async` | | async DNS via `hickory-resolver` ([docs](docs/RESOLVE.md)) |
| `whois` | | whois lookups via `whoizz` ([docs](docs/WHOIS.md)) |
| `psl` | | public suffix / eTLD+1 via `psl` ([docs](docs/PSL.md)) |
| `ipnet` | | CIDR membership tests via `ipnet` ([docs](docs/IPNET.md)) |
| `arbitrary` | | `arbitrary::Arbitrary` for fuzzing ([docs](docs/ARBITRARY.md)) |
| `proptest` | | `proptest::Arbitrary` strategy ([docs](docs/PROPTEST.md)) |

The crate ships `default = ["macros", "redact"]`. Disable
defaults with `default-features = false` if you want `http` as the bare-host
scheme or a leaner build.

## More integrations

The widely used ones are above. The rest each have a short page with a runnable
example:

- [docs/MACROS.md](docs/MACROS.md) — `addr!`, `try_addr!`, `addrezz!`, `addrezz_vec!`
- [docs/RESOLVE.md](docs/RESOLVE.md) — DNS resolution, sync and async
- [docs/WHOIS.md](docs/WHOIS.md) — registry/RIR whois lookups
- [docs/PSL.md](docs/PSL.md) — public suffix, registered domain, subdomain
- [docs/IPNET.md](docs/IPNET.md) — CIDR / subnet membership
- [docs/REDACT.md](docs/REDACT.md) — keeping passwords out of logs
- [docs/PROPTEST.md](docs/PROPTEST.md) — property-testing with `Addr`
- [docs/ARBITRARY.md](docs/ARBITRARY.md) — fuzzing with `Addr`

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
