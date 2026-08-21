use crate::config::Configuration;
use crate::decode::JwtDecode;
use crate::errors::invalid_data;
use crate::key::KeyType;
use jsonwebtoken as jwt;

#[derive(Debug, Clone)]
pub struct JwtEncode {
    config: Configuration,
    header: jwt::Header,
    key: jwt::EncodingKey,
}

use jwt::errors::Result;
use serde::Serialize;
use std::io;

impl JwtEncode {
    pub(crate) fn new(config: Configuration) -> io::Result<JwtEncode> {
        Ok(JwtEncode {
            key: key_of(&config.key, config.algorithm)?,
            header: jwt::Header::new(config.algorithm),
            config,
        })
    }

    pub fn decoder(&self) -> io::Result<JwtDecode> {
        self.config.transform().and_then(JwtDecode::new)
    }

    pub fn encode<T: Serialize>(&self, claims: &T) -> Result<String> {
        jwt::encode(&self.header, claims, &self.key)
    }
}

fn key_of(key: &[u8], algorithm: jwt::Algorithm) -> io::Result<jwt::EncodingKey> {
    let key = match KeyType::from(algorithm) {
        KeyType::Secret => jwt::EncodingKey::from_secret(key),
        KeyType::Rsa => jwt::EncodingKey::from_rsa_pem(key).map_err(invalid_data)?,
        KeyType::Ec => jwt::EncodingKey::from_ec_pem(key).map_err(invalid_data)?,
    };

    Ok(key)
}
