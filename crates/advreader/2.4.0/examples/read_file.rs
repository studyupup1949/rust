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
        .unwrap_or("testdata/big.txt");
    println!("Read file {path}");
    let verbose = false;
    let now = Instant::now();
    #[allow(non_snake_case)]
    let FALSE = Some(b"False".to_vec());
    #[allow(non_snake_case)]
    let TRUE = Some(b"True".to_vec());
    let reader = AdvReader::new(
        &Path::new(path),
        Some(65536),                      // Buffer size. Default is 65536.
        None,                             // Trim. Default is false.
        None,                             // Line ending. Default is '\n'.
        Some(false),                      // Skip comments. Default is false.
        Some(true), // Convert comments to UTF8. Default is same as convert option.
        Some(true), // Convert Strings and (line) comments to UTF8. Default is false.
        Some("windows-1252".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
        Some("replace".to_string()),      // Allow invalid UTF8 characters. Default is false.
        Some(false),                      // Extended word separation. Default is false.
        Some(true),                       // Double double quote escaping. Default is false.
        Some(true),  // Try to convert words into numbers (int, float). Default is false.
        Some(false), // Keep number base. Default is false.
        FALSE,       // BOOL false
        TRUE,        // BOOL true
        None,
    ); // Callback function for block mode

    let reader = match reader {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    let mut bytes = Vec::new();
    let mut strings = Vec::new();
    let mut comments = Vec::new();
    let mut line_comments = Vec::new();
    let mut strings_utf8 = Vec::new();
    let mut comments_utf8 = Vec::new();
    let mut line_comments_utf8 = Vec::new();
    let mut bools = Vec::new();
    let mut ints = Vec::new();
    let mut hexs = Vec::new();
    let mut octs = Vec::new();
    let mut bins = Vec::new();
    let mut floats = Vec::new();
    let mut blocks = Vec::new();
    let mut reader_iter = reader.into_iter();
    let mut line_nr = 0;

    while let Some(result) = reader_iter.next() {
        line_nr = reader_iter.line_nr();
        match result {
            Ok(r) => {
                match r {
                    AdvReturnValue::Bytes(v) => {
                        if verbose {
                            println!("#Bytes {line_nr}: {:?}", std::str::from_utf8(&v));
                        }
                        bytes.push(v);
                    }
                    AdvReturnValue::String(v) => {
                        if verbose {
                            println!("#String {line_nr}: {:?}", std::str::from_utf8(&v));
                        }
                        strings.push(v);
                    }
                    AdvReturnValue::Comment(v) => {
                        if verbose {
                            println!("#Comment {line_nr}: {:?}", std::str::from_utf8(&v));
                        }
                        comments.push(v);
                    }
                    AdvReturnValue::LineComment(v) => {
                        if verbose {
                            println!("#LineComment {line_nr}: {:?}", std::str::from_utf8(&v));
                        }
                        line_comments.push(v);
                    }
                    AdvReturnValue::StringUtf8(v) => {
                        if verbose {
                            println!("#StringUtf8 {line_nr}: {v}");
                        }
                        strings_utf8.push(v);
                    }
                    AdvReturnValue::CommentUtf8(v) => {
                        if verbose {
                            println!("#CommentUtf8 {line_nr}: {v}");
                        }
                        comments_utf8.push(v);
                    }
                    AdvReturnValue::LineCommentUtf8(v) => {
                        if verbose {
                            println!("#LineCommentUtf8 {line_nr}: {v}");
                        }
                        line_comments_utf8.push(v);
                    }
                    AdvReturnValue::Bool(v) => {
                        if verbose {
                            println!("#Bool {line_nr}: {v}");
                        }
                        bools.push(v);
                    }
                    AdvReturnValue::Int(v) => {
                        if verbose {
                            println!("#Int {line_nr}: {v}");
                        }
                        ints.push(v);
                    }
                    AdvReturnValue::Float(v) => {
                        if verbose {
                            println!("#Float {line_nr}: {v}");
                        }
                        floats.push(v);
                    }
                    AdvReturnValue::Hex(v) => {
                        if verbose {
                            println!("#Hex {line_nr}: {v}");
                        }
                        hexs.push(v);
                    }
                    AdvReturnValue::Oct(v) => {
                        if verbose {
                            println!("#Oct {line_nr}: {v}");
                        }
                        octs.push(v);
                    }
                    AdvReturnValue::Bin(v) => {
                        if verbose {
                            println!("#Bin {line_nr}: {v}");
                        }
                        bins.push(v);
                    }
                    AdvReturnValue::Block(v) => {
                        if verbose {
                            println!("#Block {line_nr}: {:?}", std::str::from_utf8(&v));
                        }
                        blocks.push(v);
                    }
                };
            }
            Err(e) => {
                eprintln!("ERROR {line_nr}: {e}");
                break;
            }
        }
    }
    println!("dt={}ms", now.elapsed().as_millis());
    println!("state={:?}", reader_iter.stop());
    println!("lines={line_nr}");
    if bytes.is_empty() {
        println!("bytes={}", bytes.len());
    } else {
        let last_bytes = bytes.get(bytes.len() - 1).unwrap();
        println!(
            "bytes={} LAST({})={:?}",
            bytes.len(),
            last_bytes.len(),
            str::from_utf8(last_bytes)
        );
    }
    println!("strings={}", strings.len());
    println!("comments={}", comments.len());
    println!("line_comments={}", line_comments.len());
    println!("strings_utf8={}", strings_utf8.len());
    println!("comments_utf8={}", comments_utf8.len());
    println!("line_comments_utf8={}", line_comments_utf8.len());
    println!("bools={}", bools.len());
    println!("ints={}", ints.len());
    println!("floats={}", floats.len());
    println!("hexs={}", hexs.len());
    println!("octs={}", octs.len());
    println!("bins={}", bins.len());
    println!("blocks={}", blocks.len());
}
