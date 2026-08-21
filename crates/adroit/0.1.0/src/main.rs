use std::{env, fs};

use adroit::Module;

fn main() {
    let mut args = env::args().skip(1);
    let source = fs::read_to_string(args.next().unwrap()).unwrap();
    let module: Module = source.parse().unwrap();
    print!("{module}");
}
