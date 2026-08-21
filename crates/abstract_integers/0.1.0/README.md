# Rust abstract integers

This crate defines specification-friendly natural integers with an upper bound. Operations on these integers can be defined as modular (modulo the upper bound) or regular (with a panic on underflow or overflow).

# Defining a new integer type

Here is the macro used to defined the `SizeNat` type of this crate:

```rust
define_abstract_integer_checked!(SizeNat, 8, BigUint::from(std::usize::MAX));
```

`SizeNat` is the name of the newly-created type. `8` is the length in bytes of the byte array that will hold values of this type. The third argument is the upper bound of this integer type, given as a `num::BigUint`. The resulting integer type is copyable, and supports addition, substraction, multiplication, integer division, remainder, comparison and equality. The `from_literal` method allows you to convert integer literals into your new type.
