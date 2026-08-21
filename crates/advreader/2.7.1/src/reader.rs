use std::{
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind, Read},
    iter::Enumerate,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use flume::Sender;

use crate::{AdvReaderOptions, AdvReturnValue, Block, ReaderState, Source, decode};

pub struct AdvReaderThread {
    // Internal
    pub source: Source,
    pub tx: Sender<Option<(usize, Result<AdvReturnValue, Error>)>>,
    pub stop: Arc<AtomicBool>,
    pub buffer_size: usize,
    pub start: Option<usize>,
    pub buffer_end: usize,
    pub line_num: usize,
    pub slash: bool,
    pub state: ReaderState,
    // Options
    pub encode_comments: bool,
    pub encode_strings: bool,
    pub convert2numbers: bool,
    pub keep_base: bool,
    pub skip_comments: bool,
    pub trim: bool,
    pub bool_false: Option<Vec<u8>>,
    pub bool_true: Option<Vec<u8>>,
    pub line_end: u8,
    pub double_quote_escape: bool,
    pub extended_word_separation: bool,
    // read
    pub escape: bool,
    // read comment
    pub asterisk: bool,
    // read string
    pub string: bool,
    pub quote: bool,
    // Block support
    pub block_reader: Option<Box<dyn Block + Send + Sync>>,
    // Encoder
    pub encoding: Option<String>,
    pub encoder_errors: Option<String>,
}

impl AdvReaderThread {
    pub fn new(
        options: AdvReaderOptions,
        tx: Sender<Option<(usize, Result<AdvReturnValue, Error>)>>,
        stop: Arc<AtomicBool>,
        block_reader: Option<Box<dyn Block + Send + Sync>>,
    ) -> Result<Self, Error> {
        if options.encoding.is_none()
            && !(options.encoding_errors.is_none()
                || options.encoding_errors == Some("strict".to_string()))
        {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "Invalid encoding error handling: {}",
                    options.encoding_errors.unwrap()
                ),
            ));
        }
        if let Source::File(ref path) = options.source {
            File::open(path)?;
        }
        Ok(Self {
            source: options.source,
            tx,
            stop,
            buffer_size: options.buffer_size,
            start: None,
            buffer_end: 0,
            line_num: 1,
            slash: false,
            state: ReaderState::Default,
            encode_comments: options.encode_comments,
            encode_strings: options.encode_strings,
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
            string: false,
            quote: false,
            encoding: options.encoding.clone(),
            encoder_errors: options.encoding_errors.clone(),
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
            AdvReturnValue::LineCommentUtf8(
                decode(
                    buf,
                    self.encoding.as_deref(),
                    self.encoder_errors.as_deref(),
                )?
                .to_string(),
            )
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
            AdvReturnValue::CommentUtf8(
                decode(
                    buf,
                    self.encoding.as_deref(),
                    self.encoder_errors.as_deref(),
                )?
                .to_string(),
            )
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
            AdvReturnValue::StringUtf8(
                decode(
                    buf,
                    self.encoding.as_deref(),
                    self.encoder_errors.as_deref(),
                )?
                .to_string(),
            )
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

    pub fn read_number(
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
        // self.quote is for start and end of a string (" char")
        // self.double_quote_escape enables "" detect as escaped ". This is from Spec V1.2 and prior.
        // Skip first character, because it's the start of the string (")
        loop {
            if self.escape {
                self.escape = false;
            } else if c == b'\\' {
                self.escape = true;
                self.quote = false;
            } else if c == b'"' {
                if !self.string {
                    self.string = true;
                } else if self.quote {
                    self.quote = false;
                    if !self.double_quote_escape {
                        self.string = false;
                        return Some((i - 1, c));
                    }
                } else {
                    self.quote = true;
                }
            } else if self.quote {
                self.quote = false;
                self.string = false;
                return Some((i - 1, c));
            } else if c == self.line_end {
                self.line_num += 1;
                self.quote = false;
            } else {
                self.quote = false;
            }
            (i, c) = match itr.next() {
                Some((i, c)) => (i, *c),
                None => break,
            };
        }
        None
    }

    pub fn read_line_comment(
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

    pub fn read_comment(
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
        let source = self.source.clone();
        let mut file = BufReader::new(match source {
            Source::File(path) => Box::new(BufReader::new(File::open(path)?)) as Box<dyn BufRead>,
            Source::String(ref content) => {
                Box::new(BufReader::new(content.as_bytes())) as Box<dyn BufRead>
            }
            Source::Bytes(ref bytes) => Box::new(BufReader::new(&bytes[..])) as Box<dyn BufRead>,
        });

        loop {
            if self.stop.load(Ordering::SeqCst) {
                break;
            }
            let nread = match file.read(&mut buffer[offset..]) {
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

            self.buffer_end = offset + nread;
            let buf = &mut buffer[..self.buffer_end];
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
                        if (self.state == ReaderState::Default || e > 0)
                            && let Some(s) = self.start.take()
                        {
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
        }
        // Do we have a string at the end of the file?
        if self.state == ReaderState::String && self.quote {
            if let Some(start) = self.start.take() {
                self.send_string(&buffer[start..offset])?;
            }
            self.state = ReaderState::Default;
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
