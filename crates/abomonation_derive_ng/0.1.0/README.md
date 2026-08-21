# abomonation_derive_ng

Derive macro implementation for the Abomonation crate. Forked & improved from [the original `abomonation_derive` crate from @mystor](https://github.com/mystor/abomonation_derive).

This crate provides a custom derive macro `#[derive(Abomonation)]` for types that should be serializable with the [abomonation crate](https://github.com/frankmcsherry/abomonation).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
abomonation_derive_ng = "0.1"
abomonation = "0.5"
```

Then, derive `Abomonation` on your types:

```rust
#[macro_use]
extern crate abomonation_derive_ng;
extern crate abomonation;
use abomonation::Abomonation;

#[derive(Abomonation, PartialEq, Debug)]
struct MyStruct {
    a: u64,
    b: String,
    c: Vec<u8>,
}

// Now you can use `MyStruct` with `abomonation`'s `encode` and `decode`.
```

## Attributes

- `#[abomonation_omit_bounds]` : Omits automatic trait bound generation for the Abomonation trait.
- `#[abomonation_bounds(Trait1, Trait2)]` : Adds custom trait bounds.
- `#[abomonate_with = "path::to::function"]` : Specifies a custom method to use for serialization.
- `#[abomonation_skip]` : Skips the field when serializing.

## Limitations

The crate does not currently support union types.
The use of `unsafe` is necessary due to the nature of `abomonation`'s serialization technique.

## Safety
This crate relies on the same unsafe serialization strategy as `abomonation`, and the same caveats apply. It is not suitable for untrusted inputs and requires care when used.
For detailed information on the safety concerns and how to use this crate, consult the abomonation documentation.

## License

This project is licensed under the MIT License, same  as the original [`abomonation-derive`](https://github.com/mystor/abomonation_derive/blob/e0980ee7f4fb7c4ccadd85dfc770273a9068aef0/Cargo.toml#L6) crate.

