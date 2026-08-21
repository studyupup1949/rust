//TODO: implement VDFs, multi-signatures with Schnorr
use crate::os_random::OsRandom;
use schnorrkel::{Keypair,SecretKey,PublicKey,Signature};
use schnorrkel::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, KEYPAIR_LENGTH, SIGNATURE_LENGTH};
use zeroize::Zeroize;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use crate::seed::*;
use crate::os_random::*;

use serde::{Serialize,Deserialize};

big_array! { BigArray; }

#[derive(Zeroize,Debug,Clone,PartialEq,Serialize,Deserialize)]
#[zeroize(drop)]
pub struct SchnorrKeypair {
    pk: [u8;PUBLIC_KEY_LENGTH],
    #[serde(with = "BigArray")]
    sk: [u8;SECRET_KEY_LENGTH],
}

#[derive(Zeroize,Debug,Clone,PartialEq,Serialize,Deserialize)]
#[zeroize(drop)]
pub struct SchnorrSignature {
    pk: [u8;PUBLIC_KEY_LENGTH],
    #[serde(with = "BigArray")]
    signature: [u8;SIGNATURE_LENGTH],
    context: Vec<u8>,
    message: Vec<u8>,
}

impl SchnorrSignature {
    /// Verifies signature
    pub fn verify(&self) -> bool {
        // Construct Public Key
        let publickey = PublicKey::from_bytes(&self.pk).expect("Failed To Construct Schnorr Public Key In Verification");
        
        // Construct Signature
        let sig = Signature::from_bytes(&self.signature).expect("Failed To Construct Schnorr Signature In Verification");

        // Simple Verification using context bytes, message, and signature
        let verification = publickey.verify_simple(&self.context,&self.message,&sig);

        if verification.is_ok() {
            return true
        }
        else {
            return false
        }
    }
    /// # From Encoding
    /// 
    /// Converts Public Key (Hex), Signature (Base64), context (bytes), message (bytes) to Schnorr Signature
    pub fn from_encoding<T: AsRef<str>>(pk: T, signature: T, context: &[u8], message: &[u8]) -> Self {
        let mut pk_output: [u8;PUBLIC_KEY_LENGTH] = [0u8;PUBLIC_KEY_LENGTH];
        let mut signature_output: [u8;SIGNATURE_LENGTH] = [0u8;SIGNATURE_LENGTH];
        
        let public_key_decoded = hex::decode(pk.as_ref()).expect("Failed To Decode Public Key From Hex For Schnorr Signatures");
        let signature = base64::decode(signature.as_ref()).expect("Failed To Decode Signature From Base64 For Schnorr Signatures");

        let mut i: usize = 0;

        for x in public_key_decoded {
            pk_output[i] = x;
            i += 1;
        }

        i = 0;

        for x in signature {
            signature_output[i] = x;
            i += 1;
        }

        return Self {
            pk: pk_output,
            signature: signature_output,
            context: context.to_vec(),
            message: message.to_vec(),
        }
    }
    pub fn return_pk_as_hex(&self) -> String {
        return hex::encode_upper(self.pk)
    }
    pub fn return_signature_as_base64(&self) -> String {
        return base64::encode(self.signature)
    }
    pub fn return_context(&self) -> Vec<u8> {
        return self.context.clone()
    }
    pub fn return_message(&self) -> Vec<u8> {
        return self.message.clone()
    }
    pub fn return_all(&self) -> (String,String,Vec<u8>,Vec<u8>) {
        return (hex::encode_upper(self.pk),base64::encode(self.signature),self.context.clone(),self.message.clone())
    }
}

impl SchnorrKeypair {
    /// Constructs seed into 32 byte seed that is used to generate the keypair.
    pub fn new(seed: &[u8]) -> Self {
        let constructed_seed = SchnorrKeypair::construct_seed(seed);

        let rng = ChaCha20Rng::from_seed(constructed_seed);
        let keypair: Keypair = Keypair::generate_with(rng);

        return Self {
            pk: keypair.public.to_bytes(),
            sk: keypair.secret.to_bytes(),
        }

    }
    pub fn sign_with_rng(&self, message: &[u8]) -> SchnorrSignature {
        let keypair = self.construct();

        let rng = OsRandom::rand_64().expect("Failed To Get CSPRNG");

        let sig = keypair.sign_simple_doublecheck(&rng,message).expect("Failed To Sign");

        return SchnorrSignature {
            pk: self.pk,
            signature: sig.to_bytes(),
            context: rng.clone().to_vec(),
            message: message.clone().to_vec(),
        }
    }
    pub fn simple_sign(&self, context: &[u8], message: &[u8]) -> SchnorrSignature {
        let keypair = self.construct();
        let sig = keypair.sign_simple_doublecheck(context,message).expect("Failed To Sign");
        
        return SchnorrSignature {
            pk: self.pk,
            signature: sig.to_bytes(),
            context: context.clone().to_vec(),
            message: message.clone().to_vec(),
        }
    }
    fn construct(&self) -> Keypair {
        let secret_key: SecretKey = SecretKey::from_bytes(&self.sk).expect("Failed to get Schnorr Secret Key From Bytes");
        let keypair = secret_key.to_keypair();
        return keypair
    }
    fn construct_from_bytes(pk_input: &[u8],sk_input: &[u8]) -> ([u8;PUBLIC_KEY_LENGTH],[u8;SECRET_KEY_LENGTH]) {
        let mut pk: [u8;PUBLIC_KEY_LENGTH] = [0u8;PUBLIC_KEY_LENGTH];
        let mut sk: [u8;SECRET_KEY_LENGTH] = [0u8;SECRET_KEY_LENGTH];

        let mut i: usize = 0;

        for x in pk_input {
            pk[i] = *x;
            i += 1usize;
        }

        i = 0;

        for x in sk_input {
            sk[i] = *x;
            i += 1usize;
        }

        return (pk,sk)

    }
    fn construct_seed(seed: &[u8]) -> [u8;32] {
        let mut output: [u8;32] = [0u8;32];
        let mut i: usize = 0;

        for x in 0..32  {
            output[i] = seed[i];
            i += 1usize;
        }
        return output

    }
}

#[test]
fn schnorr_generation_from_seed(){
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = SchnorrKeypair::new(&seed.as_bytes());
    let signature = keypair.simple_sign(b"This is the Context",b"This message is being signed");
    let is_valid = signature.verify();

    let mut signature2 = keypair.sign_with_rng(b"Hello World");
    signature2.context = b"Hello".to_vec();
    let is_valid2 = signature2.verify();

    println!("Is Valid: {}",is_valid2);
}