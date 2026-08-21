//
// Copyright 2016 The IHEX Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>.
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms.
//

use std::io::Error;

use advreader::{
    block::{A2LBlock, Block},
    AdvReader, ReaderState,
};
use common::{get_example_path, parse_file, Results};

mod common;

#[allow(non_snake_case)]
fn check_ASAP2_Demo_V171(
    results: Results,
    bytes_cnt: usize,
    strings_utf8_cnt: usize,
    comments_utf8_cnt: usize,
    ints_cnt: usize,
    hexs_cnt: usize,
    floats_cnt: usize,
    blocks_cnt: usize,
) -> Result<(), Error> {
    assert_eq!(
        results.state.unwrap().ok(),
        Some((5978, ReaderState::Default))
    );
    assert_eq!(results.last_bytes, Some(b"PROJECT".to_vec()));
    assert_eq!(results.bytes.len(), bytes_cnt);
    assert_eq!(results.strings.len(), 0);
    assert_eq!(results.comments.len(), 0);
    assert_eq!(results.strings_utf8.len(), strings_utf8_cnt);
    assert_eq!(results.comments_utf8.len(), comments_utf8_cnt);
    assert_eq!(results.line_comments_utf8.len(), 83);
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
fn test_a2l() -> Result<(), Error> {
    let reader = AdvReader::new(
        &get_example_path("ASAP2_Demo_V171.a2l"),
        None,                     // Buffer size. Default is 65536.
        None,                     // Trim. Default is false.
        None,                     // Line ending. Default is '\n'.
        Some(false),              // Skip comments. Default is false.
        Some(true),               // Convert comments to UTF8. Default is true.
        Some(true),               // Convert strings to UTF8. Default is true.
        Some("ansi".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
        Some(true),               // Allow invalid UTF8 characters. Default is false.
        Some(false),              // Extended word separation. Default is false.
        Some(true),               // Double double quote escaping. Default is false.
        Some(false), // Try to convert words into numbers (int, float). Default is false.
        Some(false), // Keep number base. Default is false.
        None,        // BOOL false
        None,        // BOOL true
        None,        // Callback function for block mode
    )?;

    check_ASAP2_Demo_V171(parse_file(reader), 8997, 1688, 682, 0, 0, 0, 0)
}

#[test]
fn test_a2l_convert2numbers() -> Result<(), Error> {
    let reader = AdvReader::new(
        &get_example_path("ASAP2_Demo_V171.a2l"),
        None,                     // Buffer size. Default is 65536.
        None,                     // Trim. Default is false.
        None,                     // Line ending. Default is '\n'.
        Some(false),              // Skip comments. Default is false.
        Some(true),               // Convert comments to UTF8. Default is true.
        Some(true),               // Convert strings to UTF8. Default is true.
        Some("ansi".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
        Some(true),               // Allow invalid UTF8 characters. Default is false.
        Some(false),              // Extended word separation. Default is false.
        Some(true),               // Double double quote escaping. Default is false.
        Some(true),  // Try to convert words into numbers (int, float). Default is false.
        Some(false), // Keep number base. Default is false.
        None,        // BOOL false
        None,        // BOOL true
        None,        // Callback function for block mode
    )?;

    check_ASAP2_Demo_V171(parse_file(reader), 7417, 1688, 682, 2271, 0, 27, 0)
}

#[test]
fn test_a2l_convert2numbers_keep_base() -> Result<(), Error> {
    let reader = AdvReader::new(
        &get_example_path("ASAP2_Demo_V171.a2l"),
        None,                     // Buffer size. Default is 65536.
        None,                     // Trim. Default is false.
        None,                     // Line ending. Default is '\n'.
        Some(false),              // Skip comments. Default is false.
        Some(true),               // Convert comments to UTF8. Default is true.
        Some(true),               // Convert strings to UTF8. Default is true.
        Some("ansi".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
        Some(true),               // Allow invalid UTF8 characters. Default is false.
        Some(false),              // Extended word separation. Default is false.
        Some(true),               // Double double quote escaping. Default is false.
        Some(true), // Try to convert words into numbers (int, float). Default is false.
        Some(true), // Keep number base. Default is false.
        None,       // BOOL false
        None,       // BOOL true
        None,       // Callback function for block mode
    )?;

    check_ASAP2_Demo_V171(parse_file(reader), 7417, 1688, 682, 1999, 272, 27, 0)
}

#[test]
fn test_a2l_convert2numbers_keep_base_bool() -> Result<(), Error> {
    #[allow(non_snake_case)]
    let FALSE = Some(b"False".to_vec());
    #[allow(non_snake_case)]
    let TRUE = Some(b"True".to_vec());
    let reader = AdvReader::new(
        &get_example_path("ASAP2_Demo_V171.a2l"),
        None,                     // Buffer size. Default is 65536.
        None,                     // Trim. Default is false.
        None,                     // Line ending. Default is '\n'.
        Some(false),              // Skip comments. Default is false.
        Some(true),               // Convert comments to UTF8. Default is true.
        Some(true),               // Convert strings to UTF8. Default is true.
        Some("ansi".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
        Some(true),               // Allow invalid UTF8 characters. Default is false.
        Some(false),              // Extended word separation. Default is false.
        Some(true),               // Double double quote escaping. Default is false.
        Some(true), // Try to convert words into numbers (int, float). Default is false.
        Some(true), // Keep number base. Default is false.
        FALSE,      // BOOL false
        TRUE,       // BOOL true
        None,       // Callback function for block mode
    )?;

    check_ASAP2_Demo_V171(parse_file(reader), 7417, 1688, 682, 1999, 272, 27, 0)
}

#[test]
fn test_a2l_convert2numbers_keep_base_bool_block_mode() -> Result<(), Error> {
    #[allow(non_snake_case)]
    let FALSE = Some(b"False".to_vec());
    #[allow(non_snake_case)]
    let TRUE = Some(b"True".to_vec());
    let reader = AdvReader::new(
        &get_example_path("ASAP2_Demo_V171.a2l"),
        Some(26400000),           // Buffer size. Default is 65536.
        None,                     // Trim. Default is false.
        None,                     // Line ending. Default is '\n'.
        Some(false),              // Skip comments. Default is false.
        Some(true),               // Convert comments to UTF8. Default is true.
        Some(true),               // Convert strings to UTF8. Default is true.
        Some("ansi".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
        Some(true),               // Allow invalid UTF8 characters. Default is false.
        Some(false),              // Extended word separation. Default is false.
        Some(true),               // Double double quote escaping. Default is false.
        Some(true), // Try to convert words into numbers (int, float). Default is false.
        Some(true), // Keep number base. Default is false.
        FALSE,      // BOOL false
        TRUE,       // BOOL true
        Some(<A2LBlock as Block>::new(b'\n', false)),
    )?;

    check_ASAP2_Demo_V171(parse_file(reader), 3299, 419, 389, 1002, 140, 27, 34)
}
