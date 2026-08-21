use serde::{Deserialize, Serialize};

use crate::{Error, Result, impl_default, impl_display};

/// Represents HTTP MIME types.
///
/// MIME types should be listed in the [official IANA MIME types](https://www.iana.org/assignments/media-types/media-types.xhtml).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum MimeType {
    #[serde(rename = "application")]
    Application,
    #[serde(rename = "application/x-git-patch")]
    ApplicationGitPatch,
    #[serde(rename = "application/javascript")]
    ApplicationJavascript,
    #[serde(rename = "application/javascript; charset=utf-8")]
    ApplicationJavascriptUtf8,
    #[serde(rename = "application/json")]
    ApplicationJson,
    #[serde(rename = "application/activity+json")]
    ApplicationActivityJson,
    #[serde(rename = "application/msgpack")]
    ApplicationMsgpack,
    #[serde(rename = "application/octet-stream")]
    ApplicationOctetStream,
    #[serde(rename = "application/pdf")]
    ApplicationPdf,
    #[serde(rename = "application/x-www-form-urlencoded")]
    ApplicationWwwFormUrlEncoded,
    #[serde(rename = "audio")]
    Audio,
    #[serde(rename = "audio/3gpp")]
    Audio3gpp,
    #[serde(rename = "audio/3gpp2")]
    Audio3gpp2,
    #[serde(rename = "audio/aac")]
    AudioAac,
    #[serde(rename = "audio/midi")]
    AudioMidi,
    #[serde(rename = "audio/x-midi")]
    AudioXMidi,
    #[serde(rename = "audio/mp3")]
    AudioMp3,
    #[serde(rename = "audio/mpeg")]
    AudioMpeg,
    #[serde(rename = "audio/ogg")]
    AudioOgg,
    #[serde(rename = "audio/wav")]
    AudioWav,
    #[serde(rename = "audio/webm")]
    AudioWebm,
    #[serde(rename = "basic")]
    Basic,
    #[serde(rename = "bmp")]
    Bmp,
    #[serde(rename = "boundary")]
    Boundary,
    #[serde(rename = "charset")]
    Charset,
    #[serde(rename = "css")]
    Css,
    #[serde(rename = "csv")]
    Csv,
    #[serde(rename = "event-stream")]
    EventStream,
    #[serde(rename = "font")]
    Font,
    #[serde(rename = "font/woff")]
    FontWoff,
    #[serde(rename = "font/woff2")]
    FontWoff2,
    #[serde(rename = "form-data")]
    FormData,
    #[serde(rename = "gif")]
    Gif,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "image")]
    Image,
    #[serde(rename = "image/bmp")]
    ImageBmp,
    #[serde(rename = "image/gif")]
    ImageGif,
    #[serde(rename = "image/jpeg")]
    ImageJpeg,
    #[serde(rename = "image/jxl")]
    ImageJpegXl,
    #[serde(rename = "image/png")]
    ImagePng,
    #[serde(rename = "image/*")]
    ImageStar,
    #[serde(rename = "image/svg+xml")]
    ImageSvgXml,
    #[serde(rename = "javascript")]
    Javascript,
    #[serde(rename = "jpeg")]
    Jpeg,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "message")]
    Message,
    #[serde(rename = "model")]
    Model,
    #[serde(rename = "mp4")]
    Mp4,
    #[serde(rename = "mpeg")]
    Mpeg,
    #[serde(rename = "msgpack")]
    Msgpack,
    #[serde(rename = "multipart")]
    Multipart,
    #[serde(rename = "multipart/form-data")]
    MultipartFormData,
    #[serde(rename = "octet-stream")]
    OctetStream,
    #[serde(rename = "ogg")]
    Ogg,
    #[serde(rename = "pdf")]
    Pdf,
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "*")]
    Star,
    #[serde(rename = "*/*")]
    StarStar,
    #[serde(rename = "svg")]
    Svg,
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "text/css")]
    TextCss,
    #[serde(rename = "text/css; charset=utf-8")]
    TextCssUtf8,
    #[serde(rename = "text/csv")]
    TextCsv,
    #[serde(rename = "text/csv; charset=utf-8")]
    TextCsvUtf8,
    #[serde(rename = "text/x-diff")]
    TextDiff,
    #[serde(rename = "text/event-stream")]
    TextEventStream,
    #[serde(rename = "text/html")]
    TextHtml,
    #[serde(rename = "text/html; charset=utf-8")]
    TextHtmlUtf8,
    #[serde(rename = "text/markdown")]
    TextMarkdown,
    #[serde(rename = "text/markdown; variant=CommonMark")]
    TextMarkdownCommonMark,
    #[serde(rename = "text/plain")]
    TextPlain,
    #[serde(rename = "text/*")]
    TextStar,
    #[serde(rename = "text/tab-separated-values")]
    TextTsv,
    #[serde(rename = "text/tab-separated-values; charset=utf-8")]
    TextTsvUtf8,
    #[serde(rename = "text/vcard")]
    TextVcard,
    #[serde(rename = "text/xml")]
    TextXml,
    #[serde(rename = "utf-8")]
    Utf8,
    #[serde(rename = "vcard")]
    Vcard,
    #[serde(rename = "video")]
    Video,
    #[serde(rename = "video/3gpp")]
    Video3gpp,
    #[serde(rename = "video/3gpp2")]
    Video3gpp2,
    #[serde(rename = "video/x-msvideo")]
    VideoAvi,
    #[serde(rename = "video/mp4")]
    VideoMp4,
    #[serde(rename = "video/mpeg")]
    VideoMpeg,
    #[serde(rename = "video/ogg")]
    VideoOgg,
    #[serde(rename = "video/mp2t")]
    VideoTs,
    #[serde(rename = "video/webm")]
    VideoWebm,
    #[serde(rename = "woff")]
    Woff,
    #[serde(rename = "woff2")]
    Woff2,
    #[serde(rename = "x-www-form-urlencoded")]
    WwwFormUrlEncoded,
    #[serde(rename = "xml")]
    Xml,
}

