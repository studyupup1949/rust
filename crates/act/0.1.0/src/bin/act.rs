extern crate act;

use act::*;
use std::fs::File;
use std::io::prelude::*;
use std::env;

fn main() {
    let mut f = File::open(env::args().nth(1).expect("Usage: act <file>")).expect("Couldn't open game file!");
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    let mut g = load_game(&s).unwrap();
    g.play();
}
