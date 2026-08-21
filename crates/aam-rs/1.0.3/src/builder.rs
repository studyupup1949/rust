use std::fmt::Display;
use std::io;
use std::ops::Deref;
use std::path::Path;

pub struct AAMBuilder {
    buffer: String,
}

impl AAMBuilder {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: String::with_capacity(capacity),
        }
    }

    pub fn add_line(&mut self, key: &str, value: &str) {
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }

        self.buffer.push_str(key);
        self.buffer.push_str(" = ");

        self.buffer.push_str(value);
    }

    pub fn add_raw(&mut self, raw_line: &str) {
        if !self.buffer.is_empty() {
            self.buffer.push('\n');
        }
        self.buffer.push_str(raw_line);
    }
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        std::fs::write(path, self.buffer.as_bytes())
    }

    pub fn build(self) -> String {
        self.buffer
    }

    pub fn as_string(&self) -> String {
        self.buffer.clone()
    }
}

impl Deref for AAMBuilder {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl Default for AAMBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for AAMBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.buffer)
    }
}