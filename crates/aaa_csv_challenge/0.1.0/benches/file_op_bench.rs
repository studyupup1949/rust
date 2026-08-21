//! 基准性能测试

#![feature(test)]
extern crate test;

use std::path::PathBuf;
use test::Bencher;

use aaa_csv_challenge::{
    replace_column, Opt, {load_csv, write_csv},
};
//or: use aaa_csv_challenge::load_csv;

#[bench]
fn bench_read_100times(b: &mut Bencher) {
    b.iter(|| {
        let n = test::black_box(100);
        (0..n).fold(0, |_, _| {
            test_load_csv();
            0
        });
    })
}

fn test_load_csv() {
    let filename = PathBuf::from("./input/challenge.csv");
    load_csv(filename);
}
