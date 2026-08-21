use std::fs;
use ades::{Padding, KtStd, FnOnceExt, des_enc, des_dec};

// des: enc/dec file-name password file-name

fn main() {
    // Get command line arguments:
    let args = std::env::args().collect::<Vec<_>>();

    // Get arguments values: (delay initialize [3] because of more elegant code)
    let mode = &args[1];
    let file_in = &args[2];
    let file_out = &args[4];

    // Read file data & Write file as partially applied function:
    let data = fs::read(file_in).unwrap();
    let write = |text| fs::write.partial(text)(file_out.clone()).unwrap();

    // Initialize the remaining arguments:
    args[3].padding(24).as_bytes().then(|key| {
        // Crypto as partially applied function:
        let des_enc = |data| des_enc(key)(data);
        let des_dec = |data| des_dec(key)(data);

        // Encryption and decryption:
        match &**mode {
            "enc" => des_enc(&data).then(|byt| write(byt)),
            "dec" => des_dec(&data).then(|byt| write(byt)),
            _ => ()
        }
    })
}