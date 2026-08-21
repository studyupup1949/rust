#![doc = include_str!("../Readme.md")]
#![no_std]
#![allow(non_camel_case_types)] // c_enum_u7 etc.
#![allow(dead_code)] // 128-bit nonsense

#[allow(unused_imports)] use core::ffi::*;



/// `enum { Min = -128, Max = 127 }`'s underlying type.<br>
/// Frequently a larger type than [`i8`], such as [`c_int`] ≈ [`i32`].<br>
/// <br>
pub use imp::c_enum_i8;

/// `enum { Min = -32768, Max = 32767 }`'s underlying type.<br>
/// Frequently a larger type than [`i16`], such as [`c_int`] ≈ [`i32`].<br>
/// <br>
pub use imp::c_enum_i16;

/// `enum { Min = -2147483648, Max = 2147483647 }`'s underlying type.<br>
/// Typically [`c_int`].<br>
/// <br>
pub use imp::c_enum_i32;

/// <code>enum { Min = -2<sup>63</sup>, Max = 2<sup>63</sup>-1 }</code>'s underlying type.<br>
/// Frequently [`c_int`], even when that loses information truncating to 32 bits!<br>
/// <br>
pub use imp::c_enum_i64;

#[cfg(nope)] // MSVC/Windows fails to compile 128-bit stuff.  Clang/linux truncates 128-bit stuff to 64-bit.  Just don't.
/// <code>enum { Min = -2<sup>127</sup>, Max = 2<sup>127</sup>-1 }</code>'s underlying type.<br>
/// Frequently unavailable, in which case this will be [`never`] or some equivalent.<br>
/// <br>
pub use imp::c_enum_i128;



/// `enum { Min = 0, Max = 255 }`'s underlying type.<br>
/// Frequently a larger type than [`u8`], such as a 32-bit [`c_int`] or [`c_uint`].<br>
/// <br>
pub use imp::c_enum_u8;

/// `enum { Min = 0, Max = 65535 }`'s underlying type.<br>
/// Frequently a larger type than [`u16`], such as a 32-bit [`c_int`] or [`c_uint`].<br>
/// <br>
pub use imp::c_enum_u16;

/// `enum { Min = 0, Max = 4294967295 }`'s underlying type.<br>
/// Frequently [`c_int`] (Windows), even when that truncates 4294967295 to -1.<br>
/// <br>
pub use imp::c_enum_u32;

/// <code>enum { Min = 0, Max = 2<sup>64</sup>-1 }</code>'s underlying type.<br>
/// Frequently [`c_int`], even when that loses information truncating to 32 bits!<br>
/// <br>
pub use imp::c_enum_u64;

#[cfg(nope)] // MSVC/Windows fails to compile 128-bit stuff.  Clang/linux truncates 128-bit stuff to 64-bit.  Just don't.
/// <code>enum { Min = 0, Max = 2<sup>128</sup>-1 }</code>'s underlying type.<br>
/// Frequently unavailable, in which case this will be [`never`] or some equivalent.<br>
/// <br>
pub use imp::c_enum_u128;



/// `enum { Min = 0, Max = 127 }`'s underlying type.<br>
/// Frequently a larger type than [`i8`], such as a 32-bit [`c_int`] or [`c_uint`].<br>
/// <br>
pub use imp::c_enum_u7;

/// `enum { Min = 0, Max = 32767 }`'s underlying type.<br>
/// Frequently a larger type than [`i16`], such as a 32-bit [`c_int`] or [`c_uint`].<br>
/// <br>
pub use imp::c_enum_u15;

/// `enum { Min = 0, Max = 2147483647 }`'s underlying type.<br>
/// Typically [`c_int`] (Windows) or [`c_uint`] (\*nix).<br>
/// <br>
pub use imp::c_enum_u31;

/// <code>enum { Min = 0, Max = 2<sup>63</sup>-1 }</code>'s underlying type.<br>
/// Frequently [`c_int`], even when that loses information truncating to 32 bits!<br>
/// <br>
pub use imp::c_enum_u63;

