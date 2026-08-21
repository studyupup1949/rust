//! `advreader` is a simple library crate offering an iterator which splits a file into
//! - text sequences, separated by characters with ASCII codes <=32 and >=127.
//! - strings with double quotes as delimiters.
//! - line comments with '//' as start sequence.
//! - comment blocks with '/*' as start sequence and '*/' as end sequence.
//!
//! Results can be obatined through the `next` method.
//! Property `line_nr` provides the current line in the text file.
#![doc(html_root_url = "https://docs.rs/advreader/2.0.0")]
use std::fs::{metadata, File};
use std::io::{prelude::*, Error, ErrorKind};
use std::iter::Enumerate;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};

use flume::{bounded, Receiver, Sender};
use local_encoding::{Encoder, Encoding};

pub mod block;
mod common;

pub use common::{
    AdvReaderOptions, AdvReturnValue, FnBlockReturnType, FnReadBlockType, ReaderState,
};

#[inline]
fn encode(data: &[u8], encoder: Option<&str>, allow_invalid_utf8: bool) -> Result<String, Error> {
    let result = match encoder {
        Some("ansi2") => {
            let mut result = Vec::with_capacity(data.len());
            for c in data.iter() {
                match *c {
                    196 => {
                        // ä
                        result.push(195);
                        result.push(132)
                    }
                    214 => {
                        // ö
                        result.push(195);
                        result.push(150)
                    }
                    220 => {
                        // ü
                        result.push(195);
                        result.push(156)
                    }
                    223 => {
                        // Ä
                        result.push(195);
                        result.push(159)
                    }
                    228 => {
                        // Ö
                        result.push(195);
                        result.push(164)
                    }
                    246 => {
                        // Ü
                        result.push(195);
                        result.push(182)
                    }
                    252 => {
                        // ß
                        result.push(195);
                        result.push(188)
                    }
                    _ => result.push(*c),
                }
            }
            std::str::from_utf8(&result)
                .map(|v| v.to_string())
                .map_err(|e| Error::new(ErrorKind::InvalidInput, e.to_string()))
        }
        Some("ansi") => Encoding::ANSI.to_string(data),
        Some("oem") => Encoding::OEM.to_string(data),
        _ => std::str::from_utf8(data)
            .map(|v| v.to_string())
            .map_err(|e| Error::new(ErrorKind::InvalidInput, e.to_string())),
    };
    match result {
        Ok(_) => result,
        Err(e) => {
            if allow_invalid_utf8 {
                Ok(String::from_utf8_lossy(data).to_string())
            } else {
                Err(Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid UTF8 character: {e}"),
                ))
            }
        }
    }
}

struct AdvReaderThread {
    // Internal
    file: File,
    tx: Sender<Option<(usize, Result<AdvReturnValue, Error>)>>,
    stop: Arc<AtomicBool>,
    buffer_size: usize,
    start: Option<usize>,
    line_num: usize,
    slash: bool,
    state: ReaderState,
    // Options
    encode_comments: bool,
    encode_strings: bool,
    encoder: Option<String>,
    allow_invalid_utf8: bool,
    convert2numbers: bool,
    keep_base: bool,
    skip_comments: bool,
    trim: bool,
    bool_false: Option<Vec<u8>>,
    bool_true: Option<Vec<u8>>,
    line_end: u8,
    double_quote_escape: bool,
    extended_word_separation: bool,
    // read
    escape: bool,
    // read comment
    asterisk: bool,
    // read string
    quote: bool,
    quote_before: usize, // True if we had a quote just before
    // Block support
    block_reader: Option<Box<dyn block::Block + Send + Sync>>,
}

impl AdvReaderThread {
    pub fn new(
        options: AdvReaderOptions,
        tx: Sender<Option<(usize, Result<AdvReturnValue, Error>)>>,
        stop: Arc<AtomicBool>,
        block_reader: Option<Box<dyn block::Block + Send + Sync>>,
    ) -> Result<Self, Error> {
        let file = File::open(&options.path)?;
        Ok(Self {
            file,
            tx,
            stop,
            buffer_size: options.buffer_size,
            start: None,
            line_num: 1,
            slash: false,
            state: ReaderState::Default,
            encode_comments: options.encode_comments,
            encode_strings: options.encode_strings,
            encoder: options.encoder,
            allow_invalid_utf8: options.allow_invalid_utf8,
            convert2numbers: options.convert2numbers,
            keep_base: options.keep_base,
            skip_comments: options.skip_comments,
            trim: options.trim,
            bool_false: options.bool_false,
            bool_true: options.bool_true,
            line_end: options.line_end,
            double_quote_escape: options.double_quote_escape,
            extended_word_separation: options.extended_word_separation,
            block_reader,
            escape: false,
            asterisk: false,
            quote: false,
            quote_before: 0,
        })
    }

