# acme-types-rs

This crate defines primitives for implementing ACME providers and clients.

See [RFC 8555](https://datatracker.ietf.org/doc/html/rfc8555) for more information.

- This crate does not provide any functionality in terms of actually interacting with an ACME client or provider, e.g. in terms of a server or client or HTTP library (beyond support for de/serialization, as documented below)
- This crate does not provide any of the cryptographic functions necessary for implementing an ACME provider or client

```rust
use acme_types::v2 as ACME;

let resp = reqwest::get("https://acme-v02.api.letsencrypt.org/directory")
    .await?
    .text()
    .await?;

let directory = ACME::Directory::from_str(&resp).unwrap();

println!("{:#?}", directory);

>>> Directory {
>>>     new_nonce: "https://acme-v02.api.letsencrypt.org/acme/new-nonce",
>>>     new_account: "https://acme-v02.api.letsencrypt.org/acme/new-acct",
>>>     new_order: "https://acme-v02.api.letsencrypt.org/acme/new-order",
>>>     new_authorization: None,
>>>     revoke_certificate: "https://acme-v02.api.letsencrypt.org/acme/revoke-cert",
>>>     key_change: "https://acme-v02.api.letsencrypt.org/acme/key-change",
>>>     metadata: Some(
>>>         DirectoryMetadata {
>>>             terms_of_service: Some(
>>>                 "https://letsencrypt.org/documents/LE-SA-v1.2-November-15-2017.pdf",
>>>             ),
>>>             website: Some(
>>>                 "https://letsencrypt.org",
>>>             ),
>>>             caa_identities: Some(
>>>                 [
>>>                     "letsencrypt.org",
>>>                 ],
>>>             ),
>>>             external_account_required: None,
>>>         },
>>>     ),
>>> }
```

## JSON De/serialization

Serialization and deserialization to and from JSON is supported using the `serde` (and `serde_json`) crate(s). This integration is optional (feature `json`):

```toml
acme-types = { version = "*", features = ["json"] }
```

When this feature is enabled, `from_str` and `to_string` are implemented on top-level ACME objects and resources.