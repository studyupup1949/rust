use crate::{Padding, des_enc, des_dec};
use aoko::{l, no_std::functions::ext::{AnyExt1, FnOnceExt}};
use std::fs;

pub fn des(r#in: String, out: String, des: String, encrypt: bool) {
    // Read file data & Write file as partially applied function:
    l!(
        data = fs::read(r#in).unwrap();
        write = |text| fs::write.partial2(text)(out).unwrap());

    // Initialize the remaining arguments:
    des.padding(24).as_bytes().let_owned(|key| {
        // Crypto as partially applied function:
        l!(
            des_enc = |data| des_enc(key)(data);
            des_dec = |data| des_dec(key)(data));

        // Encryption and decryption:
        match encrypt {
            true => des_enc(&data).let_owned(write),
            false => des_dec(&data).let_owned(write)
        }
    })
}