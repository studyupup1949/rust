use crate::{certificate::Certificate, digest::DigestAlgorithm, error::AdesError};

/// Abstraction over a signing key, compatible with software keys, DNIe, HSM, and WebCrypto.
///
/// The private key never leaves the device: only the digest is passed to `sign_digest`.
/// This design ensures hardware tokens (PKCS#11, WebCrypto) can implement this trait
/// without ever exposing the raw key material.
///
/// # Example
///
/// ```no_run
/// use ades::signer::{Signer, SoftSigner};
///
/// let signer = SoftSigner::generate(2048).unwrap();
/// ```
pub trait Signer {
    /// The error type returned by signing operations.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Signs `digest` (the pre-computed hash of the data) and returns the raw signature bytes.
    ///
    /// The digest length must match the algorithm returned by [`Self::digest_algorithm`].
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the signing operation fails.
    fn sign_digest(&self, digest: &[u8]) -> Result<Vec<u8>, Self::Error>;

    /// Returns the signer's certificate. Used to embed the signing certificate in the signature.
    fn certificate(&self) -> &Certificate;

    /// Returns the digest algorithm used by this signer.
    fn digest_algorithm(&self) -> DigestAlgorithm;
}

/// Private key material held by [`SoftSigner`], keyed by algorithm family.
#[cfg(feature = "soft")]
enum SoftKey {
    Rsa(Box<rsa::RsaPrivateKey>),
    Ec(p256::ecdsa::SigningKey),
}

/// Software signing backend — holds the private key in memory.
///
/// Supports RSA (PKCS#1 v1.5) and EC (ECDSA P-256) keys.
///
/// Intended for testing and development. Do not use in production with real keys.
///
/// Enabled only when the `soft` feature is active (on by default).
#[cfg(feature = "soft")]
pub struct SoftSigner {
    key: SoftKey,
    certificate: Certificate,
    digest: DigestAlgorithm,
}

#[cfg(feature = "soft")]
impl SoftSigner {
    /// Generates a fresh RSA key pair of `bits` size and a self-signed certificate.
    ///
    /// `bits` should be 2048 or 4096. Using 2048 is sufficient for testing.
    ///
    /// # Errors
    ///
    /// Returns [`AdesError`] if key generation or certificate construction fails.
    pub fn generate(bits: usize) -> Result<Self, AdesError> {
        use der::{Decode, Encode};
        use rand_core::OsRng;
        use rsa::pkcs1v15::SigningKey;
        use sha2::Sha256;
        use signature::Keypair;
        use spki::{EncodePublicKey, SubjectPublicKeyInfoOwned};
        use x509_cert::{
            builder::{Builder, CertificateBuilder, Profile},
            name::RdnSequence,
            serial_number::SerialNumber,
            time::Validity,
        };

        let private_key = rsa::RsaPrivateKey::new(&mut OsRng, bits).map_err(AdesError::Rsa)?;
        let signing_key = SigningKey::<Sha256>::new(private_key.clone());

        // Extract SPKI from the verifying key
        let verifying_key = signing_key.verifying_key();
        let pub_key_doc = verifying_key
            .to_public_key_der()
            .map_err(|e| AdesError::Signer(Box::new(e)))?;
        let spki = SubjectPublicKeyInfoOwned::from_der(pub_key_doc.as_bytes())?;

        // 10-year validity, empty subject (self-signed test cert)
        let validity = Validity::from_now(std::time::Duration::from_secs(60 * 60 * 24 * 365 * 10))?;
        let subject = RdnSequence::default();
        let serial = SerialNumber::from(1u32);

        let builder =
            CertificateBuilder::new(Profile::Root, serial, validity, subject, spki, &signing_key)?;

        let cert = builder.build::<rsa::pkcs1v15::Signature>()?;
        let cert_der = cert.to_der()?;
        let certificate = Certificate::from_der(&cert_der)?;

        Ok(Self {
            key: SoftKey::Rsa(Box::new(private_key)),
            certificate,
            digest: DigestAlgorithm::Sha256,
        })
    }