#[cfg(nope)] // MSVC/Windows fails to compile 128-bit stuff.  Clang/linux truncates 128-bit stuff to 64-bit.  Just don't.
/// <code>enum { Min = 0, Max = 2<sup>127</sup>-1 }</code>'s underlying type.<br>
/// Frequently unavailable, in which case this will be [`never`] or some equivalent.<br>
/// <br>
pub use imp::c_enum_u127;



#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(hidden)] pub enum _never {}

mod imp {
    use super::_never as never;

    #[cfg(c_enum_i8  ="char")] pub type c_enum_i8    = core::ffi::c_char;
    #[cfg(c_enum_i16 ="char")] pub type c_enum_i16   = core::ffi::c_char;
    #[cfg(c_enum_i32 ="char")] pub type c_enum_i32   = core::ffi::c_char;
    #[cfg(c_enum_i64 ="char")] pub type c_enum_i64   = core::ffi::c_char;
    #[cfg(c_enum_i128="char")] pub type c_enum_i128  = core::ffi::c_char;
    #[cfg(c_enum_u7  ="char")] pub type c_enum_u7    = core::ffi::c_char;
    #[cfg(c_enum_u8  ="char")] pub type c_enum_u8    = core::ffi::c_char;
    #[cfg(c_enum_u15 ="char")] pub type c_enum_u15   = core::ffi::c_char;
    #[cfg(c_enum_u16 ="char")] pub type c_enum_u16   = core::ffi::c_char;
    #[cfg(c_enum_u31 ="char")] pub type c_enum_u31   = core::ffi::c_char;
    #[cfg(c_enum_u32 ="char")] pub type c_enum_u32   = core::ffi::c_char;
    #[cfg(c_enum_u63 ="char")] pub type c_enum_u63   = core::ffi::c_char;
    #[cfg(c_enum_u64 ="char")] pub type c_enum_u64   = core::ffi::c_char;
    #[cfg(c_enum_u127="char")] pub type c_enum_u127  = core::ffi::c_char;
    #[cfg(c_enum_u128="char")] pub type c_enum_u128  = core::ffi::c_char;

    #[cfg(c_enum_i8  ="signed char")] pub type c_enum_i8    = core::ffi::c_schar;
    #[cfg(c_enum_i16 ="signed char")] pub type c_enum_i16   = core::ffi::c_schar;
    #[cfg(c_enum_i32 ="signed char")] pub type c_enum_i32   = core::ffi::c_schar;
    #[cfg(c_enum_i64 ="signed char")] pub type c_enum_i64   = core::ffi::c_schar;
    #[cfg(c_enum_i128="signed char")] pub type c_enum_i128  = core::ffi::c_schar;
    #[cfg(c_enum_u7  ="signed char")] pub type c_enum_u7    = core::ffi::c_schar;
    #[cfg(c_enum_u8  ="signed char")] pub type c_enum_u8    = core::ffi::c_schar;
    #[cfg(c_enum_u15 ="signed char")] pub type c_enum_u15   = core::ffi::c_schar;
    #[cfg(c_enum_u16 ="signed char")] pub type c_enum_u16   = core::ffi::c_schar;
    #[cfg(c_enum_u31 ="signed char")] pub type c_enum_u31   = core::ffi::c_schar;
    #[cfg(c_enum_u32 ="signed char")] pub type c_enum_u32   = core::ffi::c_schar;
    #[cfg(c_enum_u63 ="signed char")] pub type c_enum_u63   = core::ffi::c_schar;
    #[cfg(c_enum_u64 ="signed char")] pub type c_enum_u64   = core::ffi::c_schar;
    #[cfg(c_enum_u127="signed char")] pub type c_enum_u127  = core::ffi::c_schar;
    #[cfg(c_enum_u128="signed char")] pub type c_enum_u128  = core::ffi::c_schar;

