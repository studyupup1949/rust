use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageResource{
    pub key: Option<String>,
    pub message: String
}

impl MessageResource{
    pub fn new_from_str_with_type(msg: &str) -> MessageResource{
        MessageResource { key: None, message: String::from(msg) }
    }
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