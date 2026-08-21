use pqcrypto_falcon::falcon1024::*;
use pqcrypto_traits::sign::PublicKey;
use pqcrypto_traits::sign::SecretKey;
use pqcrypto_traits::Result;
use pqcrypto_traits::sign::DetachedSignature;

use crate::encoding::{EncodingAPI,HashEncoderAPI};

use serde::{Serialize,Deserialize};
use serde::{Deserializer, Serializer};

#[derive(Copy,Clone,Serialize,Deserialize)]
pub struct Falcon1024Keypair {
    pk: pqcrypto_falcon::falcon1024::PublicKey,
    sk: pqcrypto_falcon::falcon1024::SecretKey,
}

#[derive(Clone,Serialize,Deserialize)]
pub struct Falcon1024Signature {
    pk: pqcrypto_falcon::falcon1024::PublicKey,
    signature: String,
    message: String,
}

impl Falcon1024Keypair {
    /// This builds the keypair struct from the constructed Public Key and Secret Key
    pub fn new(pk: pqcrypto_falcon::falcon1024::PublicKey, sk: pqcrypto_falcon::falcon1024::SecretKey) -> Self {
        return Self {
            pk: pk,
            sk: sk,
        }
    }
    /// This builds the keypair struct from hexadecimal public key and secret key
    pub fn from_hex<T: AsRef<str>>(pk: T, sk: T) -> Self {
        let pk_bytes = hex::decode(pk.as_ref()).expect("Failed To Decode Public Key From Hex (Falcon1024)");
        let sk_bytes = hex::decode(sk.as_ref()).expect("Failed To Decode Secret Key From Hex (Falcon1024)");

        let pk_output = pqcrypto_falcon::falcon1024::PublicKey::from_bytes(&pk_bytes).expect("Failed To Construct Public Key From Bytes (Falcon1024) (from_hex)");
        let sk_output = pqcrypto_falcon::falcon1024::SecretKey::from_bytes(&sk_bytes).expect("Failed To Construct Secret Key From Bytes (Falcon1024) (from_hex)");

        return Self {
            pk: pk_output,
            sk: sk_output,
        }
    }
    /// This builds the keypair struct from bytes array for public key and secret key
    pub fn from_bytes(pk: &[u8],sk: &[u8]) -> Self {
        return Self {
            pk: pqcrypto_falcon::falcon1024::PublicKey::from_bytes(pk).expect("Failed To Get Public Key From Bytes (Falcon1024) (from_bytes)"),
            sk: pqcrypto_falcon::falcon1024::SecretKey::from_bytes(sk).expect("Failed To Get Secret Key From Bytes (Falcon1024) (from_bytes)"),
        }
    }
    /// This generates a new keypair struct. These keypairs cannot be constructed from a seed.
    pub fn generate() -> Self {
        let (pk,sk) = pqcrypto_falcon::falcon1024::keypair();

        return Self {
            pk: pk,
            sk: sk,
        }
    }
    /// Returns the signature
    pub fn sign<T: AsRef<str>>(&self, msg: T) -> Falcon1024Signature {
        
        let signature = detached_sign(msg.as_ref().as_bytes(),&self.sk);

        //assert_eq!(signature.as_bytes().len(),1280);

        return Falcon1024Signature {
            pk: self.pk,
            signature: EncodingAPI::encode_into_base64(signature.as_bytes()),
            message: msg.as_ref().to_owned(),
        }
    }
    pub fn return_public_key_as_hex(&self) -> String {
        return hex::encode_upper(self.pk.as_bytes());
    }
    pub fn return_public_key_as_bytes(&self) -> &[u8] {
        return self.pk.as_bytes()
    }
    pub fn return_secret_key_as_hex(&self) -> String {
        return hex::encode_upper(self.sk.as_bytes())
    }
    pub fn return_secret_key_as_bytes(&self) -> &[u8] {
        return self.sk.as_bytes()
    }
    /// # Return Address
    /// 
    /// Returns a **48 byte** (384 bit) hash of the public key using blake2b.
    pub fn return_address(&self) -> String {
        return  self.return_public_key_as_48_byte_hash()
    }
    /// # Return Address (Using Key)
    /// 
    /// This will return the 48 byte hash of the address keyed with a **key**.
    pub fn return_address_using_key<T: AsRef<[u8]>>(&self, key: T) -> String {
        return self.return_public_key_as_48_byte_hash_with_key(key.as_ref())
    }
    fn return_public_key_as_48_byte_hash(&self) -> String {
        return HashEncoderAPI::return_48_byte_blake2b_hash_as_hex(self.pk.as_bytes())
    }
    fn return_public_key_as_48_byte_hash_with_key<T: AsRef<[u8]>>(&self, key: T) -> String {
        return HashEncoderAPI::return_48_byte_blake2b_hash_with_key_as_hex(self.pk.as_bytes(), key.as_ref())
    }
}

