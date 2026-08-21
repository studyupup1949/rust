#[cfg(test)]
mod tests {
    use std::{
        io::Error,
        path::PathBuf,
        sync::{Arc, atomic::AtomicBool},
    };

    use flume::bounded;

    use crate::{AdvReaderOptions, AdvReaderThread, Source};

    fn get_example_path(s: &str) -> PathBuf {
        let example_path = PathBuf::from(format!("../testdata/{s}"));
        if example_path.exists() {
            return example_path;
        }
        PathBuf::from(format!("testdata/{s}"))
    }

    fn read(reader: &mut AdvReaderThread, buf: &[u8]) -> Option<(usize, u8)> {
        let mut itr = buf.iter().enumerate();
        let (i, c) = itr.next().unwrap();
        reader.string = false;
        reader.quote = false;
        reader.start = Some(0);
        reader.buffer_end = buf.len() - 1;
        match reader.read_string(&mut itr, i, *c) {
            Some((i, c)) => Some((i, c)),
            None => {
                if let Some(_) = reader.start.take()
                    && reader.quote
                {
                    return Some((reader.buffer_end, b'"'));
                }
                None
            }
        }
    }

    #[test]
    fn read_string() -> Result<(), Error> {
        let (tx, _) = bounded(256);
        let stop = Arc::new(AtomicBool::new(false));
        let options =
            AdvReaderOptions::new(Source::File(get_example_path("empty.txt")), None, None);
        let mut reader = AdvReaderThread::new(options, tx, stop, None)?;
        assert_eq!(read(&mut reader, b"\"\"\"A\"\"\""), Some((1, 34)));
        assert_eq!(read(&mut reader, b"\"Test\""), Some((5, 34)));
        assert_eq!(read(&mut reader, b"\"Test\" "), Some((5, 32)));
        assert_eq!(read(&mut reader, b"\"Test\"\r\n"), Some((5, 13)));
        assert_eq!(read(&mut reader, b"\" Test \n\"\r\n"), Some((8, 13)));
        assert_eq!(read(&mut reader, b"\" Test \"\" \n\"\r\n"), Some((7, 34)));
        assert_eq!(read(&mut reader, b"\" Test \"\"\"\r\n"), Some((7, 34)));
        assert_eq!(read(&mut reader, b"\"\\\" Test \\\"\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\" \\\" Tst \\\"\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\"\"\" Test \"\"\"\r\n"), Some((1, 34)));
        assert_eq!(read(&mut reader, b"\" \"\" Tst \"\"\"\r\n"), Some((2, 34)));
        Ok(())
    }

    #[test]
    fn read_string_trim() -> Result<(), Error> {
        let (tx, _) = bounded(256);
        let stop = Arc::new(AtomicBool::new(false));
        let mut options =
            AdvReaderOptions::new(Source::File(get_example_path("empty.txt")), None, None);
        options.trim = true;
        let mut reader = AdvReaderThread::new(options, tx, stop, None)?;
        assert_eq!(read(&mut reader, b"\"\"\"A\"\"\""), Some((1, 34)));
        assert_eq!(read(&mut reader, b"\"Test\""), Some((5, 34)));
        assert_eq!(read(&mut reader, b"\"Test\" "), Some((5, 32)));
        assert_eq!(read(&mut reader, b"\"Test\"\r\n"), Some((5, 13)));
        assert_eq!(read(&mut reader, b"\" Test \n\"\r\n"), Some((8, 13)));
        assert_eq!(read(&mut reader, b"\" Test \"\" \n\"\r\n"), Some((7, 34)));
        assert_eq!(read(&mut reader, b"\" Test \"\"\"\r\n"), Some((7, 34)));
        assert_eq!(read(&mut reader, b"\"\\\" Test \\\"\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\" \\\" Tst \\\"\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\"\"\" Test \"\"\"\r\n"), Some((1, 34)));
        assert_eq!(read(&mut reader, b"\" \"\" Tst \"\"\"\r\n"), Some((2, 34)));
        Ok(())
    }

    #[test]
    fn read_string_double_quotes() -> Result<(), Error> {
        let (tx, _) = bounded(256);
        let stop = Arc::new(AtomicBool::new(false));
        let mut options =
            AdvReaderOptions::new(Source::File(get_example_path("empty.txt")), None, None);
        options.double_quote_escape = true;
        let mut reader = AdvReaderThread::new(options, tx, stop, None)?;
        assert_eq!(read(&mut reader, b"\"\"\"A\"\"\""), Some((6, 34)));
        assert_eq!(read(&mut reader, b"\"Test\""), Some((5, 34)));
        assert_eq!(read(&mut reader, b"\"Test\" "), Some((5, 32)));
        assert_eq!(read(&mut reader, b"\"Test\"\r\n"), Some((5, 13)));
        assert_eq!(read(&mut reader, b"\" Test \n\"\r\n"), Some((8, 13)));
        assert_eq!(read(&mut reader, b"\" Test \"\" \n\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\" Test \"\"\"\r\n"), Some((9, 13)));
        assert_eq!(read(&mut reader, b"\"\\\" Test \\\"\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\" \\\" Tst \\\"\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\"\"\" Test \"\"\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\" \"\" Tst \"\"\"\r\n"), Some((11, 13)));
        Ok(())
    }

    #[test]
    fn read_string_double_quotes_trim() -> Result<(), Error> {
        let (tx, _) = bounded(256);
        let stop = Arc::new(AtomicBool::new(false));
        let mut options =
            AdvReaderOptions::new(Source::File(get_example_path("empty.txt")), None, None);
        options.trim = true;
        options.double_quote_escape = true;
        let mut reader = AdvReaderThread::new(options, tx, stop, None)?;
        assert_eq!(read(&mut reader, b"\"\"\"A\"\"\""), Some((6, 34)));
        assert_eq!(read(&mut reader, b"\"Test\""), Some((5, 34)));
        assert_eq!(read(&mut reader, b"\"Test\" "), Some((5, 32)));
        assert_eq!(read(&mut reader, b"\"Test\"\r\n"), Some((5, 13)));
        assert_eq!(read(&mut reader, b"\" Test \n\"\r\n"), Some((8, 13)));
        assert_eq!(read(&mut reader, b"\" Test \"\" \n\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\" Test \"\"\"\r\n"), Some((9, 13)));
        assert_eq!(read(&mut reader, b"\"\\\" Test \\\"\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\" \\\" Tst \\\"\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\"\"\" Test \"\"\"\r\n"), Some((11, 13)));
        assert_eq!(read(&mut reader, b"\" \"\" Tst \"\"\"\r\n"), Some((11, 13)));
        Ok(())
    }
}
