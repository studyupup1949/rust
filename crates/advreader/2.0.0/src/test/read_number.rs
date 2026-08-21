#[cfg(test)]
mod tests {
    use std::{
        io::Error,
        path::PathBuf,
        sync::{atomic::AtomicBool, Arc},
    };

    use flume::bounded;

    use crate::{AdvReaderOptions, AdvReaderThread, AdvReturnValue};

    fn get_example_path(s: &str) -> PathBuf {
        let example_path = PathBuf::from(format!("../testdata/{s}"));
        if example_path.exists() {
            return example_path;
        }
        PathBuf::from(format!("testdata/{s}"))
    }

    fn check(
        buf: &[u8],
        keep_base: bool,
        // Check status and results
        expected_i: usize,
        expected_c: u8,
        expected_result: Option<AdvReturnValue>,
    ) -> Result<(), Error> {
        let (tx, _) = bounded(256);
        let stop = Arc::new(AtomicBool::new(false));
        let mut options = AdvReaderOptions::new(&get_example_path("empty.txt"));
        options.keep_base = keep_base;
        let mut reader = AdvReaderThread::new(options, tx, stop, None)?;
        let mut itr = buf.iter().enumerate();
        let (i, c) = itr.next().unwrap();
        let (i, c, result) = reader.read_number(&mut itr, i, *c);
        assert_eq!(
            i,
            expected_i,
            "Wrong i with {:?}",
            std::str::from_utf8(&buf)
        );
        assert_eq!(
            c,
            expected_c,
            "Wrong c with {:?}",
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
    fn read_number() -> Result<(), Error> {
        check(b"0", false, 0, 48, None)?;
        check(b"123", false, 2, 51, None)?;
        check(b"+12 34", false, 3, 32, Some(AdvReturnValue::Int(12)))?;
        check(b"12 34", false, 2, 32, Some(AdvReturnValue::Int(12)))?;
        check(b"-12 34", false, 3, 32, Some(AdvReturnValue::Int(-12)))?;
        check(b"0x12 34", false, 4, 32, Some(AdvReturnValue::Int(18)))?;
        check(b"0xaF 34", false, 4, 32, Some(AdvReturnValue::Int(175)))?;
        check(b"+0xaF 34", false, 5, 32, Some(AdvReturnValue::Int(175)))?;
        check(b"-0xaF 34", false, 5, 32, Some(AdvReturnValue::Int(-175)))?;
        check(b"0xaFk 34", false, 4, 107, Some(AdvReturnValue::Int(175)))?;
        check(b"0o12 34", false, 4, 32, Some(AdvReturnValue::Int(10)))?;
        check(b"+0o12 34", false, 5, 32, Some(AdvReturnValue::Int(10)))?;
        check(b"-0o12 34", false, 5, 32, Some(AdvReturnValue::Int(-10)))?;
        check(b"0b12 34", false, 4, 32, Some(AdvReturnValue::Int(1)))?;
        check(b"0b101 34", false, 5, 32, Some(AdvReturnValue::Int(5)))?;
        check(b"+0b101 34", false, 6, 32, Some(AdvReturnValue::Int(5)))?;
        check(b"-0b101 34", false, 6, 32, Some(AdvReturnValue::Int(-5)))?;
        check(b"12.0 34", false, 4, 32, Some(AdvReturnValue::Float(12.0)))?;
        check(b"12.9 34", false, 4, 32, Some(AdvReturnValue::Float(12.9)))?;
        check(b"0.9 34", false, 3, 32, Some(AdvReturnValue::Float(0.9)))?;
        check(b"+0.9 34", false, 4, 32, Some(AdvReturnValue::Float(0.9)))?;
        check(b"-0.9 34", false, 4, 32, Some(AdvReturnValue::Float(-0.9)))?;
        check(b".9 34", false, 2, 32, Some(AdvReturnValue::Float(0.9)))?;
        check(b"+.9 34", false, 3, 32, Some(AdvReturnValue::Float(0.9)))?;
        check(b"-.9 34", false, 3, 32, Some(AdvReturnValue::Float(-0.9)))?;
        check(b"0.9a 34", false, 3, 97, Some(AdvReturnValue::Float(0.9)))?;
        check(
            b"12.9e0 34",
            false,
            6,
            32,
            Some(AdvReturnValue::Float(12.9)),
        )?;
        check(
            b"12.9e1 34",
            false,
            6,
            32,
            Some(AdvReturnValue::Float(129.0)),
        )?;
        check(
            b"12.9e+1 34",
            false,
            7,
            32,
            Some(AdvReturnValue::Float(129.0)),
        )?;
        check(
            b"12.9e-1 34",
            false,
            7,
            32,
            Some(AdvReturnValue::Float(1.29)),
        )?;
        check(
            b"12.9e+1k 34",
            false,
            7,
            107,
            Some(AdvReturnValue::Float(129.0)),
        )?;
        Ok(())
    }

    #[test]
    fn read_number_keep_base() -> Result<(), Error> {
        check(b"0", true, 0, 48, None)?;
        check(b"123", true, 2, 51, None)?;
        check(b"+12 34", true, 3, 32, Some(AdvReturnValue::Int(12)))?;
        check(b"12 34", true, 2, 32, Some(AdvReturnValue::Int(12)))?;
        check(b"-12 34", true, 3, 32, Some(AdvReturnValue::Int(-12)))?;
        check(b"0x12 34", true, 4, 32, Some(AdvReturnValue::Hex(18)))?;
        check(b"0xaF 34", true, 4, 32, Some(AdvReturnValue::Hex(175)))?;
        check(b"+0xaF 34", true, 5, 32, Some(AdvReturnValue::Hex(175)))?;
        check(b"-0xaF 34", true, 5, 32, Some(AdvReturnValue::Hex(-175)))?;
        check(b"0xaFk 34", true, 4, 107, Some(AdvReturnValue::Hex(175)))?;
        check(b"0o12 34", true, 4, 32, Some(AdvReturnValue::Oct(10)))?;
        check(b"+0o12 34", true, 5, 32, Some(AdvReturnValue::Oct(10)))?;
        check(b"-0o12 34", true, 5, 32, Some(AdvReturnValue::Oct(-10)))?;
        check(b"0b12 34", true, 4, 32, Some(AdvReturnValue::Bin(1)))?;
        check(b"0b101 34", true, 5, 32, Some(AdvReturnValue::Bin(5)))?;
        check(b"+0b101 34", true, 6, 32, Some(AdvReturnValue::Bin(5)))?;
        check(b"-0b101 34", true, 6, 32, Some(AdvReturnValue::Bin(-5)))?;
        check(b"12.0 34", false, 4, 32, Some(AdvReturnValue::Float(12.0)))?;
        check(b"12.9 34", false, 4, 32, Some(AdvReturnValue::Float(12.9)))?;
        check(b"0.9 34", false, 3, 32, Some(AdvReturnValue::Float(0.9)))?;
        check(b"+0.9 34", false, 4, 32, Some(AdvReturnValue::Float(0.9)))?;
        check(b"-0.9 34", false, 4, 32, Some(AdvReturnValue::Float(-0.9)))?;
        check(b".9 34", false, 2, 32, Some(AdvReturnValue::Float(0.9)))?;
        check(b"+.9 34", false, 3, 32, Some(AdvReturnValue::Float(0.9)))?;
        check(b"-.9 34", false, 3, 32, Some(AdvReturnValue::Float(-0.9)))?;
        check(b"0.9a 34", false, 3, 97, Some(AdvReturnValue::Float(0.9)))?;
        check(
            b"12.9e0 34",
            false,
            6,
            32,
            Some(AdvReturnValue::Float(12.9)),
        )?;
        check(
            b"12.9e1 34",
            false,
            6,
            32,
            Some(AdvReturnValue::Float(129.0)),
        )?;
        check(
            b"12.9e+1 34",
            false,
            7,
            32,
            Some(AdvReturnValue::Float(129.0)),
        )?;
        check(
            b"12.9e-1 34",
            false,
            7,
            32,
            Some(AdvReturnValue::Float(1.29)),
        )?;
        check(
            b"12.9e+1k 34",
            false,
            7,
            107,
            Some(AdvReturnValue::Float(129.0)),
        )?;
        Ok(())
    }
}
