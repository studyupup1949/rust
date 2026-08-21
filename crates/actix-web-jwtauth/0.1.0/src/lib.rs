mod actix;
mod claims;
mod config;
mod decode;
mod encode;
mod key;

pub use claims::Jwt;
pub use config::JwtConfig;
pub use decode::JwtDecode;
pub use encode::JwtEncode;

pub use jsonwebtoken::Algorithm;

mod errors {
    use std::io;

    pub(crate) fn invalid_input<E>(error: E) -> io::Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        io::Error::new(io::ErrorKind::InvalidInput, error)
    }

    pub(crate) fn invalid_data<E>(error: E) -> io::Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        io::Error::new(io::ErrorKind::InvalidData, error)
    }
}
