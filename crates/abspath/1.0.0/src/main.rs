//! Return the absolute path of a file in linux.

extern crate clap;
extern crate mhlog;

use clap::{App, Arg};
use mhlog::fatal;
use std::path::Path;
use std::process::exit;

fn main() {
    let matches = App::new("abspath")
        .version("1.0")
        .author("m <mhmorgan42@gmail.com>")
        .about("Print absolute path of a file.")
        .arg(Arg::with_name("file")
                .value_name("FILE")
                .required(true)
                .takes_value(true))
        .get_matches();

    let fname = matches.value_of("file").unwrap();
    let path  = Path::new(fname);

    if !path.exists() {
        fatal!("{} doesn't exists", fname);
        exit(1);
    }
    match path.canonicalize() {
        Ok(p) => println!("{}", p.display()),
        Err(e) => fatal!("{}", e),
    }
}
