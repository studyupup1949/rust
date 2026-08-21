use std::fmt::Display;

use serde::{Serialize, Deserialize};

//TODO: Add examples
/// This is for sending errors back from requests conveniently.
/// This struct contains an optional key just in 
/// case you want to deal with internationalization.
/// It was left as optional just in case you don't
/// have the time to yet...
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageResource{
    pub key: Option<String>,
    pub message: String
}
impl Display for MessageResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MessageResource Key: {:?}, Message: {}", self.key, self.message)
    }
}

impl MessageResource{
    pub fn new_empty() -> MessageResource{
        MessageResource { key: None, message: String::from("") }
    }
    pub fn new_from_str(msg: &str) -> MessageResource{
        MessageResource { key: None, message: String::from(msg) }
    }
    /// Just takes any error that implements display (Has a .to_string() method)
    pub fn new_from_err<E: Display>(error: E) -> MessageResource{
        MessageResource { key: None, message: error.to_string() }
    }
    pub fn new(key: &str, msg: &str) -> MessageResource{
        MessageResource { key: Some(String::from(key)), message: String::from(msg) }
    }
}