    #[inline]
    fn send(&mut self, v: AdvReturnValue) -> Result<(), Error> {
        self.tx
            .send(Some((self.line_num, Ok(v))))
            .map_err(|e| Error::new(ErrorKind::BrokenPipe, format!("Failed to send token: {e}")))
    }

    #[inline]
    fn send_line_comment(&mut self, buf: &[u8]) -> Result<(), Error> {
        let v = if self.encode_comments {
            AdvReturnValue::LineCommentUtf8(encode(
                buf,
                self.encoder.as_deref(),
                self.allow_invalid_utf8,
            )?)
        } else {
            AdvReturnValue::LineComment(buf.to_vec())
        };
        self.tx
            .send(Some((self.line_num, Ok(v))))
            .map_err(|e| Error::new(ErrorKind::BrokenPipe, format!("Failed to send string: {e}")))
    }

    #[inline]
    fn send_comment(&mut self, buf: &[u8]) -> Result<(), Error> {
        let v = if self.encode_comments {
            AdvReturnValue::CommentUtf8(encode(
                buf,
                self.encoder.as_deref(),
                self.allow_invalid_utf8,
            )?)
        } else {
            AdvReturnValue::Comment(buf.to_vec())
        };
        self.tx
            .send(Some((self.line_num, Ok(v))))
            .map_err(|e| Error::new(ErrorKind::BrokenPipe, format!("Failed to send string: {e}")))
    }

    #[inline]
    fn send_string(&mut self, buf: &[u8]) -> Result<(), Error> {
        let v = if self.encode_strings {
            AdvReturnValue::StringUtf8(encode(
                buf,
                self.encoder.as_deref(),
                self.allow_invalid_utf8,
            )?)
        } else {
            AdvReturnValue::String(buf.to_vec())
        };
        self.tx
            .send(Some((self.line_num, Ok(v))))
            .map_err(|e| Error::new(ErrorKind::BrokenPipe, format!("Failed to send string: {e}")))
    }

