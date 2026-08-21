# abac-rs

Attribute-Based Access Control (ABAC) evaluation engine for Rust. Define
policies with any number of typed dimensions — principal, resource, action,
tenant, source IP, or whatever your access model requires.

## What it does

- **Arbitrary dimensions** — not limited to a fixed set; define whatever
  policy dimensions you need
- **Typed attributes** — String, Integer, Float, IpAddr, and IpCidr out of
  the box
- **Custom matchers** — plug in your own predicate logic per dimension
- **Policy composition** — combine ABAC with RBAC in four modes (And, Or,
  AbacFirst, RbacFirst) using the
  [`acls-rs`](https://crates.io/crates/acls-rs) permission primitives
- **Performance** — caching and indexing for fast evaluation at scale; see
  the [documentation](https://akamu.dev/bac-rules/) for benchmarks
- No unsafe code

## Quick start

```rust
use abac_rs::{AbacPolicy, AbacRule, AbacRequest, AttributeType};

let mut policy = AbacPolicy::new();

// Engineers can read production resources
let rule = AbacRule::builder("allow-engineers-prod-read")
    .dimension_values("user", vec![
        AttributeType::String("group:engineers".into()),
    ])
    .dimension_values("resource", vec![
        AttributeType::String("prod:db-01".into()),
    ])
    .dimension_values("action", vec![
        AttributeType::String("read".into()),
    ])
    .enabled(true)
    .build();
policy.add_rule(rule).unwrap();

// Evaluate a request
let mut request = AbacRequest::new();
request.add_attribute(
    "user",
    AttributeType::String("alice".into()),
    vec![AttributeType::String("group:engineers".into())],
).unwrap();
request.add_attribute("resource", AttributeType::String("prod:db-01".into()), vec![]).unwrap();
request.add_attribute("action", AttributeType::String("read".into()), vec![]).unwrap();

assert!(policy.evaluate(&request).is_allowed());
```

## Features

- `bloom` *(default)* — Bloom-filter pre-screening for faster rule rejection
- `serde` — serialization support

## Documentation

Architecture, performance benchmarks, and the full API reference:
<https://akamu.dev/bac-rules/>

## License

Licensed under either of Apache License 2.0 or MIT license, at your option.