    #[cfg(c_enum_i8  ="unsigned char")] pub type c_enum_i8    = core::ffi::c_uchar;
    #[cfg(c_enum_i16 ="unsigned char")] pub type c_enum_i16   = core::ffi::c_uchar;
    #[cfg(c_enum_i32 ="unsigned char")] pub type c_enum_i32   = core::ffi::c_uchar;
    #[cfg(c_enum_i64 ="unsigned char")] pub type c_enum_i64   = core::ffi::c_uchar;
    #[cfg(c_enum_i128="unsigned char")] pub type c_enum_i128  = core::ffi::c_uchar;
    #[cfg(c_enum_u7  ="unsigned char")] pub type c_enum_u7    = core::ffi::c_uchar;
    #[cfg(c_enum_u8  ="unsigned char")] pub type c_enum_u8    = core::ffi::c_uchar;
    #[cfg(c_enum_u15 ="unsigned char")] pub type c_enum_u15   = core::ffi::c_uchar;
    #[cfg(c_enum_u16 ="unsigned char")] pub type c_enum_u16   = core::ffi::c_uchar;
    #[cfg(c_enum_u31 ="unsigned char")] pub type c_enum_u31   = core::ffi::c_uchar;
    #[cfg(c_enum_u32 ="unsigned char")] pub type c_enum_u32   = core::ffi::c_uchar;
    #[cfg(c_enum_u63 ="unsigned char")] pub type c_enum_u63   = core::ffi::c_uchar;
    #[cfg(c_enum_u64 ="unsigned char")] pub type c_enum_u64   = core::ffi::c_uchar;
    #[cfg(c_enum_u127="unsigned char")] pub type c_enum_u127  = core::ffi::c_uchar;
    #[cfg(c_enum_u128="unsigned char")] pub type c_enum_u128  = core::ffi::c_uchar;

    #[cfg(c_enum_i8  ="unknown")] pub type c_enum_i8    = never;
    #[cfg(c_enum_i16 ="unknown")] pub type c_enum_i16   = never;
    #[cfg(c_enum_i32 ="unknown")] pub type c_enum_i32   = never;
    #[cfg(c_enum_i64 ="unknown")] pub type c_enum_i64   = never;
    #[cfg(c_enum_i128="unknown")] pub type c_enum_i128  = never;
    #[cfg(c_enum_u7  ="unknown")] pub type c_enum_u7    = never;
    #[cfg(c_enum_u8  ="unknown")] pub type c_enum_u8    = never;
    #[cfg(c_enum_u15 ="unknown")] pub type c_enum_u15   = never;
    #[cfg(c_enum_u16 ="unknown")] pub type c_enum_u16   = never;
    #[cfg(c_enum_u31 ="unknown")] pub type c_enum_u31   = never;
    #[cfg(c_enum_u32 ="unknown")] pub type c_enum_u32   = never;
    #[cfg(c_enum_u63 ="unknown")] pub type c_enum_u63   = never;
    #[cfg(c_enum_u64 ="unknown")] pub type c_enum_u64   = never;
    #[cfg(c_enum_u127="unknown")] pub type c_enum_u127  = never;
    #[cfg(c_enum_u128="unknown")] pub type c_enum_u128  = never;

    #[cfg(c_enum_i8  ="short")] pub type c_enum_i8    = core::ffi::c_short;
    #[cfg(c_enum_i16 ="short")] pub type c_enum_i16   = core::ffi::c_short;
    #[cfg(c_enum_i32 ="short")] pub type c_enum_i32   = core::ffi::c_short;
    #[cfg(c_enum_i64 ="short")] pub type c_enum_i64   = core::ffi::c_short;
    #[cfg(c_enum_i128="short")] pub type c_enum_i128  = core::ffi::c_short;
    #[cfg(c_enum_u7  ="short")] pub type c_enum_u7    = core::ffi::c_short;
    #[cfg(c_enum_u8  ="short")] pub type c_enum_u8    = core::ffi::c_short;
    #[cfg(c_enum_u15 ="short")] pub type c_enum_u15   = core::ffi::c_short;
    #[cfg(c_enum_u16 ="short")] pub type c_enum_u16   = core::ffi::c_short;
    #[cfg(c_enum_u31 ="short")] pub type c_enum_u31   = core::ffi::c_short;
    #[cfg(c_enum_u32 ="short")] pub type c_enum_u32   = core::ffi::c_short;
    #[cfg(c_enum_u63 ="short")] pub type c_enum_u63   = core::ffi::c_short;
    #[cfg(c_enum_u64 ="short")] pub type c_enum_u64   = core::ffi::c_short;
    #[cfg(c_enum_u127="short")] pub type c_enum_u127  = core::ffi::c_short;
    #[cfg(c_enum_u128="short")] pub type c_enum_u128  = core::ffi::c_short;

