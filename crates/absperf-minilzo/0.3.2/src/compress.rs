use crate::error::check;
use crate::Result;
use absperf_minilzo_sys::{lzo1x_1_compress, lzo_uint};
use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::size_of;
use std::os::raw::c_char;

const LZO1X_MEM_COMPRESS: usize = 16384 * size_of::<*const c_char>();

thread_local!(
    static WRKMEM: RefCell<[u8; LZO1X_MEM_COMPRESS]> = RefCell::new([0u8; LZO1X_MEM_COMPRESS])
);

/// Compression trait.  Implemented for [u8], allowing compressing into [u8] and Vec<u8>
pub trait CompressInto<T: ?Sized> {
    /// LZO1X compresses into destination, returning a subslice of the compressed data
    fn compress_into<'buffer>(&self, destination: &'buffer mut T) -> Result<&'buffer [u8]>;
}

impl CompressInto<[u8]> for [u8] {
    fn compress_into<'buffer>(&self, destination: &'buffer mut [u8]) -> Result<&'buffer [u8]> {
        WRKMEM.with(move |wrkmem| {
            let mut wrkmem = wrkmem.borrow_mut();
            let mut out_len = destination.len() as lzo_uint;
            let ret = unsafe {
                lzo1x_1_compress(
                    self.as_ptr(),
                    self.len() as lzo_uint,
                    destination.as_mut_ptr(),
                    &mut out_len,
                    wrkmem.as_mut_ptr() as *mut c_void,
                )
            };
            check(ret)?;
            Ok(&destination[..out_len as usize])
        })
    }
}

impl CompressInto<Vec<u8>> for [u8] {
    /// Compresses into the indicated vector, automatically resizing it to fit the result.
    /// On error, the destination vector's contents likely will be corrupted.
    fn compress_into<'buffer>(&self, destination: &'buffer mut Vec<u8>) -> Result<&'buffer [u8]> {
        WRKMEM.with(move |wrkmem| {
            let mut wrkmem = wrkmem.borrow_mut();
            let in_len = self.len() as lzo_uint;
            let mut out_len = in_len + in_len / 16 + 64 + 3;
            if destination.capacity() < out_len as usize {
                destination.reserve_exact(out_len as usize - destination.len());
            }
            // Reserving may reserve more than requested
            out_len = destination.capacity() as lzo_uint;
            let ret = unsafe {
                lzo1x_1_compress(
                    self.as_ptr(),
                    in_len,
                    destination.as_mut_ptr(),
                    &mut out_len,
                    wrkmem.as_mut_ptr() as *mut c_void,
                )
            };
            check(ret)?;
            unsafe {
                destination.set_len(out_len as usize);
            }
            Ok(&destination[..out_len as usize])
        })
    }
}