    /// Creates a `SoftSigner` from an existing RSA private key and DER-encoded certificate.
    ///
    /// # Errors
    ///
    /// Returns [`AdesError`] if the certificate DER is invalid.
    pub fn from_parts(
        private_key: rsa::RsaPrivateKey,
        cert_der: &[u8],
        digest: DigestAlgorithm,
    ) -> Result<Self, AdesError> {
        let certificate = Certificate::from_der(cert_der)?;
        Ok(Self {
            key: SoftKey::Rsa(Box::new(private_key)),
            certificate,
            digest,
        })
    }

    /// Generates a fresh P-256 (ECDSA) key pair and a self-signed certificate.
    ///
    /// # Errors
    ///
    /// Returns [`AdesError`] if certificate construction fails.
    ///
    /// # Example
    ///
    /// ```
    /// use ades::signer::SoftSigner;
    ///
    /// let signer = SoftSigner::generate_ec().unwrap();
    /// ```
    pub fn generate_ec() -> Result<Self, AdesError> {
        use der::{Decode, Encode};
        use p256::ecdsa::{DerSignature, SigningKey};
        use rand_core::OsRng;
        use spki::{EncodePublicKey, SubjectPublicKeyInfoOwned};
        use x509_cert::{
            builder::{Builder, CertificateBuilder, Profile},
            name::RdnSequence,
            serial_number::SerialNumber,
            time::Validity,
        };

        let signing_key = SigningKey::random(&mut OsRng);

        // Extract SPKI from the verifying key
        let verifying_key = signing_key.verifying_key();
        let pub_key_doc = verifying_key
            .to_public_key_der()
            .map_err(|e| AdesError::Signer(Box::new(e)))?;
        let spki = SubjectPublicKeyInfoOwned::from_der(pub_key_doc.as_bytes())?;

        // 10-year validity, empty subject (self-signed test cert)
        let validity = Validity::from_now(std::time::Duration::from_secs(60 * 60 * 24 * 365 * 10))?;
        let subject = RdnSequence::default();
        let serial = SerialNumber::from(1u32);

        let builder =
            CertificateBuilder::new(Profile::Root, serial, validity, subject, spki, &signing_key)?;

        let cert = builder.build::<DerSignature>()?;
        let cert_der = cert.to_der()?;
        let certificate = Certificate::from_der(&cert_der)?;

        Ok(Self {
            key: SoftKey::Ec(signing_key),
            certificate,
            digest: DigestAlgorithm::Sha256,
        })
    }

    /// Creates a `SoftSigner` from an existing P-256 (ECDSA) private key and DER-encoded
    /// certificate.
    ///
    /// # Errors
    ///
    /// Returns [`AdesError`] if the certificate DER is invalid.
    pub fn from_ec_parts(
        signing_key: p256::ecdsa::SigningKey,
        cert_der: &[u8],
        digest: DigestAlgorithm,
    ) -> Result<Self, AdesError> {
        let certificate = Certificate::from_der(cert_der)?;
        Ok(Self {
            key: SoftKey::Ec(signing_key),
            certificate,
            digest,
        })
    }
}

#[cfg(feature = "soft")]
impl Signer for SoftSigner {
    type Error = AdesError;

    fn sign_digest(&self, digest: &[u8]) -> Result<Vec<u8>, Self::Error> {
        match &self.key {
            SoftKey::Rsa(private_key) => {
                use rsa::pkcs1v15::SigningKey;
                use rsa::signature::hazmat::PrehashSigner;
                use rsa::signature::SignatureEncoding;
                use sha2::Sha256;

                let signing_key = SigningKey::<Sha256>::new((**private_key).clone());
                let signature: rsa::pkcs1v15::Signature = signing_key.sign_prehash(digest)?;
                Ok(signature.to_vec())
            }
            // ECDSA: `sign_prehash` produces the DER-encoded ECDSA-Sig-Value directly
            // (SEQUENCE { r INTEGER, s INTEGER }), which is exactly the format CMS expects
            // in `SignerInfo.signature` — no manual r||s → DER conversion needed.
            SoftKey::Ec(signing_key) => {
                use p256::ecdsa::signature::hazmat::PrehashSigner;
                use p256::ecdsa::signature::SignatureEncoding;
                use p256::ecdsa::DerSignature;

                let signature: DerSignature = signing_key.sign_prehash(digest)?;
                Ok(signature.to_vec())
            }
        }
    }

    fn certificate(&self) -> &Certificate {
        &self.certificate
    }

    fn digest_algorithm(&self) -> DigestAlgorithm {
        self.digest
    }
}
