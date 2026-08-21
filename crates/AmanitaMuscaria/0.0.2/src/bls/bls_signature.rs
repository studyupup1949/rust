use bls_signatures::Serialize;
use bls_signatures::*;
use hex::*;
use base64::*;

use crate::traits::signature::DigitalSignatureTrait;
use crate::constants::bls_constants::*;

use crate::prelude::*;
use crate::os_random::OsRandom;

use crate::encoding::EncodingAPI;

pub struct BLSKeypair {
    pk: [u8;BLS_PK_SIZE_IN_BYTES],
    sk: [u8;BLS_SK_SIZE_IN_BYTES],
}

pub struct VerifyBLSSignature;

impl VerifyBLSSignature {
    /// Verifies signatures with prepended keys to the message. Accepts as input the messages (as Strings)
    pub fn verify_prepended_keys<T: AsRef<str>>(messages: Vec<String>,signature: T) -> bool {
        let mut pk: Vec<String> = vec![];
        let mut messages_as_bytes: Vec<&[u8]> = vec![];

        let mut hex_limit: usize = BLS_PK_SIZE_IN_BYTES * 2usize;
        
        for x in messages.iter() {
            let single_pk: String = x[0..hex_limit].to_ascii_uppercase();
            println!("Single PK: {}",single_pk);
            pk.push(single_pk);
        }

        for x in messages.iter() {
            messages_as_bytes.push(x.as_bytes());
        }

        Self::verify_with_hex(pk, messages_as_bytes, signature.as_ref().to_string())
    }
    pub fn verify<T: AsRef<[u8]>>(pk: Vec<&[u8]>,message: Vec<T>,signature: String) {
        // TODO: Do same as below without decoding from hex
    }
    pub fn verify_with_hex(pk: Vec<String>, messages: Vec<&[u8]>, signature: String) -> bool {
        let mut is_signing_single_message_with_multiple_pks: bool = false;
        let mut is_single_pk: bool = false;

        // Checks whether there are no inputs
        if pk.len() == 0usize || messages.len() == 0usize {
            panic!("No Public Key or Message Available")
        }

        // Checks if there is only one message
        if pk.len() == 1usize {
            is_single_pk = true;
        }

        // Checks if message is aggregated
        if pk.len() != messages.len() && messages.len() == 1usize {
            is_signing_single_message_with_multiple_pks = true;
        }

        let base64_decoded = EncodingAPI::decode_from_base64(&signature);
        let output_sig = bls_signatures::Signature::from_bytes(&base64_decoded).expect("Failed To Convert From Bytes To Signature In Verification Function For Signature");



        if is_single_pk == true {
            println!("Single PK");
            // Decode
            let hex_decoded = EncodingAPI::decode_from_hex(&pk[0]);

            // Construct Public Key and Signature
            let single_pk = bls_signatures::PublicKey::from_bytes(&hex_decoded).expect("Failed To Convert From Bytes To Signature In Verification Function For Public Key");
            //let signature = bls_signatures::Signature::from_bytes(&base64_decoded).expect("Failed To Convert From Bytes To Signature In Verification Function For Signature");
        
            let is_valid: bool = bls_signatures::verify_messages(&output_sig, &[&messages[0].as_ref()], &[single_pk]);
            return is_valid
        }
        else if is_single_pk == false && is_signing_single_message_with_multiple_pks == false {
            println!("Signing Multiple Message");
            let mut public_keys: Vec<bls_signatures::PublicKey> = vec![];
            
            let mut i: usize = 0;
            
            for x in pk {
                let hex_decoded = EncodingAPI::decode_from_hex(x);
                let single_pk = bls_signatures::PublicKey::from_bytes(&hex_decoded).expect("Failed To Convert From Bytes To Signature In Verification Function For Public Key");

                public_keys.push(single_pk);

                i += 1usize;
            }

            // TODO: Must be error with messages

            let is_valid: bool = bls_signatures::verify_messages(&output_sig, &messages, &public_keys);

            return is_valid


        }
        else if is_single_pk == false && is_signing_single_message_with_multiple_pks == true {
            println!("Signing Single Messages with multiple public keys");
            let mut public_keys: Vec<bls_signatures::PublicKey> = vec![];
            let mut message_vec = vec![];

            let num_of_pks: usize = pk.len();

            for x in 0..num_of_pks {
                message_vec.push(messages[0])
            }

            let mut i: usize = 0;

            for x in pk {
                let hex_decoded = EncodingAPI::decode_from_hex(x);
                let single_pk = bls_signatures::PublicKey::from_bytes(&hex_decoded).expect("Failed To Convert From Bytes To Signature In Verification Function For Public Key");

                public_keys.push(single_pk);

                i += 1usize;
            }

            let is_valid: bool = bls_signatures::verify_messages(&output_sig, &message_vec, &public_keys);

            return is_valid
        }
        else {
            panic!("Fail")
        }

        

        
        
    }
    /// Gets message without the prepended key and the underscore
    pub fn get_message_from_prepended_key(message: String) -> String {
        // Removes "PK_"
        let mut prepended_data: usize = BLS_PK_SIZE_IN_BYTES * 2usize + 1;

        let mut data_to_remove: String = message[0..prepended_data].to_string();

        let final_message = message.replace(&data_to_remove,"");

        return final_message
        
    }
}

