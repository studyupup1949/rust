mod helpers;
mod commands;
mod structs;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // TODO: Replace this error with a help command instead.
    if args.len() <= 1 {
        panic!("You did not enter a command! Options: init, search")
    }
    let command = &args[1];

    if command == "init" {
        commands::init::init();
    } else if command == "search" {
        let abbr = helpers::load_abbr::load_abbr();
        if args.len() <= 2 {
            panic!("You did not enter an abbreviation to search!");
        }

        commands::search::search(&args[2], &abbr);
    }
}
