use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        println!("{}", Request::create(Method::Read).with_content_string("Hello World".to_string(), ContentType::Text).build());
        println!("{}", Response::create(ResponseCode::Ok).with_content_string("Hello World".to_string(), ContentType::Text).build());
        panic!();
    }    
}

#[derive(Debug,Clone, Serialize, Deserialize)]
pub enum Method {
    /// Used to read existing data
    Read,
    /// Used to create new data
    Insert,
    /// Used to update existing data
    Set,
    /// Used to delete existing data
    Remove,
    /// Used to check if data exists
    Check
}

#[derive(Debug,Clone, Serialize, Deserialize)]
pub enum ContentType {
    Text, // Text in UTF-8
    Binary, // Raw 1s and 0s
    None
}

#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct Request {
    version: i32,
    pub method: Method,
    pub content_type: ContentType,
    pub content: Vec<u8>,
}
impl Request {
    pub fn create(method: Method) -> Request {
        Request {
            version: 1,
            method,
            content_type: ContentType::None,
            content: vec![],
        }
    }
    
    pub fn with_content(mut self, content: Vec<u8>, content_type: ContentType) -> Request {
        self.content = content;
        self.content_type = content_type;
        self
    }
    
    pub fn with_content_string(mut self, content: String, content_type: ContentType) -> Request {
        self.content = content.into_bytes();
        self.content_type = content_type;
        self
    }
    
    pub fn build(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
impl From<String> for Request {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}
impl From<Vec<u8>> for Request {
    fn from(value: Vec<u8>) -> Self {
        serde_json::from_str(&String::from_utf8(value).unwrap()).unwrap()
    }
}

#[derive(Debug,Clone, Serialize, Deserialize)]
pub enum ResponseCode {
    SwitchProtocols, // Server is switching protocol. Check content for protocol type
    Ok, // Server has acknowledged your request.
    Pending, // Request is pending. Wait for follow up.
    Redirect, // Content is no longer at requested location. Check content for new location.
    Denied, // Server has denied request.
    BadRequest, // Request is malformed.
    Unauthorized, // You are not authorized to view the content
    NotFound, // Server could not find the requested content.
    TooManyRequests, // Server has received too many requests from this client
    InternalServerError, // Server has encountered an error
}

#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct Response {
    version: i32,
    pub code: ResponseCode,
    pub content_type: ContentType,
    pub content: Vec<u8>,
}
impl Response {
    pub fn create(code: ResponseCode) -> Response {
        Response {
            version: 1,
            code,
            content_type: ContentType::None,
            content: vec![],
        }
    }
    
    pub fn with_content(mut self, content: Vec<u8>, content_type: ContentType) -> Response {
        self.content = content;
        self.content_type = content_type;
        self
    }
    
    pub fn with_content_string(mut self, content: String, content_type: ContentType) -> Response {
        self.content = content.into_bytes();
        self.content_type = content_type;
        self
    }
    
    pub fn build(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
impl From<String> for Response {
    fn from(value: String) -> Self {
        serde_json::from_str(&value).unwrap()
    }
}
impl From<Vec<u8>> for Response {
    fn from(value: Vec<u8>) -> Self {
        serde_json::from_str(&String::from_utf8(value).unwrap()).unwrap()
    }
}