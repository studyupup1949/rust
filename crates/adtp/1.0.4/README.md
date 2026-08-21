# The ADTP Protocol

ADTP (**A**lula's **D**ata **T**ransfer **P**rotocol)

## The Design

Here is a simple ATDP Request

```json
{
  "version": "ADTP/1.0",
  "uri": "example.com",
  "sent": "2025-05-20T23:47:24.854941600Z",
  "agent": "test/1.0",
  "method": "check",
  "lang": "us-english",
  "cookies": [],
  "encryption": "none",
  "nonce": "",
  "contentType": "none",
  "content": []
}
```

Here is a simple ATDP Response

```json
{
  "version": "ADTP/1.0",
  "status": "ok",
  "sent": "2025-05-20T23:47:24.854941600Z",
  "agent": "test/1.0",
  "language": "us-english",
  "cookies": [],
  "encryption": "none",
  "nonce": "",
  "contentType": "text",
  "content": []
}
```

While I think encryption is very annoying to impliment, It is an industry standard, and if I want to even come close to being a viable replacement for http, it must be present.
ADTP naitively supports aes256 gcm encryption. an encryption handshake goes as follows.
```
---REQUEST---
method: key
encryption: aes-gcm-256

---RESPONSE---
status: ok
encryption: aes-gcm-256
content: [the encryption key]
```

From this point on, all messages are encrypted with the key and a nonce generated for each request/response.

The following methods are avalible for ADTP.

read: functions as get does in HTTP

create: Create new data on the server

update: Update existing data on the server

destroy: Destroy/delete existing data on the server

check: used to check if data exists. if uri is *, this acts as a server health check

auth: used to authorize with a server. authorization is not standardized.

key: used to request the sessions encryption key from the server.

The following status codes are avalible for ADTP

switch-protocols: The server requests to switch protocols. Check content for new protocol.

ok: server has handled your request

pending: server has acknowledged your request. stand by for an ok upon handling.

redirect: content is no longer located at requested location. Check content for new location

denied: Server has denied your request

bad-request: request is malformed.

unauthorized: you are not authorized to view this content

not-found: server could not find the requested content

too-many-requests: server has recieved too many requests from this client

internal-error: server has encountered an internal error.

Here is a rust enum representing all the avalible content types

```rust
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
```