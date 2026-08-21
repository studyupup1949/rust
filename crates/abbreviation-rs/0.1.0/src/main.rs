mod helpers;
mod commands;
mod structs;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let command = &args[1];

    if command == "init" {
        commands::init::init();
    } else if command == "search" {
        let abbr = helpers::load_abbr::load_abbr();
        if &args[2] == "" {
            panic!("You did not enter an abbreviation to search!");
        }

        commands::search::search(&args[2], &abbr);
    }
}
