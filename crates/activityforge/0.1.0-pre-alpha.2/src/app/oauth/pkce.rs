use async_trait::async_trait;
use base64::Encoding;
use serde::{Deserialize, Serialize};
use sha2::Digest;

use oauth::code_grant::accesstoken::Request as TokenRequest;
use oauth::code_grant::authorization::Request as AuthRequest;
use oauth::frontends::simple::extensions::{AccessTokenAddon, AddonResult, AuthorizationAddon};
use oauth::primitives::grant::{Extensions, GrantExtension, Value};

use oauth_async::code_grant::access_token::Extension as AccessTokenExtension;
use oauth_async::code_grant::authorization::Extension as AuthorizationExtension;
use oauth_async::code_grant::client_credentials::Extension as ClientCredentialsExtension;
use oauth_async::endpoint::Extension;

use activitystreams_vocabulary::{impl_default, impl_display};

use crate::{Error, Result};

/// Represents the [PKCE](https://www.rfc-editor.org/rfc/rfc7636) challenge method used by the OAuth 2.0 client.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "challenge_method")]
pub enum ChallengeMethod {
    #[serde(rename = "plain")]
    #[sqlx(rename = "plain")]
    Plain,
    S256,
}

impl ChallengeMethod {
    pub const PLAIN_STR: &str = "plain";
    pub const S256_STR: &str = "S256";

    /// Creates a new [ChallengeMethod].
    pub const fn new() -> Self {
        Self::S256
    }

    /// Gets the [ChallengeMethod] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Plain => Self::PLAIN_STR,
            Self::S256 => Self::S256_STR,
        }
    }
}

impl TryFrom<&str> for ChallengeMethod {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        match val {
            Self::PLAIN_STR => Ok(Self::Plain),
            Self::S256_STR => Ok(Self::S256),
            _ => Err(Error::http(format!(
                "oauth: invalid PKCE code_challenge_method: {val}"
            ))),
        }
    }
}

impl_default!(ChallengeMethod);
impl_display!(ChallengeMethod, str);

/// Represents the [PKCE](https://www.rfc-editor.org/rfc/rfc7636) challenge method used by the OAuth 2.0 client.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "code_challenge")]
pub struct CodeChallenge {
    #[serde(
        rename = "code_challenge",
        serialize_with = "CodeChallenge::ser_code",
        deserialize_with = "CodeChallenge::des_code"
    )]
    code: [u8; 32],
    #[serde(rename = "code_challenge_method")]
    method: ChallengeMethod,
}

impl CodeChallenge {
    pub const LEN: usize = 32;

    /// Creates a new [CodeChallenge].
    pub const fn new() -> Self {
        Self {
            code: [0u8; Self::LEN],
            method: ChallengeMethod::new(),
        }
    }

    /// Creates a new [CodeChallenge].
    ///
    /// **NOTE**: only the [S256](ChallengeMethod::S256) PKCE challenge method is supported.
    pub fn create(code_b64: &str, method: ChallengeMethod) -> Result<Self> {
        Self::check_method(method).and_then(|_| {
            let mut code = [0u8; Self::LEN];

            base64::Base64UrlUnpadded::decode(code_b64, &mut code).map_err(|err| {
                Error::http(format!("oauth: error decoding PKCE challenge code: {err}"))
            })?;

            Ok(Self { code, method })
        })
    }

    /// Constructs the [CodeChallenge] from the `verifier` input.
    pub fn from_verifier(verifier: &str) -> Result<Self> {
        base64::Base64UrlUnpadded::decode_vec(verifier)
            .map_err(|err| Error::http(format!("oauth: pkce: decoding verifier: {err}")))
            .map(|v| Self {
                code: sha2::Sha256::digest(v.as_slice()).into(),
                method: ChallengeMethod::S256,
            })
    }

    /// Gets a reference to the hash of the [CodeChallenge] verifier.
    ///
    /// This corresponds to the `challenge_code` sent in the initial OAuth 2.0 PKCE Grant.
    #[inline]
    pub const fn code(&self) -> &[u8] {
        &self.code
    }

    /// Gets the [CodeChallenge] Base64URL-encoded value.
    ///
    /// This corresponds to the `challenge_code` sent in the initial OAuth 2.0 PKCE Grant.
    pub fn code_str(&self) -> String {
        base64::Base64UrlUnpadded::encode_string(self.code())
    }

