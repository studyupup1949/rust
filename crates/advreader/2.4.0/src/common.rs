use std::iter::Enumerate;
use std::path::{Path, PathBuf};
use std::str::from_utf8;

pub type FnBlockReturnType = (usize, isize, Option<Vec<AdvReturnValue>>);

pub type FnReadBlockType = Option<
    Box<
        dyn Fn(
                Option<&[u8]>, // item
                &[u8],         // buffer
                &mut Enumerate<std::slice::Iter<'_, u8>>,
                usize,      // i
                u8,         // c
                &mut usize, // line_num
                u8,         // line_end
            ) -> FnBlockReturnType
            + Send
            + 'static,
    >,
>;

#[derive(PartialEq, Clone, Debug)]
pub enum ReaderState {
    Default,
    Number,
    String,
    LineComment,
    Comment,
    Block,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AdvReturnValue {
    Bytes(Vec<u8>),
    String(Vec<u8>),
    Comment(Vec<u8>),
    LineComment(Vec<u8>),
    StringUtf8(String),
    CommentUtf8(String),
    LineCommentUtf8(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Hex(i64),
    Oct(i64),
    Bin(i64),
    Block(Vec<u8>),
}

impl AdvReturnValue {
    pub fn as_string(&self) -> String {
        match &self {
            AdvReturnValue::Bytes(v) => format!("Bytes({:?})", from_utf8(v)),
            AdvReturnValue::String(v) => format!("String({:?})", from_utf8(v)),
            AdvReturnValue::Comment(v) => format!("Comment({:?})", from_utf8(v)),
            AdvReturnValue::LineComment(v) => format!("LineComment({:?})", from_utf8(v)),
            AdvReturnValue::StringUtf8(v) => format!("StringUtf8({v})"),
            AdvReturnValue::CommentUtf8(v) => format!("CommentUtf8({v})"),
            AdvReturnValue::LineCommentUtf8(v) => format!("LineCommentUtf8({v})"),
            AdvReturnValue::Bool(v) => format!("Bool({v:?})"),
            AdvReturnValue::Int(v) => format!("Int({v:?})"),
            AdvReturnValue::Float(v) => format!("Float({v:?})"),
            AdvReturnValue::Hex(v) => format!("Hex({v:?})"),
            AdvReturnValue::Oct(v) => format!("Oct({v:?})"),
            AdvReturnValue::Bin(v) => format!("Bin({v:?})"),
            AdvReturnValue::Block(v) => format!("Block({:?})", from_utf8(v)),
        }
    }
}

#[derive(Debug)]
pub struct AdvReaderOptions {
    pub path: PathBuf,
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
    pub fn new(path: &Path, encoding: Option<String>, decoder_errors: Option<String>) -> Self {
        Self {
            path: path.to_path_buf(),
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
