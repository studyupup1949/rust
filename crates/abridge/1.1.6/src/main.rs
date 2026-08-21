#[macro_use]
extern crate clap;
use clap::App;

use std::io::{self, Read};

pub mod lib;

pub fn main() {
    // read CLI config
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let mut buffer = String::new();

    // read stdin
    match io::stdin().read_to_string(&mut buffer) {
        Ok(_) => {
            // result to stdout
            if matches.is_present("compress") {
                print!("{}", lib::compress(&buffer));
            } else if matches.is_present("decompress") {
                print!("{}", lib::decompress(&buffer));
            }
        }
        Err(why) => panic!("stdin input could not be read -> {}", why),
    }
}
