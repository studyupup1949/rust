use crate::aead::OsRng;
use crate::aes::Aes256;
use aes_gcm::aead::Aead;
pub use aes_gcm::*;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

const ADTP_VERSION: &str = "ADTP/1.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_test() {
        let request = Request::create(
            Method::Create,
            "buss://napture.web".to_string(),
            "test/0.0".to_string(),
        )
        .with_content_string(ContentType::Text, "Content".to_string())
        .with_language("us-english".to_string());
        println!("{}", request.clone().build());
        assert_eq!(
            request.clone().build().as_str(),
            Request::try_from(request.build()).unwrap().build()
        );
    }

    #[test]
    fn responses_test() {
        let response = Response::create(Status::Ok, "test/0.0".to_string())
            .with_language("us-english".to_string())
            .with_content_string(ContentType::Text, "Content".to_string());
        println!("{}", response.clone().build());
        assert_eq!(
            response.clone().build().as_str(),
            Response::try_from(response.build())
                .unwrap()
                .build()
        );
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cookie {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "value")]
    pub value: String,
}
impl Cookie {
    pub fn new(key: String, value: String) -> Cookie {
        Cookie { key, value }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    /// Used to read existing data
    #[serde(rename = "read")]
    Read,
    /// Used to create new data
    #[serde(rename = "create")]
    Create,
    /// Used to update existing data
    #[serde(rename = "update")]
    Update,
    /// Used to delete existing data
    #[serde(rename = "destroy")]
    Destroy,
    /// Used to check if data exists. If uri is *, this acts as a server health check
    #[serde(rename = "check")]
    Check,
    /// Used to authorize with a server
    #[serde(rename = "auth")]
    Auth,
    /// Used to get this sessions encryption key. assume unencrypted data unless this is sent
    #[serde(rename = "key")]
    Key
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContentType {
    /// AAC Audio
    #[serde(rename = "aac")]
    AAC,
    /// AVIF Image
    #[serde(rename = "avif")]
    AVIF,
    /// Audio Video Interleaf
    #[serde(rename = "avi")]
    AVI,
    /// Raw Binary Data
    #[serde(rename = "binary")]
    Binary,
    /// Bitmap Image
    #[serde(rename = "bmp")]
    BMP,
    /// Cascading Style Sheets in UTF-8
    #[serde(rename = "css")]
    CSS,
    /// Comma Seperated Values in UTF-8
    #[serde(rename = "csv")]
    CSV,
    /// Microsoft Word Document
    #[serde(rename = "docx")]
    DOCX,
    /// EPUB Book
    #[serde(rename = "epub")]
    EPUB,
    /// GIF Image
    #[serde(rename = "gif")]
    GIF,
    /// HTML in UTF-8
    #[serde(rename = "html")]
    HTML,
    /// Icon File
    #[serde(rename = "ico")]
    ICO,
    /// JPEG Image File
    #[serde(rename = "jpeg")]
    JPEG,
    /// JavaScript in UTF-8
    #[serde(rename = "javascript")]
    JavaScript,
    /// JavaScript Object Notation in UTF-8
    #[serde(rename = "json")]
    JSON,
    /// Markdown in UTF-8
    #[serde(rename = "markdown")]
    MarkDown,
    /// MIDI
    #[serde(rename = "midi")]
    MIDI,
    /// MP3 Audio
    #[serde(rename = "mp3")]
    MP3,
    /// MP4 Video
    #[serde(rename = "mp4")]
    MP4,
    /// MPEG Video
    #[serde(rename = "mpeg")]
    MPEG,
    /// OpenDocument Presentation
    #[serde(rename = "odp")]
    ODP,
    /// OpenDocument Spreadsheet
    #[serde(rename = "ods")]
    ODS,
    /// OpenDocument Text
    #[serde(rename = "odt")]
    ODT,
    /// Ogg Audio
    #[serde(rename = "oga")]
    OGA,
    /// Ogg Video
    #[serde(rename = "ogv")]
    OGV,
    /// Ogg
    #[serde(rename = "ogx")]
    OGX,
    /// Opus audio in Ogg Container
    #[serde(rename = "opus")]
    Opus,
    /// OpenType Font
    #[serde(rename = "otf")]
    OTF,
    /// PNG Image
    #[serde(rename = "png")]
    PNG,
    /// PDF Document
    #[serde(rename = "pdf")]
    PDF,
    /// PHP Page
    #[serde(rename = "php")]
    PHP,
    /// Microsoft PowerPoint Presentation
    #[serde(rename = "pptx")]
    PPTX,
    /// RAR Archive
    #[serde(rename = "rar")]
    RAR,
    /// SVG Graphics
    #[serde(rename = "svg")]
    SVG,
    /// TAR Archive
    #[serde(rename = "tar")]
    TAR,
    /// TIFF Image
    #[serde(rename = "tiff")]
    TIFF,
    /// TrueType Font
    #[serde(rename = "ttf")]
    TTF,
    /// Plain Text (UTF-8)
    #[serde(rename = "text")]
    Text,
    /// Waveform Audio
    #[serde(rename = "wav")]
    WAV,
    /// Webm Audio
    #[serde(rename = "weba")]
    WEBA,
    /// Webm Video
    #[serde(rename = "webm")]
    WEBM,
    /// Webm Image
    #[serde(rename = "webp")]
    WEBP,
    /// Woff font (WOFF1 or WOFF2)
    #[serde(rename = "woff")]
    WOFF,
    /// Microsoft Excel Spreadsheet
    #[serde(rename = "xlsx")]
    XLSX,
    /// XML (UTF-8)
    #[serde(rename = "xml")]
    XML,
    /// Zip Archive
    #[serde(rename = "zip")]
    ZIP,
    /// 7z archive
    #[serde(rename = "7z")]
    SevenZ,
    #[serde(rename = "none")]
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Encryption {
    #[serde(rename = "aes-gcm-256")]
    AESGCM256,
    #[serde(rename = "none")]
    None
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Request {
    #[serde(rename = "version")]
    version: String,
    #[serde(rename = "uri")]
    pub uri: String,
    #[serde(rename = "sent")]
    pub sent: String,
    #[serde(rename = "agent")]
    pub agent: String,
    #[serde(rename = "method")]
    pub method: Method,
    #[serde(rename = "lang")]
    pub language: String,
    #[serde(rename = "cookies")] // Cookies requested by the server
    pub cookies: Vec<Cookie>,
    #[serde(rename = "encryption")]
    pub encryption: Encryption,
    #[serde(skip)]
    encryption_key: Option<Key<Aes256>>,
    #[serde(rename = "nonce")]
    pub nonce: String,
    #[serde(rename = "contentType")]
    pub content_type: ContentType,
    #[serde(rename = "content")]
    pub content: Vec<u8>,
}

pub fn dateTimeString(date_time: DateTime<Utc>) -> String {
    date_time.to_rfc3339()
}

impl Request {
    pub fn create(method: Method, uri: String, agent: String) -> Request {
        Request {
            version: ADTP_VERSION.to_string(),
            uri,
            sent: dateTimeString(Utc::now()),
            agent,
            language: "none".to_string(),
            cookies: vec![],
            encryption: Encryption::None,
            encryption_key: None,
            nonce: "".to_string(),
            method,
            content_type: ContentType::None,
            content: vec![],
        }
    }

    pub fn with_time(mut self, sent: DateTime<Utc>) -> Request {
        self.sent = dateTimeString(sent);
        self
    }

    pub fn with_language(mut self, language: String) -> Request {
        self.language = language;
        self
    }

    pub fn with_cookie(mut self, cookie: Cookie) -> Request {
        self.cookies.push(cookie);
        self
    }

    pub fn with_content(mut self, content_type: ContentType, content: Vec<u8>) -> Request {
        self.content_type = content_type;
        self.content = content;
        self
    }

    pub fn with_encryption(mut self, encryption: Encryption, key: Key<Aes256>) -> Request {
        self.encryption = encryption;
        self.encryption_key = Some(key);
        let cipher = Aes256Gcm::new(&self.encryption_key.unwrap());
        let n = Aes256Gcm::generate_nonce(&mut OsRng);
        self.nonce = STANDARD.encode(n.as_slice());
        let ciphertext = cipher.encrypt(&n, self.content.clone().as_slice()).unwrap();

        self.nonce = STANDARD.encode(n.as_slice());
        self.content = ciphertext;
        self
    }
    
    pub fn decrypt_content(&mut self, key: Key<Aes256>) -> Vec<u8> {
        let cipher = Aes256Gcm::new(Key::<Aes256>::from_slice(key.as_slice()));
        cipher.decrypt(Nonce::from_slice(STANDARD.decode(self.nonce.clone()).unwrap().as_slice()), self.content.as_slice()).unwrap()
    }

    pub fn with_content_string(mut self, content_type: ContentType, content: String) -> Request {
        self.content_type = content_type;
        self.content = content.into_bytes();
        self
    }

    pub fn build(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
impl TryFrom<String> for Request {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match serde_json::from_str(&value) {
            Ok(r) => Ok(r),
            Err(e) => Err(format!("Failed to parse JSON: {}", e)),
        }
    }
}
impl TryFrom<Vec<u8>> for Request {
    type Error = String;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match String::from_utf8(value) {
            Ok(r) => match serde_json::from_str(r.as_str()) {
                Ok(r) => Ok(r),
                Err(e) => Err(format!("Failed to parse JSON: {}", e)),
            },
            Err(e) => Err(format!("Failed to parse JSON: {}", e)),
        }
    }
}
impl Display for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Status {
    #[serde(rename = "switch-protocols")]
    SwitchProtocols, // Server is switching protocol. Check content for protocol type
    #[serde(rename = "ok")]
    Ok, // server has handled your request
    #[serde(rename = "pending")]
    Pending, // Request is pending. Wait for follow up.
    #[serde(rename = "redirect")]
    Redirect, // Content is no longer at requested location. Check content for new location.
    #[serde(rename = "denied")]
    Denied, // Server has denied request.
    #[serde(rename = "bad-request")]
    BadRequest, // Request is malformed.
    #[serde(rename = "unauthorized")]
    Unauthorized, // You are not authorized to view the content
    #[serde(rename = "not-found")]
    NotFound, // Server could not find the requested content.
    #[serde(rename = "too-many-requests")]
    TooManyRequests, // Server has received too many requests from this client
    #[serde(rename = "internal-error")]
    InternalError, // Server has encountered an error
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Response {
    #[serde(rename = "version")]
    version: String,
    #[serde(rename = "status")]
    pub status: Status,
    #[serde(rename = "sent")]
    pub sent: String,
    #[serde(rename = "agent")]
    pub agent: String,
    #[serde(rename = "language")]
    pub language: String,
    #[serde(rename = "cookies")] // Cookies for the client to save
    pub cookies: Vec<Cookie>,
    #[serde(rename = "encryption")]
    pub encryption: Encryption,
    #[serde(skip)]
    encryption_key: Option<Key<Aes256>>,
    #[serde(rename = "nonce")]
    nonce: String,
    #[serde(rename = "contentType")]
    pub content_type: ContentType,
    #[serde(rename = "content")]
    pub content: Vec<u8>,
}
impl Response {
    pub fn create(status: Status, agent: String) -> Response {
        Response {
            version: ADTP_VERSION.to_string(),
            status,
            sent: dateTimeString(Utc::now()),
            agent,
            language: "none".to_string(),
            encryption: Encryption::None,
            encryption_key: None,
            nonce: "".to_string(),
            cookies: vec![],
            content_type: ContentType::None,
            content: vec![],
        }
    }

    pub fn with_time(mut self, sent: DateTime<Utc>) -> Response {
        self.sent = dateTimeString(sent);
        self
    }

    pub fn with_encryption(mut self, encryption: Encryption, key: Key<Aes256>) -> Response {
        self.encryption = encryption;
        self.encryption_key = Some(key);
        let cipher = Aes256Gcm::new(&self.encryption_key.unwrap());
        let n = Aes256Gcm::generate_nonce(&mut OsRng);
        self.nonce = STANDARD.encode(n.as_slice());
        let ciphertext = cipher.encrypt(&n, self.content.clone().as_slice()).unwrap();

        self.nonce = STANDARD.encode(n.as_slice());
        self.content = ciphertext;
        self
    }

    pub fn with_language(mut self, language: String) -> Response {
        self.language = language;
        self
    }

    pub fn with_cookie(mut self, cookie: Cookie) -> Response {
        self.cookies.push(cookie);
        self
    }

    pub fn with_content(mut self, content_type: ContentType, content: Vec<u8>) -> Response {
        self.content_type = content_type;
        self.content = content;
        self
    }

    pub fn decrypt_content(&mut self, key: Key<Aes256>) -> Vec<u8> {
        let cipher = Aes256Gcm::new(Key::<Aes256>::from_slice(key.as_slice()));
        cipher.decrypt(Nonce::from_slice(STANDARD.decode(self.nonce.clone()).unwrap().as_slice()), self.content.as_slice()).unwrap()
    }

    pub fn with_content_string(mut self, content_type: ContentType, content: String) -> Response {
        self.content_type = content_type;
        self.content = content.into_bytes();
        self
    }

    pub fn build(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl TryFrom<String> for Response {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match serde_json::from_str(&value) {
            Ok(r) => Ok(r),
            Err(e) => Err(format!("Failed to parse JSON: {}", e)),
        }
    }
}
impl TryFrom<Vec<u8>> for Response {
    type Error = String;
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        match String::from_utf8(value) {
            Ok(r) => match serde_json::from_str(r.as_str()) {
                Ok(r) => Ok(r),
                Err(e) => Err(format!("Failed to parse JSON: {}", e)),
            },
            Err(e) => Err(format!("Failed to parse JSON: {}", e)),
        }
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}
