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
    // Check strings
    assert_eq!(results.strings_utf8.get(0), Some(&r#""""#.to_string()));
    assert_eq!(results.strings_utf8.get(1), Some(&r#""A""#.to_string()));
    assert_eq!(results.strings_utf8.get(2), Some(&r#"" B ""#.to_string()));
    assert_eq!(results.strings_utf8.get(3), Some(&r#""\"""#.to_string()));
    assert_eq!(results.strings_utf8.get(4), Some(&r#""""A""""#.to_string()));
    assert_eq!(
        results.strings_utf8.get(5),
        Some(&r#"" ABC "" bla bla "" Test ""#.to_string())
    );
    assert_eq!(
        results.strings_utf8.get(6),
        Some(&r#"" abc \" bla BLA \" TEST ""#.to_string())
    );
    assert_eq!(
        results.strings_utf8.get(7),
        Some(&r#""\"\n\t""#.to_string())
    );
    assert_eq!(results.strings_utf8.get(8), Some(&r#""C\´C""#.to_string()));
    assert_eq!(results.strings_utf8.get(9), Some(&r#""\"\"""#.to_string()));
    assert_eq!(results.strings_utf8.get(10), Some(&r#""""""#.to_string()));
    assert_eq!(
        results.strings_utf8.get(11),
        Some(&r#"" "" Tst "" ""#.to_string())
    );
    assert_eq!(
        results.strings_utf8.get(12),
        Some(&r#"""" Tst "" ""#.to_string())
    );
    assert_eq!(
        results.strings_utf8.get(13),
        Some(&r#"" "" Tst """"#.to_string())
    );
    assert_eq!(
        results.strings_utf8.get(14),
        Some(&r#"""" Tst """"#.to_string())
    );
    assert_eq!(results.strings_utf8.get(15), Some(&r#""123""#.to_string()));
    assert_eq!(results.strings_utf8.get(16), None);
    Ok(())
}

#[test]
fn test_strings() -> Result<(), Error> {
    let reader = AdvReader::new(
        Source::File(get_example_path("strings.txt")),
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
        (34, ReaderState::Default),
        Some(b"Name17".to_vec()),
        17,
        0,
        0,
        16,
        0,
        0,
        0,
        0,
        0,
        0,
    )
}
