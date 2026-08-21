pub mod signature;

pub use signature::SignatureGenerator;

#[derive(Debug, Clone)]
pub enum AuthMethod {
    #[cfg(feature = "mp")]
    JwtToken(String),
    #[cfg(feature = "mp")]
    ApiKey(String),
    #[cfg(feature = "app")]
    AppSignature { app_id: String, secret: String },
    #[cfg(feature = "app")]
    WorldSignature { world_id: String, secret: String },
}

impl AuthMethod {
    #[cfg(feature = "mp")]
    pub fn jwt(token: impl Into<String>) -> Self {
        Self::JwtToken(token.into())
    }

    #[cfg(feature = "mp")]
    pub fn api_key(key: impl Into<String>) -> Self {
        Self::ApiKey(key.into())
    }

    #[cfg(feature = "app")]
    pub fn app_signature(app_id: impl Into<String>, secret: impl Into<String>) -> Self {
        Self::AppSignature {
            app_id: app_id.into(),
            secret: secret.into(),
        }
    }

    #[cfg(feature = "app")]
    pub fn world_signature(world_id: impl Into<String>, secret: impl Into<String>) -> Self {
        Self::WorldSignature {
            world_id: world_id.into(),
            secret: secret.into(),
        }
    }
}