// Good; Audited
impl BLSKeypair {
    pub fn return_sk_as_hex(&self) -> String {
        let output = hex::encode(&self.sk);
        assert_eq!(output.len(),BLS_HEX_SK_SIZE_IN_BYTES);

        return output
    }
    pub fn from_hex<T: AsRef<str>>(pk: T,sk: T) -> Self {
        let mut pk_output: [u8;BLS_PK_SIZE_IN_BYTES] = [0u8;BLS_PK_SIZE_IN_BYTES];
        let mut sk_output: [u8;BLS_SK_SIZE_IN_BYTES] = [0u8;BLS_SK_SIZE_IN_BYTES];
        
        let pk = hex::decode(pk.as_ref()).expect("Failed To Decode Public Key From Hex For BLS Keypair");
        let sk = hex::decode(sk.as_ref()).expect("Failed To Decode Secret Key From Hex For BLS Keypair");

        let mut i: usize = 0;

        for x in pk {
            pk_output[i] = x;
            i += 1;
        }

        i = 0;

        for x in sk {
            sk_output[i] = x;
            i += 1;
        }

        return Self {
            pk: pk_output,
            sk: sk_output,
        }
    }
    pub fn aggregate(sigs: &[String]) -> String {
        // Initialize List of Signatures
        let mut list_of_sigs: Vec<bls_signatures::Signature> = vec![];
        
        // Decode, Construct, Push to List of Sigs
        for x in sigs {
            let decoded_sig = EncodingAPI::decode_from_base64(x);
            let constructed_sig = bls_signatures::Signature::from_bytes(&decoded_sig).expect("[Error] Failed To Construct Signature From Bytes");
            list_of_sigs.push(constructed_sig);
        }

        let output = base64::encode(bls_signatures::aggregate(&list_of_sigs).expect("Failed To Aggregate Signatures For BLS").as_bytes());

        return output
    }
    /// Returns
    /// 
    /// 1.Message
    /// 
    /// 2.Signature (Base64)
    pub fn sign_by_prepending_key<T: AsRef<str>>(self,message: T) -> (String,String) {
        let mut msg: String = String::new();
        
        let mut key: String = self.return_pk_as_hex();

        key.push_str(BLS_CHARACTER_IN_BETWEEN_MESSAGE_WITH_PK);

        msg.push_str(&key);
        msg.push_str(message.as_ref());

        let key = bls_signatures::PrivateKey::from_bytes(&self.sk).expect("[Error] Failed To Deserialize Private Key For BLS");

        let signature = key.sign(msg.clone());

        let final_signature = base64::encode(signature.as_bytes());

        return (msg.clone(),final_signature)
    }
}

impl DigitalSignatureTrait for BLSKeypair {
    fn new(seed: &[u8]) -> Self {
        // Initialize Secret Key and Public Key arrays
        let mut sk: [u8;BLS_SK_SIZE_IN_BYTES] = [0u8;BLS_SK_SIZE_IN_BYTES];
        let mut pk: [u8;BLS_PK_SIZE_IN_BYTES] = [0u8;BLS_PK_SIZE_IN_BYTES];

        // i
        let mut i: usize = 0;

        // Asserts Seed is 64 bytes in size
        assert_eq!(seed.len(),64usize);

        // Derive Secret Key From Seed
        let secret_key = bls_signatures::PrivateKey::new(seed);

        // Convert Keys To Bytes
        let secret_key_bytes = secret_key.as_bytes();
        let public_key_bytes = secret_key.public_key().as_bytes();

        let mut i: usize = 0;

        for x in secret_key_bytes {
            sk[i] = x;
            i += 1usize;
        }

        let mut i: usize = 0;

        for x in public_key_bytes {
            pk[i] = x;
            i += 1usize;
        }

        return Self {
            pk: pk,
            sk: sk,
        }
    }
    fn generate() -> Self {
        // Initialize Secret Key and Public Key arrays
        let mut sk: [u8;BLS_SK_SIZE_IN_BYTES] = [0u8;BLS_SK_SIZE_IN_BYTES];
        let mut pk: [u8;BLS_PK_SIZE_IN_BYTES] = [0u8;BLS_PK_SIZE_IN_BYTES];

        // i
        let mut i: usize = 0;

        let randomness = OsRandom::rand_64().expect("[Error] Failed To Get Randomness");

        // Derive Secret Key From Seed
        let secret_key = bls_signatures::PrivateKey::new(randomness);

        // Convert Keys To Bytes
        let secret_key_bytes = secret_key.as_bytes();
        let public_key_bytes = secret_key.public_key().as_bytes();

        let mut i: usize = 0;

        for x in secret_key_bytes {
            sk[i] = x;
            i += 1usize;
        }

        let mut i: usize = 0;

        for x in public_key_bytes {
            pk[i] = x;
            i += 1usize;
        }

        return Self {
            pk: pk,
            sk: sk,
        }
    }
    fn sign<T: AsRef<[u8]>>(&self, message: T) -> String {
        // Construct Key From Secret Key + Sign Message
        let key = bls_signatures::PrivateKey::from_bytes(&self.sk).expect("[Error] Failed To Deserialize Private Key For BLS");
        let signature = key.sign(message);

        // Asserts Signature is 96 bytes
        assert_eq!(signature.as_bytes().len(),96usize);

        // Encoded In Hexadecimal
        let final_signature = base64::encode(signature.as_bytes());

        return final_signature
    }

