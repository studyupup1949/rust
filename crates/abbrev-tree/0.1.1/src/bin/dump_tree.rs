use abbrev_tree::AbbrevTree;
use std::io::{
    BufRead,
    stdin,
};

fn main() {
    let s = stdin();
    let sl = s.lock();
    let mut t = AbbrevTree::new();

    for line in sl.lines() {
        match line {
            Ok(line) => t.add(&line, ()),
            Err(e) => panic!("{}", e),
        }
    }
    println!("{:?}", t);
}
