use std::io::{Error, ErrorKind};

use crate::{AdvReader, AdvReturnValue, ReaderState};

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
pub struct AdvReaderIter {
    pub inner: AdvReader,
    pub line: Vec<AdvReturnValue>,
    pub line_nr: usize,
    pub reader_died: bool,
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
