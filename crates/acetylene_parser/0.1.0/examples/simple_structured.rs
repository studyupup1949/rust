extern crate acetylene_parser;

use acetylene_parser::tokenize;

/// Show each of the functional groups named in the formula string.
fn main() {
  let res = tokenize("(SO4)H2", "formula");
  println!{"Complete substance is: "};
  println!("{:?}", &res);
  
  for sub in *(res.groups.unwrap()) {
    println!("{}, {},", sub.symbol, sub.quantity);
  }
}