    #[inline]
    fn read_bytes(
        &mut self,
        itr: &mut Enumerate<std::slice::Iter<'_, u8>>,
        mut i: usize,
        mut c: u8,
    ) -> Option<(usize, u8)> {
        // Escaping is done with \" and ""
        loop {
            if self.start.is_none() {
                if c <= b' ' || c >= b'\x7f' {
                    if c == self.line_end {
                        self.line_num += 1;
                    }
                    (i, c) = match itr.next() {
                        Some((i, c)) => (i, *c),
                        None => break,
                    };
                    continue;
                }
                self.start = Some(i);
                if self.convert2numbers
                    && (c.is_ascii_digit() || c == b'.' || c == b'-' || c == b'+')
                {
                    self.state = ReaderState::Number;
                    return Some((i, c));
                }
            }
            if self.escape {
                self.escape = false;
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => break,
                };
                continue;
            }
            if c == b'\\' {
                self.escape = true;
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => break,
                };
                continue;
            }
            if self.slash {
                self.slash = false;
                if c == b'/' {
                    self.state = ReaderState::LineComment;
                    return Some((i, c));
                }
                if c == b'*' {
                    self.state = ReaderState::Comment;
                    return Some((i, c));
                }
            }
            if c == b'/' {
                self.slash = true;
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => break,
                };
                continue;
            }
            if c == b'"' {
                self.state = ReaderState::String;
                return Some((i, c));
            }
            if self.extended_word_separation {
                if c != b'.' && c != b'_' && !c.is_ascii_alphanumeric() {
                    return Some((i, c));
                }
            } else if c <= b' ' || c >= b'\x7f' {
                return Some((i, c));
            }
            (i, c) = match itr.next() {
                Some((i, c)) => (i, *c),
                None => break,
            };
        }
        None
    }

    #[inline]
    fn read_number(
        &mut self,
        itr: &mut Enumerate<std::slice::Iter<'_, u8>>,
        mut i: usize,
        mut c: u8,
    ) -> (usize, u8, Option<AdvReturnValue>) {
        let mut i_value: i64 = 0;
        let mut f_value: f64 = 0.0;
        let mut e_value: i32 = 0; // Exponent for float value
        let mut is_hex = false;
        let mut is_oct = false;
        let mut is_bin = false;
        let mut is_exp = false;
        let mut f_mul = 0.0; // Float multiplicator
        let mut is_exp_pos = false;
        let mut is_exp_neg = false; // Exponent is negative
        let is_neg = c == b'-';
        let is_int = c.is_ascii_digit() || is_neg || (c == b'+');
        let mut is_float = c == b'.';
        if is_int {
            let mut first_digit = true;
            let mut has_leading_zero = false;
            if is_neg || c == b'+' {
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => return (i, c, None),
                };
            }
            loop {
                if c.is_ascii_digit() {
                    if first_digit {
                        first_digit = false;
                        if c == b'0' {
                            has_leading_zero = true;
                        }
                    } else if has_leading_zero {
                        if c == b'0' {
                            // Double 0 means: Bytes
                            while c > b' ' && c < b'\x7f' {
                                (i, c) = match itr.next() {
                                    Some((i, c)) => (i, *c),
                                    None => break,
                                };
                            }
                            if c == self.line_end {
                                self.line_num += 1;
                            }
                            return (i, c, Some(AdvReturnValue::Bytes(vec![])));
                        }
                        has_leading_zero = false;
                    }
                    if i_value <= 0xCCCCCCCCCCCCCCC {
                        i_value = i_value * 10 + (c - b'0') as i64;
                        (i, c) = match itr.next() {
                            Some((i, c)) => (i, *c),
                            None => return (i, c, None),
                        };
                        continue;
                    }
                    // To avoid an overflow exception we need to switch to float
                    f_value = i_value as f64 * 10.0 + (c - b'0') as f64;
                    is_float = true;
                    f_mul = 1.0;
                    break;
                }
                if c <= b' ' || c >= b'\x7f' || (c != b'.' && c.is_ascii_punctuation()) {
                    if is_neg {
                        i_value = -i_value;
                    }
                    return (i, c, Some(AdvReturnValue::Int(i_value)));
                } else if c == b'.' {
                    f_value = i_value as f64;
                    is_float = true;
                    f_mul = 0.1;
                } else if c == b'e' || c == b'E' {
                    f_value = i_value as f64;
                    e_value = 0;
                    is_exp = true;
                    f_mul = 0.1;
                } else if has_leading_zero {
                    has_leading_zero = false;
                    if c == b'x' || c == b'X' {
                        is_hex = true;
                    } else if c == b'o' || c == b'O' {
                        is_oct = true;
                    } else if c == b'b' || c == b'B' {
                        is_bin = true;
                    } else {
                        // Invalid character
                        (i, c) = match itr.next() {
                            Some((i, c)) => (i, *c),
                            None => return (i, c, None),
                        };
                        return (i, c, None);
                    }
                } else {
                    // Invalid character
                    return (i, c, None);
                }
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => return (i, c, None),
                };
                if is_float || is_exp || is_hex || is_oct || is_bin {
                    break;
                }
            }
        }
        if is_float {
            loop {
                if c.is_ascii_digit() {
                    if f_mul == 1.0 {
                        f_value = f_value * 10.0 + (c - b'0') as f64;
                    } else {
                        f_value += f_mul * (c - b'0') as f64;
                        f_mul *= 0.1;
                    }
                } else if f_mul == 0.0 {
                    if c.is_ascii_digit() {
                        f_value = f_value * 10.0 + (c - b'0') as f64;
                    } else if c == b'.' {
                        f_mul = 0.1;
                    } else if c == b'e' || c == b'E' {
                        e_value = 0;
                        is_exp = true;
                    }
                } else if c == b'e' || c == b'E' {
                    e_value = 0;
                    is_exp = true;
                } else if c == b'.' {
                    f_mul = 0.1;
                } else {
                    if is_neg {
                        f_value = -f_value;
                    }
                    return (i, c, Some(AdvReturnValue::Float(f_value)));
                }
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => return (i, c, None),
                };
                if is_exp {
                    break;
                }
            }
        }
        if is_exp {
            loop {
                if c.is_ascii_digit() {
                    e_value = e_value * 10 + (c - b'0') as i32;
                } else if c == b'-' {
                    if !is_exp_pos && !is_exp_neg {
                        is_exp_neg = true;
                    }
                } else if c == b'+' {
                    if !is_exp_pos && !is_exp_neg {
                        is_exp_pos = true;
                    }
                } else {
                    let mut exp = 10_f64.powi(e_value);
                    if is_neg {
                        f_value = -f_value;
                    }
                    if is_exp_neg {
                        exp = 1.0 / exp;
                    }
                    return (i, c, Some(AdvReturnValue::Float(f_value * exp)));
                }
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => return (i, c, None),
                };
            }
        } else if is_hex {
            loop {
                if c.is_ascii_digit() {
                    i_value = (i_value << 4) + (c - b'0') as i64;
                } else if (b'a'..=b'f').contains(&c) {
                    i_value = (i_value << 4) + (c - b'a') as i64 + 10;
                } else if (b'A'..=b'F').contains(&c) {
                    i_value = (i_value << 4) + (c - b'A') as i64 + 10;
                } else {
                    if is_neg {
                        i_value = -i_value;
                    }
                    if self.keep_base {
                        return (i, c, Some(AdvReturnValue::Hex(i_value)));
                    } else {
                        return (i, c, Some(AdvReturnValue::Int(i_value)));
                    }
                }
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => return (i, c, None),
                };
            }
        } else if is_oct {
            loop {
                if (b'0'..=b'7').contains(&c) {
                    i_value = (i_value << 3) + (c - b'0') as i64;
                } else if c <= b' ' || c >= b'\x7f' {
                    if is_neg {
                        i_value = -i_value;
                    }
                    if self.keep_base {
                        return (i, c, Some(AdvReturnValue::Oct(i_value)));
                    } else {
                        return (i, c, Some(AdvReturnValue::Int(i_value)));
                    }
                }
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => return (i, c, None),
                };
            }
        } else if is_bin {
            loop {
                if (b'0'..=b'1').contains(&c) {
                    i_value = (i_value << 1) + (c - b'0') as i64;
                } else if c <= b' ' || c >= b'\x7f' {
                    if is_neg {
                        i_value = -i_value;
                    }
                    if self.keep_base {
                        return (i, c, Some(AdvReturnValue::Bin(i_value)));
                    } else {
                        return (i, c, Some(AdvReturnValue::Int(i_value)));
                    }
                }
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => return (i, c, None),
                };
            }
        }
        (i, c, None)
    }

    #[inline]
    pub(crate) fn read_string(
        &mut self,
        itr: &mut Enumerate<std::slice::Iter<'_, u8>>,
        mut i: usize,
        mut c: u8,
    ) -> Option<(usize, u8)> {
        // Escaping is done with \" and ""
        let start = self.start.unwrap_or(i);
        loop {
            if self.escape {
                self.escape = false;
                self.quote = false;
            } else if c == b'\\' {
                self.escape = true;
                self.quote = false;
            } else if c == b'"' {
                if i > start {
                    if self.quote_before == 2 {
                        self.quote_before = 1;
                    } else if !self.quote {
                        self.quote = true;
                    } else if self.double_quote_escape {
                        self.quote_before = 2;
                    } else {
                        self.quote = false;
                        self.quote_before = 0;
                        if i > start + 1 {
                            return Some((i - 1, c));
                        }
                        return Some((i, c));
                    }
                } else {
                    self.quote = true;
                }
            } else if self.quote {
                self.quote = false;
                if self.quote_before == 2 {
                    self.quote_before = 0;
                    if i == start + 2 {
                        return Some((i - 1, c));
                    }
                } else if self.quote_before == 1 {
                    self.quote_before = 0;
                    if i > start + 3 {
                        return Some((i - 1, c));
                    }
                } else if i > start + 1 {
                    return Some((i - 1, c));
                }
            } else if c == self.line_end {
                self.line_num += 1;
            }
            (i, c) = match itr.next() {
                Some((i, c)) => (i, *c),
                None => break,
            };
        }
        if self.quote && self.quote_before < 2 {
            self.quote = false;
            self.quote_before = 0;
            return Some((i, c));
        }
        None
    }

    #[inline]
    fn read_line_comment(
        &mut self,
        itr: &mut Enumerate<std::slice::Iter<'_, u8>>,
        mut i: usize,
        mut c: u8,
    ) -> Option<(usize, u8)> {
        loop {
            if self.escape {
                self.escape = false;
            } else if c == b'\\' {
                self.escape = true;
            } else if c == self.line_end {
                return Some((i, c));
            }
            (i, c) = match itr.next() {
                Some((i, c)) => (i, *c),
                None => break,
            };
        }
        None
    }

    #[inline]
    fn read_comment(
        &mut self,
        itr: &mut Enumerate<std::slice::Iter<'_, u8>>,
        mut i: usize,
        mut c: u8,
    ) -> Option<(usize, u8)> {
        loop {
            if self.escape {
                self.escape = false;
            } else if self.asterisk {
                if c == b'/' {
                    self.asterisk = false;
                    return Some((i, c));
                } else if c != b'*' {
                    self.asterisk = false;
                }
            } else if c == b'\\' {
                self.escape = true;
            } else if c == b'*' {
                self.asterisk = true;
            }
            if c == self.line_end {
                self.line_num += 1;
            }
            (i, c) = match itr.next() {
                Some((i, c)) => (i, *c),
                None => break,
            };
        }
        None
    }

    pub fn read(&mut self) -> Result<(usize, ReaderState), Error> {
        let check_true_false =
            self.convert2numbers && (self.bool_true.is_some() || self.bool_false.is_some());
        let mut buffer = vec![0u8; self.buffer_size];
        let mut offset: usize = 0;
        let mut restart = false;
        //let mut pos: usize = 0;
        loop {
            if self.stop.load(Ordering::SeqCst) {
                break;
            }
            let nread = match self.file.read(&mut buffer[offset..]) {
                Ok(num) => num,
                Err(e) => {
                    self.tx.send(Some((self.line_num, Err(e)))).map_err(|e| {
                        Error::new(
                            ErrorKind::BrokenPipe,
                            format!("Failed to send error message for file read: {e}"),
                        )
                    })?;
                    break;
                }
            };
            if nread == 0 {
                if offset > 0 {
                    self.send(AdvReturnValue::Bytes(buffer[..offset].to_vec()))?;
                }
                break;
            }

            let buf = &mut buffer[..offset + nread];
            let buf_len = buf.len();
            let mut itr = buf.iter().enumerate();
            let mut i: usize;
            let mut c: u8;

            if restart {
                restart = false;
            } else if offset > 0 {
                itr.nth(offset - 1);
            }
            (i, c) = match itr.next() {
                Some((i, c)) => (i, *c),
                None => break,
            };
            loop {
                match self.state {
                    ReaderState::Default => {
                        (i, c) = match self.read_bytes(&mut itr, i, c) {
                            Some((i, c)) => (i, c),
                            None => break,
                        };
                        let mut e = match self.state {
                            ReaderState::LineComment => i - 1,
                            ReaderState::Comment => i - 1,
                            ReaderState::String => i,
                            _ => 0,
                        };
                        if self.state == ReaderState::Default || e > 0 {
                            if let Some(s) = self.start.take() {
                                if e == 0 {
                                    e = i;
                                }
                                if e > s {
                                    let v = buf[s..e].to_vec();
                                    if check_true_false {
                                        if Some(&v) == self.bool_false.as_ref() {
                                            self.send(AdvReturnValue::Bool(false))?;
                                            continue;
                                        }
                                        if Some(&v) == self.bool_true.as_ref() {
                                            self.send(AdvReturnValue::Bool(true))?;
                                            continue;
                                        }
                                    }
                                    if let Some(ref mut block_reader) = self.block_reader {
                                        let (tmp_i, tmp_c, items) = block_reader.read_block(
                                            &v,
                                            buf,
                                            &mut itr,
                                            i,
                                            c,
                                            &mut self.line_num,
                                        );
                                        i = tmp_i;
                                        if tmp_c == -1 {
                                            self.start = Some(buf_len); // Skip all
                                            if block_reader.is_block_mode() {
                                                self.state = ReaderState::Block;
                                            }
                                            break;
                                        } else if tmp_c < 0 {
                                            self.start = Some(-tmp_c as usize - 1);
                                            if block_reader.is_block_mode() {
                                                self.state = ReaderState::Block;
                                            }
                                            restart = true;
                                            break;
                                        } else {
                                            if let Some(items) = items {
                                                for item in items {
                                                    self.send(item)?;
                                                }
                                            } else {
                                                self.send(AdvReturnValue::Bytes(v))?;
                                            }
                                            if tmp_c > 255 {
                                                break;
                                            }
                                            c = tmp_c as u8;
                                        }
                                    } else {
                                        self.send(AdvReturnValue::Bytes(v))?;
                                    }
                                }
                                if self.state != ReaderState::Default {
                                    self.start = Some(e);
                                }
                            }
                        }
                    }
                    ReaderState::Number => {
                        let v;
                        (i, c, v) = self.read_number(&mut itr, i, c);
                        match v {
                            Some(v) => {
                                if let AdvReturnValue::Bytes(_) = v {
                                    let s = self.start.take().unwrap_or_default();
                                    self.send(AdvReturnValue::Bytes(buf[s..i].to_vec()))?;
                                } else {
                                    self.send(v)?;
                                }
                                self.start = None;
                                self.state = ReaderState::Default;
                            }
                            None => {
                                restart = true;
                                break;
                            }
                        }
                    }
                    ReaderState::String => {
                        (i, c) = match self.read_string(&mut itr, i, c) {
                            Some((mut i, c)) => {
                                if let Some(mut s) = self.start.take() {
                                    if self.trim {
                                        s += 1;
                                    } else if i < buf_len {
                                        if s == i && buf[s] == b'"' {
                                            self.start = Some(s);
                                            break;
                                        }
                                        i += 1;
                                    } else if s == i {
                                        self.state = ReaderState::Default;
                                        break;
                                    }
                                    self.send_string(&buf[s..i])?;
                                    self.state = ReaderState::Default;
                                }
                                (i, c)
                            }
                            None => {
                                self.start = Some(i);
                                break;
                            }
                        };
                    }
                    ReaderState::LineComment => {
                        (i, c) = match self.read_line_comment(&mut itr, i, c) {
                            Some((i, c)) => {
                                let mut s = self.start.take().unwrap_or(0);
                                if !self.skip_comments {
                                    if self.trim {
                                        s += 2;
                                    }
                                    self.send_line_comment(&buf[s..i])?;
                                }
                                self.state = ReaderState::Default;
                                (i, c)
                            }
                            None => {
                                self.start = None;
                                break;
                            }
                        };
                    }
                    ReaderState::Comment => {
                        (i, c) = match self.read_comment(&mut itr, i, c) {
                            Some((mut i, c)) => {
                                let mut s = self.start.take().unwrap_or(0);
                                if self.trim {
                                    s += 1;
                                } else {
                                    i += 1;
                                }
                                if !self.skip_comments {
                                    self.send_comment(&buf[s..i])?;
                                }
                                self.state = ReaderState::Default;
                                (i, c)
                            }
                            None => {
                                self.start = None;
                                break;
                            }
                        };
                    }
                    ReaderState::Block => {
                        /*if pos > 111800 {
                            println!("# {pos} {}", self.line_num);
                        }*/
                        let (tmp_i, tmp_c, items) = self
                            .block_reader
                            .as_deref_mut()
                            .unwrap()
                            .read_block(&Vec::new(), buf, &mut itr, i, c, &mut self.line_num);
                        i = tmp_i;
                        if tmp_c == -1 {
                            // Skip full buffer
                            self.start = Some(buf_len);
                            break;
                        } else if tmp_c < 0 {
                            // Skip part of buffer
                            self.start = Some(-tmp_c as usize - 1);
                            restart = true;
                            break;
                        } else {
                            if let Some(items) = items {
                                for item in items {
                                    self.send(item)?;
                                }
                                self.start = None;
                                self.state = ReaderState::Default;
                            }
                            if tmp_c > 255 {
                                break;
                            }
                            c = tmp_c as u8;
                        }
                    }
                } // match
            } // loop
            if let Some(ref mut s) = self.start {
                offset = buf_len - *s;
                buf.copy_within(*s.., 0);
                *s = 0;
            } else {
                offset = 0;
            }
            //pos += nread;
        }
        if self.tx.send(None).is_err() {
            return Err(Error::new(
                ErrorKind::BrokenPipe,
                "Failed to send finish token!",
            ));
        }
        Ok((self.line_num, self.state.clone()))
    }
}

