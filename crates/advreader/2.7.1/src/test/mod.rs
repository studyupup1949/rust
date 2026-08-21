use std::io::Error;

use crate::{AdvReader, AdvReturnValue, ReaderState};

mod read_block_a2l;
mod read_comment;
mod read_line_comment;
mod read_number;
mod read_push_back;
mod read_string;
mod reader_read;

#[derive(Default)]
pub struct Results {
    pub bytes: Vec<Vec<u8>>,
    pub strings: Vec<Vec<u8>>,
    pub comments: Vec<Vec<u8>>,
    pub line_comments: Vec<Vec<u8>>,
    pub strings_utf8: Vec<String>,
    pub comments_utf8: Vec<String>,
    pub line_comments_utf8: Vec<String>,
    pub bools: Vec<bool>,
    pub ints: Vec<i64>,
    pub hexs: Vec<i64>,
    pub octs: Vec<i64>,
    pub bins: Vec<i64>,
    pub floats: Vec<f64>,
    pub blocks: Vec<Vec<u8>>,
    pub errors: Vec<String>,
    pub state: Option<Result<(usize, ReaderState), Error>>,
    pub last_bytes: Option<Vec<u8>>,
    pub line_nr: usize,
}

#[allow(dead_code)]
pub fn parse_file(reader: AdvReader) -> Results {
    let mut results = Results {
        ..Default::default()
    };
    let mut reader_iter = reader.into_iter();

    while let Some(result) = reader_iter.next() {
        results.line_nr = reader_iter.line_nr();
        match result {
            Ok(r) => match r {
                AdvReturnValue::Bytes(v) => results.bytes.push(v),
                AdvReturnValue::String(v) => results.strings.push(v),
                AdvReturnValue::Comment(v) => results.comments.push(v),
                AdvReturnValue::LineComment(v) => results.line_comments.push(v),
                AdvReturnValue::StringUtf8(v) => results.strings_utf8.push(v),
                AdvReturnValue::CommentUtf8(v) => results.comments_utf8.push(v),
                AdvReturnValue::LineCommentUtf8(v) => results.line_comments_utf8.push(v),
                AdvReturnValue::Bool(v) => results.bools.push(v),
                AdvReturnValue::Int(v) => results.ints.push(v),
                AdvReturnValue::Hex(v) => results.hexs.push(v),
                AdvReturnValue::Oct(v) => results.octs.push(v),
                AdvReturnValue::Bin(v) => results.bins.push(v),
                AdvReturnValue::Float(v) => results.floats.push(v),
                AdvReturnValue::Block(v) => results.blocks.push(v),
            },
            Err(e) => {
                results
                    .errors
                    .push(format!("ERROR ({}): {e}", reader_iter.line_nr()));
                break;
            }
        }
    }
    if !results.bytes.is_empty() {
        results.last_bytes = Some(results.bytes.get(results.bytes.len() - 1).unwrap().clone());
    }
    results.state = Some(reader_iter.stop());
    results
}
