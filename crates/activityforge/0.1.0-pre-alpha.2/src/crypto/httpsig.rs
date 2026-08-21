use std::future::Future;
use std::str::FromStr;

use axum::body::{Body, Bytes};
use base64::Encoding;
use http::HeaderMap;
use indexmap::IndexMap;
use sha2::Digest;

mod keys;

pub use keys::*;

pub use httpsig::prelude::message_component::{
    DerivedComponentName, HttpMessageComponent, HttpMessageComponentId, HttpMessageComponentName,
    HttpMessageComponentParam,
};
pub use httpsig::prelude::{
    AlgorithmName, HttpSignatureBase, HttpSignatureHeaders, HttpSignatureHeadersMap,
    HttpSignatureParams, SigningKey, VerifyingKey,
};

use activitystreams_vocabulary::{impl_default, impl_display};

use crate::{Error, Result};

/// Convenience alias to mark the index used for the header map.
pub type SignatureName = String;

/// Convenience alias to mark a key ID component.
pub type KeyId = String;

/// Convenience alias for a map of signature bases and headers.
pub type SignatureHeaderMap = IndexMap<SignatureName, (HttpSignatureBase, HttpSignatureHeaders)>;

/// Convenience alias for an algorithm-key ID map indexed by signature name.
pub type AlgoKeyIdMap = IndexMap<SignatureName, (Option<AlgorithmName>, Option<KeyId>)>;

/// Performs the complex transformations needed to sign/verify HTTP Message Signatures (RFC 9421).
pub trait HttpMessageSignature {
    /// Gets a reference to the HTTP header map.
    fn headers(&self) -> &HeaderMap;

    /// Gets a mutable reference to the HTTP header map.
    fn headers_mut(&mut self) -> &mut HeaderMap;

    /// Gets the HTTP method.
    ///
    /// Only valid for request types.
    fn method(&self) -> Result<&http::Method>;

    /// Gets the HTTP URI.
    ///
    /// Only valid for request types.
    fn uri(&self) -> Result<&http::Uri>;

    /// Gets the HTTP status.
    ///
    /// Only valid for response types.
    fn status(&self) -> Result<http::StatusCode>;

    /// Gets whether the message is a request type.
    fn is_request(&self) -> bool;

    /// Gets whether the message is a response type.
    fn is_response(&self) -> bool {
        !self.is_request()
    }

    /// Gets whether the headers contain HTTP Message Signature headers.
    fn has_message_signature(&self) -> bool {
        let headers = self.headers();
        headers.contains_key("signature") && headers.contains_key("signature-input")
    }

    /// Sets the HTTP message signature from given http signature params and signing key
    ///
    /// Based on [httpsig-hyper](https://docs.rs/httpsig-hyper)
    fn set_message_signature<T, S>(
        &mut self,
        signature_params: &HttpSignatureParams,
        signing_key: &T,
        signature_name: Option<S>,
    ) -> Result<()>
    where
        T: SigningKey + Sync,
        S: AsRef<str>,
    {
        self.set_message_signatures(&[(signature_params, signing_key, signature_name)])
    }

    /// Sets the HTTP message signature from given list of http signature params and signing keys.
    ///
    /// Based on [httpsig-hyper](https://docs.rs/httpsig-hyper)
    fn set_message_signatures<T, S>(
        &mut self,
        params_key_name: &[(&HttpSignatureParams, &T, Option<S>)],
    ) -> Result<()>
    where
        T: SigningKey + Sync,
        S: AsRef<str>,
    {
        let vec_signature_bases = params_key_name
            .iter()
            .map(|(params, key, name)| {
                self.build_signature_base(params)
                    .map(|base| (base, *key, name.as_ref()))
            })
            .collect::<Result<Vec<_>>>()?;

        for (base, key, name) in vec_signature_bases.into_iter() {
            let headers = base.build_signature_headers(key, name.map(|s| s.as_ref()))?;

            headers
                .signature_input_header_value()
                .parse()
                .map(|h| self.headers_mut().append("signature-input", h))
                .map_err(|err| Error::crypto(format!("httpsig: {err}")))?;

            headers
                .signature_header_value()
                .parse()
                .map(|h| self.headers_mut().append("signature", h))
                .map_err(|err| Error::crypto(format!("httpsig: {err}")))?;
        }

        Ok(())
    }

