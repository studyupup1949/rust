extern crate acetylene_parser;

use acetylene_parser::tokenize;

/// Show each of the functional groups named in the formula string.
fn main() {
  let benzene_ring = r"c1ccccc1";
  let res = tokenize(benzene_ring, "smiles");
  
  println!("Main substance: {:?}", &res.symbol);
  println!("Groups: ");
  for sub in *(res.groups.unwrap()) {
    println!("{:?}", sub);
  }
}
