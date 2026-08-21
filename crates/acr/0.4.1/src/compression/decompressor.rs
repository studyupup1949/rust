use super::{
    algorithms::{deflate, lzma},
    Method,
};
use dh::{Readable, Writable};
use std::io::{Error, ErrorKind, Result};

/// Decompresses data from a reader and writes it to a target.
///
/// If successful, returns the number of bytes written to the target.
pub fn decompress<'a, W: Writable<'a>>(
    reader: &'a mut dyn Readable<'a>,
    offset: u64,
    size: u64,
    method: &Method,
    target: &mut W,
    buffer_size: u64,
) -> Result<u64> {
    use Method::*;
    match method {
        None => {
            reader.copy_at(offset, size, target, buffer_size)?;
            Ok(size)
        }
        Lzma => {
            let mut source = reader.limit(offset, size)?;
            let size = lzma::decompress(&mut source, target)?;
            Ok(size)
        }
        Deflate => {
            let mut source = reader.limit(offset, size)?;
            let size = deflate::decompress(&mut source, target)?;
            Ok(size)
        }
        _ => Err(Error::new(
            ErrorKind::Unsupported,
            "Unsupported compression method",
        )),
    }
}
