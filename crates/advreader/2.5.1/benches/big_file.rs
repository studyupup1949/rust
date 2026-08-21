#![feature(test)]

extern crate test;

use std::{path::Path, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};

use advreader::{AdvReader, AdvReturnValue, Source};

fn read_file(
    path: &Path,
    encode_comments: Option<bool>,
    encode_strings: Option<bool>,
    encoding: Option<String>,
    encoding_errors: Option<String>,
) {
    #[allow(non_snake_case)]
    let FALSE = None; //Some(b"False".to_vec());
    #[allow(non_snake_case)]
    let TRUE = None; //Some(b"True".to_vec());
    let reader = AdvReader::new(
        Source::File(path.to_path_buf()),
        None,
        None,            // Trim. Default is false.
        None,            // Line ending. Default is '\n'.
        Some(false),     // Skip comments. Default is false.
        encode_comments, // Convert comments to UTF8. Default is same as convert option.
        encode_strings,  // Convert Strings and (line) comments to UTF8. Default is false.
        encoding,
        encoding_errors, // Allow invalid UTF8 characters. Default is false.
        Some(false),     // Extended word separation. Default is false.
        Some(true),      // Double double quote escaping. Default is false.
        Some(true),      // Try to convert words into numbers (int, float). Default is false.
        Some(true),      // Keep number base. Default is false.
        FALSE,           // BOOL false
        TRUE,            // BOOL true
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

    while let Some(result) = reader_iter.next() {
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
                eprintln!("ERROR ({}): {e}", reader_iter.line_nr());
                break;
            }
        }
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("advreader");
    group.sample_size(10);
    group.measurement_time(Duration::new(10, 0));
    group.bench_function("big_file no encoding, errors replace", |b| {
        b.iter(|| read_file(Path::new("../testdata/big.txt"), None, None, None, None))
    });
    group.bench_function("big_file no encoding, strict", |b| {
        b.iter(|| {
            read_file(
                Path::new("../testdata/big.txt"),
                Some(true),
                Some(true),
                None,
                Some("strict".to_string()), // Allow invalid UTF8 characters. Default is false.
            )
        })
    });
    group.bench_function("big_file windows-1252 encoding, errors replace", |b| {
        b.iter(|| {
            read_file(
                Path::new("../testdata/big.txt"),
                Some(true),
                Some(true),
                Some("windows-1252".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
                Some("replace".to_string()), // Allow invalid UTF8 characters. Default is false.
            )
        })
    });
    group.bench_function("big_file utf-8 encoding, errors ignore", |b| {
        b.iter(|| {
            read_file(
                Path::new("../testdata/big.txt"),
                Some(true),
                Some(true),
                Some("utf-8".to_string()), // Convert Strings and (line) comments to UTF8. Default is no encoder.
                Some("ignore".to_string()), // Allow invalid UTF8 characters. Default is false.
            )
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
