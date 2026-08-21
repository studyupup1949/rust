//! Framework-independent core types shared by all store backends.
//!
//! This module mirrors the
//! [`core/`](https://github.com/LdDl/echo-jwt/tree/master/core) package from the Go implementation and
//! contains no actix-web dependency, making it reusable with any Rust web
//! framework.
//!
//! # Contents
//!
//! * [`TokenStore`] - async trait defining the refresh-token storage contract.
//! * [`Token`] - a complete JWT token pair (access + optional refresh).
//! * [`RefreshTokenData`] - data stored alongside each refresh token.

pub mod store;
pub mod token;

pub use store::*;
pub use token::*;
