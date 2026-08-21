#[cfg(test)]
mod tests {
    use std::{io::Error, path::PathBuf};

    use crate::{AdvReader, AdvReturnValue};

    fn get_example_path(s: &str) -> PathBuf {
        let example_path = PathBuf::from(format!("../testdata/{s}"));
        if example_path.exists() {
            return example_path;
        }
        PathBuf::from(format!("testdata/{s}"))
    }

    #[test]
    fn read_mixed() -> Result<(), Error> {
        // This test checks integer numbers, bytes, UTF8-Strings and push_back method.
        // In addition the special cases 00 and 000 are checked.
        let reader = AdvReader::new(
            &get_example_path("mixed.txt"),
            None,                             // Buffer size. Default is 65536.
            Some(true),                       // Trim. Default is false.
            None,                             // Line ending. Default is '\n'.
            Some(false),                      // Skip comments. Default is false.
            Some(true),                       // Convert comments to UTF8. Default is true.
            Some(true),                       // Convert strings to UTF8. Default is true.
            Some("windows-1252".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
            Some("replace".to_string()),      // Allow invalid UTF8 characters. Default is false.
            Some(false),                      // Extended word separation. Default is false.
            Some(true),                       // Double double quote escaping. Default is false.
            Some(true), // Try to convert words into numbers (int, float). Default is false.
            Some(false), // Keep number base. Default is false.
            None,       // BOOL false
            None,       // BOOL true
            None,       // Callback function for block mode
        )?;
        let mut reader_iter = reader.into_iter();
        let token = reader_iter.next();
        println!("{} {token:?}", reader_iter.line_nr());
        assert_eq!(
            token.as_ref().unwrap().as_ref().unwrap(),
            &AdvReturnValue::Int(123),
            "Wrong result with {token:?}"
        );
        let token = reader_iter.next();
        println!("{} {token:?}", reader_iter.line_nr());
        assert_eq!(
            token.as_ref().unwrap().as_ref().unwrap(),
            &AdvReturnValue::Int(0),
            "Wrong result with {token:?}"
        );
        let token = reader_iter.next();
        println!("{} {token:?}", reader_iter.line_nr());
        assert_eq!(
            token.as_ref().unwrap().as_ref().unwrap(),
            &AdvReturnValue::Bytes(vec![48, 48]),
            "Wrong result with {token:?}"
        );
        let token = reader_iter.next();
        println!("{} {token:?}", reader_iter.line_nr());
        assert_eq!(
            token.as_ref().unwrap().as_ref().unwrap(),
            &AdvReturnValue::Bytes(vec![48, 48, 48]),
            "Wrong result with {token:?}"
        );
        let token = reader_iter.next();
        println!("{} {token:?}", reader_iter.line_nr());
        assert_eq!(
            token.as_ref().unwrap().as_ref().unwrap(),
            &AdvReturnValue::Bytes(vec![84, 101, 115, 116]),
            "Wrong result with {token:?}"
        );
        let token = reader_iter.next();
        println!("{} {token:?}", reader_iter.line_nr());
        assert_eq!(
            token.as_ref().unwrap().as_ref().unwrap(),
            &AdvReturnValue::StringUtf8("Test".to_string()),
            "Wrong result with {token:?}"
        );
        reader_iter
            .push_back(Some((reader_iter.line_nr(), token.unwrap())))
            .unwrap();
        let token = reader_iter.next();
        println!("{} {token:?}", reader_iter.line_nr());
        assert_eq!(
            token.as_ref().unwrap().as_ref().unwrap(),
            &AdvReturnValue::StringUtf8("Test".to_string()),
            "Wrong result with {token:?}"
        );
        let token = reader_iter.next();
        println!("{} {token:?}", reader_iter.line_nr());
        assert_eq!(
            token.as_ref().unwrap().as_ref().unwrap(),
            &AdvReturnValue::Bytes(vec![69, 110, 100]),
            "Wrong result with {token:?}"
        );
        let token = reader_iter.next();
        println!("{} {token:?}", reader_iter.line_nr());
        assert_eq!(token.is_none(), true, "Wrong result with {token:?}");
        Ok(())
    }
}
