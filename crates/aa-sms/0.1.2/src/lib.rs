//! # aa-sms
//!
//! The `aa-sms` crate provides a [`Client`] for sending [`Message`]s with Andrew & Arnold's SMS API.

mod client;
pub use crate::client::Client;
pub use crate::client::ClientBuilder;

mod message;
pub use crate::message::Message;
pub use crate::message::MessageBuilder;
