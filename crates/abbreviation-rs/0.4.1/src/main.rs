mod helpers;
mod commands;
mod structs;

use std::{env, process::exit};
use colored::Colorize;

fn main() {
    let args: Vec<String> = env::args().collect();

    // * Offer the list of commands when no command is entered.
    if args.len() <= 1 {
        commands::help::help(None);
        exit(0);
    }
    let command = &args[1];

    if command == "init" {
        commands::init::init();
    } else if command == "search" {
        let abbr = helpers::load_abbr::load_abbr();
        if args.len() <= 2 {
            println!("{}: You did not enter an abbreviation to search!", "Error".red().bold());
            exit(1);
        }

        commands::search::search(&args[2], &abbr);
    } else if command == "help" {
        let cmd: Option<&str>;

        if args.len() <= 2 {
            cmd = None;
        } else {
            cmd = Some(&args[2]);
        }

        commands::help::help(cmd);
    } else if command == "version" {
        commands::version::version();
    } else if command == "list" {
        let abbr = helpers::load_abbr::load_abbr();
        commands::list::list(&abbr);
    }
}
