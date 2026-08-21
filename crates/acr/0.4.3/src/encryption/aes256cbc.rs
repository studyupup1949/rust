use dh::Readable;
use libaes::Cipher;
use std::io::Result;

pub fn encrypt<'a>(
    file: &'a mut dyn Readable<'a>,
    key: &[u8; 32],
    iv: &[u8; 16],
    offset: u64,
    size: u64,
) -> Result<Vec<u8>> {
    let cipher = Cipher::new_256(key);
    let data = file.read_bytes_at(offset, size)?;
    Ok(cipher.cbc_encrypt(iv, &data))
}

pub fn decrypt<'a>(
    file: &'a mut dyn Readable<'a>,
    key: &[u8; 32],
    iv: &[u8; 16],
    offset: u64,
    size: u64,
) -> Result<Vec<u8>> {
    let cipher = Cipher::new_256(key);
    let data = file.read_bytes_at(offset, size)?;
    Ok(cipher.cbc_decrypt(iv, &data))
}