    #[cfg(c_enum_i8  ="unsigned short")] pub type c_enum_i8    = core::ffi::c_ushort;
    #[cfg(c_enum_i16 ="unsigned short")] pub type c_enum_i16   = core::ffi::c_ushort;
    #[cfg(c_enum_i32 ="unsigned short")] pub type c_enum_i32   = core::ffi::c_ushort;
    #[cfg(c_enum_i64 ="unsigned short")] pub type c_enum_i64   = core::ffi::c_ushort;
    #[cfg(c_enum_i128="unsigned short")] pub type c_enum_i128  = core::ffi::c_ushort;
    #[cfg(c_enum_u7  ="unsigned short")] pub type c_enum_u7    = core::ffi::c_ushort;
    #[cfg(c_enum_u8  ="unsigned short")] pub type c_enum_u8    = core::ffi::c_ushort;
    #[cfg(c_enum_u15 ="unsigned short")] pub type c_enum_u15   = core::ffi::c_ushort;
    #[cfg(c_enum_u16 ="unsigned short")] pub type c_enum_u16   = core::ffi::c_ushort;
    #[cfg(c_enum_u31 ="unsigned short")] pub type c_enum_u31   = core::ffi::c_ushort;
    #[cfg(c_enum_u32 ="unsigned short")] pub type c_enum_u32   = core::ffi::c_ushort;
    #[cfg(c_enum_u63 ="unsigned short")] pub type c_enum_u63   = core::ffi::c_ushort;
    #[cfg(c_enum_u64 ="unsigned short")] pub type c_enum_u64   = core::ffi::c_ushort;
    #[cfg(c_enum_u127="unsigned short")] pub type c_enum_u127  = core::ffi::c_ushort;
    #[cfg(c_enum_u128="unsigned short")] pub type c_enum_u128  = core::ffi::c_ushort;

    #[cfg(c_enum_i8  ="int")] pub type c_enum_i8    = core::ffi::c_int;
    #[cfg(c_enum_i16 ="int")] pub type c_enum_i16   = core::ffi::c_int;
    #[cfg(c_enum_i32 ="int")] pub type c_enum_i32   = core::ffi::c_int;
    #[cfg(c_enum_i64 ="int")] pub type c_enum_i64   = core::ffi::c_int;
    #[cfg(c_enum_i128="int")] pub type c_enum_i128  = core::ffi::c_int;
    #[cfg(c_enum_u7  ="int")] pub type c_enum_u7    = core::ffi::c_int;
    #[cfg(c_enum_u8  ="int")] pub type c_enum_u8    = core::ffi::c_int;
    #[cfg(c_enum_u15 ="int")] pub type c_enum_u15   = core::ffi::c_int;
    #[cfg(c_enum_u16 ="int")] pub type c_enum_u16   = core::ffi::c_int;
    #[cfg(c_enum_u31 ="int")] pub type c_enum_u31   = core::ffi::c_int;
    #[cfg(c_enum_u32 ="int")] pub type c_enum_u32   = core::ffi::c_int;
    #[cfg(c_enum_u63 ="int")] pub type c_enum_u63   = core::ffi::c_int;
    #[cfg(c_enum_u64 ="int")] pub type c_enum_u64   = core::ffi::c_int;
    #[cfg(c_enum_u127="int")] pub type c_enum_u127  = core::ffi::c_int;
    #[cfg(c_enum_u128="int")] pub type c_enum_u128  = core::ffi::c_int;

