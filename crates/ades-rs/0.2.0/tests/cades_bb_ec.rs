/// CAdES B-B roundtrip integration test — software ECDSA P-256 signer.
#[test]
fn cades_bb_roundtrip_ec() {
    use ades::{cades, signer::SoftSigner};

    // 1. Generate a P-256 key pair in memory
    let signer = SoftSigner::generate_ec().expect("EC key generation failed");

    // 2. Sign b"hello world" as CAdES B-B
    let data = b"hello world ec";
    let signed = cades::sign(data, &signer).expect("CAdES B-B signing failed");

    // 3. Basic structural check: must start with a DER SEQUENCE tag (0x30)
    assert!(!signed.is_empty(), "signature must not be empty");
    assert_eq!(signed[0], 0x30, "CMS ContentInfo must be a DER SEQUENCE");
}

/// `from_ec_parts` roundtrip — mirrors the `eidas-testenv` use case of loading a P-256
/// key that already exists on disk (e.g. PKCS#8 PEM) alongside its certificate.
#[test]
fn cades_bb_from_ec_parts_roundtrip() {
    use ades::{cades, digest::DigestAlgorithm, signer::SoftSigner};

    let (signing_key, cert_der) = self_signed_ec_key_and_cert();

    let signer = SoftSigner::from_ec_parts(signing_key, &cert_der, DigestAlgorithm::Sha256)
        .expect("from_ec_parts failed");

    let data = b"hello world ec from_ec_parts";
    let signed = cades::sign(data, &signer).expect("CAdES B-B signing failed");

    assert!(!signed.is_empty(), "signature must not be empty");
    assert_eq!(signed[0], 0x30, "CMS ContentInfo must be a DER SEQUENCE");
}

/// Builds a P-256 key pair and a matching self-signed certificate, independently of
/// `SoftSigner::generate_ec`, so `from_ec_parts` is exercised with externally-supplied
/// key material rather than one produced by the crate's own generator.
fn self_signed_ec_key_and_cert() -> (p256::ecdsa::SigningKey, Vec<u8>) {
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
    let pub_key_doc = signing_key
        .verifying_key()
        .to_public_key_der()
        .expect("SPKI encoding failed");
    let spki = SubjectPublicKeyInfoOwned::from_der(pub_key_doc.as_bytes()).unwrap();

    let validity = Validity::from_now(std::time::Duration::from_secs(60 * 60 * 24 * 365 * 10))
        .expect("validity construction failed");
    let subject = RdnSequence::default();
    let serial = SerialNumber::from(1u32);

    let builder =
        CertificateBuilder::new(Profile::Root, serial, validity, subject, spki, &signing_key)
            .expect("certificate builder construction failed");
    let cert = builder
        .build::<DerSignature>()
        .expect("certificate build failed");

    (
        signing_key,
        cert.to_der().expect("cert DER encoding failed"),
    )
}
