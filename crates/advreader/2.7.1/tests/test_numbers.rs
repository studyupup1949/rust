//
// Copyright 2016 The IHEX Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms.
//

use std::io::Error;

use advreader::{AdvReader, ReaderState, Source};

mod common;
use common::{Results, get_example_path, parse_file};

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
    bools_cnt: usize,
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
    assert_eq!(results.bools.len(), bools_cnt);
    assert_eq!(results.ints.len(), ints_cnt);
    assert_eq!(results.hexs.len(), hexs_cnt);
    assert_eq!(results.octs.len(), 0);
    assert_eq!(results.bins.len(), 0);
    assert_eq!(results.floats.len(), floats_cnt);
    assert_eq!(results.blocks.len(), blocks_cnt);
    // Check values
    if bools_cnt > 0 {
        assert_eq!(results.bools.get(0), Some(&false));
        assert_eq!(results.bools.get(1), Some(&true));
    }
    assert_eq!(results.ints.get(0), Some(&1234));
    assert_eq!(results.ints.get(1), Some(&1234));
    assert_eq!(results.ints.get(2), Some(&-1234));
    assert_eq!(results.hexs.get(0), Some(&305419896));
    assert_eq!(results.floats.get(0), Some(&200.0));
    assert_eq!(results.floats.get(1), Some(&2000000000.0));
    assert_eq!(results.floats.get(2), Some(&2e23));
    assert_eq!(results.floats.get(3), Some(&1.9999999999999996e38));
    assert_eq!(results.floats.get(4), Some(&1.9999999999999996e38));
    assert_eq!(results.floats.get(5), Some(&-1.0000000000000001e37));
    assert_eq!(results.floats.get(6), Some(&1.0000000000000001e37));
    assert_eq!(results.floats.get(7), Some(&56.789));
    assert_eq!(results.floats.get(8), Some(&0.00314));
    assert_eq!(results.floats.get(9), Some(&586.3999999999999));
    assert_eq!(results.floats.get(10), None);
    Ok(())
}

#[test]
fn test_convert_numbers() -> Result<(), Error> {
    #[allow(non_snake_case)]
    let FALSE = None;
    #[allow(non_snake_case)]
    let TRUE = None;
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
        0,
        3,
        1,
        10,
        0,
    )
}

#[test]
fn test_convert_numbers_bools() -> Result<(), Error> {
    #[allow(non_snake_case)]
    let FALSE = Some(b"False".to_vec());
    #[allow(non_snake_case)]
    let TRUE = Some(b"True".to_vec());
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
        52,
        0,
        0,
        11,
        6,
        6,
        2,
        3,
        1,
        10,
        0,
    )
}
