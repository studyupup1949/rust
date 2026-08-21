use dh::Readable;
use murmur3::murmur3_32;
use std::io::Result;

pub fn hash<'a>(file: &'a mut dyn Readable<'a>, offset: u64, size: u64, seed: u32) -> Result<u32> {
    let pos_before = file.pos()?;

    let mut limited = file.limit(offset, size)?;

    let result = murmur3_32(&mut limited, seed)?;
    let file = limited.unlimit();

    file.to(pos_before)?;

    Ok(result)
}
