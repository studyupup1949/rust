use std::fs;
use ades::{Padding, KtStd, FnOnceExt, aes_enc, aes_dec, des_enc, des_dec};

// ades: enc/dec file-name aes_passwd des_passwd file-name

fn main() {
    // Get command line arguments:
    let args = std::env::args().collect::<Vec<_>>();

    // Get arguments values: (delay initialize [3] & [4] because of more elegant code)
    let mode = &args[1];
    let file_in = &args[2];
    let file_out = &args[5];

    // Read file data & Write file as partially applied function:
    let data = fs::read(file_in).unwrap();
    let write = |text| fs::write.partial(text)(file_out.clone()).unwrap();

    // Initialize the remaining arguments:
    args[3].padding(32).as_bytes().then(|aes_key|
        args[4].padding(24).as_bytes().then(|des_key| {
            // Crypto as partially applied function:
            let aes_enc = |data| aes_enc(aes_key)(data);
            let aes_dec = |data: &_| aes_dec(aes_key)(data);
            let des_enc = |data: &_| des_enc(des_key)(data);
            let des_dec = |data| des_dec(des_key)(data);

            // Encryption and decryption:
            match &**mode {
                "enc" => aes_enc(&data)
                            .then(|ctx| des_enc(&ctx))
                            .then(|byt| write(byt)),
                "dec" => des_dec(&data)
                            .then(|ctx| aes_dec(&ctx))
                            .then(|byt| write(byt)),
                _ => ()
            }
        })
    )
}