/// Provides iteration over bytes or utf8 string of words, strings and (line) comments.
///
/// ```rust
/// use std::path::PathBuf;
/// use advreader::*;
///
/// // construct our iterator from our file input
/// let reader = AdvReader::default(&PathBuf::from("../testdata/example.txt"));
///
/// let mut reader_ok = reader.unwrap();
///
/// // walk our item using `while` syntax
/// for item in reader_ok.into_iter() {
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
/// let reader = AdvReader::default(&PathBuf::from("../testdata/example.txt"));
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
    thread_handle: Option<JoinHandle<Result<(usize, ReaderState), Error>>>,
    items: Receiver<Option<(usize, Result<AdvReturnValue, Error>)>>,
    // Items which were pushed back
    stack_tx: Sender<Option<(usize, Result<AdvReturnValue, Error>)>>,
    stack_rx: Receiver<Option<(usize, Result<AdvReturnValue, Error>)>>,
    stop: Arc<AtomicBool>,
}

impl AdvReader {
    /// Constructs a new `AdvReader`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: &Path,
        buffer_size: Option<usize>,
        trim: Option<bool>,
        line_end: Option<u8>,
        skip_comments: Option<bool>,
        encode_comments: Option<bool>,
        encode_strings: Option<bool>,
        encoder: Option<String>,
        allow_invalid_utf8: Option<bool>,
        extended_word_separation: Option<bool>,
        double_quote_escape: Option<bool>,
        convert2numbers: Option<bool>,
        keep_base: Option<bool>,
        bool_false: Option<Vec<u8>>,
        bool_true: Option<Vec<u8>>,
        block_reader: Option<Box<dyn block::Block + Send + Sync>>,
    ) -> Result<Self, Error> {
        AdvReader::with_capacity(
            path,
            buffer_size,
            trim,
            line_end,
            skip_comments,
            encode_comments,
            encode_strings,
            encoder,
            allow_invalid_utf8,
            extended_word_separation,
            double_quote_escape,
            convert2numbers,
            keep_base,
            bool_false,
            bool_true,
            block_reader,
        )
    }

    pub fn default(path: &Path) -> Result<Self, Error> {
        AdvReader::new(
            path, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        )
    }

    /// Constructs a new `AdvReader`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_capacity(
        path: &Path,
        buffer_size: Option<usize>,
        trim: Option<bool>,
        line_end: Option<u8>,
        skip_comments: Option<bool>,
        encode_comments: Option<bool>,
        encode_strings: Option<bool>,
        encoder: Option<String>,
        allow_invalid_utf8: Option<bool>,
        extended_word_separation: Option<bool>,
        double_quote_escape: Option<bool>,
        convert2numbers: Option<bool>,
        keep_base: Option<bool>,
        bool_false: Option<Vec<u8>>,
        bool_true: Option<Vec<u8>>,
        block_reader: Option<Box<dyn block::Block + Send + Sync>>,
    ) -> Result<Self, Error> {
        if metadata(path).is_err() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("File {path:?} not found or is not readable!"),
            ));
        }
        let buffer_size = buffer_size.unwrap_or(65536);
        if buffer_size < 64 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Buffer size is too small! Minimum value is 64!",
            ));
        }
        let (tx, rx) = bounded(256);
        let (stack_tx, stack_rx) = bounded(256);
        let stop = Arc::new(AtomicBool::new(false));
        let options = AdvReaderOptions {
            path: path.to_owned(),
            buffer_size,
            trim: trim.unwrap_or(false),
            line_end: line_end.unwrap_or(b'\n'),
            skip_comments: skip_comments.unwrap_or(false),
            encode_comments: encode_comments.unwrap_or(false),
            encode_strings: encode_strings.unwrap_or(false),
            encoder,
            allow_invalid_utf8: allow_invalid_utf8.unwrap_or(false),
            extended_word_separation: extended_word_separation.unwrap_or(false),
            double_quote_escape: double_quote_escape.unwrap_or(false),
            convert2numbers: convert2numbers.unwrap_or(false),
            keep_base: keep_base.unwrap_or(false),
            bool_false,
            bool_true,
        };
        Ok(Self {
            thread_handle: Some(AdvReader::reader_thread(
                options,
                tx,
                stop.clone(),
                block_reader,
            )),
            items: rx,
            stack_tx,
            stack_rx,
            stop,
        })
    }

    fn reader_thread(
        options: AdvReaderOptions,
        tx: Sender<Option<(usize, Result<AdvReturnValue, Error>)>>,
        stop: Arc<AtomicBool>,
        block_reader: Option<Box<dyn block::Block + Send + Sync>>,
    ) -> JoinHandle<Result<(usize, ReaderState), Error>> {
        thread::spawn(move || AdvReaderThread::new(options, tx, stop, block_reader)?.read())
    }

    pub fn stop(&mut self) -> Result<(usize, ReaderState), Error> {
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
}

