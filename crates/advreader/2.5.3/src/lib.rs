//! `advreader` is a simple library crate offering an iterator which splits a file into
//! - text sequences, separated by characters with ASCII codes <=32 and >=127.
//! - strings with double quotes as delimiters.
//! - line comments with '//' as start sequence.
//! - comment blocks with '/*' as start sequence and '*/' as end sequence.
//!
//! Results can be obatined through the `next` method.
//! Property `line_nr` provides the current line in the text file.

#![doc(html_root_url = "https://docs.rs/advreader/2.5.0")]
use std::fs::metadata;
use std::io::{Error, ErrorKind};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;

use flume::{Receiver, Sender, bounded};

pub mod block;
mod common;
mod iter;
mod options;
mod reader;

pub use block::Block;
pub use common::{AdvReturnValue, FnBlockReturnType, FnReadBlockType, ReaderState};
pub use iter::AdvReaderIter;
pub use options::AdvReaderOptions;
pub use options::Source;
pub use reader::AdvReaderThread;

/// Provides iteration over bytes or utf8 string of words, strings and (line) comments.
///
/// ```rust
/// use std::path::PathBuf;
/// use advreader::*;
///
/// // construct our iterator from our file input
/// let reader = AdvReader::with_defaults(Source::File(PathBuf::from("../testdata/example.txt")));
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
/// let reader = AdvReader::with_defaults(Source::File(PathBuf::from("../testdata/example.txt")));
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
        source: Source,
        buffer_size: Option<usize>,
        trim: Option<bool>,
        line_end: Option<u8>,
        skip_comments: Option<bool>,
        encode_comments: Option<bool>,
        encode_strings: Option<bool>,
        encoding: Option<String>,
        encoding_errors: Option<String>,
        extended_word_separation: Option<bool>,
        double_quote_escape: Option<bool>,
        convert2numbers: Option<bool>,
        keep_base: Option<bool>,
        bool_false: Option<Vec<u8>>,
        bool_true: Option<Vec<u8>>,
        block_reader: Option<Box<dyn block::Block + Send + Sync>>,
    ) -> Result<Self, Error> {
        AdvReader::with_capacity(
            source,
            buffer_size,
            trim,
            line_end,
            skip_comments,
            encode_comments,
            encode_strings,
            encoding,
            encoding_errors,
            extended_word_separation,
            double_quote_escape,
            convert2numbers,
            keep_base,
            bool_false,
            bool_true,
            block_reader,
        )
    }

    pub fn with_defaults(path: Source) -> Result<Self, Error> {
        Self::new(
            path, None, None, None, None, None, None, None, None, None, None, None, None, None,
            None, None,
        )
    }

    /// Constructs a new `AdvReader`.
    #[allow(clippy::too_many_arguments)]
    pub fn with_capacity(
        source: Source,
        buffer_size: Option<usize>,
        trim: Option<bool>,
        line_end: Option<u8>,
        skip_comments: Option<bool>,
        encode_comments: Option<bool>,
        encode_strings: Option<bool>,
        encoding: Option<String>,
        encoding_errors: Option<String>,
        extended_word_separation: Option<bool>,
        double_quote_escape: Option<bool>,
        convert2numbers: Option<bool>,
        keep_base: Option<bool>,
        bool_false: Option<Vec<u8>>,
        bool_true: Option<Vec<u8>>,
        block_reader: Option<Box<dyn block::Block + Send + Sync>>,
    ) -> Result<Self, Error> {
        if let Source::File(ref path) = source
            && metadata(path).is_err()
        {
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
            source,
            buffer_size,
            trim: trim.unwrap_or(false),
            line_end: line_end.unwrap_or(b'\n'),
            skip_comments: skip_comments.unwrap_or(false),
            encode_comments: encode_comments.unwrap_or(false),
            encode_strings: encode_strings.unwrap_or(false),
            encoding,
            encoding_errors,
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
        std::thread::spawn(move || AdvReaderThread::new(options, tx, stop, block_reader)?.read())
    }

    pub fn stop(&mut self) -> Result<(usize, ReaderState), Error> {
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.items.try_recv(); // Read last entry
        let _ = self.items.try_recv(); // Read None
        match self.thread_handle.take() {
            Some(h) => match h.join() {
                Ok(result) => result,
                Err(e) => Err(Error::other(format!("Failed to join thread: {e:?}"))),
            },
            None => Err(Error::other("No thread to stop!")),
        }
    }
}

impl Default for AdvReader {
    fn default() -> Self {
        Self::new(
            Source::String("".to_string()),
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
            None,
            None,
            None,
        )
        .unwrap()
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

#[cfg(test)]
mod test;