    /// Gets the [ChallengeMethod].
    #[inline]
    pub const fn method(&self) -> ChallengeMethod {
        self.method
    }

    /// Attempts to verify the provided PKCE verifier against the `challenge_code`.
    pub fn verify(&self, verifier: &str) -> Result<()> {
        Self::check_method(self.method).and_then(|_| {
            let out = base64::Base64UrlUnpadded::decode_vec(verifier)
                .map_err(|err| Error::http(format!("oauth: pkce: error decoding verifier: {err}")))
                .map(|v| sha2::Sha256::digest(&v))?;

            if out.as_slice() == self.code {
                Ok(())
            } else {
                Err(Error::http("oauth: invalid PKCE verifier"))
            }
        })
    }

    /// Checks that the PKCE `code_challenge_method` field is valid.
    ///
    /// **NOTE**: only the [S256](ChallengeMethod::S256) PKCE challenge method is supported.
    #[inline]
    fn check_method(method: ChallengeMethod) -> Result<()> {
        match method {
            ChallengeMethod::Plain => Err(Error::http(format!(
                "oauth: unsupported PKCE challenge method: {method}"
            ))),
            ChallengeMethod::S256 => Ok(()),
        }
    }

    fn ser_code<S>(code: &[u8; Self::LEN], s: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::Serialize as _;

        base64::Base64UrlUnpadded::encode_string(code).serialize(s)
    }

    fn des_code<'de, D>(d: D) -> core::result::Result<[u8; Self::LEN], D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use serde::de::Deserialize as _;

        <&str>::deserialize(d).and_then(|s| {
            let mut dec = [0u8; Self::LEN];
            base64::Base64UrlUnpadded::decode(s, &mut dec).map_err(|err| {
                serde::de::Error::custom(format!(
                    "http: oauth: invalid code_challenge encoding: {err}"
                ))
            })?;
            Ok(dec)
        })
    }
}

impl_default!(CodeChallenge);
impl_display!(CodeChallenge, json);

/// Represents the base type for implementing the PKCE extension traits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Pkce;

impl Pkce {
    /// Creates a new [Pkce].
    #[inline]
    pub const fn new() -> Self {
        Self {}
    }

    /// Gets the [Pkce] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        "pkce"
    }
}

impl_default!(Pkce);
impl_display!(Pkce, str);

impl AuthorizationAddon for Pkce {
    fn execute(&self, request: &dyn AuthRequest) -> AddonResult {
        let Some(method) = request.extension("code_challenge_method") else {
            log::error!("oauth: pkce: missing code_challenge_method");
            return AddonResult::Err;
        };

        let Some(code) = request.extension("code_challenge") else {
            log::error!("oauth: pkce: missing code_challenge");
            return AddonResult::Err;
        };

        log::debug!("oauth: pkce: code_challenge: {code}");

        match ChallengeMethod::try_from(method.as_ref())
            .and_then(|m| CodeChallenge::create(code.as_ref(), m))
        {
            Ok(c) => AddonResult::Data(Value::private(Some(c.code_str()))),
            Err(err) => {
                log::error!("oauth: pkce: {err}");
                AddonResult::Err
            }
        }
    }
}

impl AccessTokenAddon for Pkce {
    fn execute(&self, request: &dyn TokenRequest, data: Option<Value>) -> AddonResult {
        let Some(verifier) = request.extension("code_verifier") else {
            log::error!("oauth: pkce: missing code_verifier");
            return AddonResult::Err;
        };

        let Some(Some(code)) = data.and_then(|d| d.into_private_value().ok()) else {
            log::error!("oauth: pkce: invalid or missing challenge verifier");
            return AddonResult::Err;
        };

        match CodeChallenge::create(&code, ChallengeMethod::S256)
            .and_then(|c| c.verify(verifier.as_ref()))
        {
            Ok(_) => AddonResult::Ok,
            Err(err) => {
                log::error!("oauth: pkce: error validating verifier: {err}");
                AddonResult::Err
            }
        }
    }
}

impl Extension for Pkce {
    fn authorization(&mut self) -> Option<&mut (dyn AuthorizationExtension + Send)> {
        Some(self)
    }

    fn access_token(&mut self) -> Option<&mut (dyn AccessTokenExtension + Send)> {
        Some(self)
    }

    fn client_credentials(&mut self) -> Option<&mut (dyn ClientCredentialsExtension + Send)> {
        None
    }
}