impl MimeType {
    pub const APPLICATION: &str = "application";
    pub const APPLICATION_GIT_PATCH: &str = "application/x-git-patch";
    pub const APPLICATION_JAVASCRIPT: &str = "application/javascript";
    pub const APPLICATION_JAVASCRIPT_UTF8: &str = "application/javascript; charset=utf-8";
    pub const APPLICATION_JSON: &str = "application/json";
    pub const APPLICATION_ACTIVITY_JSON: &str = "application/activity+json";
    pub const APPLICATION_MSGPACK: &str = "application/msgpack";
    pub const APPLICATION_OCTET_STREAM: &str = "application/octet-stream";
    pub const APPLICATION_PDF: &str = "application/pdf";
    pub const APPLICATION_WWW_FORM_URL_ENCODED: &str = "application/x-www-form-urlencoded";
    pub const AUDIO: &str = "audio";
    pub const AUDIO_3GPP: &str = "audio/3gpp";
    pub const AUDIO_3GPP2: &str = "audio/3gpp2";
    pub const AUDIO_AAC: &str = "audio/aac";
    pub const AUDIO_MIDI: &str = "audio/midi";
    pub const AUDIO_X_MIDI: &str = "audio/x-midi";
    pub const AUDIO_MP3: &str = "audio/mp3";
    pub const AUDIO_MPEG: &str = "audio/mpeg";
    pub const AUDIO_OGG: &str = "audio/ogg";
    pub const AUDIO_WAV: &str = "audio/wav";
    pub const AUDIO_WEBM: &str = "audio/webm";
    pub const BASIC: &str = "basic";
    pub const BMP: &str = "bmp";
    pub const BOUNDARY: &str = "boundary";
    pub const CHARSET: &str = "charset";
    pub const CSS: &str = "css";
    pub const CSV: &str = "csv";
    pub const EVENT_STREAM: &str = "event-stream";
    pub const FONT: &str = "font";
    pub const FONT_WOFF: &str = "font/woff";
    pub const FONT_WOFF2: &str = "font/woff2";
    pub const FORM_DATA: &str = "form-data";
    pub const GIF: &str = "gif";
    pub const HTML: &str = "html";
    pub const IMAGE: &str = "image";
    pub const IMAGE_BMP: &str = "image/bmp";
    pub const IMAGE_GIF: &str = "image/gif";
    pub const IMAGE_JPEG: &str = "image/jpeg";
    pub const IMAGE_JPEG_XL: &str = "image/jxl";
    pub const IMAGE_PNG: &str = "image/png";
    pub const IMAGE_STAR: &str = "image/*";
    pub const IMAGE_SVG_XML: &str = "image/svg+xml";
    pub const JAVASCRIPT: &str = "javascript";
    pub const JPEG: &str = "jpeg";
    pub const JSON: &str = "json";
    pub const MESSAGE: &str = "message";
    pub const MODEL: &str = "model";
    pub const MP4: &str = "mp4";
    pub const MPEG: &str = "mpeg";
    pub const MSGPACK: &str = "msgpack";
    pub const MULTIPART: &str = "multipart";
    pub const MULTIPART_FORM_DATA: &str = "multipart/form-data";
    pub const OCTET_STREAM: &str = "octet-stream";
    pub const OGG: &str = "ogg";
    pub const PDF: &str = "pdf";
    pub const PLAIN: &str = "plain";
    pub const STAR: &str = "*";
    pub const STAR_STAR: &str = "*/*";
    pub const SVG: &str = "svg";
    pub const TEXT: &str = "text";
    pub const TEXT_CSS: &str = "text/css";
    pub const TEXT_CSS_UTF8: &str = "text/css; charset=utf-8";
    pub const TEXT_CSV: &str = "text/csv";
    pub const TEXT_CSV_UTF8: &str = "text/csv; charset=utf-8";
    pub const TEXT_DIFF: &str = "text/x-diff";
    pub const TEXT_EVENT_STREAM: &str = "text/event-stream";
    pub const TEXT_HTML: &str = "text/html";
    pub const TEXT_HTML_UTF8: &str = "text/html; charset=utf-8";
    pub const TEXT_MARKDOWN: &str = "text/markdown";
    pub const TEXT_MARKDOWN_COMMON_MARK: &str = "text/markdown; variant=CommonMark";
    pub const TEXT_PLAIN: &str = "text/plain";
    pub const TEXT_STAR: &str = "text/*";
    pub const TEXT_TSV: &str = "text/tab-separated-values";
    pub const TEXT_TSV_UTF8: &str = "text/tab-separated-values; charset=utf-8";
    pub const TEXT_VCARD: &str = "text/vcard";
    pub const TEXT_XML: &str = "text/xml";
    pub const UTF8: &str = "utf-8";
    pub const VCARD: &str = "vcard";
    pub const VIDEO: &str = "video";
    pub const VIDEO_3GPP: &str = "video/3gpp";
    pub const VIDEO_3GPP2: &str = "video/3gpp2";
    pub const VIDEO_AVI: &str = "video/x-msvideo";
    pub const VIDEO_MP4: &str = "video/mp4";
    pub const VIDEO_MPEG: &str = "video/mpeg";
    pub const VIDEO_OGG: &str = "video/ogg";
    pub const VIDEO_TS: &str = "video/mp2t";
    pub const VIDEO_WEBM: &str = "video/webm";
    pub const WOFF: &str = "woff";
    pub const WOFF2: &str = "woff2";
    pub const WWW_FORM_URL_ENCODED: &str = "x-www-form-urlencoded";
    pub const XML: &str = "xml";