/// `IntoIterator` conversion for `AdvReader` to provide `Iterator` APIs.
impl IntoIterator for AdvReader {
    type Item = Result<AdvReturnValue, Error>;
    type IntoIter = AdvReaderIter;

    /// Constructs a `advreaderIter` to provide an `Iterator` API.
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        AdvReaderIter {
            inner: self,
            line: Vec::with_capacity(10),
            line_nr: 0,
            reader_died: false,
        }
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
/// let reader = AdvReader::default(&PathBuf::from("../testdata/example.txt"));
///
/// let mut reader_ok = reader.unwrap();
///
/// // walk our items using `for` syntax
/// for item in reader_ok.into_iter() {
///     // do something with the item, which is Result<AdvReturnValue, Error>
/// }
/// ```

#[derive(Debug)]
pub struct AdvReaderIter {
    inner: AdvReader,
    line: Vec<AdvReturnValue>,
    line_nr: usize,
    reader_died: bool,
}

impl AdvReaderIter {
    /// Returns the corresponding line in the file for the latest returned item.
    pub fn line_nr(&self) -> usize {
        self.line_nr
    }

    pub fn reader_died(&self) -> bool {
        self.reader_died
    }

    pub fn stop(&mut self) -> Result<(usize, ReaderState), Error> {
        self.inner.stop()
    }

