use dh::{Readable, Writable};
use lzma_rs::lzma_decompress;
use std::io::{BufReader, Error, ErrorKind, Result};

pub fn decompress<'a, W: Writable<'a>>(source: &mut dyn Readable, target: &mut W) -> Result<u64> {
    let start = target.pos()?;
    let mut buf = BufReader::new(source);
    match lzma_decompress(&mut buf, target) {
        Ok(_) => Ok(target.pos()? - start),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            "LZMA decompression error: ".to_string() + &e.to_string(),
        )),
    }
}
