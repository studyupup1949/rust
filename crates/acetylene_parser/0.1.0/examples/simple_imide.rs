extern crate acetylene_parser;
use acetylene_parser::tokenize;

fn main() {
  println!("{:?}", tokenize("(HCO)2NH", "formula"));
}

