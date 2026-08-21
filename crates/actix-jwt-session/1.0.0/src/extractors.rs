//! Allow to create own session extractor and extract from cookie or header.

use crate::*;

/// Trait allowing to extract JWt token from [actix_web::dev::ServiceRequest]
///
/// Two extractor are implemented by default
/// * [HeaderExtractor] which is best for any PWA or micro services requests
/// * [CookieExtractor] which is best for simple server with session stored in cookie
///
/// It's possible to implement GraphQL, JSON payload or query using `req.extract::<JSON<YourStruct>>()` if this is needed.
///
/// All implementation can use [SessionExtractor::decode] method for decoding raw JWT string into
/// Claims and then [SessionExtractor::validate] to validate claims agains session stored in [SessionStorage]
#[async_trait(?Send)]
pub trait SessionExtractor<ClaimsType: Claims>: Send + Sync + 'static {
    /// Extract claims from [actix_web::dev::ServiceRequest]
    ///
    /// Examples:
    ///
    /// ```
    /// use actix_web::dev::ServiceRequest;
    /// use jsonwebtoken::*;
    /// use actix_jwt_session::*;
    /// use std::sync::Arc;
    /// use actix_web::HttpMessage;
    /// use std::borrow::Cow;
    ///
    /// # #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    /// # pub struct Claims { id: uuid::Uuid, sub: String }
    /// # impl actix_jwt_session::Claims for Claims {
    /// #     fn jti(&self) -> uuid::Uuid { self.id }
    /// #     fn subject(&self) -> &str { &self.sub }
    /// # }
    ///
    /// #[derive(Debug, Clone, Copy, Default)]
    /// struct ExampleExtractor;
    ///
    /// #[async_trait::async_trait(?Send)]
    /// impl SessionExtractor<Claims> for ExampleExtractor {
    ///     async fn extract_claims(
    ///         &self,
    ///         req: &mut ServiceRequest,
    ///         jwt_encoding_key: Arc<EncodingKey>,
    ///         jwt_decoding_key: Arc<DecodingKey>,
    ///         algorithm: Algorithm,
    ///         storage: SessionStorage,
    ///     ) -> Result<(), Error> {
    ///         if req.peer_addr().unwrap().ip().is_multicast() {
    ///            req.extensions_mut().insert(Authenticated {
    ///                claims: Arc::new(Claims { id: uuid::Uuid::default(), sub: "HUB".into() }),
    ///                jwt_encoding_key,
    ///                algorithm,
    ///            });
    ///         }
    ///         Ok(())
    ///     }
    ///
    ///     async fn extract_token_text<'req>(&self, req: &'req mut ServiceRequest) -> Option<Cow<'req, str>> { None }
    /// }
    /// ```
    async fn extract_claims(
        &self,
        req: &mut ServiceRequest,
        jwt_encoding_key: Arc<EncodingKey>,
        jwt_decoding_key: Arc<DecodingKey>,
        algorithm: Algorithm,
        storage: SessionStorage,
    ) -> Result<(), Error> {
        let Some(as_str) = self.extract_token_text(req).await else {
            return Ok(());
        };
        let decoded_claims = self.decode(&as_str, jwt_decoding_key, algorithm)?;
        self.validate(&decoded_claims, storage).await?;
        req.extensions_mut().insert(Authenticated {
            claims: Arc::new(decoded_claims),
            jwt_encoding_key,
            algorithm,
        });
        Ok(())
    }

    /// Decode encrypted JWT to structure
    fn decode(
        &self,
        value: &str,
        jwt_decoding_key: Arc<DecodingKey>,
        algorithm: Algorithm,
    ) -> Result<ClaimsType, Error> {
        let mut validation = Validation::new(algorithm);
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.leeway = 0;
        validation.required_spec_claims.clear();

        decode::<ClaimsType>(value, &jwt_decoding_key, &validation)
            .map_err(|e| {
                #[cfg(feature = "use-tracing")]
                tracing::debug!("Failed to decode claims: {e:?}. {e}");
                Error::CantDecode
            })
            .map(|t| t.claims)
    }

    /// Validate JWT Claims agains stored in storage tokens.
    ///
    /// * Token must exists in storage
    /// * Token must be exactly the same as token from storage
    async fn validate(&self, claims: &ClaimsType, storage: SessionStorage) -> Result<(), Error> {
        let stored = storage
            .clone()
            .find_jwt::<ClaimsType>(claims.jti())
            .await
            .map_err(|e| {
                #[cfg(feature = "use-tracing")]
                tracing::debug!(
                    "Failed to load {} from storage: {e:?}",
                    std::any::type_name::<ClaimsType>()
                );
                Error::LoadError
            })?;

        if &stored != claims {
            #[cfg(feature = "use-tracing")]
            tracing::debug!("{claims:?} != {stored:?}");
            Err(Error::DontMatch)
        } else {
            Ok(())
        }
    }

    /// Lookup for session data as a string in [actix_web::dev::ServiceRequest]
    ///
    /// If there's no token data in request you should returns `None`. This is not considered as an
    /// error and until endpoint requires `Authenticated` this will not results in `401`.
    async fn extract_token_text<'req>(
        &self,
        req: &'req mut ServiceRequest,
    ) -> Option<Cow<'req, str>>;
}

/// Extracts JWT token from HTTP Request cookies. This extractor should be used when you can't set
/// your own header, for example when user enters http links to browser and you don't have any
/// advanced frontend.
///
/// This exractor is may be used by PWA application or micro services but [HeaderExtractor] is much
/// more suitable for this purpose.
pub struct CookieExtractor<ClaimsType> {
    __ty: PhantomData<ClaimsType>,
    cookie_name: &'static str,
}

impl<ClaimsType: Claims> CookieExtractor<ClaimsType> {
    /// Creates new cookie extractor.
    /// It will extract token data from cookie with given name
    pub fn new(cookie_name: &'static str) -> Self {
        Self {
            __ty: Default::default(),
            cookie_name,
        }
    }
}

#[async_trait(?Send)]
impl<ClaimsType: Claims> SessionExtractor<ClaimsType> for CookieExtractor<ClaimsType> {
    async fn extract_token_text<'req>(
        &self,
        req: &'req mut ServiceRequest,
    ) -> Option<Cow<'req, str>> {
        req.cookie(self.cookie_name)
            .map(|c| c.value().to_string().into())
    }
}

/// Extracts JWT token from HTTP Request headers
///
/// This exractor is very useful for all PWA application or for micro services
/// because you can set your own headers while making http requests.
///
/// If you want to have users authorized using simple html anchor (tag A) you should use [CookieExtractor]
pub struct HeaderExtractor<ClaimsType> {
    __ty: PhantomData<ClaimsType>,
    header_name: &'static str,
}

impl<ClaimsType: Claims> HeaderExtractor<ClaimsType> {
    /// Creates new header extractor.
    /// It will extract token data from header with given name
    pub fn new(header_name: &'static str) -> Self {
        Self {
            __ty: Default::default(),
            header_name,
        }
    }
}

#[async_trait(?Send)]
impl<ClaimsType: Claims> SessionExtractor<ClaimsType> for HeaderExtractor<ClaimsType> {
    async fn extract_token_text<'req>(
        &self,
        req: &'req mut ServiceRequest,
    ) -> Option<Cow<'req, str>> {
        req.headers()
            .get(self.header_name)
            .and_then(|h| h.to_str().ok())
            .map(|h| h.to_owned().into())
    }
}
