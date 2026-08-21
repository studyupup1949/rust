use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::str;
use std::time::Instant;
use std::{env, fs};

use advreader::block::{A2LBlock, Block};
use advreader::{AdvReader, AdvReturnValue};

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let dir: &str = args.get(1).map(|x| x.as_str()).unwrap_or("testdata");
    let a2l_ext = OsStr::new("a2l");
    let paths: Vec<PathBuf> = fs::read_dir(dir)?
        .into_iter()
        .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
        .map(|r| r.unwrap().path()) // This is safe, since we only have the Ok variants
        .filter(|r| r.is_file()) // Filter out folders
        .filter(|d| d.extension() == Some(a2l_ext))
        .collect();
    let verbose = false;
    #[allow(non_snake_case)]
    let FALSE = Some(b"False".to_vec());
    #[allow(non_snake_case)]
    let TRUE = Some(b"True".to_vec());
    let now_all = Instant::now();
    let cnt = paths.len();
    println!("Parse {cnt} A2L-files...");
    for (nr, path) in paths.iter().enumerate() {
        println!("Parsing {nr}/{cnt}: {path:?}...");
        let now = Instant::now();
        let reader = AdvReader::new(
            &path,
            None,          // Buffer size. Default is 65536.
            None,          // Trim. Default is false.
            None,          // Line ending. Default is '\n'.
            Some(true),    // Skip comments. Default is false.
            Some(true),    // Convert comments to UTF8. Default is same as convert option.
            Some(true),    // Convert Strings and (line) comments to UTF8. Default is false.
            Some(true),    // Allow invalid UTF8 characters. Default is false.
            Some(false),   // Extended word separation. Default is false.
            Some(true),    // Double double quote escaping. Default is false.
            Some(true),    // Try to convert words into numbers (int, float). Default is false.
            Some(false),   // Keep number base. Default is false.
            FALSE.clone(), // BOOL false
            TRUE.clone(),  // BOOL true
            Some(<A2LBlock as Block>::new(b'\n', true)),
        )?; // Callback function for block mode

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
                    eprintln!("ERROR ({}): {e}", reader_iter.line_nr());
                    break;
                }
            }
        }

        let last_bytes;
        println!("dt={}ms", now.elapsed().as_millis());
        println!("state={:?}", reader_iter.stop());
        println!("lines={line_nr}");
        if bytes.is_empty() {
            println!("bytes={}", bytes.len());
            last_bytes = Vec::new();
        } else {
            last_bytes = bytes.get(bytes.len() - 1).unwrap().to_vec();
            println!(
                "bytes={} LAST({})={:?}",
                bytes.len(),
                last_bytes.len(),
                str::from_utf8(&last_bytes)
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
        println!("blocks={}\n", blocks.len());
        if last_bytes != b"PROJECT" {
            return Err(Error::new(ErrorKind::InvalidData, "FAILED"));
        }
    }
    let dt = now_all.elapsed().as_millis();
    println!(
        "dtAll ({cnt} files)={}ms  {:.2} files/s",
        dt,
        1000.0 * cnt as f64 / dt as f64
    );
    Ok(())
}
