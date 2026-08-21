use std::iter::Enumerate;

use crate::{AdvReturnValue, FnBlockReturnType};

use super::Block;

pub struct A2LBlock {
    pub(crate) line_end: u8,        // Line end character
    pub(crate) skip_comments: bool, // Skip comments
    // Status
    pub(crate) begin: bool,      // /begin detected
    pub(crate) if_data: bool,    // Begin of IF_DATA block. Next token is key.
    pub(crate) block_mode: bool, // We are within a block
    pub(crate) escape: bool,
    pub(crate) slash: bool,
    pub(crate) asterisk: bool,
    pub(crate) comment: bool,
    pub(crate) line_comment: bool,
    pub(crate) level: i32,            // /begin - /end level
    pub(crate) end_pos: usize,        // Start position of /end keyword
    pub(crate) end_before: usize,     // Reverse word count after last /end beginning from 2
    pub(crate) item: Option<Vec<u8>>, // Block type A2ML or IF_DATA
    pub(crate) key: Option<Vec<u8>>,  // Block name in case of IF_DATA
    pub(crate) block: Vec<Vec<u8>>,   // Block contents
}

impl Block for A2LBlock {
    fn new(line_end: u8, skip_comments: bool) -> Box<Self> {
        Box::new(Self {
            line_end,
            skip_comments,
            begin: false,
            if_data: false,
            block_mode: false,
            escape: false,
            slash: false,
            asterisk: false,
            comment: false,
            line_comment: false,
            level: 1,
            end_pos: 0,
            end_before: 0,
            item: None,
            key: None,
            block: Vec::new(),
        })
    }

    fn is_block_mode(&self) -> bool {
        self.block_mode
    }

    fn start_block(&mut self) {
        self.block_mode = true;
        self.escape = false;
        self.slash = false;
        self.asterisk = false;
        self.comment = false;
        self.line_comment = false;
        self.level = 1;
        self.end_pos = 0;
        self.end_before = 0;
    }

    fn build_result(&mut self) -> Vec<AdvReturnValue> {
        let mut result = vec![AdvReturnValue::Bytes(self.item.take().unwrap())];
        //result.push(AdvReturnValue::StringUtf8(s));
        if let Some(v) = self.key.take() {
            result.push(AdvReturnValue::Bytes(v))
        }
        let buf = self.block.drain(..).flatten().collect::<Vec<u8>>();
        result.push(AdvReturnValue::Block(buf));
        result
    }

