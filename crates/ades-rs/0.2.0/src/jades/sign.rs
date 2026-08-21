use base64ct::{Base64, Base64UrlUnpadded, Encoding};
use sha2::{Digest, Sha512};

use crate::{error::AdesError, signer::Signer};

const RSA_OID: &str = "1.2.840.113549.1.1.1";
const EC_OID: &str = "1.2.840.10045.2.1";

/// Internal JWS components shared between B-B, B-T, and B-LT builders.
pub(super) struct JwsComponents {
    pub(super) header_b64url: String,
    pub(super) payload_b64url: String,
    pub(super) sig_b64url: String,
}

/// Produces a JAdES B-B signature over `data` in JWS JSON Serialization format.
///
/// The protected header follows the DSS/JAdES format with `alg`, `cty`, `iat`,
/// `typ`, `x5c`, and `x5t#o` (SHA-512 cert thumbprint).
/// The unprotected header contains an empty `etsiU` array, required by DSS for
/// JAdES classification.
///
/// # Errors
///
/// Returns [`AdesError`] if signing fails or system time is unavailable.
///
/// # Example
///
/// ```no_run
/// use ades::{jades, signer::SoftSigner};
///
/// let signer = SoftSigner::generate(2048).unwrap();
/// let jws = jades::sign(b"hello world", &signer).unwrap();
/// ```
pub fn sign<S: Signer>(data: &[u8], signer: &S) -> Result<Vec<u8>, AdesError>
where
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let c = sign_inner(data, signer)?;
    // etsiU must be present (even if empty) for DSS JAdES classification
    Ok(assemble_jws(
        &c.header_b64url,
        &c.payload_b64url,
        &c.sig_b64url,
        r#"{"etsiU":[]}"#,
    ))
}

/// Builds JWS components without assembling the final JSON document.
///
/// Used by `sign`, `sign_t`, and `sign_lt` to share the core signing logic.
pub(super) fn sign_inner<S: Signer>(data: &[u8], signer: &S) -> Result<JwsComponents, AdesError>
where
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let cert = signer.certificate();
    let cert_der = cert.to_der();
    let iat = unix_now()?;

    let key_alg_oid = cert
        .inner()
        .tbs_certificate
        .subject_public_key_info
        .algorithm
        .oid
        .to_string();

    let (alg, is_ecdsa) = match key_alg_oid.as_str() {
        RSA_OID => ("RS256", false),
        EC_OID => ("ES256", true),
        _ => {
            return Err(AdesError::NotImplemented(
                "JAdES: unsupported key type — only RSA and EC are supported",
            ))
        }
    };

    // x5c: regular base64 per RFC 7517 §4.7 (NOT base64url)
    let cert_b64 = Base64::encode_string(cert_der);

    // x5t#o: SHA-512 thumbprint of the DER cert, base64url-encoded.
    // DSS JAdES uses this instead of sigX5ts for the signing certificate reference.
    let cert_sha512_b64url = Base64UrlUnpadded::encode_string(&Sha512::digest(cert_der));

    // Protected header keys in alphabetical order.
    // Uses iat (integer Unix timestamp) instead of sigT per DSS JAdES format.
    // Uses x5t#o (SHA-512) instead of sigX5ts per DSS JAdES format.
    let header_json = format!(
        r#"{{"alg":"{alg}","cty":"application/octet-stream","iat":{iat},"typ":"jose+json","x5c":["{cert_b64}"],"x5t#o":{{"digAlg":"S512","digVal":"{cert_sha512_b64url}"}}}}"#
    );

    let header_b64url = Base64UrlUnpadded::encode_string(header_json.as_bytes());
    let payload_b64url = Base64UrlUnpadded::encode_string(data);

    // JWS signing input: BASE64URL(UTF8(protected)) || "." || BASE64URL(payload)
    let signing_input = format!("{header_b64url}.{payload_b64url}");

    let digest = signer.digest_algorithm().hash(signing_input.as_bytes());
    let raw_sig = signer
        .sign_digest(&digest)
        .map_err(|e| AdesError::Signer(Box::new(e)))?;

    // JWS requires ECDSA in IEEE P1363 (raw r||s), not DER
    let sig_bytes = if is_ecdsa {
        ecdsa_der_to_raw(&raw_sig)?
    } else {
        raw_sig
    };

    let sig_b64url = Base64UrlUnpadded::encode_string(&sig_bytes);

    Ok(JwsComponents {
        header_b64url,
        payload_b64url,
        sig_b64url,
    })
}

/// Assembles a JWS JSON Serialization document (general form per RFC 7515 §7.2.1).
pub(super) fn assemble_jws(
    header_b64url: &str,
    payload_b64url: &str,
    sig_b64url: &str,
    unprotected: &str,
) -> Vec<u8> {
    format!(
        r#"{{"payload":"{payload_b64url}","signatures":[{{"protected":"{header_b64url}","header":{unprotected},"signature":"{sig_b64url}"}}]}}"#
    )
    .into_bytes()
}

/// Converts ECDSA DER signature (SEQUENCE { INTEGER r, INTEGER s }) to IEEE P1363 (raw r||s)
/// as required by JWS (RFC 7518 §3.4).
fn ecdsa_der_to_raw(der: &[u8]) -> Result<Vec<u8>, AdesError> {
    let err = || AdesError::NotImplemented("JAdES: malformed ECDSA DER signature");
    if der.len() < 6 || der[0] != 0x30 {
        return Err(err());
    }
    let seq_len = der[1] as usize;
    if der.len() < 2 + seq_len {
        return Err(err());
    }
    let content = &der[2..2 + seq_len];

    if content.len() < 2 || content[0] != 0x02 {
        return Err(err());
    }
    let r_len = content[1] as usize;
    if content.len() < 2 + r_len {
        return Err(err());
    }
    let r_bytes = &content[2..2 + r_len];

    let rest = &content[2 + r_len..];
    if rest.len() < 2 || rest[0] != 0x02 {
        return Err(err());
    }
    let s_len = rest[1] as usize;
    if rest.len() < 2 + s_len {
        return Err(err());
    }
    let s_bytes = &rest[2..2 + s_len];

    let r = strip_leading_zero(r_bytes);
    let s = strip_leading_zero(s_bytes);
    let coord = r.len().max(s.len());

    let mut out = vec![0u8; 2 * coord];
    out[coord - r.len()..coord].copy_from_slice(r);
    out[2 * coord - s.len()..].copy_from_slice(s);
    Ok(out)
}

fn strip_leading_zero(b: &[u8]) -> &[u8] {
    if b.first() == Some(&0x00) {
        &b[1..]
    } else {
        b
    }
}

fn unix_now() -> Result<u64, AdesError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| AdesError::NotImplemented("system time before Unix epoch"))
}
