//
// Copyright (C) Bosch Engineering GmbH, 2023.
//
// All rights reserved, also regarding any disposal, exploitation,
// reproduction, editing, distribution, as well as in the event of
// applications for industrial property rights.
//

use std::{
    fs::File,
    io::{Error, Write},
    path::PathBuf,
    str,
    sync::mpsc::sync_channel,
    time::{Duration, Instant},
};

use advreader::{
    block::{A2LBlock, Block},
    AdvReader, AdvReturnValue,
};
use threadpool::ThreadPool;

mod common;

const MIN_BLOCK_SIZE: usize = 140;

fn get_example_path(s: &str) -> PathBuf {
    let example_path = PathBuf::from(format!("../testdata/{s}"));
    if example_path.exists() {
        return example_path;
    }
    PathBuf::from(format!("testdata/{s}"))
}

fn new_reader(
    block_size: usize,
    block_reader: Option<Box<dyn Block + Send + Sync>>,
) -> Result<AdvReader, Error> {
    #[allow(non_snake_case)]
    let FALSE = Some(b"False".to_vec());
    #[allow(non_snake_case)]
    let TRUE = Some(b"True".to_vec());
    AdvReader::new(
        &get_example_path("ASAP2_Demo_V171.a2l"),
        Some(block_size), // Buffer size. Default is 65536.
        None,             // Trim. Default is false.
        None,             // Line ending. Default is '\n'.
        Some(false),      // Skip comments. Default is false.
        Some(true),       // Convert comments to UTF8. Default is same as convert option.
        Some(true),       // Convert Strings and (line) comments to UTF8. Default is false.
        Some(true),       // Allow invalid UTF8 characters. Default is false.
        Some(false),      // Extended word separation. Default is false.
        Some(true),       // Double double quote escaping. Default is false.
        Some(true),       // Try to convert words into numbers (int, float). Default is false.
        Some(true),       // Keep number base. Default is false.
        FALSE,            // BOOL false
        TRUE,             // BOOL true
        block_reader,
    ) // Callback function for block mode
}

