//
// Copyright 2016 The IHEX Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms.
//

use std::{io::Error, path::PathBuf};

use advreader::{AdvReader, ReaderState, Source};
use common::Results;

use crate::common::{get_example_path, parse_file};

mod common;
#[allow(non_snake_case)]
fn check_results(
    results: Results,
    state: (usize, ReaderState),
    last_bytes: Option<Vec<u8>>,
    bytes_cnt: usize,
    strings_cnt: usize,
    comments_cnt: usize,
    strings_utf8_cnt: usize,
    comments_utf8_cnt: usize,
    line_comments_utf8_cnt: usize,
    ints_cnt: usize,
    hexs_cnt: usize,
    floats_cnt: usize,
    blocks_cnt: usize,
) -> Result<(), Error> {
    assert_eq!(results.state.unwrap().ok(), Some(state));
    assert_eq!(results.last_bytes, last_bytes);
    assert_eq!(results.bytes.len(), bytes_cnt);
    assert_eq!(results.strings.len(), strings_cnt);
    assert_eq!(results.comments.len(), comments_cnt);
    assert_eq!(results.strings_utf8.len(), strings_utf8_cnt);
    assert_eq!(results.comments_utf8.len(), comments_utf8_cnt);
    assert_eq!(results.line_comments_utf8.len(), line_comments_utf8_cnt);
    assert_eq!(results.bools.len(), 0);
    assert_eq!(results.ints.len(), ints_cnt);
    assert_eq!(results.hexs.len(), hexs_cnt);
    assert_eq!(results.octs.len(), 0);
    assert_eq!(results.bins.len(), 0);
    assert_eq!(results.floats.len(), floats_cnt);
    assert_eq!(results.blocks.len(), blocks_cnt);
    Ok(())
}

#[test]
fn test_basic_iterator() -> Result<(), Error> {
    let reader = AdvReader::with_defaults(Source::File(get_example_path("example.txt")))?;
    check_results(
        parse_file(reader),
        (71, ReaderState::Default),
        Some(b"end".to_vec()),
        71,
        19,
        6,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    )
}

#[test]
fn test_convert_numbers() -> Result<(), Error> {
    #[allow(non_snake_case)]
    let FALSE = None; //Some(b"False".to_vec());
    #[allow(non_snake_case)]
    let TRUE = None; //Some(b"True".to_vec());
    let reader = AdvReader::new(
        Source::File(get_example_path("example.txt")),
        None,                             // Buffer size. Default is 65536.
        None,                             // Trim. Default is false.
        None,                             // Line ending. Default is '\n'.
        Some(false),                      // Skip comments. Default is false.
        Some(true), // Convert comments to UTF8. Default is same as convert option.
        Some(true), // Convert Strings and (line) comments to UTF8. Default is false.
        Some("windows-1252".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
        Some("replace".to_string()),      // Allow invalid UTF8 characters. Default is false.
        Some(false),                      // Extended word separation. Default is false.
        Some(true),                       // Double double quote escaping. Default is false.
        Some(true), // Try to convert words into numbers (int, float). Default is false.
        Some(true), // Keep number base. Default is false.
        FALSE,      // BOOL false
        TRUE,       // BOOL true
        None,       // Callback function for block mode
    )?;
    check_results(
        parse_file(reader),
        (71, ReaderState::Default),
        Some(b"end".to_vec()),
        54,
        0,
        0,
        11,
        6,
        6,
        3,
        1,
        10,
        0,
    )
}

#[test]
fn test_empty_line() -> Result<(), Error> {
    let reader = AdvReader::with_defaults(Source::File(get_example_path("empty.txt")))?;
    check_results(
        parse_file(reader),
        (2, ReaderState::Default),
        None,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    )
}

#[test]
fn test_ansi_encoding() -> Result<(), Error> {
    let reader = AdvReader::new(
        Source::File(get_example_path("ansi.txt")),
        None,                             // Buffer size. Default is 65536.
        None,                             // Trim. Default is false.
        None,                             // Line ending. Default is '\n'.
        Some(false),                      // Skip comments. Default is false.
        Some(true), // Convert comments to UTF8. Default is same as convert option.
        Some(true), // Convert Strings and (line) comments to UTF8. Default is false.
        Some("windows-1252".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
        Some("replace".to_string()),      // Allow invalid UTF8 characters. Default is false.
        Some(false),                      // Extended word separation. Default is false.
        Some(true),                       // Double double quote escaping. Default is false.
        Some(true), // Try to convert words into numbers (int, float). Default is false.
        Some(true), // Keep number base. Default is false.
        None,       // BOOL false
        None,       // BOOL true
        None,       // Callback function for block mode
    )?;
    check_results(
        parse_file(reader),
        (3, ReaderState::Default),
        None,
        0,
        0,
        0,
        2,
        0,
        0,
        0,
        0,
        0,
        0,
    )
}

#[test]
fn test_ansi2_encoding() -> Result<(), Error> {
    let reader = AdvReader::new(
        Source::File(get_example_path("ansi.txt")),
        None,                             // Buffer size. Default is 65536.
        None,                             // Trim. Default is false.
        None,                             // Line ending. Default is '\n'.
        Some(false),                      // Skip comments. Default is false.
        Some(true), // Convert comments to UTF8. Default is same as convert option.
        Some(true), // Convert Strings and (line) comments to UTF8. Default is false.
        Some("windows-1252".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
        Some("replace".to_string()),      // Allow invalid UTF8 characters. Default is false.
        Some(false),                      // Extended word separation. Default is false.
        Some(true),                       // Double double quote escaping. Default is false.
        Some(true), // Try to convert words into numbers (int, float). Default is false.
        Some(true), // Keep number base. Default is false.
        None,       // BOOL false
        None,       // BOOL true
        None,       // Callback function for block mode
    )?;
    check_results(
        parse_file(reader),
        (3, ReaderState::Default),
        None,
        0,
        0,
        0,
        2,
        0,
        0,
        0,
        0,
        0,
        0,
    )
}

#[test]
fn test_invalid_filename() {
    let reader = AdvReader::with_defaults(Source::File(PathBuf::from("../testdata/invalid.txt")));

    assert_eq!(reader.is_ok(), false);
}
