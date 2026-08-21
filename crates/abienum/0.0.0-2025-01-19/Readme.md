# abienum - underlying types for C enums

[![GitHub](https://img.shields.io/github/stars/MaulingMonkey/abienum.svg?label=GitHub&style=social)](https://github.com/MaulingMonkey/abienum)
[![crates.io](https://img.shields.io/crates/v/abienum.svg)](https://crates.io/crates/abienum)
[![docs.rs](https://docs.rs/abienum/badge.svg)](https://docs.rs/abienum)
[![License](https://img.shields.io/crates/l/abienum.svg)](https://github.com/MaulingMonkey/abienum)
[![Build Status](https://github.com/MaulingMonkey/abienum/workflows/Rust/badge.svg)](https://github.com/MaulingMonkey/abienum/actions?query=workflow%3Arust)
<!-- [![dependency status](https://deps.rs/repo/github/MaulingMonkey/abienum/status.svg)](https://deps.rs/repo/github/MaulingMonkey/abienum) -->

Attempts to define the implicit underlying types of C and C++ enums, when compiled via the [`cc`] crate using default settings.
That is, enums that follow any of these styles:

```cpp
enum Test { Hello = 1 };
typedef enum { Hello = 1 } Test;
typedef enum Test { Hello = 1 } Test;
typedef enum _Test { Hello = 1 } Test;
```

Would presumably be FFI-compatible with the following Rust type:

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] pub struct Test(abienum::c_enum_u7);
impl Test { pub const Hello : Test = Test(1 as _); }
```

You are expected to use the `c_enum_*` with the smallest compatible range &mdash; e.g. `c_enum_u7` over `c_enum_u8` or `c_enum_i8`.



## Caveats

The underlying type of enums is implementation specific, to the point of being potentially finicky in practice.

1.  Compilers may disagree on enum size.
    For example, [ARM delegates type selection to the platform ABI](https://github.com/ARM-software/abi-aa/blob/05abf4f78dd7837774c4880fc0e6c01ce9e41ba8/aapcs32/aapcs32.rst#enumerated-types),
    which leads to [Clang and GCC disagreeing on enum size](https://godbolt.org/z/MajYq4o68) for unknown/none, where no platform ABI is specified or agreed upon.
    If using the [`cc`] crate, prefer a global [`CC=...`] over [`.compiler("...")`](https://docs.rs/cc/1/cc/struct.Build.html#method.compiler).
    If linking against prebuilt libraries, specify the same compiler as was used for said libraries via [`CC=...`]

2.  Compilers provide flags controlling enum size, such as `-f[no-]short-enums`
    <sup>\[[clang](https://clang.llvm.org/docs/ClangCommandLineReference.html#cmdoption-clang-fshort-enums),
    [gcc](https://gcc.gnu.org/onlinedocs/gcc-4.9.2/gcc/Structures-unions-enumerations-and-bit-fields-implementation.html)\]</sup>.
    If you need these, prefer a global [`CFLAGS=...`] over [`.flag("...")`](https://docs.rs/cc/1/cc/struct.Build.html#method.flag).

3.  Compilers may provide extensions controlling enum size, such as `__attribute__((packed))` or `#pragma`s.
    You're completely on your own for those.
    I recommend a lot of `static_assert`s on the C++ side and `const _ : () = assert!(...);`s on the Rust side.
    Good luck.

4.  Compilers may or may not actually respect the flags, attributes, and pragmas specified.
    E.g. LLVM seems to ignore them on Win32 despite parsing them ([llvm/llvm-project #70607](https://github.com/llvm/llvm-project/issues/70607))

5.  Type selection beyond the basics (e.g. involving potentially typed expressions that depend on previous enumerands) is enough of a potential mess that I haven't tackled it.
    See also these C23 proposals:
    *   [N3029 Improved Normal Enumerations](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3029.htm),
        [ThePHD's Coverage](https://thephd.dev/c23-is-coming-here-is-what-is-on-the-menu#n3029---improved-normal-enumerations)
    *   [N3030 Enhancements to Enumerations](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3030.htm),
        [ThePHD's Coverage](https://thephd.dev/c23-is-coming-here-is-what-is-on-the-menu#n3030---enhanced-enumerations)



## Alternatives

C++11 (and perhaps C23?) provides syntax to explicitly control the underlying type of enums, which &mdash; assuming
you can sanely modify the C/C++ &mdash; I strongly recommend over resorting to this crate's nonsense:

```cpp
enum Test : int { Hello = 1 };
```
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] pub struct Test(core::ffi::c_int);
impl Test { pub const Hello : Test = Test(1 as _); }
```

If you're stuck in earlier versions of C or C++, the old "force dword" trick will at least get the size right (signedness may still vary):
```cpp
typedef enum _D3DLIGHTTYPE {
    D3DLIGHT_POINT          = 1,
    D3DLIGHT_SPOT           = 2,
    D3DLIGHT_DIRECTIONAL    = 3,
    D3DLIGHT_FORCE_DWORD    = 0x7fffffff, /* force 32-bit size enum */
} D3DLIGHTTYPE;
```
```rust
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] pub struct D3DLIGHTTYPE(i32);
pub const D3DLIGHT_POINT        : D3DLIGHTTYPE = D3DLIGHTTYPE(1);
pub const D3DLIGHT_SPOT         : D3DLIGHTTYPE = D3DLIGHTTYPE(2);
pub const D3DLIGHT_DIRECTIONAL  : D3DLIGHTTYPE = D3DLIGHTTYPE(3);
//  const D3DLIGHT_FORCE_DWORD  : D3DLIGHTTYPE = D3DLIGHTTYPE(0x7fffffff); // impl detail
```



<h2 name="license">License</h2>

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.



<h2 name="contribution">Contribution</h2>

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.



<!-- references -->

[`cc`]:             https://docs.rs/cc/
[`CC=...`]:         https://docs.rs/cc/1/cc/index.html#external-configuration-via-environment-variables
[`CFLAGS=...`]:     https://docs.rs/cc/1/cc/index.html#external-configuration-via-environment-variables