    fn read_block(
        &mut self,
        item: &[u8],
        buffer: &[u8],
        itr: &mut Enumerate<std::slice::Iter<'_, u8>>,
        mut i: usize,
        mut c: u8,
        line_num: &mut usize,
    ) -> FnBlockReturnType {
        // Check for block start
        let mut block_start = 0;
        if !self.block_mode {
            if self.if_data {
                if item == b"/end" {
                    self.if_data = false;
                }
                if self.key.is_none() {
                    self.key = Some(item.to_vec());
                    self.block_mode = true;
                }
            } else if self.begin {
                if item == b"IF_DATA" {
                    self.if_data = true;
                    self.block_mode = true;
                    self.item = Some(item.to_vec());
                } else if item == b"A2ML" {
                    self.block_mode = true;
                    self.item = Some(item.to_vec());
                } else {
                    let item_len = item.len();
                    if i == buffer.len() && item_len < 7 {
                        return (i, -(item_len as isize) - 1, None);
                    }
                }
                self.begin = false;
            } else if item == b"/begin" {
                self.begin = true;
            }
            if self.block_mode {
                self.start_block();
                self.item = Some(item.to_vec());
                block_start = i;
            } else {
                return (i, c as isize, None);
            }
        }
        if self.block_mode {
            let mut line_cnt = 0;
            let mut word_start = None; // Start of current word
            let mut block_end = None; // Start of last /end word. May be end of block data.
            if self.escape {
                // If ESC was last character of last buffer contents.
                self.escape = false;
            } else if c == b'\\' {
                self.escape = true;
            } else if c == self.line_end {
                line_cnt += 1;
                self.line_comment = false;
            }
            if self.comment {
                if self.asterisk {
                    self.asterisk = false;
                    if c == b'/' {
                        self.comment = false;
                    }
                }
                if c == b'*' {
                    self.asterisk = true;
                }
            } else if c == b'/' {
                if self.slash {
                    self.slash = false;
                    self.line_comment = true;
                } else {
                    self.slash = true;
                }
            } else if c == b'*' {
                if self.slash {
                    self.slash = false;
                    self.comment = true;
                }
            } else if self.slash {
                self.slash = false;
            }
            if c > b' ' && c < b'\x7f' {
                word_start = Some(i);
            }
            loop {
                (i, c) = match itr.next() {
                    Some((i, c)) => (i, *c),
                    None => break,
                };
                if self.escape {
                    self.escape = false;
                    continue;
                }
                if c == b'\\' {
                    self.escape = true;
                    continue;
                }
                if c == self.line_end {
                    line_cnt += 1;
                    if self.line_comment {
                        self.line_comment = false;
                        if !self.skip_comments && block_start < i - 1 {
                            let buf = &buffer[block_start..i];
                            self.block.push(buf.to_vec());
                        }
                        block_start = i;
                        continue;
                    }
                }
                if self.comment {
                    if self.asterisk {
                        self.asterisk = false;
                        if c == b'/' {
                            self.comment = false;
                            if !self.skip_comments && block_start < i - 1 {
                                let buf = &buffer[block_start..i];
                                self.block.push(buf.to_vec());
                            }
                            block_start = i;
                            word_start = None;
                        }
                    }
                    if c == b'*' {
                        self.asterisk = true;
                    }
                    continue;
                }
                if self.line_comment {
                    continue;
                }
                if c == b'/' {
                    if self.slash {
                        self.slash = false;
                        self.line_comment = true;
                        if block_start < i - 1 {
                            let buf = &buffer[block_start..i];
                            self.block.push(buf.to_vec());
                        }
                        block_start = i;
                        continue;
                    }
                    self.slash = true;
                } else if c == b'*' {
                    if self.slash {
                        self.slash = false;
                        self.comment = true;
                        if block_start < i - 1 {
                            let buf = &buffer[block_start..i - 1];
                            self.block.push(buf.to_vec());
                        }
                        block_start = i - 1;
                        continue;
                    }
                } else if self.slash {
                    self.slash = false;
                }
                if word_start.is_none() {
                    if c <= b' ' || c >= b'\x7f' {
                        continue;
                    }
                    word_start = Some(i);
                } else if c == b'/' {
                    word_start = Some(i);
                }
                if c > b' ' && c < b'\x7f' {
                    continue;
                }
                // Check word
                if self.end_before > 0 {
                    self.end_before -= 1;
                }
                let ws = word_start.take().unwrap();
                let buf = &buffer[ws..i];
                if self.if_data && self.key.is_none() {
                    self.key = Some(buf.to_vec());
                    block_start = i;
                    continue;
                }
                // Have we reached end of block?
                if self.level <= 1 && self.end_before > 0 && Some(buf) == self.item.as_deref() {
                    if let Some(be) = block_end.take() {
                        let buf = &buffer[block_start..be];
                        self.block.push(buf.to_vec());
                    }
                    self.block_mode = false;
                    self.if_data = false;
                    *line_num += line_cnt;
                    return (i, c as isize, Some(self.build_result()));
                } else if buf == b"/begin" {
                    self.level += 1;
                    block_end = None;
                } else if buf == b"/end" {
                    self.level -= 1;
                    self.end_pos = i - 4;
                    if i >= 4 {
                        block_end = Some(i - 4);
                        self.end_before = 2;
                    }
                } else if self.end_before > 0 {
                    self.end_pos = 0;
                    self.end_before = 0;
                    block_end = None;
                }
            }
            // We have reached the end of the buffer
            let mut s;
            if let Some(ws) = word_start.take() {
                let last_word = &buffer[ws..];
                // Have we reached end of block?
                if self.level <= 1 && self.end_before > 0 && Some(last_word) == self.item.as_deref()
                {
                    let buf = &buffer[block_start..self.end_pos];
                    self.block.push(buf.to_vec());
                    self.block_mode = false;
                    self.if_data = false;
                    *line_num += line_cnt;
                    return (i + 1, c as isize, Some(self.build_result()));
                }
                if last_word == b"/begin" {
                    self.level += 1;
                    s = buffer.len();
                } else if last_word == b"/end" {
                    self.level -= 1;
                    self.end_before = 2;
                    s = ws;
                } else if self.end_before == 2 {
                    s = self.end_pos;
                } else {
                    s = ws;
                }
                if last_word == b"/" {
                    self.slash = false;
                }
            } else {
                s = buffer.len();
            }
            if block_start < s && (!self.if_data || self.key.is_some()) {
                if let Some(be) = block_end.take()
                    && be > block_start
                {
                    s = be;
                }
                let buf = &buffer[block_start..s];
                self.block.push(buf.to_vec());
            }
            *line_num += line_cnt;
            // Restart parsing from begin of last word
            return (i, -(s as isize) - 1, None);
        }
        (i, c as isize, None)
    }
}
