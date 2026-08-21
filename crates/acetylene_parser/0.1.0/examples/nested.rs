extern crate acetylene_parser;

use acetylene_parser::tokenize;

/// Show each of the functional groups named in the formula string.
fn main() {
  for sub in *(tokenize("(CH3)3CH", "formula").groups.unwrap()) {
    println!("symbol: {}, quantity: {}, charge: {:?}",
             sub.symbol, sub.quantity, sub.charge);
  }
}
