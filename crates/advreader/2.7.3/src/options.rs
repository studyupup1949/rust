use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Source {
    File(PathBuf),
    String(String),
    Bytes(Vec<u8>),
}

#[derive(Debug)]
pub struct AdvReaderOptions {
    pub source: Source,
    pub buffer_size: usize,
    pub trim: bool,
    pub line_end: u8,
    pub skip_comments: bool,
    /// Convert (Line) Comments into UTF8
    pub encode_comments: bool,
    /// Convert Strings into UTF8
    pub encode_strings: bool,
    /// Optional decoder for converting strings into UTF8
    pub encoding: Option<String>,
    /// Decoder error handling: strict, replace, ignore. Default is replace.
    pub encoding_errors: Option<String>,
    /// Valid characters for word: 0-9a-zA-Z_.
    pub extended_word_separation: bool,
    /// Special support for escaping double quote is: ""
    pub double_quote_escape: bool,
    /// Convert text to numbers (int, float)
    pub convert2numbers: bool,
    /// Keep base of number
    pub keep_base: bool,
    /// If defined boolean False detection is enabled.
    pub bool_false: Option<Vec<u8>>,
    /// If defined boolean True detection is enabled.
    pub bool_true: Option<Vec<u8>>,
}

impl AdvReaderOptions {
    pub fn new(source: Source, encoding: Option<String>, decoder_errors: Option<String>) -> Self {
        Self {
            source,
            buffer_size: 65536,
            trim: false,
            line_end: b'\n',
            skip_comments: false,
            encode_comments: true,
            encode_strings: true,
            encoding,
            encoding_errors: decoder_errors,
            extended_word_separation: false,
            double_quote_escape: false,
            convert2numbers: true,
            keep_base: false,
            bool_false: None,
            bool_true: None,
        }
    }
}
