# Simplee > Aabel > Identifier

[![Crates.io][crates-badge]][crates-url]
[![CI][ci-badge]][ci-url]
![GitHub top language][lang-badge]
[![License:MIT][license-badge]][license-url]
![GitHub code size in bytes][size-badge]
![GitHub last commit][last-commit-badge]
![GitHub watchers][watchers-badge]


## Description
A Rust crate that defines some behavior related to identifiers, such:
- any type which implements Eq will be an Identifier.
- any type which implements PartialOrd will be a PartialOrdIdentifier.
- capabilities to generate sequences of identifiers.

## Examples

Any type which implements the [`Eq`] trait will automatically be an identifier.

```rust
use aabel_identifier_rs::*;

fn test_identifier(_id: impl Identifier) {
    assert!(true);
}

let id = 10_u8;
test_identifier(id);
```

You can create an iterator which generates new idenfiers, given a starting point and a function which
compute a new identifier value from a previous one.

```rust
use aabel_identifier_rs::*;

let id = 10_u8;
let mut iter = id.into_ids_iterator(|id| id + 1);

assert_eq!(iter.next(), Some(10));
assert_eq!(iter.next(), Some(11));
```

## About
> Code designed and written on the beautiful island of [**Saaremaa**](https://goo.gl/maps/DmB9ewY2R3sPGFnTA), Estonia.


[crates-badge]: https://img.shields.io/crates/v/aabel-identifier-rs.svg
[crates-url]: https://crates.io/crates/aabel-identifier-rs
[ci-badge]: https://github.com/veminovici/aabel-identifier-rs/actions/workflows/ci.yml/badge.svg?branch=main
[ci-url]: https://github.com/veminovici/aabel-identifier-rs/actions/workflows/ci.yml
[lang-badge]: https://img.shields.io/github/languages/top/veminovici/aabel-identifier-rs
[license-badge]: https://img.shields.io/badge/License-MIT-yellow.svg
[license-url]: https://opensource.org/licenses/MIT
[size-badge]: https://img.shields.io/github/languages/code-size/veminovici/aabel-identifier-rs
[last-commit-badge]: https://img.shields.io/github/last-commit/veminovici/aabel-identifier-rs
[watchers-badge]: https://img.shields.io/github/watchers/veminovici/aabel-identifier-rs