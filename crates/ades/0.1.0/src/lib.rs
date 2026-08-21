#[macro_use]
extern crate lazy_static;
use openssl::symm::{Cipher, encrypt, decrypt};

pub const AES_IV: &[u8] = b",E@*{O.x=#z~}C>Z";
pub const DES_IV: &[u8] = b"k+Q|R](h";

lazy_static! {
    pub static ref AES_CIPHER: Cipher = Cipher::aes_256_cfb128();
    pub static ref DES_CIPHER: Cipher = Cipher::des_ede3_cfb64();
}

pub fn aes_enc(key: &[u8]) -> impl FnOnce(&[u8]) -> Vec<u8> + '_ {
    move |data| encrypt(*AES_CIPHER, key, Some(AES_IV), data).unwrap()
}

pub fn aes_dec(key: &[u8]) -> impl FnOnce(&[u8]) -> Vec<u8> + '_ {
    move |data| decrypt(*AES_CIPHER, key, Some(AES_IV), data).unwrap()
}

pub fn des_enc(key: &[u8]) -> impl FnOnce(&[u8]) -> Vec<u8> + '_ {
    move |data| encrypt(*DES_CIPHER, key, Some(DES_IV), data).unwrap()
}

pub fn des_dec(key: &[u8]) -> impl FnOnce(&[u8]) -> Vec<u8> + '_ {
    move |data| decrypt(*DES_CIPHER, key, Some(DES_IV), data).unwrap()
}

pub trait Padding {
    fn padding(self, n: usize) -> String;
}

impl Padding for &String {
    fn padding(self, n: usize) -> String {
        if self.len() < n {
            let init = String::from("<Ra{V)*t%o&:`1q^/Y|#U+k-W$'Fl7cJ");
            format!("{}{}", self, &init[..n-self.len()])
        } else { self.clone() }
    }
}

pub trait KtStd {
    fn then<R>(self, block: impl FnOnce(Self) -> R) -> R where Self: Sized {
        block(self)
    }
}

impl<T> KtStd for T {}

pub trait FnOnceExt<P1, P2, R> {
    fn partial(self, p2: P2) -> Box<dyn FnOnce(P1) -> R>;
}

impl<P1, P2: 'static, R, F> FnOnceExt<P1, P2, R> for F where F: FnOnce(P1, P2) -> R + 'static {
    fn partial(self, p2: P2) -> Box<dyn FnOnce(P1) -> R> {
        Box::new(move |p1| self(p1, p2))
    }
}
