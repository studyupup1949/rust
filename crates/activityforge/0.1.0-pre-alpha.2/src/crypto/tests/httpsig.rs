use bytes::Bytes;
use http::Request;

use httpsig::prelude::message_component::HttpMessageComponentId;
use httpsig::prelude::{AlgorithmName, HttpSignatureParams, PublicKey, SecretKey};

use crate::Result;
use crate::crypto::{DigestAlgorithm, HttpContentDigest, HttpMessageSignature};

const ED25519_PUBLIC_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEAJrQLj5P/89iXES9+vFgrIy29clF9CC/oPPsw3c5D0bs=
-----END PUBLIC KEY-----
"#;

const ED25519_PRIVATE_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIJ+DYvh6SEqVTm50DFtMDoQikTmiCqirVv9mWG9qfSnF
-----END PRIVATE KEY-----"#;

#[test]
fn test_ed25519_rfc_request() -> Result<()> {
    let body = br#"{"hello": "world"}"#;

    let req = Request::builder()
        .method("POST")
        .uri("https://example.com/foo")
        .header("date", "Tue, 20 Apr 2021 02:07:55 GMT")
        .header("content-type", "application/json")
        .header("Signature-Input", r##"sig-b26=("date" "@method" "@path" "@authority" "content-type" "content-length");created=1618884473;keyid="test-key-ed25519""##)
        .header("Signature", "sig-b26=:wqcAqbmYJ2ji2glfAMaRy4gruYYnx2nEFN2HN6jrnDnQCK1u02Gb04v9EDgwUPiu4A0w6vuQv5lIp5WPpBKRCw==:")
        .body(Bytes::from(body.as_ref()))?;

    let pubkey = PublicKey::from_pem(&AlgorithmName::Ed25519, ED25519_PUBLIC_PEM)?;
    let key_id = "test-key-ed25519";

    let no_filter: Option<&str> = None;

    req.verify_message_signature(&pubkey, no_filter)?;
    req.verify_message_signature(&pubkey, Some(key_id))?;

    Ok(())
}

#[tokio::test]
async fn test_ed25519_full_coverage() -> Result<()> {
    let body = br#"{"hello": "world"}"#;

    let req = Request::builder()
        .method("POST")
        .uri("https://example.com/foo?param=Value&Pet=dog")
        .header("date", "Tue, 20 Apr 2021 02:07:55 GMT")
        .header("content-type", "application/json")
        .body(Bytes::from(body.as_ref()))?;

    let mut req = req.set_content_digest(DigestAlgorithm::Sha512).await?;

    let privkey = SecretKey::from_pem(&AlgorithmName::Ed25519, ED25519_PRIVATE_PEM)?;
    let pubkey = PublicKey::from_pem(&AlgorithmName::Ed25519, ED25519_PUBLIC_PEM)?;
    let key_id = "test-key-ed25519";

    let sig_params = HttpSignatureParams::try_new(&[
        HttpMessageComponentId::try_from("date")?,
        HttpMessageComponentId::try_from("@method")?,
        HttpMessageComponentId::try_from("@path")?,
        HttpMessageComponentId::try_from("@query")?,
        HttpMessageComponentId::try_from("@authority")?,
        HttpMessageComponentId::try_from("content-type")?,
        HttpMessageComponentId::try_from("content-digest")?,
        HttpMessageComponentId::try_from("content-length")?,
    ])
    .map(|mut s| {
        s.set_keyid(key_id)
            .set_alg(&AlgorithmName::Ed25519)
            .set_created(1618884473);
        s
    })?;

    let sig_name = "isig-b23";

    req.set_message_signature(&sig_params, &privkey, Some(sig_name))?;
    req.verify_message_signature(&pubkey, Some(sig_name))?;

    let sig_input = r#"isig-b23=("date" "@method" "@path" "@query" "@authority" "content-type" "content-digest" "content-length");created=1618884473;alg="ed25519";keyid="test-key-ed25519""#;
    assert_eq!(
        req.headers()
            .get("signature-input")
            .unwrap()
            .to_str()
            .unwrap(),
        sig_input
    );
    assert!(
        req.headers()
            .get("signature")
            .unwrap()
            .to_str()
            .unwrap()
            .contains(sig_name)
    );

    req.verify_content_digest().await?;

    Ok(())
}
