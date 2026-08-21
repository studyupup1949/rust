use alloc::vec::Vec;

use aes::Aes256;
use chacha20::{XChaCha12, XChaCha20, XChaCha8};
use serde::Deserialize;

use crate::{
    cipher::{generic_array::GenericArray, KeyInit},
    Cipher,
};

#[derive(Deserialize)]
struct Vector {
    input: Input,
    plaintext_hex: &'static str,
    ciphertext_hex: &'static str,
    description: &'static str,
}

#[derive(Deserialize)]
struct Input {
    key_hex: &'static str,
    tweak_hex: &'static str,
}

macro_rules! test_cipher_impl {
    ($type:ty, $test_data:literal, $test_name:ident) => {
        #[test]
        fn $test_name() {
            let data = serde_json::from_str::<Vec<Vector>>(include_str!($test_data)).unwrap();
            for Vector {
                input: Input { key_hex, tweak_hex },
                plaintext_hex,
                ciphertext_hex,
                description,
            } in data
            {
                let test_cipher = stringify!($test_name);
                let key = hex::decode(key_hex).unwrap();
                assert_eq!(key.len(), 0x20);

                let cipher = <$type>::new(GenericArray::from_slice(&key));

                let mut msg = hex::decode(plaintext_hex).unwrap();
                assert!((0x10..=0x1000).contains(&msg.len()), "Cipher: {test_cipher}, description {description}, plaintext_hex: {plaintext_hex}, ciphertext_hex: {ciphertext_hex}");

                let tweak = hex::decode(tweak_hex).unwrap();
                assert!(tweak.len() <= 0x20, "Cipher: {test_cipher}, description {description}, plaintext_hex: {plaintext_hex}, ciphertext_hex: {ciphertext_hex}");

                cipher.encrypt(&mut msg, &tweak);
                assert_eq!(hex::encode(&msg), ciphertext_hex, "Cipher: {test_cipher}, description {description}, plaintext_hex: {plaintext_hex}, ciphertext_hex: {ciphertext_hex}");

                cipher.decrypt(&mut msg, &tweak);
                assert_eq!(hex::encode(&msg), plaintext_hex, "Cipher: {test_cipher}, description {description}, plaintext_hex: {plaintext_hex}, ciphertext_hex: {ciphertext_hex}");
            }
        }
    };
}

test_cipher_impl!(Cipher<XChaCha8, Aes256>, "test_data/Adiantum_XChaCha8_32_AES256.json", x_cha_cha8);
test_cipher_impl!(Cipher<XChaCha12, Aes256>, "test_data/Adiantum_XChaCha12_32_AES256.json", x_cha_cha12);
test_cipher_impl!(Cipher<XChaCha20, Aes256>, "test_data/Adiantum_XChaCha20_32_AES256.json", x_cha_cha20);
