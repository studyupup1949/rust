//! Read JSON and write ADI file
//!
//! licensed under CC BY-SA 4.0,
//! To view a copy of this license, visit <http://creativecommons.org/licenses/by-sa/4.0/>
//!
//! Author: Andreas, <df1asc@darc.de>

use adif_io::{Doc, SerializeADI};
use clap::Parser;
use env_logger;
use log::info;
use std::{env, fs};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// the JSON file to read
    #[arg(short, long)]
    file: String,

    /// the ADI file to write
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

    info!("Reading JSON file '{}'...", &args.file);
    let content = fs::read_to_string(args.file).expect("error reading JSON file: {err}");

    let doc: Doc = serde_json::from_str(&content).expect("could not deserialize from JSON");

    let adi = doc.serialize_adi();

    if let Some(out) = args.out {
        info!("Writing ADI file '{}'...", &out);
        fs::write(out, adi).expect("could not write ADI output file");
    } else {
        println!("{adi}");
    }
}
