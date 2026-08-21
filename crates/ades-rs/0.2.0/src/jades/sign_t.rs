use base64ct::{Base64, Base64UrlUnpadded, Encoding};
use sha2::{Digest, Sha256};

use crate::{digest::DigestAlgorithm, error::AdesError, signer::Signer, tsp::TspClient};

use super::sign::{assemble_jws, sign_inner};

/// Produces a JAdES B-T signature: B-B plus an RFC 3161 timestamp from a TSA.
///
/// The timestamp token is placed in the JWS unprotected header under
/// `etsiU[sigTst].tstTokens`. The message imprint is SHA-256 of the
/// base64url-encoded JWS signature string (matching DSS validation).
///
/// Each `etsiU` element is a base64url-encoded JSON string (DSS format).
/// Inside the JSON, the token value uses standard base64 (not base64url).
///
/// # Errors
///
/// Returns [`AdesError`] if signing or the TSA request fails.
///
/// # Example
///
/// ```no_run
/// use ades::{jades, signer::SoftSigner, tsp::TspClient};
///
/// let signer = SoftSigner::generate(2048).unwrap();
/// let tsa = TspClient::new("https://freetsa.org/tsr");
/// let jws = jades::sign_t(b"hello world", &signer, &tsa).unwrap();
/// ```
#[cfg(feature = "tsp")]
pub fn sign_t<S>(data: &[u8], signer: &S, tsa: &TspClient) -> Result<Vec<u8>, AdesError>
where
    S: Signer,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let c = sign_inner(data, signer)?;
    let unprotected = timestamp_unprotected(&c.sig_b64url, tsa)?;
    Ok(assemble_jws(
        &c.header_b64url,
        &c.payload_b64url,
        &c.sig_b64url,
        &unprotected,
    ))
}

/// Produces a JAdES B-LT signature: B-T plus embedded OCSP revocation data.
///
/// Revocation values are placed under `etsiU[rVals].ocspVals`. If the signing
/// certificate has no OCSP URL (e.g. self-signed test certs), silently returns B-T.
///
/// # Errors
///
/// Returns [`AdesError`] if any step fails.
///
/// # Example
///
/// ```no_run
/// use ades::{jades, signer::SoftSigner, tsp::TspClient, ocsp::OcspClient};
///
/// let signer = SoftSigner::generate(2048).unwrap();
/// let tsa = TspClient::new("https://freetsa.org/tsr");
/// let ocsp = OcspClient::new();
/// let jws = jades::sign_lt(b"hello world", &signer, &tsa, &ocsp).unwrap();
/// ```
#[cfg(all(feature = "tsp", feature = "ocsp"))]
pub fn sign_lt<S>(
    data: &[u8],
    signer: &S,
    tsa: &TspClient,
    ocsp: &crate::ocsp::OcspClient,
) -> Result<Vec<u8>, AdesError>
where
    S: Signer,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    let c = sign_inner(data, signer)?;

    let cert = signer.certificate();
    let ocsp_resp = match ocsp.raw_response(cert, cert) {
        Ok(resp) => Some(resp),
        Err(AdesError::Ocsp(_)) => None,
        Err(e) => return Err(e),
    };

    let unprotected = match ocsp_resp {
        Some(ocsp_bytes) => {
            let tst_entry = tst_etsi_u_entry(&c.sig_b64url, tsa)?;
            let rvals_json = format!(
                r#"{{"rVals":{{"ocspVals":["{}"]}}}}"#,
                Base64::encode_string(&ocsp_bytes)
            );
            let rvals_entry = Base64UrlUnpadded::encode_string(rvals_json.as_bytes());
            format!(r#"{{"etsiU":["{tst_entry}","{rvals_entry}"]}}"#)
        }
        None => timestamp_unprotected(&c.sig_b64url, tsa)?,
    };

    Ok(assemble_jws(
        &c.header_b64url,
        &c.payload_b64url,
        &c.sig_b64url,
        &unprotected,
    ))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns a base64url-encoded JSON string for the sigTst etsiU entry.
///
/// DSS JAdES format: each etsiU element is `base64url(JSON string)`.
/// The message imprint is SHA-256 of the base64url-encoded JWS signature string
/// (not the raw bytes), matching DSS validation behaviour.
/// Inside the JSON, the TSR DER bytes use standard base64 (not base64url).
#[cfg(feature = "tsp")]
fn tst_etsi_u_entry(sig_b64url: &str, tsa: &TspClient) -> Result<String, AdesError> {
    let imprint = Sha256::digest(sig_b64url.as_bytes());
    let tsr = tsa.timestamp(&imprint, DigestAlgorithm::Sha256)?;
    // Inner val: standard base64 of the DER-encoded TimeStampToken
    let tsr_b64 = Base64::encode_string(&tsr);
    let sig_tst_json = format!(r#"{{"sigTst":{{"tstTokens":[{{"val":"{tsr_b64}"}}]}}}}"#);
    Ok(Base64UrlUnpadded::encode_string(sig_tst_json.as_bytes()))
}

#[cfg(feature = "tsp")]
fn timestamp_unprotected(sig_b64url: &str, tsa: &TspClient) -> Result<String, AdesError> {
    let entry = tst_etsi_u_entry(sig_b64url, tsa)?;
    Ok(format!(r#"{{"etsiU":["{entry}"]}}"#))
}
