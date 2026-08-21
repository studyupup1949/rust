extern crate acetylene_parser;

use acetylene_parser::tokenize;

/// Show each of the functional groups named in the formula string.
fn main() {
  let test_str = r"[OH3+]";
  let res = tokenize(test_str, "smiles");

  println!("{:?}", res);
}