    fn return_pk_as_bytes(&self) -> &[u8] {
        return &self.pk
    }
    fn return_pk_as_hex(&self) -> String {
        return hex::encode_upper(self.pk)
    }
}

#[test]
fn test_bls_generation(){
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = BLSKeypair::new(&seed.as_bytes());
}

#[test]
fn test_signing_true(){
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = BLSKeypair::new(&seed.as_bytes());

    let pk = keypair.return_pk_as_hex();

    let sig = keypair.sign("Hello World.");

    let is_valid: bool = VerifyBLSSignature::verify_with_hex(vec![pk], vec![b"Hello World."], sig);

    assert!(is_valid);
}

#[test]
fn test_signing_invalid() {
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = BLSKeypair::new(&seed.as_bytes());

    let pk = keypair.return_pk_as_hex();

    let sig = keypair.sign("Hello World.");

    let is_valid: bool = VerifyBLSSignature::verify_with_hex(vec![pk], vec![b"Other"], sig);

    assert_eq!(is_valid,false);

}

// This test has the following:
// - 3 Public Keys
// - 3 Unique Messages
// - 3 Signatures aggregated into 1
#[test]
fn test_bls_aggregation_with_multiple_messages(){

    // Issue1: GOOD
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = BLSKeypair::new(&seed.as_bytes());

    let pk = keypair.return_pk_as_hex();

    let sig = keypair.sign("Message 1");

    //

    // Issue1: GOOD
    let phrase2 = BIP39API::generate(Language::English);
    let seed2 = phrase2.derive_seed("Password123", Language::English);

    let keypair2 = BLSKeypair::new(&seed2.as_bytes());

    let pk2 = keypair2.return_pk_as_hex();

    let sig2 = keypair2.sign("Message 2");

    //

    let phrase3 = BIP39API::generate(Language::English);
    let seed3 = phrase3.derive_seed("Password1234", Language::English);

    let keypair3 = BLSKeypair::new(&seed3.as_bytes());

    let pk3 = keypair3.return_pk_as_hex();

    let sig3 = keypair3.sign("Message 3");

    let output = BLSKeypair::aggregate(&vec![sig,sig2,sig3]);

    let is_valid = VerifyBLSSignature::verify_with_hex(vec![pk,pk2,pk3], vec![b"Message 1",b"Message 2",b"Message 3"], output);

    println!("is_valid for Aggregation with single message: {}",is_valid);

    assert!(is_valid);

}

#[test]
fn test_bls_aggregation_with_single_message_done_multiple_times(){

    // Issue1: GOOD
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = BLSKeypair::new(&seed.as_bytes());

    let pk = keypair.return_pk_as_hex();

    let sig = keypair.sign(b"m");

    //

    // Issue1: GOOD
    let phrase2 = BIP39API::generate(Language::English);
    let seed2 = phrase2.derive_seed("Password123", Language::English);

    let keypair2 = BLSKeypair::new(&seed2.as_bytes());

    let pk2 = keypair2.return_pk_as_hex();

    let sig2 = keypair2.sign(b"m");

    //

    let phrase3 = BIP39API::generate(Language::English);
    let seed3 = phrase3.derive_seed("Password12345", Language::English);
    let keypair3 = BLSKeypair::new(&seed3.as_bytes());
    let pk3 = keypair3.return_pk_as_hex();
    let sig3 = keypair3.sign(b"m");

    let output = BLSKeypair::aggregate(&vec![sig,sig2,sig3]);
    let is_valid = VerifyBLSSignature::verify_with_hex(vec![pk,pk2,pk3], vec![b"m",b"m",b"m"], output);

    println!("is_valid for Aggregation with single message: {}",is_valid);

    assert!(is_valid);

}

