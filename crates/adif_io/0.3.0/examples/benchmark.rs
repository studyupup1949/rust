//! Performance test for read and write ADIF formated files
//!
//! licensed under CC BY-SA 4.0,
//! To view a copy of this license, visit <http://creativecommons.org/licenses/by-sa/4.0/>
//!
//! Author: Andreas, <df1asc@darc.de>

use adif_io::{DeserializeADI, Doc, SerializeADI};
use clap::Parser;
use env_logger;
use log::info;
use std::time::Instant;
use std::{env, fs};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// the ADI file to read
    #[arg(short, long, default_value = "test_data/big_testfile_10000.adi")]
    file: String,

    /// the ADI output file to write
    #[arg(
        short,
        long,
        conflicts_with = "print",
        default_value = "test_data/test.adi"
    )]
    out: String,

    /// print parsed document (and skip 2nd parse and write)
    #[arg(long, default_value_t = false)]
    print: bool,
}

fn main() {
    let _ = env::var("RUST_LOG").is_err_and(|_| {
        unsafe { env::set_var("RUST_LOG", "info") }
        false
    });
    env_logger::init();

    let args = Args::parse();

    info!("Running benchmark...");
    let t_read = Instant::now();

    let content = fs::read_to_string(args.file).expect("error reading ADI file: {err}");

    info!("Read : {} ms", t_read.elapsed().as_millis());
    let t_parse = Instant::now();
    let mut doc = Doc::new();
    doc.deserialize_adi(&content)
        .expect("could not deserialize");
    info!("1st Parse: {} ms", t_parse.elapsed().as_millis());
    let t_parse2 = Instant::now();
    let mut doc = Doc::new();
    doc.deserialize_adi(&content)
        .expect("could not deserialize");

    if args.print {
        println!("{doc:#?}");
    } else {
        info!("2nd Parse: {} ms", t_parse2.elapsed().as_millis());

        let t_dump = Instant::now();
        let adi_str = doc.serialize_adi();
        info!("Dump: {} ms", t_dump.elapsed().as_millis());

        let t_write = Instant::now();
        fs::write(args.out, adi_str).expect("could not write ADI file");
        info!("Write: {} ms", t_write.elapsed().as_millis());
        info!("...finished after {} ms!", t_parse.elapsed().as_millis());
    }
}
