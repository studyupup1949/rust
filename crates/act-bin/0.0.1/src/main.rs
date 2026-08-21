extern crate act_file;

use act_file::Parser;

fn main() {
    let mut parser = Parser::new();
    let result = parser.parse("2 + 2 * 2");

    println!("parse result: {:?} !", result);
}