impl GrantExtension for Pkce {
    fn identifier(&self) -> &'static str {
        "pkce"
    }
}

#[async_trait]
impl AuthorizationExtension for Pkce {
    async fn extend(
        &mut self,
        request: &(dyn AuthRequest + Sync),
    ) -> core::result::Result<Extensions, ()> {
        let mut result_data = Extensions::new();

        match <Self as AuthorizationAddon>::execute(self, request) {
            AddonResult::Ok => (),
            AddonResult::Data(data) => result_data.set(self, data),
            AddonResult::Err => return Err(()),
        }

        Ok(result_data)
    }
}

#[async_trait]
impl AccessTokenExtension for Pkce {
    async fn extend(
        &mut self,
        request: &(dyn TokenRequest + Sync),
        mut data: Extensions,
    ) -> core::result::Result<Extensions, ()> {
        let mut result_data = Extensions::new();

        let ext_data = data.remove(self);

        match <Self as AccessTokenAddon>::execute(self, request, ext_data) {
            AddonResult::Ok => (),
            AddonResult::Data(data) => result_data.set(self, data),
            AddonResult::Err => return Err(()),
        }

        Ok(result_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Nonce;

    #[test]
    fn test_challenge_code() {
        let verifier = sha2::Sha256::digest(Nonce::random().as_ref());
        let verifier_b64 = base64::Base64UrlUnpadded::encode_string(&verifier);
        let code = CodeChallenge::from_verifier(&verifier_b64).unwrap();

        let code_challenge = sha2::Sha256::digest(verifier);
        let code_challenge_b64 =
            base64::Base64UrlUnpadded::encode_string(code_challenge.as_slice());

        let code_json = format!(
            r#"{{"code_challenge":"{code_challenge_b64}","code_challenge_method":"S256"}}"#
        );
        assert_eq!(
            serde_json::from_str::<CodeChallenge>(&code_json).unwrap(),
            code
        );
        assert!(code.verify(&verifier_b64).is_ok());
        assert_eq!(code.code(), code_challenge.as_slice());
        assert_eq!(code.code_str(), code_challenge_b64);

        let bad_code = sha2::Sha256::digest(b"bad_verifier");
        let bad_code_b64 = base64::Base64UrlUnpadded::encode_string(bad_code.as_slice());
        let bad_code_json =
            format!(r#"{{"code_challenge":"{bad_code_b64}","code_challenge_method":"S256"}}"#);

        let bad_code = CodeChallenge {
            code: bad_code.into(),
            method: ChallengeMethod::S256,
        };

        assert_eq!(
            serde_json::from_str::<CodeChallenge>(&bad_code_json).unwrap(),
            bad_code
        );
        assert!(bad_code.verify(&verifier_b64).is_err());

        let code: &[u8] = &verifier;
        let bad_method = CodeChallenge {
            code: code.as_ref().try_into().unwrap(),
            method: ChallengeMethod::Plain,
        };
        let plain_b64 = base64::Base64UrlUnpadded::encode_string(&verifier);
        let bad_method_json =
            format!(r#"{{"code_challenge":"{plain_b64}","code_challenge_method":"plain"}}"#);
        assert_eq!(
            serde_json::from_str::<CodeChallenge>(&bad_method_json).unwrap(),
            bad_method
        );
        assert!(bad_method.verify(&verifier_b64).is_err());

        let unknown_method_str = "weird_meth";
        let unknown_method_json = format!(
            r#"{{"code_challenge":"{verifier_b64}","code_challenge_method":"{unknown_method_str}"}}"#
        );
        assert!(serde_json::from_str::<CodeChallenge>(&unknown_method_json).is_err());
        assert!(ChallengeMethod::try_from(unknown_method_str).is_err())
    }

    #[test]
    fn test_challenge_method() {
        [
            (ChallengeMethod::Plain, ChallengeMethod::PLAIN_STR),
            (ChallengeMethod::S256, ChallengeMethod::S256_STR),
        ]
        .into_iter()
        .for_each(|(method, method_str)| {
            let method_json = format!(r#""{method_str}""#);

            assert_eq!(method.as_str(), method_str);
            assert_eq!(ChallengeMethod::try_from(method_str).unwrap(), method);

            assert_eq!(serde_json::to_string(&method).unwrap(), method_json);
            assert_eq!(
                serde_json::from_str::<ChallengeMethod>(&method_json).unwrap(),
                method
            );
        });
    }
}
