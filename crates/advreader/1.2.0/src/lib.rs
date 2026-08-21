//! `advreader` is a simple library crate offering an iterator which splits a file into
//! - text sequences, separated by characters with ASCII codes <=32 and >=127.
//! - strings with double quotes as delimiters.
//! - line comments with '//' as start sequence.
//! - comment blocks with '/*' as start sequence and '*/' as end sequence.
//!
//! Results can be obatined through the `next` method.
//! Property `line_nr` provides the current line in the text file.
#![doc(html_root_url = "https://docs.rs/advreader/1.0.0")]
use std::fs::{metadata, File};
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use flume::{bounded, Receiver, Sender};

#[derive(Debug)]
pub struct AdvReaderOptions {
    path: PathBuf,
    buffer_size: usize,
    trim: bool,
    line_end: u8,
    skip_comments: bool,
    encode_comments: bool,
    /// Convert Strings and (line) comments into UTF8
    encode: bool,
    /// Allow invalid UTF8 characters.
    allow_invalid_utf8: bool,
    /// Valid characters for word: 0-9a-zA-Z_.
    extended_word_separation: bool,
    /// Special support for escaping double quote is: ""
    double_double_quote_escape: bool,
    /// Convert text to numbers (int, float)
    convert2numbers: bool,
    /// Keep base of number
    keep_base: bool,
    /// If defined boolean False detection is enabled.
    bool_false: Option<Vec<u8>>,
    /// If defined boolean True detection is enabled.
    bool_true: Option<Vec<u8>>,
    max_block_size: usize,
}

#[derive(Clone, Debug)]
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ReaderStates {
    Default,
    Number,
    Slash,
    LineComment,
    Comment,
    CommentAsterisk,
    String,
}

/// Provides iteration over bytes or utf8 string of words, strings and (line) comments.
///
/// ```rust
/// use std::path::PathBuf;
/// use advreader::*;
///
/// // construct our iterator from our file input
/// let reader = AdvReader::default(&PathBuf::from("../res/example.txt"));
///
/// let mut reader_ok = reader.unwrap();
///
/// // walk our item using `while` syntax
/// while let Some(item) = reader_ok.next() {
///     // do something with the item, which is Result<&[u8], _>
/// }
/// ```
///
/// For those who prefer the `Iterator` API, this structure implements
/// the `IntoIterator` trait to provide it. This comes at the cost of
/// an allocation of a `Vec` for each line in the `Iterator`. This is
/// negligible in many cases, so often it comes down to which syntax
/// is preferred:
///
/// ```rust
/// use std::path::PathBuf;
/// use advreader::*;
///
/// // construct our iterator from our file input
/// let reader = AdvReader::default(&PathBuf::from("../res/example.txt"));
///
/// let mut reader_ok = reader.unwrap();
///
/// // walk our items using `for` syntax
/// for item in reader_ok.into_iter() {
///     // do something with the item, which is Result<AdvReturnValue, Error>
/// }
/// ```
#[derive(Debug)]
pub struct AdvReader {
    thread_handle: Option<JoinHandle<Result<(), Error>>>,
    items: Receiver<Option<(usize, Result<AdvReturnValue, Error>)>>,
    stop: Arc<AtomicBool>,
    line_num: usize,
    reader_died: bool,
}

impl AdvReader {
    /// Constructs a new `AdvReader`.
    pub fn new(
        path: &PathBuf,
        trim: Option<bool>,
        line_end: Option<u8>,
        skip_comments: Option<bool>,
        encode_comments: Option<bool>,
        encode: Option<bool>,
        allow_invalid_utf8: Option<bool>,
        extended_word_separation: Option<bool>,
        double_double_quote_escape: Option<bool>,
        convert2numbers: Option<bool>,
        keep_base: Option<bool>,
        bool_false: Option<Vec<u8>>,
        bool_true: Option<Vec<u8>>,
        fn_block_mode: Option<
            Box<dyn Fn(Option<&[u8]>) -> (bool, Option<Vec<Vec<u8>>>) + Send + 'static>,
        >,
    ) -> Result<Self, Error> {
        AdvReader::with_capacity(
            path,
            65536,
            trim,
            line_end,
            skip_comments,
            encode_comments,
            encode,
            allow_invalid_utf8,
            extended_word_separation,
            double_double_quote_escape,
            convert2numbers,
            keep_base,
            bool_false,
            bool_true,
            fn_block_mode,
        )
    }

