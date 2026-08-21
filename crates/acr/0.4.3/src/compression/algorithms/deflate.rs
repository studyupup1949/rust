use dh::{Readable, Writable};
use inflate::inflate_bytes;
use std::io::{Error, ErrorKind, Result};

pub fn decompress<'a, W: Writable<'a>>(source: &mut dyn Readable, target: &mut W) -> Result<u64> {
    let len = source.size()?;
    let data = source.read_bytes(len)?;
    match inflate_bytes(&data) {
        Ok(data) => {
            target.write_bytes(&data)?;
            Ok(data.len() as u64)
        }
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            "Deflate decompression error: ".to_string() + &e.to_string(),
        )),
    }
}
