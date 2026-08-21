//! Read ADI and write JSON formated
//!
//! licensed under CC BY-SA 4.0,
//! To view a copy of this license, visit <http://creativecommons.org/licenses/by-sa/4.0/>
//!
//! Author: Andreas, <df1asc@darc.de>

use adif_io::{DeserializeADI, Doc};
use clap::Parser;
use env_logger;
use log::info;
use std::{env, fs};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// the ADI file to read
    #[arg(short, long, default_value = "test_data/big_testfile_1000.adi")]
    file: String,

    /// the JSON file to write
    #[arg(short, long)]
    out: Option<String>,
}

fn main() {
    let _ = env::var("RUST_LOG").is_err_and(|_| {
        unsafe { env::set_var("RUST_LOG", "info") }
        false
    });
    env_logger::init();

    let args = Args::parse();

    info!("Reading ADI file '{}'...", &args.file);
    let content = fs::read_to_string(args.file).expect("error reading ADI file: {err}");

    let mut doc = Doc::new();
    doc.deserialize_adi(&content)
        .expect("could not deserialize from ADI");

    let json = serde_json::to_string_pretty(&doc).expect("could not serialize to JSON");

    if let Some(out) = args.out {
        info!("Writing JSON file '{}'...", &out);
        fs::write(out, json).expect("could not write JSON output file");
    } else {
        println!("{json}");
    }
}
