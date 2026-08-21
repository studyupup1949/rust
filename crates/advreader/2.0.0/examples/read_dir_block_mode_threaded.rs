use std::ffi::OsStr;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::str;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};
use std::{env, fs};

use advreader::block::{A2LBlock, Block};
use advreader::{AdvReader, AdvReturnValue, ReaderState};
use threadpool::ThreadPool;

struct Statistics {
    path: PathBuf,
    bytes: usize,
    strings: usize,
    comments: usize,
    line_comments: usize,
    strings_utf8: usize,
    comments_utf8: usize,
    line_comments_utf8: usize,
    bools: usize,
    ints: usize,
    floats: usize,
    hexs: usize,
    octs: usize,
    bins: usize,
    blocks: usize,
    //
    last_bytes: Vec<u8>,
    state: Result<(usize, ReaderState), Error>,
    lines: usize,
    errors: Vec<String>,
    dt: f64,
}

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
    let total = Instant::now();
    let cnt = paths.len();
    println!("Parse {cnt} A2L-files...");
    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = channel();
    for path in paths.into_iter() {
        let tx = tx.clone();
        pool.execute(move || {
            let now = Instant::now();
            #[allow(non_snake_case)]
            let FALSE = Some(b"False".to_vec());
            #[allow(non_snake_case)]
            let TRUE = Some(b"True".to_vec());
            let reader = AdvReader::new(
                &path,
                None,        // Buffer size. Default is 65536.
                None,        // Trim. Default is false.
                None,        // Line ending. Default is '\n'.
                Some(true),  // Skip comments. Default is false.
                Some(true),  // Convert comments to UTF8. Default is same as convert option.
                Some(true),  // Convert Strings and (line) comments to UTF8. Default is false.
                Some(true),  // Allow invalid UTF8 characters. Default is false.
                Some(false), // Extended word separation. Default is false.
                Some(true),  // Double double quote escaping. Default is false.
                Some(true),  // Try to convert words into numbers (int, float). Default is false.
                Some(false), // Keep number base. Default is false.
                FALSE,       // BOOL false
                TRUE,        // BOOL true
                Some(<A2LBlock as Block>::new(b'\n', true)),
            )
            .unwrap(); // Callback function for block mode

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
            let mut errors = Vec::new();
            let mut reader_iter = reader.into_iter();
            let mut line_nr = 0;

            while let Some(result) = reader_iter.next() {
                line_nr = reader_iter.line_nr();
                match result {
                    Ok(r) => {
                        match r {
                            AdvReturnValue::Bytes(v) => bytes.push(v),
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
                        errors.push(format!("ERROR ({}): {e}", reader_iter.line_nr()));
                        break;
                    }
                }
            }
            let last_bytes;
            if bytes.is_empty() {
                last_bytes = Vec::new();
            } else {
                last_bytes = bytes.get(bytes.len() - 1).unwrap().to_vec();
            }
            tx.send(Statistics {
                path: path.clone(),
                bytes: bytes.len(),
                strings: strings.len(),
                comments: comments.len(),
                line_comments: line_comments.len(),
                strings_utf8: strings_utf8.len(),
                comments_utf8: comments_utf8.len(),
                line_comments_utf8: line_comments_utf8.len(),
                bools: bools.len(),
                ints: ints.len(),
                floats: floats.len(),
                hexs: hexs.len(),
                octs: octs.len(),
                bins: bins.len(),
                blocks: blocks.len(),
                last_bytes: last_bytes,
                state: reader_iter.stop(),
                lines: line_nr,
                errors,
                dt: now.elapsed().as_millis() as f64 / 1000.0,
            })
            .expect("Failed to send results");
        });
    }
    // Collect results
    for nr in 0..cnt {
        let statistics = match rx.recv_timeout(Duration::new(5, 0)) {
            Ok(r) => r,
            Err(e) => {
                println!("ERROR: {e:?}");
                break;
            }
        };
        println!("{}/{}: PATH={:?}s", nr + 1, cnt, statistics.path);
        println!("dt={:.3}s", statistics.dt);
        println!("state={:?}", statistics.state);
        println!("lines={}", statistics.lines);
        if statistics.bytes == 0 {
            println!("bytes=0");
        } else {
            println!(
                "bytes={} LAST({})={:?}",
                statistics.bytes,
                statistics.last_bytes.len(),
                str::from_utf8(&statistics.last_bytes)
            );
        }
        println!("strings={}", statistics.strings);
        println!("comments={}", statistics.comments);
        println!("line_comments={}", statistics.line_comments);
        println!("strings_utf8={}", statistics.strings_utf8);
        println!("comments_utf8={}", statistics.comments_utf8);
        println!("line_comments_utf8={}", statistics.line_comments_utf8);
        println!("bools={}", statistics.bools);
        println!("ints={}", statistics.ints);
        println!("floats={}", statistics.floats);
        println!("hexs={}", statistics.hexs);
        println!("octs={}", statistics.octs);
        println!("bins={}", statistics.bins);
        println!("blocks={}\n", statistics.blocks);
        for error in statistics.errors {
            println!("ERROR: {error}");
        }
        if statistics.last_bytes != b"PROJECT" {
            return Err(Error::new(ErrorKind::InvalidData, "FAILED"));
        }
    }
    let dt = total.elapsed().as_millis() as f64 / 1000.0;
    println!(
        "dtTOTAL ({cnt} files)={dt}ms  {:.2} files/s",
        cnt as f64 / dt
    );
    Ok(())
}
