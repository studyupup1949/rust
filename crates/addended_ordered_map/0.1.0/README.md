# addended_ordered_map

![Crates.io MSRV](https://img.shields.io/crates/msrv/addended_ordered_map)
![Crates.io Version](https://img.shields.io/crates/v/addended_ordered_map)
![Crates.io License](https://img.shields.io/crates/l/addended_ordered_map)

An ordered mapping with addends for each key.

This crate provides the `AddendedOrderedMap` type, a `no_std` ordered mapping,
that allows ranged lookups for keys.
Each range is defined by the key of a pairing as the start of the range, and a
size obtained from the value of the pairing used to compute where the range ends.

For example, if a given key is `4` and the size stored in the value is `6`, then
the range for this key is `[4, 10)` (inclusive start, exclusive end). Any lookup
for the keys `4`, `5`, `6`, `7`, `8` or `9` will return this pair.

The size component of the range is obtained from the value instead of being
stored as part of the key, meaning the range size can be mutated after the pair
has been inserted into the mapping. The range doesn't need to be final at
construction time.

There's also a `AddendedOrderedMapFallible` type, that allows fallible size and
key-addition operations.

If you are interested in crates that use a range as a key then checkout
[`rangemap`](https://crates.io/crates/rangemap) or
[`nodit`](https://crates.io/crates/nodit).

## Examples

```rust
use addended_ordered_map::{AddendedOrderedMap, FindSettings};

let settings = FindSettings::new(true);
let mut map = AddendedOrderedMap::new();

// Insert a key with size 4.
// In other words, contains the range `[0x1000, 0x1004)` (closed, open).
// For plain types like integers their own value is considered the size too,
// more complex types need to implement the `SizedValue` trait.
let (value, newly_inserted) = map.find_mut_or_insert_with(0x1000, settings, || 4);
assert_eq!(value, &4);
assert_eq!(newly_inserted, true);

// This does not insert the key 0x1001 because it overlaps with the range
// of the previous key.
let (value, newly_inserted) = map.find_mut_or_insert_with(0x1001, settings, || 4);
assert_eq!(value, &4);
assert_eq!(newly_inserted, false);

assert_eq!(map.len(), 1);

assert_eq!(
    map.find(&0x1000, settings),
    Some((&0x1000, &4)),
);

// Ranged lookups work.
assert_eq!(
    map.find(&0x1002, settings),
    Some((&0x1000, &4)),
);
// We can also do exact key lookups if wanted.
assert_eq!(
    map.find(&0x1002, FindSettings::new(false)),
    None,
);

// This is outside the range of the only key.
assert_eq!(
    map.find(&0x1004, settings),
    None,
);
```

Custom types.

```rust
use addended_ordered_map::{AddendedOrderedMap, AddendableKey, FindSettings, SizedValue};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TestKey(u32);
#[derive(Debug, PartialEq)]
struct TestValue(&'static str, u32);

impl AddendableKey<u32> for TestKey {
    fn add_size(&self, size: &u32) -> Self {
        Self(self.0 + *size)
    }
}
impl SizedValue<u32> for TestValue {
    fn size(&self) -> u32 {
        self.1
    }
}

let mut map: AddendedOrderedMap<TestKey, TestValue, u32> = AddendedOrderedMap::new();

let settings = FindSettings::new(true);

map.find_mut_or_insert_with(TestKey(0x1000), settings, || TestValue("value1", 8));
map.find_mut_or_insert_with(TestKey(0x1007), settings, || TestValue("value1", 8));

assert_eq!(
    map.find(&TestKey(0x1000), settings),
    Some((&TestKey(0x1000), &TestValue("value1", 8))),
);

assert_eq!(
    map.find(&TestKey(0x1004), settings),
    Some((&TestKey(0x1000), &TestValue("value1", 8))),
);

assert_eq!(
    map.find(&TestKey(0x1004), FindSettings::new(false)),
    None,
);
```

## Crate features

This crate provides the following features. Currently none of them are enabled
by default.

- `std`: Turns on `std` (or turn off `no_std`).
  - This currently doesn't do anything codewise.
- `extract_if`: Exposes the `extract_if` method.
  - Bumps the MSRV to 1.91.
- `nightly`: Unstable stuff depending on a nightly Rust compiler.
  - Highly discouraged for common use.

## Minimum Supported Rust Version (MSRV)

The current version of rabbitizer requires **Rust 1.56.0 or greater**.

The current policy is MSRV changes may happen in minor updates.

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

## Versioning and changelog

This library follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
We try to always keep backwards compatibility, so no breaking changes should
happen until a major release (i.e. jumping from 1.X.X to 2.0.0).

To see what changed on each release visit either the
[CHANGELOG.md](https://github.com/Decompollaborate/addended_ordered_map/blob/main/CHANGELOG.md)
file or check the [releases page on Github](https://github.com/Decompollaborate/addended_ordered_map/releases).
You can also use [this link](https://github.com/Decompollaborate/addended_ordered_map/releases/latest)
to check the latest release.
