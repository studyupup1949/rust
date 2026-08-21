#[cfg(test)]
mod tests {
    use std::{
        io::Error,
        path::PathBuf,
        sync::{atomic::AtomicBool, Arc},
    };

    use flume::bounded;

    use crate::{AdvReaderOptions, AdvReaderThread};

    fn get_example_path(s: &str) -> PathBuf {
        let example_path = PathBuf::from(format!("../testdata/{s}"));
        if example_path.exists() {
            return example_path;
        }
        PathBuf::from(format!("testdata/{s}"))
    }

    fn check(
        buf: &[u8],
        // Check status and results
        escape: bool,
        asterisk: bool,
        line_num: usize,
        expected_result: Option<(usize, u8)>,
    ) -> Result<(), Error> {
        let (tx, _) = bounded(256);
        let stop = Arc::new(AtomicBool::new(false));
        let options = AdvReaderOptions::new(&get_example_path("empty.txt"), None);
        let mut reader = AdvReaderThread::new(options, tx, stop, None)?;
        let mut itr = buf.iter().enumerate();
        let (i, c) = itr.next().unwrap();
        let result = match reader.read_comment(&mut itr, i, *c) {
            Some((i, c)) => Some((i, c)),
            None => None,
        };
        assert_eq!(
            reader.escape,
            escape,
            "Wrong escape with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.asterisk,
            asterisk,
            "Wrong asterisk with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            reader.line_num,
            line_num,
            "Wrong line_num with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            result,
            expected_result,
            "Wrong result with {:?}",
            std::str::from_utf8(&buf)
        );
        Ok(())
    }

    #[test]
    fn read_comment() -> Result<(), Error> {
        check(b"Test", false, false, 1, None)?;
        check(b"/Test", false, false, 1, None)?;
        check(b"/Test/", false, false, 1, None)?;
        check(b"/*Test*", false, true, 1, None)?;
        check(b"/*Test/", false, false, 1, None)?;
        check(b"/*Test/*", false, true, 1, None)?;
        check(b"/*Test*/", false, false, 1, Some((7, 47)))?;
        check(b"/*Test\r\n\\A Next*/", false, false, 2, Some((16, 47)))?;
        check(b"Test*/", false, false, 1, Some((5, 47)))?;
        check(b"Test\\", true, false, 1, None)?;
        Ok(())
    }
}
