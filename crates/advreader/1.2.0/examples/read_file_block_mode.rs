use std::cell::RefCell;
use std::env;
use std::path::PathBuf;
use std::time::Instant;

use advreader::*;

fn fn_block_mode(item: Option<&[u8]>) -> (bool, Option<Vec<Vec<u8>>>) {
    thread_local! {
        static BEGIN: RefCell<bool> = RefCell::new(false);
        static END: RefCell<bool> = RefCell::new(false);
        static STATE: RefCell<u8> = RefCell::new(0);
    }
    if item.is_none() {
        // Initialize static variables.
        STATE.with(|s| {
            *s.borrow_mut() = 0;
        });
        BEGIN.with(|b| {
            *b.borrow_mut() = false;
        });
        END.with(|e| {
            *e.borrow_mut() = false;
        });
        return (false, None);
    }
    let item = item.unwrap();
    let mut cur_state = 0;
    let mut send: Option<Vec<Vec<u8>>> = None;
    STATE.with(|s| {
        cur_state = *s.borrow_mut();
        BEGIN.with(|b| {
            END.with(|e| {
                if item == b"/begin" {
                    *b.borrow_mut() = true;
                    *e.borrow_mut() = false;
                } else if item.ends_with(b"/end") {
                    *b.borrow_mut() = false;
                    *e.borrow_mut() = true;
                } else if *b.borrow() {
                    if cur_state < 0xff {
                        if item == b"IF_DATA" {
                            *s.borrow_mut() = cur_state + 1;
                        } else if item == b"A2ML" {
                            cur_state += 2;
                            *s.borrow_mut() = cur_state;
                            send = Some(vec![item.to_vec()]);
                        }
                    }
                    *b.borrow_mut() = false;
                } else if *e.borrow() {
                    if (item == b"IF_DATA" || item == b"A2ML") && cur_state > 0 {
                        if cur_state > 1 {
                            cur_state -= 2;
                        } else {
                            cur_state = 0;
                        }
                        *s.borrow_mut() = cur_state;
                        send = Some(vec![b"/end".to_vec(), item.to_vec()]);
                    }
                    *e.borrow_mut() = false;
                } else if cur_state & 1 != 0 {
                    cur_state += 1;
                    *s.borrow_mut() = cur_state;
                    send = Some(vec![item.to_vec()]);
                }
                println!(
                    "CB {} {:?} {:?}",
                    cur_state,
                    std::str::from_utf8(item),
                    send
                );
            });
        });
    });
    (cur_state > 1, send)
}

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
        Some(Box::new(fn_block_mode)),
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
                        println!("BYTES {:?}", std::str::from_utf8(&v));
                        bytes.push(v);
                    }
                    AdvReturnValue::String(v) => strings.push(v),
                    AdvReturnValue::Comment(v) => comments.push(v),
                    AdvReturnValue::LineComment(v) => line_comments.push(v),
                    AdvReturnValue::StringUtf8(v) => strings_utf8.push(v),
                    AdvReturnValue::CommentUtf8(v) => comments_utf8.push(v),
                    AdvReturnValue::LineCommentUtf8(v) => line_comments_utf8.push(v),
                    AdvReturnValue::Bool(v) => bools.push(v),
                    AdvReturnValue::Int(v) => {
                        println!("INT {}", v);
                        ints.push(v);
                    }
                    AdvReturnValue::Float(v) => floats.push(v),
                    AdvReturnValue::Hex(v) => hexs.push(v),
                    AdvReturnValue::Oct(v) => octs.push(v),
                    AdvReturnValue::Bin(v) => bins.push(v),
                    AdvReturnValue::Block(v) => {
                        println!(
                            "BLOCK {:?}...{:?} @ {}",
                            std::str::from_utf8(&v[..100]),
                            std::str::from_utf8(&v[v.len() - 100..]),
                            reader_ok.line_nr()
                        );
                        blocks.push(v);
                    }
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
