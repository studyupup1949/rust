use dh::Readable;
use sha2::{Digest, Sha256};
use std::io::Result;

pub fn hash<'a>(file: &'a mut dyn Readable<'a>, offset: u64, size: u64) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    let data = file.read_bytes_at(offset, size)?;
    hasher.update(data);
    Ok(hasher.finalize().into())
}
