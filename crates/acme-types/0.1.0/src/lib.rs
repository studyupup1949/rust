/// Rust primitives for [RFC 8555](https://datatracker.ietf.org/doc/html/rfc8555) resources
///
/// ```
/// use acme_types::v2 as ACME;
///
/// let resp = reqwest::blocking::get("https://acme-v02.api.letsencrypt.org/directory")
///     .unwrap()
///     .text()
///     .unwrap();
///
/// let directory = ACME::Directory::from_str(&resp).unwrap();
///
/// println!("{:?}", directory);
///```
pub mod v2;
