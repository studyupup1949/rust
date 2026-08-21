use crate::config::Configuration;
use crate::errors::invalid_data;
use crate::key::KeyType;
use jsonwebtoken as jwt;

#[derive(Debug, Clone)]
pub struct JwtDecode {
    pub validation: jwt::Validation,
    key: jwt::DecodingKey<'static>,
}

use jwt::errors::Result;
use serde::de::DeserializeOwned;
use std::io;

impl JwtDecode {
    pub(crate) fn new(config: Configuration) -> io::Result<Self> {
        let algorithm = config.algorithm;

        let mut validation = jwt::Validation::new(algorithm);
        validation.sub = config.subject;
        validation.aud = config.audience.map(|text| {
            text.split(is_seperator)
                .map(|it| it.to_string())
                .collect::<_>()
        });

        Ok(JwtDecode {
            key: key_of(&config.key, algorithm)?,
            validation,
        })
    }

    pub fn decode<T: DeserializeOwned>(&self, token: &str) -> Result<T> {
        jwt::decode(token, &self.key, &self.validation).map(|it| it.claims)
    }
}

fn is_seperator(c: char) -> bool {
    matches!(c, ',' | ';' | '/')
}

fn key_of(key: &[u8], algorithm: jwt::Algorithm) -> io::Result<jwt::DecodingKey<'static>> {
    let key = match KeyType::from(algorithm) {
        KeyType::Secret => jwt::DecodingKey::from_secret(key).into_static(),
        KeyType::Rsa => jwt::DecodingKey::from_rsa_pem(key)
            .map_err(invalid_data)?
            .into_static(),
        KeyType::Ec => jwt::DecodingKey::from_ec_pem(key)
            .map_err(invalid_data)?
            .into_static(),
    };

    Ok(key)
}
