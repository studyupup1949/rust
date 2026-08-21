use jsonwebtoken as jwt;
use serde::Deserialize;
use std::fs;
use std::io;

use crate::errors;
use crate::key;
use crate::JwtDecode;
use crate::JwtEncode;

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    key: Option<String>,
    key_file: Option<String>,
    passphrase: Option<String>,
    passphrase_file: Option<String>,
    algorithm: jwt::Algorithm,
    audience: Option<String>,
    subject: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Configuration {
    pub key: Vec<u8>,
    pub algorithm: jwt::Algorithm,
    pub audience: Option<String>,
    pub subject: Option<String>,
}

impl Configuration {
    pub fn transform(&self) -> io::Result<Self> {
        let mut next = self.clone();
        next.key = key::for_verifying(next.key, next.algorithm)?;
        Ok(next)
    }
}

fn bytes_or_read(text: Option<&String>, file: Option<&String>) -> io::Result<Option<Vec<u8>>> {
    text.map(|it| Ok(it.as_bytes().to_owned()))
        .or_else(|| file.map(fs::read))
        .transpose()
}

impl JwtConfig {
    pub fn with(algorithm: jwt::Algorithm) -> Self {
        Self {
            algorithm,
            key: None,
            key_file: None,
            passphrase: None,
            passphrase_file: None,
            audience: None,
            subject: None,
        }
    }

    pub fn key<T: ToString>(mut self, value: T) -> Self {
        self.key.replace(value.to_string());
        self
    }

    pub fn key_file<T: ToString>(mut self, value: T) -> Self {
        self.key_file.replace(value.to_string());
        self
    }

    pub fn passphrase<T: ToString>(mut self, value: T) -> Self {
        self.passphrase.replace(value.to_string());
        self
    }

    pub fn passphrase_file<T: ToString>(mut self, value: T) -> Self {
        self.passphrase_file.replace(value.to_string());
        self
    }

    pub fn audience<T: ToString>(mut self, value: T) -> Self {
        self.audience.replace(value.to_string());
        self
    }

    pub fn subject<T: ToString>(mut self, value: T) -> Self {
        self.subject.replace(value.to_string());
        self
    }

    /// Create a [`JwtDecode`] with it
    pub fn decoder(self) -> io::Result<JwtDecode> {
        self.finish().and_then(JwtDecode::new)
    }

    /// Create a [`JwtEncode`] with it
    pub fn encoder(self) -> io::Result<JwtEncode> {
        self.finish().and_then(JwtEncode::new)
    }

    fn finish(self) -> io::Result<Configuration> {
        let key = bytes_or_read(self.key.as_ref(), self.key_file.as_ref())?;
        let key = key.ok_or_else(|| errors::invalid_input("`key` or `key_file` required"))?;
        let passphrase = bytes_or_read(self.passphrase.as_ref(), self.passphrase_file.as_ref())?;

        Ok(Configuration {
            key: key::key_content(key, passphrase, self.algorithm)?,
            algorithm: self.algorithm,
            audience: self.audience,
            subject: self.subject,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Algorithm;

    #[test]
    fn it_works_with_key_file() {
        let file = include_str!("../README.md");
        let it = JwtConfig::with(Algorithm::ES256)
            .key_file("README.md")
            .finish()
            .unwrap();

        assert_eq!(file.as_bytes(), it.key);
    }
}
