//
// Copyright 2016 The IHEX Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms.
//

use std::path::PathBuf;

use advreader::*;

fn get_example_txt_path(s: &str) -> PathBuf {
    let example_txt_path = PathBuf::from(format!("../res/{s}.txt"));
    if example_txt_path.exists() {
        return example_txt_path;
    }
    PathBuf::from("res/example.txt")
}

#[test]
fn test_basic_loop() {
    let reader = AdvReader::default(&get_example_txt_path("example"));

    assert_eq!(reader.is_ok(), true);

    let mut reader_ok = reader.unwrap();

    let mut bytes = Vec::new();
    let mut strings = Vec::new();
    let mut comments = Vec::new();
    let mut line_comments = Vec::new();
    let mut others = Vec::new();

    while let Some(Ok(item)) = reader_ok.next() {
        match item {
            AdvReturnValue::Bytes(v) => bytes.push(String::from_utf8(v).unwrap()),
            AdvReturnValue::String(v) => strings.push(String::from_utf8(v).unwrap()),
            AdvReturnValue::Comment(v) => comments.push(String::from_utf8(v).unwrap()),
            AdvReturnValue::LineComment(v) => line_comments.push(String::from_utf8(v).unwrap()),
            _ => others.push(item),
        }
    }

    assert_eq!(others.len(), 0);
    assert_eq!(bytes.len(), 69);
    assert_eq!(strings.len(), 16);
    assert_eq!(comments.len(), 4);
    assert_eq!(line_comments.len(), 5);
}

#[test]
fn test_basic_iterator() {
    let reader = AdvReader::default(&get_example_txt_path("example"));

    assert_eq!(reader.is_ok(), true);

    let reader_ok = reader.unwrap();

    let mut bytes = Vec::new();
    let mut strings = Vec::new();
    let mut comments = Vec::new();
    let mut line_comments = Vec::new();
    let mut others = Vec::new();

    for item in reader_ok.into_iter() {
        match item {
            Ok(i) => match i {
                AdvReturnValue::Bytes(v) => bytes.push(String::from_utf8(v).unwrap()),
                AdvReturnValue::String(v) => strings.push(v),
                AdvReturnValue::Comment(v) => comments.push(v),
                AdvReturnValue::LineComment(v) => line_comments.push(v),
                _ => others.push(i),
            },
            Err(e) => {
                eprint!("{}", e);
                break;
            }
        }
    }

    assert_eq!(others.len(), 0);
    assert_eq!(bytes.len(), 69);
    assert_eq!(strings.len(), 16);
    assert_eq!(comments.len(), 4);
    assert_eq!(line_comments.len(), 5);
}

#[test]
fn test_convert_numbers() {
    #[allow(non_snake_case)]
    let FALSE = None; //Some(b"False".to_vec());
    #[allow(non_snake_case)]
    let TRUE = None; //Some(b"True".to_vec());
    let reader = AdvReader::new(
        &get_example_txt_path("example"),
        None,        // Trim. Default is false.
        None,        // Line ending. Default is '\n'.
        Some(false), // Skip comments. Default is false.
        Some(true),  // Convert comments to UTF8. Default is same as convert option.
        Some(true),  // Convert Strings and (line) comments to UTF8. Default is false.
        Some(true),  // Allow invalid UTF8 characters. Default is false.
        Some(false), // Extended word separation. Default is false.
        Some(true),  // Double double quote escaping. Default is false.
        Some(true),  // Try to convert words into numbers (int, float). Default is false.
        Some(true),  // Keep number base. Default is false.
        FALSE,       // BOOL false
        TRUE,        // BOOL true
        None as Option<Box<dyn Fn(Option<&[u8]>) -> (bool, Option<Vec<Vec<u8>>>) + Send + 'static>>,
    ); // Callback function for block mode

    assert_eq!(reader.is_ok(), true);

    let reader_ok = reader.unwrap();

    let mut bytes = Vec::new();
    let mut strings = Vec::new();
    let mut comments = Vec::new();
    let mut line_comments = Vec::new();
    let mut ints = Vec::new();
    let mut floats = Vec::new();
    let mut others = Vec::new();

    for item in reader_ok.into_iter() {
        match item {
            Ok(i) => match i {
                AdvReturnValue::Bytes(v) => bytes.push(String::from_utf8(v).unwrap()),
                AdvReturnValue::StringUtf8(v) => strings.push(v),
                AdvReturnValue::CommentUtf8(v) => comments.push(v),
                AdvReturnValue::LineCommentUtf8(v) => line_comments.push(v),
                AdvReturnValue::Int(v) => ints.push(v),
                AdvReturnValue::Float(v) => floats.push(v),
                _ => others.push(i),
            },
            Err(e) => {
                eprint!("{}", e);
                break;
            }
        }
    }

    assert_eq!(others.len(), 0);
    assert_eq!(ints.len(), 4);
    assert_eq!(floats.len(), 10);
    assert_eq!(bytes.len(), 57);
    assert_eq!(strings.len(), 10);
    assert_eq!(comments.len(), 4);
    assert_eq!(line_comments.len(), 4);
}

#[test]
fn test_empty_line() {
    let reader = AdvReader::default(&get_example_txt_path("empty"));

    assert_eq!(reader.is_ok(), true);

    let reader_ok = reader.unwrap();

    let mut bytes = Vec::new();
    let mut strings = Vec::new();
    let mut comments = Vec::new();
    let mut line_comments = Vec::new();

    for item in reader_ok.into_iter() {
        match item {
            Ok(i) => match i {
                AdvReturnValue::Bytes(v) => bytes.push(String::from_utf8(v).unwrap()),
                AdvReturnValue::String(v) => strings.push(v),
                AdvReturnValue::Comment(v) => comments.push(v),
                AdvReturnValue::LineComment(v) => line_comments.push(v),
                _ => {}
            },
            Err(e) => {
                eprint!("{}", e);
                break;
            }
        }
    }

    assert_eq!(bytes.len(), 1);
    assert_eq!(strings.len(), 0);
    assert_eq!(comments.len(), 0);
    assert_eq!(line_comments.len(), 0);
}

#[test]
fn test_invalid_filename() {
    let reader = AdvReader::default(&PathBuf::from("../res/invalid.txt"));

    assert_eq!(reader.is_ok(), false);
}