impl Falcon1024Signature {
    pub fn new<T: AsRef<str>>(pk: pqcrypto_falcon::falcon1024::PublicKey,signature: T, message: T) -> Self {
        return Self {
            pk: pk,
            signature: signature.as_ref().to_owned(),
            message: message.as_ref().to_owned(),
        }
    }
    pub fn new_from_string<T: AsRef<str>>(pk: T, signature: T, message: T) -> Self {
        let pk_decoded = EncodingAPI::decode_from_hex(pk.as_ref());

        return Self {
            pk: pqcrypto_falcon::falcon1024::PublicKey::from_bytes(&pk_decoded).expect("Failed To Construct Public Key From Bytes (Falcon1024-Signature) (new_from_string)"),
            signature: signature.as_ref().to_owned(),
            message: message.as_ref().to_owned(),
        }
    }
    pub fn return_signature_in_base64(&self) -> String {
        return self.signature.clone()
    }
    pub fn return_message(&self) -> String {
        return self.message.clone()
    }
    pub fn return_public_key_as_hex(&self) -> String {
        return hex::encode_upper(self.pk.as_bytes())
    }
    pub fn verify(&self) -> bool {
        let sig = pqcrypto_falcon::falcon1024::DetachedSignature::from_bytes(&EncodingAPI::decode_from_base64(&self.signature)).expect("[VERIFICATION-FAILURE] Failed To Construct Detached Signature (Falcon1024-Signature) (verify)");
        
        let output = verify_detached_signature(&sig, self.message.as_bytes(), &self.pk);

        if output.is_ok() {
            return true
        }
        else {
            return false
        }
    }
    pub fn verify_message<T: AsRef<str>>(&self, message: T) -> bool {
        let sig = pqcrypto_falcon::falcon1024::DetachedSignature::from_bytes(&EncodingAPI::decode_from_base64(&self.signature)).expect("[VERIFICATION-FAILURE] Failed To Construct Detached Signature (Falcon1024-Signature) (verify)");

        let output = verify_detached_signature(&sig, message.as_ref().as_bytes(), &self.pk);

        if output.is_ok() {
            return true
        }
        else {
            return false
        }
    }
    // TODO: Change unwraps to error handling
    /// Accepts an input
    /// - Hexadecimal Public Key
    /// - Signature in Base64
    /// - Message
    pub fn verify_with_input<T: AsRef<str>>(pk: T, signature: T,message: T) -> bool {
        let pk_decoded = hex::decode(pk.as_ref()).unwrap();
        let constructed_pk = pqcrypto_falcon::falcon1024::PublicKey::from_bytes(&pk_decoded).unwrap();

        let sig = pqcrypto_falcon::falcon1024::DetachedSignature::from_bytes(&EncodingAPI::decode_from_base64(signature)).expect("[VERIFICATION-FAILURE] Failed To Construct Detached Signature (Falcon1024-Signature) (verify-with-input)");
    
        let output = verify_detached_signature(&sig, message.as_ref().as_bytes(), &constructed_pk);

        if output.is_ok() {
            return true
        }
        else {
            return false
        }
    }
}

#[test]
fn falcon1024_generation(){
    let keypair = Falcon1024Keypair::generate();

    let signature = keypair.sign("Hello World.");

    let is_valid_signature = signature.verify();

    assert!(is_valid_signature);
}

#[test]
fn falcon1024_signing(){
    let keypair = Falcon1024Keypair::generate();

    let sig = keypair.sign("Test 1 2 3");

    let pk = sig.return_public_key_as_hex();
    let signature = sig.return_signature_in_base64();
    let msg = sig.return_message();

    let is_valid = Falcon1024Signature::verify_with_input(pk, signature, msg);

    assert!(is_valid)
}