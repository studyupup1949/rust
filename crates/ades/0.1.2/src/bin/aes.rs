use std::fs;
use ades::{Padding, aes_enc, aes_dec};
use aoko::no_std::ext::{AnyExt1, FnOnceExt};

// aes: enc/dec file-name password file-name

fn main() {
    // Get command line arguments:
    let args = std::env::args().collect::<Vec<_>>();

    // Get arguments values: (delay initialize [3] because of more elegant code)
    let mode = &args[1];
    let file_in = &args[2];
    let file_out = &args[4];

    // Read file data & Write file as partially applied function:
    let data = fs::read(file_in).unwrap();
    let write = |text| fs::write.partial2(text)(file_out.clone()).unwrap();

    // Initialize the remaining arguments:
    args[3].padding(32).as_bytes().let_owned(|key| {
        // Crypto as partially applied function:
        let aes_enc = |data| aes_enc(key)(data);
        let aes_dec = |data| aes_dec(key)(data);

        // Encryption and decryption:
        match &**mode {
            "enc" => aes_enc(&data).let_owned(|byt| write(byt)),
            "dec" => aes_dec(&data).let_owned(|byt| write(byt)),
            _ => ()
        }
    })
}