    /// Verifies a HTTP message signature (RFC 9421).
    ///
    /// Based on [httpsig-hyper](https://docs.rs/httpsig-hyper)
    fn verify_message_signature<T, S>(
        &self,
        key: &T,
        key_id: Option<S>,
    ) -> Result<Vec<Result<SignatureName>>>
    where
        T: VerifyingKey + Sync,
        S: AsRef<str>,
    {
        self.verify_message_signatures([(key, key_id)])
    }

    /// Verifies HTTP message signatures (RFC 9421).
    ///
    /// Based on [httpsig-hyper](https://docs.rs/httpsig-hyper)
    fn verify_message_signatures<'a, I, T, S>(
        &self,
        key_and_id: I,
    ) -> Result<Vec<Result<SignatureName>>>
    where
        I: IntoIterator<Item = (&'a T, Option<S>)>,
        T: VerifyingKey + Sync + 'a,
        S: AsRef<str>,
    {
        let sig_map = self.extract_signatures()?;

        // verify for each key_and_id tuple
        let res = key_and_id
            .into_iter()
            .flat_map(|(key, key_id)| {
                let filtered = if let Some(key_id) = key_id {
                    sig_map
                        .iter()
                        .filter(|(_, (base, _))| base.keyid() == Some(key_id.as_ref()))
                        .collect::<IndexMap<_, _>>()
                } else {
                    sig_map.iter().collect()
                };

                // check if any one of the signature headers is valid
                if filtered.is_empty() {
                    vec![Err(Error::crypto(
                        "httpsig: No signature as appropriate target for verification",
                    ))]
                } else {
                    // check if any one of the signature headers is valid
                    filtered
                        .iter()
                        .map(|(&name, (base, headers))| {
                            base.verify_signature_headers(key, headers)
                                .map(|_| name.clone())
                                .map_err(Error::from)
                        })
                        .collect::<Vec<_>>()
                }
            })
            .collect::<Vec<_>>();

        Ok(res)
    }

    /// Extract all signatures from the headers.
    ///
    /// Based on [httpsig-hyper](https://docs.rs/httpsig-hyper)
    fn extract_signatures(&self) -> Result<SignatureHeaderMap> {
        let signature_headers_map = self.extract_signature_headers_with_name()?;
        let extracted = signature_headers_map
            .iter()
            .filter_map(|(name, headers)| {
                self.build_signature_base(headers.signature_params())
                    .ok()
                    .map(|base| (name.clone(), (base, headers.clone())))
            })
            .collect();
        Ok(extracted)
    }

    /// Extract signature and signature-input with signature-name indication from http request and response
    ///
    /// Based on [httpsig-hyper](https://docs.rs/httpsig-hyper)
    fn extract_signature_headers_with_name(&self) -> Result<HttpSignatureHeadersMap> {
        if !self.has_message_signature() {
            Err(Error::crypto(
                "httpsig: The request does not have signature and signature-input headers",
            ))
        } else {
            let headers = self.headers();

            let sig = headers
                .get_all("signature")
                .iter()
                .map(|v| v.to_str().map_err(Error::from))
                .collect::<Result<Vec<_>>>()
                .map(|h| h.join(", "))?;

            let sig_input = headers
                .get_all("signature-input")
                .iter()
                .map(|v| v.to_str().map_err(Error::from))
                .collect::<Result<Vec<_>>>()
                .map(|h| h.join(", "))?;

            HttpSignatureHeaders::try_parse(&sig, &sig_input).map_err(Error::from)
        }
    }

    /// Build signature base from hyper http request/response and signature params
    ///
    /// - msg: the HTTP request or response
    /// - signature_params: the http signature params
    /// - req_for_param: corresponding request to be considered in the signature base in response
    ///
    /// Based on [httpsig-hyper](https://docs.rs/httpsig-hyper)
    fn build_signature_base(&self, params: &HttpSignatureParams) -> Result<HttpSignatureBase> {
        params
            .covered_components
            .iter()
            .map(|component_id| {
                if self.is_request()
                    && component_id
                        .params
                        .0
                        .contains(&HttpMessageComponentParam::Req)
                {
                    Err(Error::crypto("httpsig: `req` is not allowed in request"))
                } else {
                    self.extract_http_message_component(component_id)
                }
            })
            .collect::<Result<Vec<_>>>()
            .and_then(|lines| HttpSignatureBase::try_new(&lines, params).map_err(Error::from))
    }

    /// Extract http message component from hyper http request
    ///
    /// Based on [httpsig-hyper](https://docs.rs/httpsig-hyper)
    fn extract_http_message_component(
        &self,
        target_component_id: &HttpMessageComponentId,
    ) -> Result<HttpMessageComponent> {
        match &target_component_id.name {
            HttpMessageComponentName::HttpField(_) => self.extract_http_field(target_component_id),
            HttpMessageComponentName::Derived(_) => {
                self.extract_derived_component(target_component_id)
            }
        }
    }

    /// Extract http field from hyper http request/response
    ///
    /// Based on [httpsig-hyper](https://docs.rs/httpsig-hyper)
    fn extract_http_field(&self, id: &HttpMessageComponentId) -> Result<HttpMessageComponent> {
        let HttpMessageComponentName::HttpField(header_name) = &id.name else {
            return Err(Error::crypto(
                "httpsig: invalid http message component name as http field",
            ));
        };

        let field_values = self
            .headers()
            .get_all(header_name)
            .iter()
            .map(|v| v.to_str().map(|s| s.to_owned()).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;

        HttpMessageComponent::try_from((id, field_values.as_slice())).map_err(Error::from)
    }

    /// Extract derived component from http request
    ///
    /// Based on [httpsig-hyper](https://docs.rs/httpsig-hyper)
    fn extract_derived_component(
        &self,
        id: &HttpMessageComponentId,
    ) -> Result<HttpMessageComponent> {
        let HttpMessageComponentName::Derived(derived_id) = &id.name else {
            return Err(Error::crypto(
                "httpsig: invalid http message component name as derived component",
            ));
        };

        // Validate parameters allowed on derived components (RFC 9421).
        // - `name`: only valid on `@query-param`
        // - `req`: only valid on response messages (to reference request-derived components, §2.4)
        // - `sf`, `key`, `bs`, `tr`: only valid on HTTP field components, not derived components
        id.params.0.iter().try_for_each(|param| match param {
            HttpMessageComponentParam::Name(_)
                if matches!(derived_id, DerivedComponentName::QueryParam) =>
            {
                Ok(())
            }
            HttpMessageComponentParam::Name(_) => Err(Error::crypto(
                "httpsig: `name` parameter is only allowed for `@query-param`",
            )),
            // `req` is only meaningful in response signatures (RFC 9421 §2.4).
            // `build_signature_base` already validates this and re-dispatches extraction against the
            // original request, so `req_or_res` here should always be `Request`.
            // Guard against misuse by callers that bypass `build_signature_base`.
            HttpMessageComponentParam::Req if self.is_request() => Ok(()),
            HttpMessageComponentParam::Req => Err(Error::crypto(
                "`req`-tagged component must be extracted from the source request",
            )),
            _ => Err(Error::crypto(format!(
                "httpsig: parameter `{}` is not allowed on derived components",
                String::from(param.clone())
            ))),
        })?;

        if matches!(derived_id, DerivedComponentName::Status) {
            return Err(Error::crypto("httpsig: `status` is only for response"));
        }

        let field_values: Vec<String> = match derived_id {
            DerivedComponentName::Method => self.method().map(|m| vec![m.as_str().to_string()])?,
            DerivedComponentName::TargetUri => self.uri().map(|u| vec![u.to_string()])?,
            DerivedComponentName::Authority => self
                .uri()
                .map(|u| vec![u.authority().map(|s| s.to_string()).unwrap_or_default()])?,
            DerivedComponentName::Scheme => self
                .uri()
                .map(|u| vec![u.scheme_str().unwrap_or_default().to_string()])?,
            DerivedComponentName::RequestTarget => match self.method() {
                Ok(&http::Method::CONNECT) => self
                    .uri()
                    .map(|u| vec![u.authority().map(|s| s.to_string()).unwrap_or_default()])?,
                Ok(&http::Method::OPTIONS) => vec!["*".to_string()],
                Ok(_) => self.uri().map(|u| {
                    vec![
                        u.path_and_query()
                            .map(|p| p.to_string())
                            .unwrap_or_default(),
                    ]
                })?,
                Err(err) => return Err(Error::crypto(format!("invalid request target: {err}"))),
            },
            DerivedComponentName::Path => self.uri().map(|u| {
                if u.path().is_empty() {
                    vec!["/".to_string()]
                } else {
                    vec![u.path().to_string()]
                }
            })?,
            DerivedComponentName::Query => self.uri().map(|u| {
                vec![
                    u.query()
                        .map(|v| format!("?{v}"))
                        .unwrap_or("?".to_string()),
                ]
            })?,
            DerivedComponentName::QueryParam => {
                let query = self.uri().map(|u| u.query().unwrap_or(""))?;
                query
                    .split('&')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            }
            DerivedComponentName::Status => self.status().map(|s| vec![s.as_str().to_string()])?,
            DerivedComponentName::SignatureParams => self
                .headers()
                .get_all("signature-input")
                .iter()
                .map(|v| v.to_str().map(|s| s.to_string()).unwrap_or_default())
                .collect::<Vec<_>>(),
        };

        HttpMessageComponent::try_from((id, field_values.as_slice())).map_err(Error::from)
    }

    /// Gets the algorithm, key ID pairs from the signature input.
    fn get_alg_key_ids(&self) -> Result<AlgoKeyIdMap> {
        self.extract_signature_headers_with_name().map(|map| {
            map.iter()
                .map(|(name, headers)| {
                    // Unknown or unsupported algorithm strings are mapped to None
                    let alg = headers
                        .signature_params()
                        .alg
                        .clone()
                        .map(|a| AlgorithmName::from_str(&a))
                        .transpose()
                        .ok()
                        .flatten();
                    let key_id = headers.signature_params().keyid.clone();
                    (name.clone(), (alg, key_id))
                })
                .collect()
        })
    }
}

impl<B> HttpMessageSignature for http::Request<B> {
    fn headers(&self) -> &HeaderMap {
        http::Request::headers(self)
    }

    fn headers_mut(&mut self) -> &mut HeaderMap {
        http::Request::headers_mut(self)
    }

    fn method(&self) -> Result<&http::Method> {
        Ok(http::Request::method(self))
    }

    fn uri(&self) -> Result<&http::Uri> {
        Ok(http::Request::uri(self))
    }

    fn status(&self) -> Result<http::StatusCode> {
        Err(Error::http("`status` is only for response"))
    }

    fn is_request(&self) -> bool {
        true
    }
}

impl<B> HttpMessageSignature for http::Response<B> {
    fn headers(&self) -> &HeaderMap {
        http::Response::headers(self)
    }

    fn headers_mut(&mut self) -> &mut HeaderMap {
        http::Response::headers_mut(self)
    }

    fn method(&self) -> Result<&http::Method> {
        Err(Error::http("`method` is only for request"))
    }

    fn uri(&self) -> Result<&http::Uri> {
        Err(Error::http("`method` is only for request"))
    }

    fn status(&self) -> Result<http::StatusCode> {
        Ok(http::Response::status(self))
    }

    fn is_request(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DigestAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl DigestAlgorithm {
    pub const SHA256: &str = "sha-256";
    pub const SHA384: &str = "sha-384";
    pub const SHA512: &str = "sha-512";

    /// Creates a new [DigestAlgorithm].
    pub const fn new() -> Self {
        Self::Sha256
    }

    /// Gets the [DigestAlgorithm] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Sha256 => Self::SHA256,
            Self::Sha384 => Self::SHA384,
            Self::Sha512 => Self::SHA512,
        }
    }

    /// Attempts to convert a string into a [DigestAlgorithm].
    pub fn try_from_str<S: AsRef<str>>(val: S) -> Result<Self> {
        match val.as_ref() {
            Self::SHA256 => Ok(Self::Sha256),
            Self::SHA384 => Ok(Self::Sha384),
            Self::SHA512 => Ok(Self::Sha512),
            s => Err(Error::http(format!("unsupported digest type: {s}"))),
        }
    }
}

impl TryFrom<&str> for DigestAlgorithm {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        Self::try_from_str(val)
    }
}

