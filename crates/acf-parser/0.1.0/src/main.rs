use acf_parser::prelude::*;

fn main() {
    let result = parse_acf("./acfs/simple.acf");

    let result = result.unwrap();
    let root_entry = &result.entries[0];
    let root_contents = &root_entry.expressions;

    println!("Found root entry '{}'", root_entry.name);
    println!("App name: {}", root_contents["name"]);
    println!("App ID: {}", root_contents["appid"]);
}
