# Adv_LinAlg_Lib

⚠️ This library is currently under construction! Design is subject to change! ⚠️

The backbone of the linear algebra framework `adv_linalg`.

## Usage
#### Cargo.toml
```
adv_linalg_lib = "0.1"
```

# #![no_std] Compliance Feature
If you need a version of this library that uses only the `core` crate, simply import this library as such:

#### Cargo.toml

```toml
adv_linalg_lib = { version = "0.1", features=["no_std"] }
```

Do note that this drastically simplifies the library structure and that the dependencies of this crate do not neccessarily use `#![no_std]`.

# Basic Types

Simply, this is a linear algebra library. It features two main types:
- Vector\<T>
- Matrix\<T>

These are simple wrappers around the standard library's `Vec<T>` type.

Most developers will often only need these two types.

However, for users requiring an optimized run-time performance, this library features two options:
- use the framework `adv_linalg` alongside this library for automatic compile-time optimizations **(recommended)**
- manually use **advanced types**

*In fact,* `adv_linalg` *simply uses `adv_linalg_lib` as as a "ghost" type-system*.

To learn more, read about [advanced types](#advanced-types)

# Advanced Types

For developers just wanting a simple tool, just know that this section is optional. 

Every type in this crate is either a "Vector" or a "Matrix". All types include this base in their name.

The type's functionality is described by it's both optional:
- [prefix](#prefix) (description of mutability)
- and/or [suffix](#suffix) (special functionality).

## Prefix

There is only one prefix: Mut.

### Mut
This states explicitly to allow interior mutability.

<details>
    <summary>Click here to learn about "interior mutability"</summary>

To learn about interior mutability, first understand "**interior immutability**".

Interior immutability means that the interior of the type is unchanging. This forces the following rule: if the data mutated, then it is a different vector.

In other words, to change the data, an allocation is needed.

Below is example code for interior mutability from regular mutability:
```rust
fn main() {
    // MUTABILITY TYPES

    // initial value
    let mut std_vec = vec![1, 2, 3];
    
    // Example of mutability that follows `interior 
    // immutability`.
    // We essentially "overwrite" the variable.
    std_vec = std_vec.iter().map(|val| val + 1).collect();

    // Example of only 'interior mutability'.
    // Imagine the next three lines as a single operation.
    // We are reusing the already allocated memory.
    {
        std_vec[0] = std_vec[0] + 1;
        std_vec[1] = std_vec[1] + 1;
        std_vec[2] = std_vec[2] + 1;
    }

    // Exterior mutability "overwrote the varaiable", utilizing an entire new heap allocation in the process.
    // Interior mutability "overwrote the memory", reusuing the already allocated memory.
}

```

By being selective when to use interior mutability or not can be useful to reducing time spent allocating memory.

</details>

## Suffix

There are currently three suffixes:
1. Slice
3. Simd (planned/nightly)
2. Gpu (planned)

### Slice
A subsection view of an existing vector. This is simply a slice of a Vec under-the-hood.

#### Example
```rust
use adv_linalg_lib::vector;
use adv_linalg_lib::vectors::{Vector, VectorSlice};

fn main() {
    let vector: Vector<u32> = vector![1, 2, 3];

    let vector_slice: VectorSlice<'_, u32> = vector.as_slice(1..vector.len());

    assert_eq!(
        vector_slice.to_vector(), vector![2, 3]
    )
}
```

### Simd
⚠️Design is still under-construction and is nightly.⚠️

This feature is still in the design process. The produced design will use `core::simd::Simd`. Therefore, when the design is implemented, this require to build with `nightly` until Rust stabilizes `core::simd::Simd`.

### Gpu
⚠️Design is still under-construction.⚠️

This feature is still in the design process. The produced design will support OpenCL.

## Some Advanced Type Examples:
- MutVector
- MatrixSlice
- MutVectorSimd

# License
This software is licensed ultimately described under the terms of the SPDX 2.1 License Expression "MIT OR Apache-2.0".

see files *`'LICENSE.md'`*, *`'LICENSE-MIT'`*, and *`'LICENSE-APACHE'`* in the root of this crate for more details