fn get_result(
    reader: Result<AdvReader, Error>,
) -> Result<Vec<Result<AdvReturnValue, Error>>, Error> {
    let reader = match reader {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    let mut results = Vec::new();
    let mut reader_iter = reader.into_iter();
    while let Some(result) = reader_iter.next() {
        results.push(result);
    }
    Ok(results)
}

fn truncate(result: &AdvReturnValue, max_chars: usize) -> String {
    let s = match result {
        AdvReturnValue::Bytes(v) => str::from_utf8(v).unwrap().to_string(),
        AdvReturnValue::Block(v) => str::from_utf8(v).unwrap().to_string(),
        AdvReturnValue::CommentUtf8(_) => "".to_string(),
        AdvReturnValue::LineCommentUtf8(_) => "".to_string(),
        _ => format!("{result:?}"),
    };
    if s.len() < max_chars {
        return s;
    }
    match s.char_indices().nth(max_chars) {
        None => "".to_string(),
        Some((idx, _)) => (&s[..idx]).to_string(),
    }
    /*match s.char_indices().nth(s.len() - max_chars) {
        None => "".to_string(),
        Some((idx, _)) => (&s[idx..]).to_string(),
    }*/
}

fn write_results(path: &str, results: &Vec<Result<AdvReturnValue, Error>>) -> Result<(), Error> {
    let mut file = File::create(path)?;
    file.write_all(
        results
            .iter()
            .map(|x| {
                format!(
                    "{:?}",
                    x.as_ref().map(|x| match x {
                        AdvReturnValue::Bytes(v) => format!("Bytes({:?})", str::from_utf8(&v)),
                        AdvReturnValue::Block(v) => format!("Block({:?})", str::from_utf8(&v)),
                        AdvReturnValue::CommentUtf8(_) => format!("CommentUtf8()"),
                        AdvReturnValue::LineCommentUtf8(_) => format!("LineCommentUtf8()"),
                        _ => format!("{x:?}"),
                    })
                )
            })
            .collect::<Vec<String>>()
            .join("\n")
            .as_bytes(),
    )?;
    Ok(())
}

#[ignore]
#[test]
fn test_blocksize() -> Result<(), Error> {
    let ref_results = get_result(new_reader(256 * 1024, None)).unwrap();
    let total = Instant::now();
    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = sync_channel(256);
    for block_size in MIN_BLOCK_SIZE..65536 {
        let tx = tx.clone();
        pool.execute(move || {
            let now = Instant::now();
            let results = get_result(new_reader(block_size, None));
            let dt = now.elapsed().as_millis();
            tx.send((block_size, dt, results))
                .expect("Failed to send results");
        });
    }
    for bs in MIN_BLOCK_SIZE..65536 {
        let block_size;
        let dt;
        let mut ok = true;
        match rx.recv_timeout(Duration::new(5, 0)) {
            Ok((bs, dt_bs, results)) => {
                block_size = bs;
                dt = dt_bs;
                match results {
                    Ok(results) => {
                        for (nr, (result, ref_result)) in
                            results.iter().zip(&ref_results).enumerate()
                        {
                            match result {
                                Ok(result) => match ref_result {
                                    Ok(ref_result) => {
                                        if result != ref_result {
                                            let res = truncate(&result, 65536);
                                            let ref_res = truncate(&ref_result, 65536);
                                            if res != ref_res {
                                                ok = false;
                                                write_results("results_ref.txt", &ref_results)?;
                                                write_results("results.txt", &results)?;
                                                eprintln!("ERROR: nr={nr} bs={bs}: Inconsistent result:\n{res:?}\n{ref_res:?}!");
                                                eprintln!("");
                                            }
                                            break;
                                        }
                                    }
                                    Err(ref_e) => {
                                        ok = false;
                                        eprintln!("ERROR: nr={nr} bs={bs}: Inconsistent error: {result:?} != {ref_e}!");
                                        break;
                                    }
                                },
                                Err(e) => match ref_result {
                                    Ok(ref_result) => {
                                        ok = false;
                                        eprintln!("ERROR: nr={nr} bs={bs}: Inconsistent result: {result:?} != {ref_result:?}!");
                                        break;
                                    }
                                    Err(ref_e) => {
                                        if e.kind() != ref_e.kind() {
                                            ok = false;
                                            eprintln!("ERROR: nr={nr} bs={bs}: Inconsistent error: {e} != {ref_e}!");
                                            break;
                                        }
                                    }
                                },
                            }
                        }
                    }
                    Err(e) => {
                        ok = false;
                        eprintln!("ERROR: bs={bs}: {e:?}");
                    }
                }
            }
            Err(e) => {
                eprintln!("ERROR: {e:?}");
                break;
            }
        };
        if ok {
            if bs % 50 == 0 {
                println!("OK bs={block_size} dt={dt}");
            }
        } else {
            println!("FAILED bs={block_size} dt={dt}");
        }
    }
    let dt = total.elapsed().as_millis();
    println!("TOTAL dt={}ms", dt,);
    Ok(())
}

#[ignore]
#[test]
fn test_blocksize_block() -> Result<(), Error> {
    let ref_results = get_result(new_reader(
        256 * 1024,
        Some(<A2LBlock as Block>::new(b'\n', false)),
    ))
    .unwrap();
    let total = Instant::now();
    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = sync_channel(256);
    for block_size in MIN_BLOCK_SIZE..65536 {
        let tx = tx.clone();
        pool.execute(move || {
            let now = Instant::now();
            let results = get_result(new_reader(
                block_size,
                Some(<A2LBlock as Block>::new(b'\n', false)),
            ));
            let dt = now.elapsed().as_millis();
            tx.send((block_size, dt, results))
                .expect("Failed to send results");
        });
    }
    for bs in MIN_BLOCK_SIZE..65536 {
        let block_size;
        let dt;
        let mut ok = true;
        match rx.recv_timeout(Duration::new(5, 0)) {
            Ok((bs, dt_bs, results)) => {
                block_size = bs;
                dt = dt_bs;
                match results {
                    Ok(results) => {
                        for (nr, (result, ref_result)) in
                            results.iter().zip(&ref_results).enumerate()
                        {
                            match result {
                                Ok(result) => match ref_result {
                                    Ok(ref_result) => {
                                        if result != ref_result {
                                            let res = truncate(&result, 65536);
                                            let ref_res = truncate(&ref_result, 65536);
                                            if res != ref_res {
                                                ok = false;
                                                write_results("results_ref.txt", &ref_results)?;
                                                write_results("results.txt", &results)?;
                                                eprintln!("ERROR: nr={nr} bs={bs}: Inconsistent result:\n{res:?}\n{ref_res:?}!");
                                                eprintln!("");
                                            }
                                            break;
                                        }
                                    }
                                    Err(ref_e) => {
                                        ok = false;
                                        eprintln!("ERROR: nr={nr} bs={bs}: Inconsistent error: {result:?} != {ref_e}!");
                                        break;
                                    }
                                },
                                Err(e) => match ref_result {
                                    Ok(ref_result) => {
                                        ok = false;
                                        eprintln!("ERROR: nr={nr} bs={bs}: Inconsistent result: {result:?} != {ref_result:?}!");
                                        break;
                                    }
                                    Err(ref_e) => {
                                        if e.kind() != ref_e.kind() {
                                            ok = false;
                                            eprintln!("ERROR: nr={nr} bs={bs}: Inconsistent error: {e} != {ref_e}!");
                                            break;
                                        }
                                    }
                                },
                            }
                        }
                    }
                    Err(e) => {
                        ok = false;
                        eprintln!("ERROR: bs={bs}: {e:?}");
                    }
                }
            }
            Err(e) => {
                eprintln!("ERROR: {e:?}");
                break;
            }
        };
        if ok {
            if bs % 50 == 0 {
                println!("OK bs={block_size} dt={dt}");
            }
        } else {
            println!("FAILED bs={block_size} dt={dt}");
        }
    }
    let dt = total.elapsed().as_millis();
    println!("TOTAL dt={}ms", dt,);
    Ok(())
}
