use std::env;
use std::path::Path;
use std::str;
use std::time::Instant;

use advreader::{AdvReader, AdvReturnValue};

fn main() {
    let args: Vec<String> = env::args().collect();
    //println!("{:?}", env::current_dir());
    let path: &str = args
        .get(1)
        .map(|x| x.as_str())
        .unwrap_or("testdata/small.txt");
    let now = Instant::now();
    #[allow(non_snake_case)]
    let FALSE = None; //Some(b"False".to_vec());
    #[allow(non_snake_case)]
    let TRUE = None; //Some(b"True".to_vec());
    let reader = AdvReader::new(
        &Path::new(path),
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
        None,
    ); // Callback function for block mode

    let reader = match reader {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let mut reader_iter = reader.into_iter();

    while let Some(result) = reader_iter.next_line() {
        match result {
            Ok((line_nr, line)) => {
                println!(
                    "\n{line_nr}: {}: {:?}",
                    line.len(),
                    line[..std::cmp::min(line.len(), 10)]
                        .iter()
                        .map(|x| match x {
                            AdvReturnValue::Bytes(v) => str::from_utf8(v).unwrap().to_string(),
                            _ => format!("{x:?}"),
                        })
                        .collect::<Vec<String>>()
                );
            }
            Err(e) => {
                println!("ERR: {e:?}");
            }
        }
    }
    println!("dt={}ms", now.elapsed().as_millis());
}
