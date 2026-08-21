# Ad-hoc owning iterator types

This crate defines the macro `iter!` which produces ad-hoc iterator types that own their values and have compile-time known exact sizes.

# Usage
This macro can be used exactly like `vec!`, except it produces an `impl Iterator` that does not allocate.
This `impl Iterator` yields the values directly, **not references** to the values.

This can be useful for when you want a ad-hoc iterator of a non-copy type, as sized slices and arrays currently do not implement `IntoIterator` in a way that moves their values, instead they yield references, causing the need for cloning.

The `iter!` macro's iterator types move their values on calls to `next()` instead of returning references, and drop the non-consumed values when the iterator is dropped itself.

## Example

Concatenating a 'slice' of `String` without cloning.
``` rust
let whole: String = iter![String::from("Hell"),
			  String::from("o "),
			  String::from("world"),
			  String::from("!")]
    .collect();
assert_eq!("Hello world!", &whole[..]);
```

## Functions
The iterator types also have a few associated functions.

### The length of the whole iterator
```rust
pub const fn len(&self) -> usize
```

### The rest of the iterator that has not been consumed.
```rust
pub fn rest(&self) -> &[T]
```

###  The whole array.
All values that have not been consumed are initialised, values that have been consumed are uninitialised.
```rust
pub fn array(&self) -> &[MaybeUninit<T>; Self::LEN]
```

### How many items have since been consumed.
```rust
pub const fn consumed(&self) -> usize
```
# License
MIT
