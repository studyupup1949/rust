use serde::{Serialize, Deserialize};

/// This is for sending errors back from requests.
/// This struct contains an optional key just in 
/// case you want to deal with internationalization
/// It was left as optional just in case you don't
/// Have the time to yet...
#[derive(Serialize, Deserialize, Debug)]
pub struct MessageResource{
    pub key: Option<String>,
    pub message: String
}

impl MessageResource{
    pub fn new_empty() -> MessageResource{
        MessageResource { key: None, message: String::from("") }
    }
    pub fn new_from_str(msg: &str) -> MessageResource{
        MessageResource { key: None, message: String::from(msg) }
    }
    pub fn new_from_err(errstr: String) -> MessageResource{
        MessageResource { key: None, message: errstr }
    }
    pub fn new(key: &str, msg: &str) -> MessageResource{
        MessageResource { key: Some(String::from(key)), message: String::from(msg) }
    }
}