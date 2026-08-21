use crc32fast::Hasher;
use dh::{recommended::*, Readable};
use std::{cmp::min, io::Result};

pub fn hash(file: &mut dyn Readable, offset: &u64, size: &u64, buffer_size: &u64) -> Result<u32> {
    let pos_before = file.pos()?;
    file.to(*offset)?;

    let mut hasher = Hasher::new();

    let mut buf = vec![0; *buffer_size as usize];

    let mut remaining = *size;
    while remaining > 0 {
        let to_read = min(*buffer_size, remaining) as usize;
        let buf = &mut buf[..to_read];
        file.read_exact(buf)?;
        hasher.update(buf);
        remaining -= to_read as u64;
    }

    file.to(pos_before)?;
    Ok(hasher.finalize())
}
