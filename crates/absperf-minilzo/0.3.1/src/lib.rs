mod compress;
mod decompress;
mod error;
mod checksum;
pub use compress::CompressInto;
pub use decompress::DecompressInto;
pub use error::{Error, Result};
use error::check;
pub use checksum::{adler32, adler32_chain};

use absperf_minilzo_sys::{__lzo_init_v2, lzo_callback_t, lzo_uint, lzo_version, lzo_version_date, lzo_version_string};
use std::ffi::{c_void, CStr};
use std::mem::size_of;
use std::os::raw::{c_char, c_uint, c_int, c_long, c_short};

/// Run initialization, which currently just appears to be a simple pointer size check.
/// You should probably still do this, just in case it  gets more complex and necessary in the
/// future.
pub fn init() -> Result<()> {
    let ret = unsafe {
        __lzo_init_v2(
            version_raw(),
            size_of::<c_short>() as c_int,
            size_of::<c_int>() as c_int,
            size_of::<c_long>() as c_int,
            size_of::<u32>() as c_int, // lzo_uint32_t
            size_of::<lzo_uint>() as c_int,
            size_of::<*mut c_char>() as c_int, // lzo_sizeof_dict_t
            size_of::<*mut c_char>() as c_int,
            size_of::<*mut c_void>() as c_int,
            size_of::<lzo_callback_t>() as c_int,
        )
    };
    check(ret)
}

/// Get the raw version as reported by minilzo
pub fn version_raw() -> c_uint {
    unsafe {
        lzo_version()
    }
}

/// Get the version as a (major, minor) tuple
pub fn version() -> (u32, u32) {
    // This computation is weird.  Not sure why the version is represented as a 16 bit integer with
    // each component in the upper nibble.
    let raw = version_raw();
    (raw as u32 >> 12, (raw as u32 >> 4) & 0x0000000F)
}

/// Get the version as a string
pub fn version_str() -> &'static str {
    unsafe {
        let cstr: &'static CStr = CStr::from_ptr(lzo_version_string());
        cstr.to_str().unwrap()
    }
}

/// Get the version date as a string
pub fn version_date() -> &'static str {
    unsafe {
        let cstr: &'static CStr = CStr::from_ptr(lzo_version_date());
        cstr.to_str().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple test to make sure a library function can be correctly called
    #[test]
    fn init() {
        assert_eq!(super::init(), Ok(()));
    }

    #[test]
    fn version() {
        assert_eq!(version_raw(), 0x20a0);
        assert_eq!(super::version(), (2, 10));
        assert_eq!(version_str(), "2.10");
        assert_eq!(version_date(), "Mar 01 2017");
    }

    #[test]
    fn compress_slice() {
        let source = include_bytes!("lorem.txt");
        let mut compress_buffer = Vec::with_capacity(source.len());
        compress_buffer.resize(source.len(), 0);
        let mut decompress_buffer = compress_buffer.clone();

        let compressed = source.compress_into(compress_buffer.as_mut_slice()).unwrap();
        assert!(compressed.len() < source.len());

        let decompressed = compressed.decompress_into(decompress_buffer.as_mut_slice()).unwrap();

        assert_eq!(&source[..], decompressed);
    }

    #[test]
    fn compress_vec() {
        let source = include_bytes!("lorem.txt");
        let mut compress_buffer = Vec::new();

        let compressed = source.compress_into(&mut compress_buffer).unwrap();
        assert!(compressed.len() < source.len());

        let mut decompress_buffer = Vec::new();

        let decompressed = compressed.decompress_into(&mut decompress_buffer).unwrap();

        assert_eq!(&source[..], decompressed);
    }
}
