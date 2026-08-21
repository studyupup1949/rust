use crate::error::{check, Error};
use crate::Result;
use absperf_minilzo_sys::{lzo1x_decompress_safe, lzo_uint};
use std::ptr::null_mut;

pub trait DecompressInto<T: ?Sized> {
    /// LZO1X decompresses into destination, returning a subslice of the decompressed data
    fn decompress_into<'comp>(&self, destination: &'comp mut T) -> Result<&'comp [u8]>;
}

impl DecompressInto<[u8]> for [u8] {
    fn decompress_into<'comp>(&self, destination: &'comp mut [u8]) -> Result<&'comp [u8]> {
        let mut out_len = destination.len() as lzo_uint;
        let ret = unsafe {
            lzo1x_decompress_safe(
                self.as_ptr(),
                self.len() as lzo_uint,
                destination.as_mut_ptr(),
                &mut out_len,
                null_mut(),
            )
        };
        check(ret)?;
        Ok(&destination[..out_len as usize])
    }
}

impl DecompressInto<Vec<u8>> for [u8] {
    /// Decompresses into the indicated vector, automatically resizing it to fit the result.
    /// This can be pretty inefficient.  It first ensures that the vector's capacity matches the
    /// input size, and repeatedly tries to decompress into the vector, doubling the size each time
    /// until successful.  If you want to avoid this, ensure that your vector has sufficient
    /// capacity.  If you can know the decompressed size ahead of time, or can safely guess large
    /// enough, you can avoid any unnecessary wasted work.
    fn decompress_into<'comp>(&self, destination: &'comp mut Vec<u8>) -> Result<&'comp [u8]> {
        if destination.capacity() < self.len() {
            destination.reserve(self.len() - destination.len());
        }
        let in_len = self.len() as lzo_uint;
        loop {
            let mut out_len = destination.capacity() as lzo_uint;
            let ret = unsafe {
                lzo1x_decompress_safe(
                    self.as_ptr(),
                    in_len,
                    destination.as_mut_ptr(),
                    &mut out_len,
                    null_mut(),
                )
            };
            match check(ret) {
                Ok(()) => {
                    unsafe {
                        destination.set_len(out_len as usize);
                    }
                    return Ok(&destination[..out_len as usize]);
                }
                Err(Error::OutputOverrun) => {
                    destination.reserve(destination.capacity() * 2 - destination.len());
                }
                Err(e) => return Err(e),
            }
        }
    }
}
