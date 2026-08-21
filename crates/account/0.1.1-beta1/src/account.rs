use crate::seed;
use bip32::secp256k1::ecdsa::{
    signature::{Signer, Verifier},
    SigningKey, VerifyingKey,
};
use bip32::{secp256k1::ecdsa::Signature, Seed, XPrv};
use x25519_dalek::{self, PublicKey, StaticSecret};

pub struct Account {
    pub seed: Seed,
}

impl Account {
    pub fn new(password: Option<&str>) -> Self {
        let seed = seed::get_seed(password);
        Self { seed: seed }
    }

    pub fn get_root_key(&self) -> SigningKey {
        let root_xprv = XPrv::new(&self.seed).unwrap();
        let private_key = root_xprv.private_key();
        private_key.to_owned()
    }

    /// Gets the key used for the Diffie-Hellman protocol
    pub fn sub_dh_key(&self, child_path: &str) -> (StaticSecret, PublicKey) {
        let child_xprv = XPrv::derive_from_path(&self.seed, &child_path.parse().unwrap()).unwrap();
        let xprv = child_xprv.to_bytes();
        let child_priv_key: [u8; 32] = xprv[0..32].try_into().unwrap();

        let secret_key = StaticSecret::from(child_priv_key);
        let public_key = PublicKey::from(&secret_key);
        (secret_key, public_key)
    }

    /// Gets the key used to sign the message
    pub fn sign_key(&self, child_path: &str) -> Result<(SigningKey, VerifyingKey), bip32::Error> {
        let child_xprv = XPrv::derive_from_path(&self.seed, &child_path.parse()?)?;
        let child_xpub = child_xprv.public_key();
        Ok((child_xprv.into(), child_xpub.into()))
    }

    pub fn sign(&self, child_path: &str, message: &[u8]) -> Result<Signature, bip32::Error> {
        let (signing_key, verification_key) = self.sign_key(child_path).ok().unwrap();
        let signature: Signature = signing_key.sign(message);
        assert!(verification_key.verify(message, &signature).is_ok());
        Ok(signature)
    }

    pub fn verify(&self, child_path: &str, message: &[u8], signature: &Signature) -> bool {
        let (_, verification_key) = self.sign_key(child_path).ok().unwrap();
        verification_key.verify(message, signature).is_ok()
    }
}
