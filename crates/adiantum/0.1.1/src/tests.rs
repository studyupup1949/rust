use alloc::vec::Vec;

use chacha20::XChaCha12;
use aes::Aes256;

use crate::{
    cipher::{generic_array::GenericArray, KeyInit},
    Cipher,
};

#[test]
fn vectors() {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Vector {
        input: Input,
        plaintext_hex: &'static str,
        ciphertext_hex: &'static str,
    }

    #[derive(Deserialize)]
    struct Input {
        key_hex: &'static str,
        tweak_hex: &'static str,
    }

    let data =
        serde_json::from_str::<Vec<Vector>>(include_str!("test_data/compressed.json")).unwrap();
    for Vector {
        input: Input { key_hex, tweak_hex },
        plaintext_hex,
        ciphertext_hex,
    } in data
    {
        let key = hex::decode(key_hex).unwrap();
        assert_eq!(key.len(), 0x20);
        let cipher = Cipher::<XChaCha12, Aes256>::new(GenericArray::from_slice(&key));

        let mut msg = hex::decode(plaintext_hex).unwrap();
        assert!((0x10..=0x1000).contains(&msg.len()));

        let tweak = hex::decode(tweak_hex).unwrap();
        assert!(tweak.len() <= 0x20);

        cipher.encrypt(&mut msg, &tweak);
        assert_eq!(hex::encode(&msg), ciphertext_hex);

        cipher.decrypt(&mut msg, &tweak);
        assert_eq!(hex::encode(&msg), plaintext_hex);
    }
}
