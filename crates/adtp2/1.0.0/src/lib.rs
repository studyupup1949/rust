use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum Version {
    Adtp2,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum Method {
    Check,
    Read,
    Create,
    Update,
    Append,
    Destroy,
    Auth,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct RequestBuilder {
    version: Version,
    method: Method,
    headers: HashMap<String, String>,
    uri: String,
    content: String,
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestBuilder {
    pub fn new() -> RequestBuilder {
        RequestBuilder {
            version: Version::Adtp2,
            method: Method::Check,
            headers: HashMap::new(),
            uri: String::new(),
            content: String::new(),
        }
    }

    pub fn set_version(mut self, version: Version) -> RequestBuilder {
        self.version = version;
        self
    }

    pub fn set_method(mut self, method: Method) -> RequestBuilder {
        self.method = method;
        self
    }

    pub fn add_header(mut self, key: &str, value: &str) -> RequestBuilder {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn set_uri(mut self, uri: &str) -> RequestBuilder {
        self.uri = uri.to_string();
        self
    }

    pub fn set_content(mut self, content: &str) -> RequestBuilder {
        self.content = content.to_owned();
        self
    }

    pub fn build(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum Status {
    SwitchProtocols,
    Ok,
    Pending,
    Redirect,
    Denied,
    BadRequest,
    Unauthorized,
    NotFound,
    TooManyRequests,
    InternalError,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct ResponseBuilder {
    version: Version,
    status: Status,
    headers: HashMap<String, String>,
    content: String,
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseBuilder {
    pub fn new() -> ResponseBuilder {
        ResponseBuilder {
            version: Version::Adtp2,
            status: Status::Ok,
            headers: HashMap::new(),
            content: String::new(),
        }
    }

    pub fn set_version(mut self, version: Version) -> ResponseBuilder {
        self.version = version;
        self
    }

    pub fn set_status(mut self, status: Status) -> ResponseBuilder {
        self.status = status;
        self
    }

    pub fn add_header(mut self, key: &str, value: &str) -> ResponseBuilder {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn set_content(mut self, content: &str) -> ResponseBuilder {
        self.content = content.to_owned();
        self
    }

    pub fn build(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
