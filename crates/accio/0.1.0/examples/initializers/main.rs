use accio::*;
use std::collections::HashMap;

/*
in this example we collect a list of functions
to be executed in the beginning of the program
*/

// let them import themselves!
accio! {mods}

pub type TestMap = HashMap<&'static str, i32>;

#[accio_body(initializers)]
static INIT_FNS: &[fn(&mut TestMap)] = &[];

fn init_map() -> TestMap {
    let mut m: TestMap = HashMap::new();
    for f in INIT_FNS {
        f(&mut m)
    }
    m
}

fn main() {
    let m = init_map();
    println!("learning numbers with accio!");
    for (k, v) in m.iter() {
        println!("{} -> {}", k, v);
    }
    println!("...uh, that's all for today")
}
