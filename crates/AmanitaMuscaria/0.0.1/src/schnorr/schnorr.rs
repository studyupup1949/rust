//TODO: implement VDFs, multi-signatures with Schnorr
use schnorrkel::{Keypair,SecretKey,PublicKey,Signature};
use schnorrkel::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, KEYPAIR_LENGTH, SIGNATURE_LENGTH};
use zeroize::Zeroize;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;


#[derive(Zeroize,Debug,Clone,Copy,PartialEq)]
pub struct SchnorrKeypair {
    pk: [u8;PUBLIC_KEY_LENGTH],
    sk: [u8;SECRET_KEY_LENGTH],
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
    pub fn simple_sign(&self, context: &[u8], message: &[u8]) -> Signature {
        let keypair = self.construct();
        let sig = keypair.sign_simple_doublecheck(context,message);
        let final_sig = sig.expect("Failed To Sign");
        return final_sig
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