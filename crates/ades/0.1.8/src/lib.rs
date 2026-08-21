#[macro_use]
extern crate lazy_static;
use openssl::symm::{Cipher, encrypt, decrypt};

pub mod cli;
pub mod logic;

pub const AES_IV: &[u8] = b",E@*{O.x=#z~}C>Z";
pub const DES_IV: &[u8] = b"k+Q|R](h";

lazy_static! {
    pub static ref AES_CIPHER: Cipher = Cipher::aes_256_cfb128();
    pub static ref DES_CIPHER: Cipher = Cipher::des_ede3_cfb64();
}

pub fn aes_enc(key: &[u8]) -> impl Fn(&[u8]) -> Vec<u8> + '_ {
    move |data| encrypt(*AES_CIPHER, key, Some(AES_IV), data).unwrap()
}

pub fn aes_dec(key: &[u8]) -> impl Fn(&[u8]) -> Vec<u8> + '_ {
    move |data| decrypt(*AES_CIPHER, key, Some(AES_IV), data).unwrap()
}

pub fn des_enc(key: &[u8]) -> impl Fn(&[u8]) -> Vec<u8> + '_ {
    move |data| encrypt(*DES_CIPHER, key, Some(DES_IV), data).unwrap()
}

pub fn des_dec(key: &[u8]) -> impl Fn(&[u8]) -> Vec<u8> + '_ {
    move |data| decrypt(*DES_CIPHER, key, Some(DES_IV), data).unwrap()
}

pub trait Padding {
    fn padding(self, n: usize) -> String;
}

impl Padding for &str {
    fn padding(self, n: usize) -> String {
        if self.len() < n {
            let init = String::from("<Ra{V)*t%o&:`1q^/Y|#U+k-W$'Fl7cJ");
            format!("{}{}", self, &init[..n-self.len()])
        } else { self.to_string() }
    }
}