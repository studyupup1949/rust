use accio::*;

/*
this example demonstrates expanding statements
using code blocks from different paths. we can
differentiate code blocks by naming them.
*/

fn main() {
    let mut val = 1;
    println!("original value = {}", val);
    // expands the first scope
    accio!(firstScope);
    println!("value after first scope = {}", val);
    // expands the second scope
    accio!(secondScope);
    println!("value after second scope = {}", val);
}