    pub fn next_line(&mut self) -> Option<Result<(usize, Vec<AdvReturnValue>), Error>> {
        loop {
            match self.inner.items.recv() {
                Ok(Some((line_nr, item))) => match item {
                    Ok(item) => {
                        if line_nr != self.line_nr {
                            self.line_nr = line_nr;
                            let line = self.line.drain(..).collect();
                            self.line.push(item);
                            return Some(Ok((line_nr - 1, line)));
                        }
                        self.line.push(item);
                    }
                    Err(e) => {
                        if self.reader_died {
                            return None;
                        }
                        self.reader_died = true;
                        return Some(Err(Error::new(
                            ErrorKind::BrokenPipe,
                            format!("Reader thread died: {e}"),
                        )));
                    }
                },
                Ok(None) => {
                    if self.line.is_empty() {
                        return None;
                    }
                    return Some(Ok((self.line_nr - 1, self.line.drain(..).collect())));
                }
                Err(e) => {
                    if self.reader_died {
                        return None;
                    }
                    self.reader_died = true;
                    return Some(Err(Error::new(
                        ErrorKind::BrokenPipe,
                        format!("Reader thread died: {e}"),
                    )));
                }
            }
        }
    }

    pub fn push_back(
        &self,
        item: Option<(usize, Result<AdvReturnValue, Error>)>,
    ) -> Result<(), Error> {
        self.inner
            .stack_tx
            .send(item)
            .map_err(|e| Error::new(ErrorKind::BrokenPipe, e.to_string()))
    }
}

impl Iterator for AdvReaderIter {
    type Item = Result<AdvReturnValue, Error>;

    /// Retrieves the next item in the iterator (if any).
    #[inline]
    fn next(&mut self) -> Option<Result<AdvReturnValue, Error>> {
        if let Ok(item) = self.inner.stack_rx.try_recv() {
            if let Some((line_nr, item)) = item {
                self.line_nr = line_nr;
                return Some(item);
            } else {
                return None;
            }
        }
        match self.inner.items.recv() {
            Ok(Some((line_nr, item))) => {
                self.line_nr = line_nr;
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
                    format!("Reader thread died: {e}"),
                )))
            }
        }
    }
}

#[cfg(test)]
mod test;
