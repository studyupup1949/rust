use std::iter::Enumerate;
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

#[derive(Clone, Debug, PartialEq)]
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
