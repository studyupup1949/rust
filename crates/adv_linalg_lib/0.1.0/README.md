# Adv_LinAlg_Lib
This library is the core "primative types" used internally within optimizer macros within the `adv_linalg` crate, a automatic optimizing linear algebra library.

Obviously, this library is not intended for the end-user due mostly to the harder developer experience. Nonetheless, the library can be used independently from `adv_linalg` for building **custom runtime-optimizations**.

Information about how `adv_linalg` optimizes, and the overall type-structure are work-in-progresses, but will be available before version 0.2.0 of this crate.

# License
This software is licensed ultimately described under the terms of the SPDX 2.1 License Expression "MIT OR Apache-2.0".

see files *`'LICENSE.md'`*, *`'LICENSE-MIT'`*, and *`'LICENSE-APACHE'`* in the root of this crate for more details

# Usage
You can easily import `adv_linalg_lib` in your `Cargo.toml` file as
```toml
adv_linalg_lib = "0.1"
```

# Overall Type System
This library is split up into a "Vector" group and a "Matrix" group.

Consider the current "Vector" group for instance:

group | type | feature_enabled
---|---|---
Vector | Vector\<T> | "full"
Vector | MutVector\<T> | "full"
Vector | VectorSlice\<'v, T> | "full" or "no_std"
Vector | MutVectorSlice\<'v, T> | "full" or "no_std"

Even though there are 4 unique types, we may want cross-type functionality, like Addition that all go to a similar type (ex. `Vector<T> + MutVectorSlice<'v, T> -> Vector<T>`).

In fact, this is exactly how the crate `simp_linalg` works, but lacking mutability. `simp_linalg` lacks is a clean way to implement mutability, simd utilization, gpu accelleration, etc. The ambition of `adv_linalg` is to be a massive superset of `simp_linalg`, but keeping the simplicity in its usage in syntax.

## Why is it structured this way?

To understand why, first understand some design decisions were made early on:
1. `adv_linalg` will depend only on itself and `adv_linalg_lib`
2. `adv_linalg` will never require importing traits for functionality

*Other design decisions were made, but they pertain to `#![no_std]` functionality.*

\#1 forces a single source of truth to be only this library, making this the "core" of `adv_linalg`. This incentivises dense and low-level functionality. This usually accomplished by to defining some trait and importing them when you want to use some functionality.

However, #2 disallows this. This seems counter-intuitive, but is a driving force to not let the inner type system get out-of-control, and won't affect the end-user experience, only the developer side. #2 is subject to change before version 1.0. This involves a lot of manual implementations. However, this crate leverages procedural macros to massively reduce complexity when adding new types.

# #![no_std] Compliance
#### Heads up!
If are simply using the Rust standard library `std`, then you can safely ignore this section.
You can easily import `adv_linalg_lib` in your `Cargo.toml` file as
```toml
adv_linalg_lib = "0.1"
```

If you know you need a headless environment (using only the `core` crate, no `alloc`), then import `adv_linalg_lib` in your Cargo.toml file as
```toml
adv_linalg_lib = { version = "0.1", features=["no_std"] }
```
Continue reading for more details on what effects this has...

## Feature List

This library is fully implemented using the `#![no_std]`, with the **optional** exception of the `alloc` crate.

The two features of this crate are:
1. `no_std`
2. `full` *<-- default*

### 1. `no_std`
When `no_std` is enabled, this implements `adv_linalg_lib` using only the Rust `core` crate. Importantly, this does utilize the Rust `alloc` crate, which is useful for heap-less programs. In fact, the `core` crate will always be guaranteed to be used exclusively within the source code of this crate.

HOWEVER: Dependencies of this library do not hold this guarantee. The consequence of this is that developers of headless programs must compile on a non-headless machine and then import the source code as needed.

### 2. `full`
When `full` is enabled, this implements `adv_linalg_lin` using the Rust `core` and `alloc` crates. Specifically, the only object ever imported from `alloc` is `alloc::vec::Vec`. This one change results in a massive expansion of the type system.

By default, `full` is enabled with the intention of broader compatibility and optimizations for end-users using the Rust `std` crate, which are likely to be the majority of users.

With the intention of procedural optimizations in the `adv_linalg` crate, any unnecessary overheads introduced by `alloc::vec::Vec` can be optimized away. This is why direct usage of `adv_linalg_lib` is generally discouraged.

However, a good example of when to use this library is for testing new optimization patterns or implementing protocols with how memory shall be handled such as for gpu buffers.

### Consequences of using feature `no_std`
By switching to using `no-std`, `adv_linalg_lib` retains nearly all functionality but reduces the overall type system. Specifically, the only functionality removed is dynamic sizing of `adv_linalg` types at runtime. If disallowing dynamic sizing is a requirement, then you are encouraged to use the `no_std` feature. To do so, import this line under your `[dependencies]` in your `Cargo.toml` file:
```toml
[dependencies]
...
adv_linalg_lib = { version="*", features=["no_std"] }
```