    /// Creates a new [MimeType].
    pub const fn new() -> Self {
        Self::Application
    }

    /// Gets the string representation of the [MimeType].
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Application => Self::APPLICATION,
            Self::ApplicationGitPatch => Self::APPLICATION_GIT_PATCH,
            Self::ApplicationJavascript => Self::APPLICATION_JAVASCRIPT,
            Self::ApplicationJavascriptUtf8 => Self::APPLICATION_JAVASCRIPT_UTF8,
            Self::ApplicationJson => Self::APPLICATION_JSON,
            Self::ApplicationActivityJson => Self::APPLICATION_ACTIVITY_JSON,
            Self::ApplicationMsgpack => Self::APPLICATION_MSGPACK,
            Self::ApplicationOctetStream => Self::APPLICATION_OCTET_STREAM,
            Self::ApplicationPdf => Self::APPLICATION_PDF,
            Self::ApplicationWwwFormUrlEncoded => Self::APPLICATION_WWW_FORM_URL_ENCODED,
            Self::Audio => Self::AUDIO,
            Self::Audio3gpp => Self::AUDIO_3GPP,
            Self::Audio3gpp2 => Self::AUDIO_3GPP2,
            Self::AudioAac => Self::AUDIO_AAC,
            Self::AudioMidi => Self::AUDIO_MIDI,
            Self::AudioXMidi => Self::AUDIO_X_MIDI,
            Self::AudioMp3 => Self::AUDIO_MP3,
            Self::AudioMpeg => Self::AUDIO_MPEG,
            Self::AudioOgg => Self::AUDIO_OGG,
            Self::AudioWav => Self::AUDIO_WAV,
            Self::AudioWebm => Self::AUDIO_WEBM,
            Self::Basic => Self::BASIC,
            Self::Bmp => Self::BMP,
            Self::Boundary => Self::BOUNDARY,
            Self::Charset => Self::CHARSET,
            Self::Css => Self::CSS,
            Self::Csv => Self::CSV,
            Self::EventStream => Self::EVENT_STREAM,
            Self::Font => Self::FONT,
            Self::FontWoff => Self::FONT_WOFF,
            Self::FontWoff2 => Self::FONT_WOFF2,
            Self::FormData => Self::FORM_DATA,
            Self::Gif => Self::GIF,
            Self::Html => Self::HTML,
            Self::Image => Self::IMAGE,
            Self::ImageBmp => Self::IMAGE_BMP,
            Self::ImageGif => Self::IMAGE_GIF,
            Self::ImageJpeg => Self::IMAGE_JPEG,
            Self::ImageJpegXl => Self::IMAGE_JPEG_XL,
            Self::ImagePng => Self::IMAGE_PNG,
            Self::ImageStar => Self::IMAGE_STAR,
            Self::ImageSvgXml => Self::IMAGE_SVG_XML,
            Self::Javascript => Self::JAVASCRIPT,
            Self::Jpeg => Self::JPEG,
            Self::Json => Self::JSON,
            Self::Message => Self::MESSAGE,
            Self::Model => Self::MODEL,
            Self::Mp4 => Self::MP4,
            Self::Mpeg => Self::MPEG,
            Self::Msgpack => Self::MSGPACK,
            Self::Multipart => Self::MULTIPART,
            Self::MultipartFormData => Self::MULTIPART_FORM_DATA,
            Self::OctetStream => Self::OCTET_STREAM,
            Self::Ogg => Self::OGG,
            Self::Pdf => Self::PDF,
            Self::Plain => Self::PLAIN,
            Self::Star => Self::STAR,
            Self::StarStar => Self::STAR_STAR,
            Self::Svg => Self::SVG,
            Self::Text => Self::TEXT,
            Self::TextCss => Self::TEXT_CSS,
            Self::TextCssUtf8 => Self::TEXT_CSS_UTF8,
            Self::TextCsv => Self::TEXT_CSV,
            Self::TextCsvUtf8 => Self::TEXT_CSV_UTF8,
            Self::TextDiff => Self::TEXT_DIFF,
            Self::TextEventStream => Self::TEXT_EVENT_STREAM,
            Self::TextHtml => Self::TEXT_HTML,
            Self::TextHtmlUtf8 => Self::TEXT_HTML_UTF8,
            Self::TextMarkdown => Self::TEXT_MARKDOWN,
            Self::TextMarkdownCommonMark => Self::TEXT_MARKDOWN_COMMON_MARK,
            Self::TextPlain => Self::TEXT_PLAIN,
            Self::TextStar => Self::TEXT_STAR,
            Self::TextTsv => Self::TEXT_TSV,
            Self::TextTsvUtf8 => Self::TEXT_TSV_UTF8,
            Self::TextVcard => Self::TEXT_VCARD,
            Self::TextXml => Self::TEXT_XML,
            Self::Utf8 => Self::UTF8,
            Self::Vcard => Self::VCARD,
            Self::Video => Self::VIDEO,
            Self::Video3gpp => Self::VIDEO_3GPP,
            Self::Video3gpp2 => Self::VIDEO_3GPP2,
            Self::VideoAvi => Self::VIDEO_AVI,
            Self::VideoMp4 => Self::VIDEO_MP4,
            Self::VideoMpeg => Self::VIDEO_MPEG,
            Self::VideoOgg => Self::VIDEO_OGG,
            Self::VideoTs => Self::VIDEO_TS,
            Self::VideoWebm => Self::VIDEO_WEBM,
            Self::Woff => Self::WOFF,
            Self::Woff2 => Self::WOFF2,
            Self::WwwFormUrlEncoded => Self::WWW_FORM_URL_ENCODED,
            Self::Xml => Self::XML,
        }
    }

    /// Attempts to convert a string into a [MimeType].
    pub fn try_from_str(val: &str) -> Result<Self> {
        match val.to_lowercase().as_str() {
            Self::APPLICATION => Ok(Self::Application),
            Self::APPLICATION_GIT_PATCH => Ok(Self::ApplicationGitPatch),
            Self::APPLICATION_JAVASCRIPT => Ok(Self::ApplicationJavascript),
            Self::APPLICATION_JAVASCRIPT_UTF8 => Ok(Self::ApplicationJavascriptUtf8),
            Self::APPLICATION_JSON => Ok(Self::ApplicationJson),
            Self::APPLICATION_ACTIVITY_JSON => Ok(Self::ApplicationActivityJson),
            Self::APPLICATION_MSGPACK => Ok(Self::ApplicationMsgpack),
            Self::APPLICATION_OCTET_STREAM => Ok(Self::ApplicationOctetStream),
            Self::APPLICATION_PDF => Ok(Self::ApplicationPdf),
            Self::APPLICATION_WWW_FORM_URL_ENCODED => Ok(Self::ApplicationWwwFormUrlEncoded),
            Self::AUDIO => Ok(Self::Audio),
            Self::AUDIO_3GPP => Ok(Self::Audio3gpp),
            Self::AUDIO_3GPP2 => Ok(Self::Audio3gpp2),
            Self::AUDIO_AAC => Ok(Self::AudioAac),
            Self::AUDIO_MIDI => Ok(Self::AudioMidi),
            Self::AUDIO_X_MIDI => Ok(Self::AudioXMidi),
            Self::AUDIO_MP3 => Ok(Self::AudioMp3),
            Self::AUDIO_MPEG => Ok(Self::AudioMpeg),
            Self::AUDIO_OGG => Ok(Self::AudioOgg),
            Self::AUDIO_WAV => Ok(Self::AudioWav),
            Self::AUDIO_WEBM => Ok(Self::AudioWebm),
            Self::BASIC => Ok(Self::Basic),
            Self::BMP => Ok(Self::Bmp),
            Self::BOUNDARY => Ok(Self::Boundary),
            Self::CHARSET => Ok(Self::Charset),
            Self::CSS => Ok(Self::Css),
            Self::CSV => Ok(Self::Csv),
            Self::EVENT_STREAM => Ok(Self::EventStream),
            Self::FONT => Ok(Self::Font),
            Self::FONT_WOFF => Ok(Self::FontWoff),
            Self::FONT_WOFF2 => Ok(Self::FontWoff2),
            Self::FORM_DATA => Ok(Self::FormData),
            Self::GIF => Ok(Self::Gif),
            Self::HTML => Ok(Self::Html),
            Self::IMAGE => Ok(Self::Image),
            Self::IMAGE_BMP => Ok(Self::ImageBmp),
            Self::IMAGE_GIF => Ok(Self::ImageGif),
            Self::IMAGE_JPEG => Ok(Self::ImageJpeg),
            Self::IMAGE_JPEG_XL => Ok(Self::ImageJpegXl),
            Self::IMAGE_PNG => Ok(Self::ImagePng),
            Self::IMAGE_STAR => Ok(Self::ImageStar),
            Self::IMAGE_SVG_XML => Ok(Self::ImageSvgXml),
            Self::JAVASCRIPT => Ok(Self::Javascript),
            Self::JPEG => Ok(Self::Jpeg),
            Self::JSON => Ok(Self::Json),
            Self::MESSAGE => Ok(Self::Message),
            Self::MODEL => Ok(Self::Model),
            Self::MP4 => Ok(Self::Mp4),
            Self::MPEG => Ok(Self::Mpeg),
            Self::MSGPACK => Ok(Self::Msgpack),
            Self::MULTIPART => Ok(Self::Multipart),
            Self::MULTIPART_FORM_DATA => Ok(Self::MultipartFormData),
            Self::OCTET_STREAM => Ok(Self::OctetStream),
            Self::OGG => Ok(Self::Ogg),
            Self::PDF => Ok(Self::Pdf),
            Self::PLAIN => Ok(Self::Plain),
            Self::STAR => Ok(Self::Star),
            Self::STAR_STAR => Ok(Self::StarStar),
            Self::SVG => Ok(Self::Svg),
            Self::TEXT => Ok(Self::Text),
            Self::TEXT_CSS => Ok(Self::TextCss),
            Self::TEXT_CSS_UTF8 => Ok(Self::TextCssUtf8),
            Self::TEXT_CSV => Ok(Self::TextCsv),
            Self::TEXT_CSV_UTF8 => Ok(Self::TextCsvUtf8),
            Self::TEXT_DIFF => Ok(Self::TextDiff),
            Self::TEXT_EVENT_STREAM => Ok(Self::TextEventStream),
            Self::TEXT_HTML => Ok(Self::TextHtml),
            Self::TEXT_HTML_UTF8 => Ok(Self::TextHtmlUtf8),
            Self::TEXT_MARKDOWN => Ok(Self::TextMarkdown),
            Self::TEXT_MARKDOWN_COMMON_MARK => Ok(Self::TextMarkdownCommonMark),
            Self::TEXT_PLAIN => Ok(Self::TextPlain),
            Self::TEXT_STAR => Ok(Self::TextStar),
            Self::TEXT_TSV => Ok(Self::TextTsv),
            Self::TEXT_TSV_UTF8 => Ok(Self::TextTsvUtf8),
            Self::TEXT_VCARD => Ok(Self::TextVcard),
            Self::TEXT_XML => Ok(Self::TextXml),
            Self::UTF8 => Ok(Self::Utf8),
            Self::VCARD => Ok(Self::Vcard),
            Self::VIDEO => Ok(Self::Video),
            Self::VIDEO_3GPP => Ok(Self::Video3gpp),
            Self::VIDEO_3GPP2 => Ok(Self::Video3gpp2),
            Self::VIDEO_AVI => Ok(Self::VideoAvi),
            Self::VIDEO_MP4 => Ok(Self::VideoMp4),
            Self::VIDEO_MPEG => Ok(Self::VideoMpeg),
            Self::VIDEO_OGG => Ok(Self::VideoOgg),
            Self::VIDEO_TS => Ok(Self::VideoTs),
            Self::VIDEO_WEBM => Ok(Self::VideoWebm),
            Self::WOFF => Ok(Self::Woff),
            Self::WOFF2 => Ok(Self::Woff2),
            Self::WWW_FORM_URL_ENCODED => Ok(Self::WwwFormUrlEncoded),
            Self::XML => Ok(Self::Xml),
            bad_mime => Err(Error::Mime(format!("invalid MIME type: {bad_mime}"))),
        }
    }
}

impl_default!(MimeType);
impl_display!(MimeType, str);
