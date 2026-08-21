use crate::{Padding, aes_enc, aes_dec, des_enc, des_dec};
use aoko::{no_std::functions::ext::{AnyExt1, FnOnceExt}, l};
use std::fs;

// ades: enc/dec file-name aes_passwd des_passwd file-name

pub fn ades(r#in: String, out: String, aes: String, des: String, encrypt: bool) {
    // Read file data & Write file as partially applied function:
    l!(
        data = fs::read(r#in).unwrap();
        write = |text| fs::write.partial2(text)(out).unwrap());

    // Initialize the remaining arguments:
    aes.padding(32).as_bytes().let_owned(|aes_key|
        des.padding(24).as_bytes().let_owned(|des_key| {
            // Crypto as partially applied function:
            l!(
                aes_enc = |data| aes_enc(aes_key)(data);
                aes_dec = |data: &_| aes_dec(aes_key)(data);
                des_enc = |data: &_| des_enc(des_key)(data);
                des_dec = |data| des_dec(des_key)(data));

            // Encryption and decryption:
            match encrypt {
                true => aes_enc(&data)
                            .let_owned(|ctx| des_enc(&ctx))
                            .let_owned(write),
                false => des_dec(&data)
                            .let_owned(|ctx| aes_dec(&ctx))
                            .let_owned(write)
            }
        })
    )
}