    pub fn default(path: &PathBuf) -> Result<Self, Error> {
        AdvReader::new(
            path,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None as Option<
                Box<dyn Fn(Option<&[u8]>) -> (bool, Option<Vec<Vec<u8>>>) + Send + 'static>,
            >,
        )
    }

    /// Constructs a new `AdvReader`.
    pub fn with_capacity(
        path: &PathBuf,
        buffer_size: usize,
        trim: Option<bool>,
        line_end: Option<u8>,
        skip_comments: Option<bool>,
        encode_comments: Option<bool>,
        encode: Option<bool>,
        allow_invalid_utf8: Option<bool>,
        extended_word_separation: Option<bool>,
        double_double_quote_escape: Option<bool>,
        convert2numbers: Option<bool>,
        keep_base: Option<bool>,
        bool_false: Option<Vec<u8>>,
        bool_true: Option<Vec<u8>>,
        fn_block_mode: Option<
            Box<dyn Fn(Option<&[u8]>) -> (bool, Option<Vec<Vec<u8>>>) + Send + 'static>,
        >,
    ) -> Result<Self, Error> {
        if metadata(path).is_err() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("File {:?} not found or is not readable!", path),
            ));
        }
        if buffer_size < 4096 {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Buffer size must be at least 4096!",
            ));
        }
        let (tx, rx) = bounded(64);
        let stop = Arc::new(AtomicBool::new(false));
        let encode = encode.unwrap_or(false);
        let options = AdvReaderOptions {
            path: path.to_owned(),
            buffer_size,
            trim: trim.unwrap_or(false),
            line_end: line_end.unwrap_or(b'\n'),
            skip_comments: skip_comments.unwrap_or(false),
            encode_comments: encode_comments.unwrap_or(encode),
            encode,
            allow_invalid_utf8: allow_invalid_utf8.unwrap_or(false),
            extended_word_separation: extended_word_separation.unwrap_or(false),
            double_double_quote_escape: double_double_quote_escape.unwrap_or(false),
            convert2numbers: convert2numbers.unwrap_or(false),
            keep_base: keep_base.unwrap_or(false),
            bool_false,
            bool_true,
            max_block_size: buffer_size * 3 / 4, // 75%
        };
        Ok(Self {
            thread_handle: Some(AdvReader::reader_thread(
                options,
                tx,
                stop.clone(),
                fn_block_mode,
            )),
            reader_died: false,
            items: rx,
            stop,
            line_num: 0,
        })
    }

    fn reader_thread(
        options: AdvReaderOptions,
        tx: Sender<Option<(usize, Result<AdvReturnValue, Error>)>>,
        stop: Arc<AtomicBool>,
        fn_block_mode: Option<
            Box<dyn Fn(Option<&[u8]>) -> (bool, Option<Vec<Vec<u8>>>) + Send + 'static>,
        >,
    ) -> JoinHandle<Result<(), Error>> {
        let path = options.path;
        let buffer_size = options.buffer_size;
        let trim = options.trim;
        let line_end = options.line_end;
        let skip_comments = options.skip_comments;
        let encode_comments = options.encode_comments;
        let encode = options.encode;
        let allow_invalid_utf8 = options.allow_invalid_utf8;
        let _extended_word_separation = options.extended_word_separation; //TODO
        let double_double_quote_escape = options.double_double_quote_escape;
        let convert2numbers = options.convert2numbers;
        let keep_base = options.keep_base;
        let bool_false = options.bool_false;
        let bool_true = options.bool_true;
        let max_block_size = options.max_block_size;
        thread::spawn(move || {
            fn send_item(
                buf: &[u8],
                encode: bool,
                allow_invalid_utf8: bool,
                bool_false: &Option<Vec<u8>>,
                bool_true: &Option<Vec<u8>>,
                line_num: usize,
                state: ReaderStates,
                tx: &Sender<Option<(usize, Result<AdvReturnValue, Error>)>>,
            ) -> Result<(), Error> {
                let b = buf.to_vec();
                let mut v = None;
                if state == ReaderStates::Default {
                    if let Some(bf) = bool_false {
                        if b == *bf {
                            v = Some(AdvReturnValue::Bool(false));
                        }
                    }
                    if let Some(bf) = bool_true {
                        if b == *bf {
                            v = Some(AdvReturnValue::Bool(true));
                        }
                    }
                    if v.is_none() {
                        v = Some(AdvReturnValue::Bytes(b));
                    }
                } else if encode {
                    let s = match String::from_utf8(b.clone()) {
                        Ok(d) => d,
                        Err(e) => {
                            if allow_invalid_utf8 {
                                String::from_utf8_lossy(&b).to_string()
                            } else {
                                return Err(Error::new(
                                    ErrorKind::InvalidData,
                                    format!("Invalid UTF8 character: {}", e),
                                ));
                            }
                        }
                    };
                    v = Some(match state {
                        ReaderStates::String => AdvReturnValue::StringUtf8(s),
                        ReaderStates::Comment => AdvReturnValue::CommentUtf8(s),
                        ReaderStates::LineComment => AdvReturnValue::LineCommentUtf8(s),
                        _ => {
                            return Err(Error::new(
                                ErrorKind::Other,
                                format!("Invalid state: {:?}", state),
                            ))
                        }
                    });
                } else {
                    v = Some(match state {
                        ReaderStates::String => AdvReturnValue::String(b),
                        ReaderStates::Comment => AdvReturnValue::Comment(b),
                        ReaderStates::LineComment => AdvReturnValue::LineComment(b),
                        _ => {
                            return Err(Error::new(
                                ErrorKind::Other,
                                format!("Invalid state: {:?}", state),
                            ))
                        }
                    });
                }
                if let Err(e) = tx.send(Some((line_num, Ok(v.unwrap())))) {
                    return Err(Error::new(
                        ErrorKind::BrokenPipe,
                        format!("Failed to send string: {}", e),
                    ));
                }
                Ok(())
            }

            let mut file = match File::open(path) {
                Ok(fh) => fh,
                Err(e) => {
                    if tx.send(Some((0, Err(e)))).is_err() {
                        return Err(Error::new(
                            ErrorKind::BrokenPipe,
                            "Failed to send error message for failed open file!",
                        ));
                    }
                    return Ok(());
                }
            };
            let mut buffer = vec![0u8; buffer_size];
            let mut offset = 0;
            let mut start = None;
            let mut state = ReaderStates::Default;
            let mut escape = false;
            let mut quote = false;
            let mut line_num = 1;
            let mut i_value: i64 = 0;
            let mut f_value: f64 = 0.0;
            let mut e_value: i32 = 0; // Exponent for float value
            let mut is_neg = false; // - sign detected. Value is negative
            let mut has_leading_zero = false;
            let mut is_int = false;
            let mut is_hex = false;
            let mut is_oct = false;
            let mut is_bin = false;
            let mut is_float = false;
            let mut is_exp = false;
            let mut f_mul = 0.1; // Float multiplicator
            let mut is_exp_pos = false;
            let mut is_exp_neg = false; // Exponent is negative
            let mut block_mode = false;
            let mut block_start: usize = 0xffffffff;
            if let Some(ref cb_func) = fn_block_mode {
                let _ = cb_func(None); // Initialize static variables.
            }
            loop {
                if stop.load(Ordering::SeqCst) {
                    break;
                }
                let nread = match file.read(&mut buffer[offset..]) {
                    Ok(num) => num,
                    Err(e) => {
                        if tx.send(Some((line_num, Err(e)))).is_err() {
                            return Err(Error::new(
                                ErrorKind::BrokenPipe,
                                "Failed to send error message for file read!",
                            ));
                        }
                        break;
                    }
                };
                if nread == 0 {
                    send_item(
                        &buffer[..offset],
                        encode,
                        allow_invalid_utf8,
                        &bool_false,
                        &bool_true,
                        line_num,
                        state,
                        &tx,
                    )?;
                    break;
                }

                let buf = &mut buffer[..offset + nread];

                for i in (0..buf.len()).skip(offset) {
                    let c = buf[i];
                    match state {
                        ReaderStates::Default => {
                            if c <= b' ' || c >= b'\x7f' {
                                if let Some(s) = start.take() {
                                    let old_block_mode = block_mode;
                                    let mut send = None;
                                    if let Some(ref cb_func) = fn_block_mode {
                                        match cb_func(Some(&buf[s..i])) {
                                            (bm, Some(snd)) => {
                                                block_mode = bm;
                                                send = Some(snd);
                                            }
                                            (bm, None) => block_mode = bm,
                                        }
                                    }
                                    if block_mode || old_block_mode {
                                        if block_start == 0xffffffff {
                                            block_start = s;
                                        } else if !block_mode || i - block_start > max_block_size {
                                            let v = (buf[block_start..i]).to_vec();
                                            if let Err(e) = tx.send(Some((
                                                line_num,
                                                Ok(AdvReturnValue::Block(v)),
                                            ))) {
                                                return Err(Error::new(
                                                    ErrorKind::BrokenPipe,
                                                    format!("Failed to send block: {}", e),
                                                ));
                                            }
                                            if block_mode {
                                                block_start = i;
                                            } else {
                                                block_start = 0xffffffff;
                                            }
                                        }
                                    } else if let Err(e) = send_item(
                                        &buf[s..i],
                                        encode,
                                        allow_invalid_utf8,
                                        &bool_false,
                                        &bool_true,
                                        line_num,
                                        state,
                                        &tx,
                                    ) {
                                        return Err(e);
                                    }
                                    if let Some(snd) = send {
                                        for item in snd {
                                            send_item(
                                                &item,
                                                encode,
                                                allow_invalid_utf8,
                                                &bool_false,
                                                &bool_true,
                                                line_num,
                                                state,
                                                &tx,
                                            )?
                                        }
                                    }
                                }
                            } else {
                                if c == b'/' {
                                    state = ReaderStates::Slash;
                                } else if c == b'"' {
                                    if !block_mode {
                                        if let Some(s) = start.take() {
                                            send_item(
                                                &buf[s..i],
                                                encode,
                                                allow_invalid_utf8,
                                                &bool_false,
                                                &bool_true,
                                                line_num,
                                                state,
                                                &tx,
                                            )?
                                        }
                                    }
                                    state = ReaderStates::String;
                                }
                                if start.is_none() {
                                    if !block_mode
                                        && convert2numbers
                                        && state == ReaderStates::Default
                                    {
                                        is_int = false;
                                        is_float = false;
                                        is_neg = false;
                                        if c.is_ascii_digit() {
                                            f_value = 0.0;
                                            i_value = (c - b'0') as i64;
                                            is_int = true;
                                            if c == b'0' {
                                                has_leading_zero = true;
                                            }
                                        } else if c == b'-' {
                                            f_value = 0.0;
                                            i_value = 0;
                                            is_int = true;
                                            is_neg = true;
                                        } else if c == b'+' {
                                            f_value = 0.0;
                                            i_value = 0;
                                            is_int = true;
                                        } else if c == b'.' {
                                            is_float = true;
                                        }
                                        if is_int || is_float {
                                            is_hex = false;
                                            is_oct = false;
                                            is_bin = false;
                                            is_float = false;
                                            is_exp = false;
                                            is_exp_pos = false;
                                            is_exp_neg = false;
                                            state = ReaderStates::Number;
                                        }
                                    }
                                    start = Some(i);
                                }
                            }
                        }
                        ReaderStates::Number => {
                            let mut send = true;
                            if c == b'/' {
                                send = false;
                                state = ReaderStates::Slash;
                            } else if c == b'"' {
                                send = false;
                                state = ReaderStates::String;
                            } else if is_int {
                                if c.is_ascii_digit() {
                                    if i_value > 0xCCCCCCCCCCCCCCC {
                                        // To avoid an overflow exception we need to switch to float
                                        f_value = i_value as f64 * 10.0 + (c - b'0') as f64;
                                        is_int = false;
                                        is_float = true;
                                        f_mul = 0.0;
                                    } else {
                                        i_value = i_value * 10 + (c - b'0') as i64;
                                    }
                                } else if c == b'.' {
                                    f_value = i_value as f64;
                                    is_int = false;
                                    is_float = true;
                                    f_mul = 0.1;
                                } else if c == b'e' || c == b'E' {
                                    f_value = i_value as f64;
                                    e_value = 0;
                                    is_int = false;
                                    is_exp = true;
                                    f_mul = 0.1;
                                } else if has_leading_zero {
                                    if c == b'x' || c == b'X' {
                                        is_int = false;
                                        is_hex = true;
                                    } else if c == b'o' || c == b'O' {
                                        is_int = false;
                                        is_oct = true;
                                    } else if c == b'b' || c == b'B' {
                                        is_int = false;
                                        is_bin = true;
                                    } else {
                                        // Invalid character
                                        state = ReaderStates::Default;
                                    }
                                } else {
                                    // Invalid character
                                    state = ReaderStates::Default;
                                }
                                has_leading_zero = false;
                            } else if is_hex {
                                if c.is_ascii_digit() {
                                    i_value = (i_value << 4) + (c - b'0') as i64;
                                } else if (b'a'..=b'f').contains(&c) {
                                    i_value = (i_value << 4) + (c - b'a') as i64 + 10;
                                } else if (b'A'..=b'F').contains(&c) {
                                    i_value = (i_value << 4) + (c - b'A') as i64 + 10;
                                } else {
                                    is_hex = false;
                                    // Invalid character
                                    state = ReaderStates::Default;
                                }
                            } else if is_oct {
                                if (b'0'..=b'7').contains(&c) {
                                    i_value = (i_value << 3) + (c - b'0') as i64;
                                } else {
                                    is_oct = false;
                                    // Invalid character
                                    state = ReaderStates::Default;
                                }
                            } else if is_bin {
                                if (b'0'..=b'1').contains(&c) {
                                    i_value = (i_value << 1) + (c - b'0') as i64;
                                } else {
                                    is_bin = false;
                                    // Invalid character
                                    state = ReaderStates::Default;
                                }
                            } else if is_float {
                                if f_mul == 0.0 {
                                    if c.is_ascii_digit() {
                                        f_value = f_value * 10.0 + (c - b'0') as f64;
                                    } else if c == b'e' || c == b'E' {
                                        e_value = 0;
                                        is_float = false;
                                        is_exp = true;
                                    } else if c == b'.' {
                                        f_mul = 0.1;
                                    } else {
                                        // Invalid character
                                        state = ReaderStates::Default;
                                    }
                                } else if c.is_ascii_digit() {
                                    f_value += f_mul * (c - b'0') as f64;
                                    f_mul *= 0.1;
                                } else if c == b'e' || c == b'E' {
                                    e_value = 0;
                                    is_float = false;
                                    is_exp = true;
                                } else {
                                    // Invalid character
                                    state = ReaderStates::Default;
                                }
                            } else if is_exp {
                                if c.is_ascii_digit() {
                                    e_value = e_value * 10 + (c - b'0') as i32;
                                } else if c == b'-' {
                                    if is_exp_pos || is_exp_neg {
                                        // Invalid character
                                        state = ReaderStates::Default;
                                    } else {
                                        is_exp_neg = true;
                                    }
                                } else if c == b'+' {
                                    if is_exp_pos || is_exp_neg {
                                        // Invalid character
                                        state = ReaderStates::Default;
                                    } else {
                                        is_exp_pos = true;
                                    }
                                } else {
                                    // Invalid character
                                    state = ReaderStates::Default;
                                }
                            }
                            if state != ReaderStates::Number && send {
                                let v;
                                if is_float {
                                    if is_neg {
                                        f_value = -f_value;
                                    }
                                    v = AdvReturnValue::Float(f_value);
                                } else if is_exp {
                                    let mut exp = 10_f64.powi(e_value);
                                    if is_neg {
                                        f_value = -f_value;
                                    }
                                    if is_exp_neg {
                                        exp = 1.0 / exp;
                                    }
                                    v = AdvReturnValue::Float(f_value * exp);
                                } else {
                                    if is_neg {
                                        i_value = -i_value;
                                    }
                                    if keep_base {
                                        if is_hex {
                                            v = AdvReturnValue::Hex(i_value);
                                        } else if is_oct {
                                            v = AdvReturnValue::Oct(i_value);
                                        } else if is_bin {
                                            v = AdvReturnValue::Bin(i_value);
                                        } else {
                                            v = AdvReturnValue::Int(i_value);
                                        }
                                    } else {
                                        v = AdvReturnValue::Int(i_value);
                                    }
                                }
                                if let Err(e) = tx.send(Some((line_num, Ok(v)))) {
                                    return Err(Error::new(
                                        ErrorKind::BrokenPipe,
                                        format!("Failed to send number: {}", e),
                                    ));
                                }
                                start = None;
                                state = ReaderStates::Default;
                            }
                        }
                        ReaderStates::String => {
                            // Escaping is done with \" and ""
                            let mut string_end = false;
                            let mut e = i;
                            if c == b'\\' {
                                escape = true;
                            } else if quote {
                                if c != b'"' {
                                    e -= 1;
                                    string_end = true;
                                }
                                quote = false;
                            } else if !escape && c == b'"' {
                                if double_double_quote_escape {
                                    quote = true;
                                } else {
                                    string_end = true;
                                }
                            } else {
                                escape = false;
                            }
                            if string_end {
                                if !block_mode {
                                    if let Some(mut s) = start.take() {
                                        if trim {
                                            s += 1;
                                        } else {
                                            e += 1;
                                        }
                                        send_item(
                                            &buf[s..e],
                                            encode,
                                            allow_invalid_utf8,
                                            &bool_false,
                                            &bool_true,
                                            line_num,
                                            state,
                                            &tx,
                                        )?
                                    }
                                }
                                state = ReaderStates::Default;
                                escape = false;
                            }
                        }
                        ReaderStates::Slash => {
                            if c != b'*' && c != b'/' {
                                if c == b'"' {
                                    if !block_mode {
                                        if let Some(s) = start.take() {
                                            send_item(
                                                &buf[s..i],
                                                encode,
                                                allow_invalid_utf8,
                                                &bool_false,
                                                &bool_true,
                                                line_num,
                                                ReaderStates::Default,
                                                &tx,
                                            )?
                                        }
                                    }
                                    start = Some(i);
                                    state = ReaderStates::String;
                                } else {
                                    if c <= b' ' || c >= b'\x7f' {
                                        if let Some(s) = start.take() {
                                            send_item(
                                                &buf[s..i],
                                                encode,
                                                allow_invalid_utf8,
                                                &bool_false,
                                                &bool_true,
                                                line_num,
                                                ReaderStates::Default,
                                                &tx,
                                            )?
                                        }
                                    }
                                    state = ReaderStates::Default;
                                }
                            } else {
                                if let Some(s) = start.take() {
                                    if !block_mode && s < i - 1 {
                                        send_item(
                                            &buf[s..i - 1],
                                            encode,
                                            allow_invalid_utf8,
                                            &bool_false,
                                            &bool_true,
                                            line_num,
                                            ReaderStates::Default,
                                            &tx,
                                        )?
                                    }
                                    start = Some(i - 1);
                                }
                                if c == b'*' {
                                    state = ReaderStates::Comment;
                                } else {
                                    state = ReaderStates::LineComment;
                                }
                            }
                        }
                        ReaderStates::Comment => {
                            if c == b'*' {
                                state = ReaderStates::CommentAsterisk;
                            }
                        }
                        ReaderStates::CommentAsterisk => {
                            if c == b'/' {
                                if let Some(mut s) = start.take() {
                                    if !block_mode && !skip_comments {
                                        let mut e = i + 1;
                                        if trim {
                                            s += 2;
                                            e -= 2;
                                        }
                                        send_item(
                                            &buf[s..e],
                                            encode_comments,
                                            allow_invalid_utf8,
                                            &bool_false,
                                            &bool_true,
                                            line_num,
                                            ReaderStates::Comment,
                                            &tx,
                                        )?
                                    }
                                }
                                state = ReaderStates::Default;
                            }
                        }
                        ReaderStates::LineComment => {
                            if c < b' ' {
                                if let Some(mut s) = start.take() {
                                    if !block_mode && !skip_comments {
                                        if trim {
                                            s += 2;
                                        }
                                        send_item(
                                            &buf[s..i],
                                            encode_comments,
                                            allow_invalid_utf8,
                                            &bool_false,
                                            &bool_true,
                                            line_num,
                                            state,
                                            &tx,
                                        )?;
                                    }
                                }
                                state = ReaderStates::Default;
                            }
                        }
                    }
                    if c == line_end {
                        line_num += 1;
                    }
                }
                if let Some(ref mut s) = start {
                    offset = buf.len() - *s;
                    buf.copy_within(*s.., 0);
                    *s = 0;
                } else {
                    offset = 0;
                }
                if block_mode {
                    block_start = 0;
                }
            }
            if tx.send(None).is_err() {
                return Err(Error::new(
                    ErrorKind::BrokenPipe,
                    "Failed to send finish token!",
                ));
            }
            Ok(())
        })
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.items.try_recv(); // Read last entry
        let _ = self.items.try_recv(); // Read None
        match self.thread_handle.take() {
            Some(h) => match h.join() {
                Ok(result) => result,
                Err(e) => Err(Error::new(
                    ErrorKind::Other,
                    format!("Failed to join thread: {:?}", e),
                )),
            },
            None => Err(Error::new(ErrorKind::Other, "No thread to stop!")),
        }
    }

    /// Retrieves a reference to the next word, string or (line) comment of bytes in the reader (if any).
    pub fn next(&mut self) -> Option<Result<AdvReturnValue, Error>> {
        match self.items.recv() {
            Ok(Some((line_num, item))) => {
                self.line_num = line_num;
                Some(item)
            }
            Ok(None) => None,
            Err(e) => {
                if self.reader_died {
                    return None;
                }
                self.reader_died = true;
                Some(Err(Error::new(
                    ErrorKind::BrokenPipe,
                    format!("Reader thread died: {}", e),
                )))
            }
        }
    }

    /// Returns the corresponding line in the file for the latest returned item.
    pub fn line_nr(&self) -> usize {
        self.line_num
    }
}