    #[cfg(c_enum_i8  ="unsigned int")] pub type c_enum_i8    = core::ffi::c_uint;
    #[cfg(c_enum_i16 ="unsigned int")] pub type c_enum_i16   = core::ffi::c_uint;
    #[cfg(c_enum_i32 ="unsigned int")] pub type c_enum_i32   = core::ffi::c_uint;
    #[cfg(c_enum_i64 ="unsigned int")] pub type c_enum_i64   = core::ffi::c_uint;
    #[cfg(c_enum_i128="unsigned int")] pub type c_enum_i128  = core::ffi::c_uint;
    #[cfg(c_enum_u7  ="unsigned int")] pub type c_enum_u7    = core::ffi::c_uint;
    #[cfg(c_enum_u8  ="unsigned int")] pub type c_enum_u8    = core::ffi::c_uint;
    #[cfg(c_enum_u15 ="unsigned int")] pub type c_enum_u15   = core::ffi::c_uint;
    #[cfg(c_enum_u16 ="unsigned int")] pub type c_enum_u16   = core::ffi::c_uint;
    #[cfg(c_enum_u31 ="unsigned int")] pub type c_enum_u31   = core::ffi::c_uint;
    #[cfg(c_enum_u32 ="unsigned int")] pub type c_enum_u32   = core::ffi::c_uint;
    #[cfg(c_enum_u63 ="unsigned int")] pub type c_enum_u63   = core::ffi::c_uint;
    #[cfg(c_enum_u64 ="unsigned int")] pub type c_enum_u64   = core::ffi::c_uint;
    #[cfg(c_enum_u127="unsigned int")] pub type c_enum_u127  = core::ffi::c_uint;
    #[cfg(c_enum_u128="unsigned int")] pub type c_enum_u128  = core::ffi::c_uint;

    #[cfg(c_enum_i8  ="long")] pub type c_enum_i8    = core::ffi::c_long;
    #[cfg(c_enum_i16 ="long")] pub type c_enum_i16   = core::ffi::c_long;
    #[cfg(c_enum_i32 ="long")] pub type c_enum_i32   = core::ffi::c_long;
    #[cfg(c_enum_i64 ="long")] pub type c_enum_i64   = core::ffi::c_long;
    #[cfg(c_enum_i128="long")] pub type c_enum_i128  = core::ffi::c_long;
    #[cfg(c_enum_u7  ="long")] pub type c_enum_u7    = core::ffi::c_long;
    #[cfg(c_enum_u8  ="long")] pub type c_enum_u8    = core::ffi::c_long;
    #[cfg(c_enum_u15 ="long")] pub type c_enum_u15   = core::ffi::c_long;
    #[cfg(c_enum_u16 ="long")] pub type c_enum_u16   = core::ffi::c_long;
    #[cfg(c_enum_u31 ="long")] pub type c_enum_u31   = core::ffi::c_long;
    #[cfg(c_enum_u32 ="long")] pub type c_enum_u32   = core::ffi::c_long;
    #[cfg(c_enum_u63 ="long")] pub type c_enum_u63   = core::ffi::c_long;
    #[cfg(c_enum_u64 ="long")] pub type c_enum_u64   = core::ffi::c_long;
    #[cfg(c_enum_u127="long")] pub type c_enum_u127  = core::ffi::c_long;
    #[cfg(c_enum_u128="long")] pub type c_enum_u128  = core::ffi::c_long;

    #[cfg(c_enum_i8  ="unsigned long")] pub type c_enum_i8    = core::ffi::c_ulong;
    #[cfg(c_enum_i16 ="unsigned long")] pub type c_enum_i16   = core::ffi::c_ulong;
    #[cfg(c_enum_i32 ="unsigned long")] pub type c_enum_i32   = core::ffi::c_ulong;
    #[cfg(c_enum_i64 ="unsigned long")] pub type c_enum_i64   = core::ffi::c_ulong;
    #[cfg(c_enum_i128="unsigned long")] pub type c_enum_i128  = core::ffi::c_ulong;
    #[cfg(c_enum_u7  ="unsigned long")] pub type c_enum_u7    = core::ffi::c_ulong;
    #[cfg(c_enum_u8  ="unsigned long")] pub type c_enum_u8    = core::ffi::c_ulong;
    #[cfg(c_enum_u15 ="unsigned long")] pub type c_enum_u15   = core::ffi::c_ulong;
    #[cfg(c_enum_u16 ="unsigned long")] pub type c_enum_u16   = core::ffi::c_ulong;
    #[cfg(c_enum_u31 ="unsigned long")] pub type c_enum_u31   = core::ffi::c_ulong;
    #[cfg(c_enum_u32 ="unsigned long")] pub type c_enum_u32   = core::ffi::c_ulong;
    #[cfg(c_enum_u63 ="unsigned long")] pub type c_enum_u63   = core::ffi::c_ulong;
    #[cfg(c_enum_u64 ="unsigned long")] pub type c_enum_u64   = core::ffi::c_ulong;
    #[cfg(c_enum_u127="unsigned long")] pub type c_enum_u127  = core::ffi::c_ulong;
    #[cfg(c_enum_u128="unsigned long")] pub type c_enum_u128  = core::ffi::c_ulong;