// This test has the following:
// - 3 Public Keys
// - 1 Unique Message
// - 3 Signatures aggregated into 1
#[test]
fn test_bls_aggregation_with_single_message(){

    // Issue1: GOOD
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = BLSKeypair::new(&seed.as_bytes());

    let pk = keypair.return_pk_as_hex();

    let sig = keypair.sign("Message");

    //

    // Issue1: GOOD
    let phrase2 = BIP39API::generate(Language::English);
    let seed2 = phrase2.derive_seed("Password123", Language::English);

    let keypair2 = BLSKeypair::new(&seed2.as_bytes());

    let pk2 = keypair2.return_pk_as_hex();

    let sig2 = keypair2.sign("Message");

    //

    let phrase3 = BIP39API::generate(Language::English);
    let seed3 = phrase3.derive_seed("Password1234", Language::English);

    let keypair3 = BLSKeypair::new(&seed3.as_bytes());

    let pk3 = keypair3.return_pk_as_hex();

    let sig3 = keypair3.sign("Message");

    let output = BLSKeypair::aggregate(&vec![sig,sig2,sig3]);

    let is_valid = VerifyBLSSignature::verify_with_hex(vec![pk,pk2,pk3], vec![b"Message"], output);

    println!("is_valid for Aggregation with single message: {}",is_valid);

    assert!(is_valid);

}

#[test]
fn test_bls_aggregation_with_single_message_done_twice(){

    // Issue1: GOOD
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = BLSKeypair::new(&seed.as_bytes());

    let pk = keypair.return_pk_as_hex();

    let sig = keypair.sign("Message");

    //

    // Issue1: GOOD
    let phrase2 = BIP39API::generate(Language::English);
    let seed2 = phrase2.derive_seed("Password123", Language::English);

    let keypair2 = BLSKeypair::new(&seed2.as_bytes());

    let pk2 = keypair2.return_pk_as_hex();

    let sig2 = keypair2.sign("Message");

    //

    let phrase3 = BIP39API::generate(Language::English);
    let seed3 = phrase3.derive_seed("Password1234", Language::English);

    let keypair3 = BLSKeypair::new(&seed3.as_bytes());

    let pk3 = keypair3.return_pk_as_hex();

    let sig3 = keypair3.sign("Message 2");

    let output = BLSKeypair::aggregate(&vec![sig,sig2,sig3]);

    let is_valid = VerifyBLSSignature::verify_with_hex(vec![pk,pk2,pk3], vec![b"Message",b"Message",b"Message 2"], output);

    println!("is_valid for Aggregation with single message: {}",is_valid);

    assert!(is_valid);

}

#[test]
fn test_signing_one_public_key_multiple_messages(){
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = BLSKeypair::new(&seed.as_bytes());

    let pk = keypair.return_pk_as_hex();

    let pk2 = pk.clone();

    let sig = keypair.sign("Hello World.");

    let sig2 = keypair.sign("Hello World 2.");

    let is_valid: bool = VerifyBLSSignature::verify_with_hex(vec![pk,pk2], vec![b"Hello World.",b"Hello World 2."], sig);

    assert!(is_valid);
}

#[test]
fn prepended_keys(){
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);
    let keypair = BLSKeypair::new(&seed.as_bytes());
    let sig1 = keypair.sign_by_prepending_key("Hello World.");

    // 2
    let phrase2 = BIP39API::generate(Language::English);
    let seed2 = phrase2.derive_seed("Password1234", Language::English);
    let keypair2 = BLSKeypair::new(&seed2.as_bytes());
    let sig2 = keypair2.sign_by_prepending_key("Hello World.");


    let phrase3 = BIP39API::generate(Language::English);
    let seed3 = phrase3.derive_seed("Password1234", Language::English);
    let keypair3= BLSKeypair::new(&seed3.as_bytes());
    let sig3 = keypair3.sign_by_prepending_key("Hello World.");

    let final_sig = BLSKeypair::aggregate(&vec![sig1.1.clone(),sig2.1,sig3.1]);

    println!("Keypair 1 (Signed Message): {}",sig1.0);
    println!("Keypair 1 (Signature): {}",sig1.1.clone());

    let msg = VerifyBLSSignature::get_message_from_prepended_key(sig1.0.clone());
    println!("Message Without PK: {}",msg);

    let is_valid = VerifyBLSSignature::verify_prepended_keys(vec![sig1.0.clone(),sig2.0,sig3.0], final_sig);

    println!("Aggregate BLS Verify Prepended Keys Validility: {}",is_valid);

}