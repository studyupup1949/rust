use crate::prelude::*;

pub trait DigitalSignatureTrait {
    /// # New
    /// 
    /// Creates a Keypair from the Seed
    fn new(seed: &[u8]) -> Self;

    /// # Generate
    /// 
    /// Generates a keypair using CSPRNG
    fn generate() -> Self;

    /// # Sign
    /// 
    /// `Sign()` allows you to sign data using a given algorithm
    /// 
    /// TODO: Change to Signature Struct
    /// 
    /// ## Developer Notes
    /// 
    /// - Signs message as bytes
    /// 
    /// - Takes ownership of digital signature keypair and destorys it after signing
    fn sign<T: AsRef<[u8]>>(&self, message: T) -> String;

    fn return_pk_as_hex(&self) -> String;
    fn return_pk_as_bytes(&self) -> &[u8];
}