    #[cfg(c_enum_i8  ="long long")] pub type c_enum_i8    = core::ffi::c_longlong;
    #[cfg(c_enum_i16 ="long long")] pub type c_enum_i16   = core::ffi::c_longlong;
    #[cfg(c_enum_i32 ="long long")] pub type c_enum_i32   = core::ffi::c_longlong;
    #[cfg(c_enum_i64 ="long long")] pub type c_enum_i64   = core::ffi::c_longlong;
    #[cfg(c_enum_i128="long long")] pub type c_enum_i128  = core::ffi::c_longlong;
    #[cfg(c_enum_u7  ="long long")] pub type c_enum_u7    = core::ffi::c_longlong;
    #[cfg(c_enum_u8  ="long long")] pub type c_enum_u8    = core::ffi::c_longlong;
    #[cfg(c_enum_u15 ="long long")] pub type c_enum_u15   = core::ffi::c_longlong;
    #[cfg(c_enum_u16 ="long long")] pub type c_enum_u16   = core::ffi::c_longlong;
    #[cfg(c_enum_u31 ="long long")] pub type c_enum_u31   = core::ffi::c_longlong;
    #[cfg(c_enum_u32 ="long long")] pub type c_enum_u32   = core::ffi::c_longlong;
    #[cfg(c_enum_u63 ="long long")] pub type c_enum_u63   = core::ffi::c_longlong;
    #[cfg(c_enum_u64 ="long long")] pub type c_enum_u64   = core::ffi::c_longlong;
    #[cfg(c_enum_u127="long long")] pub type c_enum_u127  = core::ffi::c_longlong;
    #[cfg(c_enum_u128="long long")] pub type c_enum_u128  = core::ffi::c_longlong;

    #[cfg(c_enum_i8  ="unsigned long long")] pub type c_enum_i8    = core::ffi::c_ulonglong;
    #[cfg(c_enum_i16 ="unsigned long long")] pub type c_enum_i16   = core::ffi::c_ulonglong;
    #[cfg(c_enum_i32 ="unsigned long long")] pub type c_enum_i32   = core::ffi::c_ulonglong;
    #[cfg(c_enum_i64 ="unsigned long long")] pub type c_enum_i64   = core::ffi::c_ulonglong;
    #[cfg(c_enum_i128="unsigned long long")] pub type c_enum_i128  = core::ffi::c_ulonglong;
    #[cfg(c_enum_u7  ="unsigned long long")] pub type c_enum_u7    = core::ffi::c_ulonglong;
    #[cfg(c_enum_u8  ="unsigned long long")] pub type c_enum_u8    = core::ffi::c_ulonglong;
    #[cfg(c_enum_u15 ="unsigned long long")] pub type c_enum_u15   = core::ffi::c_ulonglong;
    #[cfg(c_enum_u16 ="unsigned long long")] pub type c_enum_u16   = core::ffi::c_ulonglong;
    #[cfg(c_enum_u31 ="unsigned long long")] pub type c_enum_u31   = core::ffi::c_ulonglong;
    #[cfg(c_enum_u32 ="unsigned long long")] pub type c_enum_u32   = core::ffi::c_ulonglong;
    #[cfg(c_enum_u63 ="unsigned long long")] pub type c_enum_u63   = core::ffi::c_ulonglong;
    #[cfg(c_enum_u64 ="unsigned long long")] pub type c_enum_u64   = core::ffi::c_ulonglong;
    #[cfg(c_enum_u127="unsigned long long")] pub type c_enum_u127  = core::ffi::c_ulonglong;
    #[cfg(c_enum_u128="unsigned long long")] pub type c_enum_u128  = core::ffi::c_ulonglong;
}
