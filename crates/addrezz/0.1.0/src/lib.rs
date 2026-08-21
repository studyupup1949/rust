//! addrezz — generic, RFC-compliant address parsing with confident defaults.
//!
//!
//! # Usage
//!
//! ```toml
//! [dependencies]
//! addrezz = "0.1"
//! ```
//!
//! ## Parsing
//!
//! Parsing addresses and hosts with varying level of completeness is possible, with minimal
//! guesswork to keep track of under the hood.
//!
//! One assumption is feature-dependent, specifically, all URLs without specified schemes are
//! assumed to be `https://` by default. To change this behavior, add the `default_http` feature.
//!
//! ```toml
//! addrezz = { version = "0.1", features = ["default_http"]}
//! ```
//!
//! ```
//! use addrezz::{Addr, Scheme, Host};
//!
//! let full_url = Addr::parse("https://user@example.com:8443/v1/items?q=rust#top").unwrap();
//! assert_eq!(full_url.scheme, Scheme::Https);
//! assert_eq!(full_url.port, Some(8443));
//! assert_eq!(full_url.path, "/v1/items");
//!
//! let hostname = Addr::parse("example.com").unwrap();
//! assert_eq!(hostname.scheme, Scheme::Https);
//!
//!
//! // Git remote with SCP-style address
//! let scp_addr = Addr::parse("git@github.com:rust-lang/rust").unwrap();
//! assert_eq!(scp_addr.scheme, Scheme::Ssh);
//! assert_eq!(scp_addr.path, "/rust-lang/rust");
//!
//! let v4 = Addr::parse("http://127.0.0.1:8080/").unwrap();
//! assert!(matches!(v4.host, Host::Ipv4(_)));
//! let v6 = Addr::parse("https://[::1]:9200").unwrap();
//! assert!(matches!(v6.host, Host::Ipv6(_)));
//! ```
//!
//! ## Schemes and their IANA semantics
//!
//! ```
//! use addrezz::Addr;
//!
//! let pg = Addr::parse("postgres://db.internal/app").unwrap();
//! assert_eq!(pg.port, None); // No port was explicitly provided
//! assert_eq!(pg.effective_port(), Some(5432)); // Scheme-derived fallback
//!
//! let websock = Addr::parse("wss://gateway.example.com").unwrap();
//! assert!(websock.scheme.is_secure()); // Scheme-derived security properties
//! ```
//!
//! ## URL path and query parameters
//!
//! ```
//! use addrezz::Addr;
//!
//! let a = Addr::parse("https://example.com/foo/bar%20baz?key=val&a=b%20c").unwrap();
//!
//! let segments: Vec<_> = a.path_segments().collect();
//! assert_eq!(segments, vec!["foo", "bar baz"]); // percent-decoded path segments
//!
//! let qparams: Vec<_> = a.query_pairs().collect();
//! assert_eq!(qparams[0], ("key".into(), "val".into()));
//! ```
//!
//! ## Properties
//!
//! You can use [`Host::is_local`] to determine if the host is a local IPv4/IPv6 address, or if it
//! has a `.local`/`.localhost` extension.
//!
//! You can use [`Origin::same_origin`] to compare whether two addresses share an origin, useful for
//! web application security policies:
//!
//! ```
//! use addrezz::Addr;
//!
//! let a = Addr::parse("https://example.com/").unwrap();
//! let b = Addr::parse("https://example.com:443/other").unwrap();
//!
//! assert!(a.origin().same_origin(&b.origin()));
//! ```
//!
//! # Optional Features
//!
//! ## Feature flags
//! 
//! | Feature | Default | What it adds |
//! |---|---|---|
//! | `default_http` | | bare hosts default to `http` (mutually exclusive with above) |
//! | `macros` | yes | the compile-time `addrezz!` / `addrezz_vec!` proc macros |
//! | `redact` | yes | `Debug` for `Userinfo` hides the password ([docs](docs/REDACT.md)) |
//! | `serde` | | `Serialize` / `Deserialize` on all public types |
//! | `url` | | conversions to/from `url::Url` |
//! | `http` | | conversions to `http::Uri`, `Authority`, `HeaderValue` |
//! | `reqwest` | | conversions to `reqwest::Url` (implies `http`) |
//! | `resolve` | | blocking DNS via `getaddrinfo` ([docs](docs/RESOLVE.md)) |
//! | `resolve_async` | | async DNS via `hickory-resolver` ([docs](docs/RESOLVE.md)) |
//! | `whois` | | whois lookups via `whoizz` ([docs](docs/WHOIS.md)) |
//! | `psl` | | public suffix / eTLD+1 via `psl` ([docs](docs/PSL.md)) |
//! | `ipnet` | | CIDR membership tests via `ipnet` ([docs](docs/IPNET.md)) |
//! | `arbitrary` | | `arbitrary::Arbitrary` for fuzzing ([docs](docs/ARBITRARY.md)) |
//! | `proptest` | | `proptest::Arbitrary` strategy ([docs](docs/PROPTEST.md)) |
//!
//!
//! ## `macros` feature
//!
//! Allows the use of both compile-time macros ([addrezz!] and [addrezz_vec!]), and runtime macros
//! ([addr!] and [try_addr!]).
//!
//! Compile-time macros ensure, that the [Addr] is correct on successful compilation, while runtime
//! macros allow different levels of validation at runtime, i.e. [addr!] panics on failure, while
//! [try_addr!] returns a result.
//!
//! Compile-time macros accept both string and token inputs (not quoted values), while runtime
//! macros accept not just quoted strings, but also non-literals (variables, references).
//!
//! **Note**: If the URL includes the scheme, only the quoted form will parse, as `scheme://` will
//! parse as a comment start before the macro could take effect.
//!
//! ```
//! use addrezz::{addrezz, addrezz_vec};
//!
//! let a = addrezz!("ssh://git@github.com/rust-lang/rust");
//! let b = addrezz! { git@github.com:rust-lang/rust };
//!
//! assert_eq!(a, b);
//!
//! let mirrors = addrezz_vec!(
//!     "https://m1.example.com",
//!     "https://m2.example.com",
//!     "https://m3.example.com",
//! );
//! assert_eq!(mirrors.len(), 3);
//! ```
//!
//! ```compile_fail
//! use addrezz::addrezz;
//!
//! let fail = addrezz!("://fail_at_compile")
//! ```
//!
//! ```
//! use addrezz::{addr, try_addr};
//!
//! let a = addr!("https://example.com/health");
//! assert_eq!(a.scheme.as_str(), "https");
//!
//! let raw = std::env::var("ENDPOINT").unwrap_or_else(|_| "https://example.com".into());
//! let a = addr!(&raw);
//!
//! match try_addr!("not a url:::") {
//!     Ok(url) => println!("parsed {url}"),
//!     Err(e) => eprintln!("bad address: {e}"),
//! }
//! ```
//!
//! 
//! ## `serde` feature
//!
//! Every public type derives `Serialize` / `Deserialize` behind the `serde`
//! feature. `Addr` serializes as a struct, so the components stay addressable in
//! the serialized form.
//!
//! ```no_run
//! use addrezz::Addr;
//!
//! let a = Addr::parse("https://user:secret@api.example.com:8443/v1?q=1").unwrap();
//! let json = serde_json::to_string_pretty(&a).unwrap();
//! // {
//! //   "scheme": "Https",
//! //   "userinfo": { "username": "user", "password": "secret" },
//! //   "host": { "Domain": "api.example.com" },
//! //   "port": 8443,
//! //   "path": "/v1",
//! //   "query": "q=1",
//! //   "fragment": null
//! // }
//!
//! let back: Addr = serde_json::from_str(&json).unwrap();
//! assert_eq!(a, back);
//! ```
//!
//! ## `url` feature
//!
//! ```no_run
//! use addrezz::Addr;
//! use url::Url;
//!
//! let a = Addr::parse("https://example.com/a/b?c=d").unwrap();
//!
//! let u: Url = a.clone().try_into().unwrap();
//! assert_eq!(u.as_str(), "https://example.com/a/b?x=1");
//!
//! let back: Addr = (&u).try_into().unwrap();
//! assert_eq!(a, back);
//! ```
//!
//! ## `http` feature
//!
//! Allows conversion from [`Addr`] to [`http::Uri`], [`http::uri::Authority`],
//! [`http::header::HOST`]-header's [`http::HeaderValue`]:
//!
//! ```no_run
//! use addrezz::Addr;
//! use http::{Uri, HeaderValue, uri::Authority};
//!
//! let a = Addr::parse("https://user@example.com:8443/v1/items").unwrap();
//!
//! let uri: Uri = (&a).try_into().unwrap();
//! let auth: Authority = (&a).try_into().unwrap();
//!
//! // For the 'Host' header value, we only keep the port intact when it's specified explicitly and
//! // differs from the schema default. If it matches the default, we strip it.
//! let host: HeaderValue = (&a).try_into().unwrap();
//! assert_eq!(host.to_str().unwrap(), "example.com:8443");
//! ```
//!
//! ## `reqwest` feature
//!
//! Allows conversion of [`Addr`] to [`reqwest::Url`] (and also enables `http` feature).
//!
//! This feature is purely convenience to reduce direct dependencies, since [`reqwest::Url`] is a
//! re-export of [`url::Url`], so if direct deps only specify `reqwest`, this should be more
//! convenient to use.
//!
//! ```no_run
//! use addrezz::Addr;
//!
//! # async fn req() -> Result<(), reqwest::Error> {
//!
//! let a = Addr::parse("https://api.example.com/search?q=rust").unwrap();
//!
//! let url: reqwest::Url = a.try_into().unwrap();
//! let resp = reqwest::Client::new().get(url).send().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## `sqlx` feature
//!
//! [`Addr`] implements [`sqlx::Type`], [sqlx::Encode], [sqlx::Decode], and is stored as `TEXT` or
//! `VARCHAR`.
//!
//! The stored [String] is decoded back through [Addr::parse]. Any backend that stores type [String]
//! can handle [Addr].
//!
//! ```toml
//! addrezz = { version = "0.1", features = ["sqlx"] }
//! ```
//!
//! ```rust
//! use addrezz::Addr;
//!
//! # async fn run(pool: sqlx::PgPool) -> Result<(), sqlx::Error> {
//! let endpoint = Addr::parse("https://hooks.example.com/incoming").unwrap();
//!
//! sqlx::query("INSERT INTO webhooks (url) VALUES ($1)")
//!     .bind(&endpoint)
//!     .execute(&pool)
//!     .await?;
//!
//! let row: (Addr,) = sqlx::query_as("SELECT url FROM webhooks LIMIT 1")
//!     .fetch_one(&pool)
//!     .await?;
//! assert_eq!(row.0.host.to_string(), "hooks.example.com");
//! # Ok(())
//! # }
//! ```

pub use addrezz_core::{Addr, Host, HostError, Origin, ParseError, ParseOptions, Scheme, Userinfo};

// Re-export the declarative macros from core.
pub use addrezz_core::{addr, try_addr};

#[cfg(feature = "macros")]
pub use addrezz_macros::{addrezz, addrezz_vec};

/// Path the proc macros expand to. Not part of the public API.
#[doc(hidden)]
pub use addrezz_core as __private;

#[cfg(feature = "whois")]
pub use addrezz_core::{WhoisError, WhoisResponse};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_parse() {
        let a = Addr::parse("https://gitlab.com/").unwrap();
        assert_eq!(a.scheme, Scheme::Https);
    }

    #[test]
    fn facade_macro() {
        let a = addr!("ssh://git@gitlab.com/foo");
        assert_eq!(a.scheme, Scheme::Ssh);
    }
}