/// `IntoIterator` conversion for `AdvReader` to provide `Iterator` APIs.
impl IntoIterator for AdvReader {
    type Item = Result<AdvReturnValue, Error>;
    type IntoIter = AdvReaderIter;

    /// Constructs a `advreaderIter` to provide an `Iterator` API.
    #[inline]
    fn into_iter(self) -> AdvReaderIter {
        AdvReaderIter { inner: self }
    }
}

/// `Iterator` implementation of `AdvReader` to provide `Iterator` APIs.
///
/// This structure enables developers the use of the `Iterator` API in
/// their code, at the cost of an allocation per returned item:
///
/// ```rust
/// use std::path::PathBuf;
/// use advreader::*;
///
/// // construct our iterator from our file input
/// let reader = AdvReader::default(&PathBuf::from("../res/example.txt"));
///
/// let mut reader_ok = reader.unwrap();
///
/// // walk our items using `for` syntax
/// for item in reader_ok.into_iter() {
///     // do something with the item, which is Result<AdvReturnValue, Error>
/// }
/// ```
pub struct AdvReaderIter {
    inner: AdvReader,
}

impl Iterator for AdvReaderIter {
    type Item = Result<AdvReturnValue, Error>;

    /// Retrieves the next item in the iterator (if any).
    #[inline]
    fn next(&mut self) -> Option<Result<AdvReturnValue, Error>> {
        self.inner.next()
    }
}