impl TryFrom<String> for DigestAlgorithm {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl TryFrom<&String> for DigestAlgorithm {
    type Error = Error;

    fn try_from(val: &String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl_default!(DigestAlgorithm);
impl_display!(DigestAlgorithm, str);

pub trait HttpContentDigest: Sized {
    /// Verifies the calculated `content-digest` matches the header value.
    fn verify_content_digest(self) -> impl Future<Output = Result<Self>> + Send;

    /// Calculates and sets the `content-digest` header value.
    fn set_content_digest(self, algo: DigestAlgorithm)
    -> impl Future<Output = Result<Self>> + Send;

    /// Attempts to get the `content-digest` header from the header map.
    fn content_digest_header(headers: &HeaderMap) -> Result<&str> {
        headers
            .get("content-digest")
            .ok_or(Error::http("missing content-digest header"))
            .and_then(|h| {
                h.to_str().map_err(|err| {
                    Error::http(format!(
                        "error converting `content-digest` to string: {err}"
                    ))
                })
            })
    }

    /// Parses the `content-digest` header into its component parts.
    fn parse_content_digest(headers: &HeaderMap) -> Result<(DigestAlgorithm, Vec<u8>)> {
        let header = Self::content_digest_header(headers)?;

        let (algo_str, b64) = header
            .split_once("=")
            .ok_or(Error::http(format!("invalid `content-digest`: {header}")))?;

        let algo = DigestAlgorithm::try_from(algo_str)?;
        let b = b64.trim_start_matches(':').trim_end_matches(':');

        base64::Base64Url::decode_vec(b)
            .map_err(|err| {
                Error::http(format!(
                    "digest: invalid base64 encoding: {err}, base64: {b}"
                ))
            })
            .map(|bytes| (algo, bytes))
    }

    /// Calculates the digest for the provided algorithm.
    fn calculate_digest(algo: DigestAlgorithm, body: &[u8]) -> Vec<u8> {
        match algo {
            DigestAlgorithm::Sha256 => sha2::Sha256::digest(body).to_vec(),
            DigestAlgorithm::Sha384 => sha2::Sha384::digest(body).to_vec(),
            DigestAlgorithm::Sha512 => sha2::Sha512::digest(body).to_vec(),
        }
    }

    /// Checks that the provided `digest` matches the calculated digest (`calc_digest`).
    fn check_content_digest(digest: &[u8], calc_digest: &[u8]) -> Result<()> {
        if digest == calc_digest {
            Ok(())
        } else {
            let have = base64::Base64Url::encode_string(digest);
            let expected = base64::Base64Url::encode_string(calc_digest);

            Err(Error::http(format!(
                "invalid digest, have: {have}, expected: {expected}"
            )))
        }
    }

    fn encode_digest(headers: &mut HeaderMap, algo: DigestAlgorithm, body: &[u8]) -> Result<()> {
        let digest = Self::calculate_digest(algo, body);
        let b64 = base64::Base64Url::encode_string(&digest);

        format!("{algo}=:{b64}:")
            .try_into()
            .map_err(|err| Error::http(format!("header: {err}")))
            .map(|hv| {
                headers.insert("content-digest", hv);
            })
    }
}

impl HttpContentDigest for http::Request<Body> {
    async fn verify_content_digest(self) -> Result<Self> {
        let (parts, body) = self.into_parts();

        let (algo, digest) = Self::parse_content_digest(&parts.headers)?;
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;
        let body_str = str::from_utf8(&body_bytes)?;
        let calc_digest = Self::calculate_digest(algo, body_str.as_bytes());

        Self::check_content_digest(&digest, &calc_digest)
            .map(|_| http::Request::from_parts(parts, body_bytes.into()))
    }

    async fn set_content_digest(self, algo: DigestAlgorithm) -> Result<Self> {
        let (mut parts, body) = self.into_parts();

        let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;

        Self::encode_digest(&mut parts.headers, algo, &body_bytes)
            .map(|_| http::Request::from_parts(parts, body_bytes.into()))
    }
}

impl HttpContentDigest for http::Request<Bytes> {
    async fn verify_content_digest(self) -> Result<Self> {
        let (parts, body) = self.into_parts();

        let (algo, digest) = Self::parse_content_digest(&parts.headers)?;
        let body_str = str::from_utf8(&body)?;
        let calc_digest = Self::calculate_digest(algo, body_str.as_bytes());

        Self::check_content_digest(&digest, &calc_digest)
            .map(|_| http::Request::from_parts(parts, body))
    }

    async fn set_content_digest(self, algo: DigestAlgorithm) -> Result<Self> {
        let (mut parts, body) = self.into_parts();

        Self::encode_digest(&mut parts.headers, algo, &body)
            .map(|_| http::Request::from_parts(parts, body))
    }
}

impl HttpContentDigest for http::Response<Body> {
    async fn verify_content_digest(self) -> Result<Self> {
        let (parts, body) = self.into_parts();

        let (algo, digest) = Self::parse_content_digest(&parts.headers)?;
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;
        let body_str = str::from_utf8(&body_bytes)?;
        let calc_digest = Self::calculate_digest(algo, body_str.as_bytes());

        Self::check_content_digest(&digest, &calc_digest)
            .map(|_| http::Response::from_parts(parts, body_bytes.into()))
    }

    async fn set_content_digest(self, algo: DigestAlgorithm) -> Result<Self> {
        let (mut parts, body) = self.into_parts();

        let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;

        Self::encode_digest(&mut parts.headers, algo, &body_bytes)
            .map(|_| http::Response::from_parts(parts, body_bytes.into()))
    }
}

impl HttpContentDigest for http::Response<Bytes> {
    async fn verify_content_digest(self) -> Result<Self> {
        let (parts, body) = self.into_parts();

        let (algo, digest) = Self::parse_content_digest(&parts.headers)?;
        let body_str = str::from_utf8(&body)?;
        let calc_digest = Self::calculate_digest(algo, body_str.as_bytes());

        Self::check_content_digest(&digest, &calc_digest)
            .map(|_| http::Response::from_parts(parts, body))
    }

    async fn set_content_digest(self, algo: DigestAlgorithm) -> Result<Self> {
        let (mut parts, body) = self.into_parts();

        Self::encode_digest(&mut parts.headers, algo, &body)
            .map(|_| http::Response::from_parts(parts, body))
    }
}
