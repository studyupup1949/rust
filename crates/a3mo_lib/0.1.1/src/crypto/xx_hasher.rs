extern crate time;
extern crate xorshift;

use std::fs::File;
use std::io::Read;

extern crate custom_error;
use custom_error::custom_error;
custom_error! {pub CryptoError
    IOError{source: std::io::Error} = "IOError",
}

fn hash_byte_vec(inp: Vec<u8>) -> u64 {
    let mut hasher = crate::crypto::XxHash64::with_seed(0);
    hasher.write(inp.as_slice());
    let f = hasher.finish();
    return f;
}

pub fn hash_path(path: &str) -> Result<u64, CryptoError> {
    let filedataresult = readfile(path)?;

    let hash = hash_byte_vec(filedataresult);

    Ok(hash)
}

fn readfile(path: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    return Ok(data);
}
