[![](https://roar.doom.fm/v1/badge.svg?title=crates.io&text=acme-types&font_size=120&text_bg_colour=%23e33b26&text_colour=%23fff)](https://crates.io/crates/acme-types)
[![](https://roar.doom.fm/v1/badge.svg?title=docs.rs&text=latest&font_size=120&text_bg_colour=%23e33b26&text_colour=%23fff)](https://docs.rs/crate/acme-types/latest)

# acme-types-rs

This crate defines types for implementing ACME ([RFC 8555](https://datatracker.ietf.org/doc/html/rfc8555)) providers and clients.

- This crate does not include any HTTP library or provide any functionality (beyond support for de/serialization, as documented below) for interacting with an ACME client or provider, e.g. a server or client
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