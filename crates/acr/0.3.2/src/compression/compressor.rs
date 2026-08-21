use super::Method;
use dh::{Readable, Writable};
use std::io::{Error, ErrorKind, Result};

/// Compresses data from a reader and writes it to a target.
///
/// If successful, returns the number of bytes written to the target.
pub fn compress<'a>(
    reader: &mut dyn Readable<'a>,
    offset: u64,
    size: u64,
    method: &Method,
    target: &mut dyn Writable<'a>,
    buffer_size: u64,
) -> Result<u64> {
    use Method::*;
    match method {
        None => {
            reader.copy_at(offset, size, target, buffer_size)?;
            Ok(size)
        }
        _ => Err(Error::new(
            ErrorKind::Unsupported,
            "Unsupported compression method",
        )),
    }
}
