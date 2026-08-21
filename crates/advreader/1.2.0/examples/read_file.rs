use std::env;
use std::path::PathBuf;
use std::time::Instant;

use advreader::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    //println!("{:?}", env::current_dir());
    //let args: Vec<String> = vec!["".to_owned(), "../res/example.txt".to_owned()];
    let now = Instant::now();
    #[allow(non_snake_case)]
    let FALSE = None; //Some(b"False".to_vec());
    #[allow(non_snake_case)]
    let TRUE = None; //Some(b"True".to_vec());
    let reader = AdvReader::new(
        &PathBuf::from(&args[1]),
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

    let mut reader_ok = match reader {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
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

    while let Some(result) = reader_ok.next() {
        match result {
            Ok(r) => {
                match r {
                    AdvReturnValue::Bytes(v) => {
                        //println!("{:?}", std::str::from_utf8(&v));
                        bytes.push(v);
                    }
                    AdvReturnValue::String(v) => strings.push(v),
                    AdvReturnValue::Comment(v) => comments.push(v),
                    AdvReturnValue::LineComment(v) => line_comments.push(v),
                    AdvReturnValue::StringUtf8(v) => strings_utf8.push(v),
                    AdvReturnValue::CommentUtf8(v) => comments_utf8.push(v),
                    AdvReturnValue::LineCommentUtf8(v) => line_comments_utf8.push(v),
                    AdvReturnValue::Bool(v) => bools.push(v),
                    AdvReturnValue::Int(v) => ints.push(v),
                    AdvReturnValue::Float(v) => floats.push(v),
                    AdvReturnValue::Hex(v) => hexs.push(v),
                    AdvReturnValue::Oct(v) => octs.push(v),
                    AdvReturnValue::Bin(v) => bins.push(v),
                    AdvReturnValue::Block(v) => blocks.push(v),
                };
            }
            Err(e) => {
                eprintln!("ERROR ({}): {}", reader_ok.line_nr(), e);
                break;
            }
        }
    }
    println!("dt={}ms", now.elapsed().as_millis());
    println!("bytes={}", bytes.len());
    println!("strings={}", strings.len());
    println!("comments={}", comments.len());
    println!("line_comments={}", line_comments.len());
    println!("strings_utf8={}", strings_utf8.len());
    println!("comments_utf8={}", comments_utf8.len());
    println!("line_comments_utf8={}", line_comments_utf8.len());
    println!("bools={}", bools.len());
    println!("ints={} {:?}", ints.len(), "int");
    println!("floats={} {:?}", floats.len(), "floats");
    println!("hexs={}", hexs.len());
    println!("octs={}", octs.len());
    println!("bins={}", bins.len());
    println!("blocks={}", blocks.len());
}
