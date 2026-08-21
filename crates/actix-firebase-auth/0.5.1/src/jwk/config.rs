use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Issuer(String);

impl Issuer {
    pub fn new(project_id: impl AsRef<str>) -> Issuer {
        let issuer =
            format!("https://securetoken.google.com/{}", project_id.as_ref());
        Issuer(issuer)
    }
}

impl Deref for Issuer {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

#[derive(Debug)]
pub struct JwkConfig {
    jwk_url: String,
    audience: String,
    issuer: Issuer,
}

impl JwkConfig {
    pub(crate) const JWK_URL: &str = "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";

    pub fn new(project_id: impl AsRef<str>) -> JwkConfig {
        JwkConfig {
            jwk_url: Self::JWK_URL.to_owned(),
            audience: project_id.as_ref().to_owned(),
            issuer: Issuer::new(project_id),
        }
    }

    #[expect(unused)]
    pub fn jwk_url(&self) -> &str {
        &self.jwk_url
    }

    pub fn audience(&self) -> &str {
        &self.audience
    }

    pub fn issuer(&self) -> &str {
        &self.issuer
    }
}
