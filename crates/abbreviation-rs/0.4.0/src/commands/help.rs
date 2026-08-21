// command that returns help info

use colored::Colorize;

pub fn help(cmd: Option<&str>) {
    match cmd {
        Some(cmd) => {
            match cmd {
                // ! COMMANDS GO HERE
                "help" => {
                    println!("{}", "Help Command".bold());
                    println!("The help command provides a list of commands or specific information on a single command.");
                    println!("  Arguments:");
                    println!("  {}: The command to provide information on. ({})", "command".bold(), "OPTIONAL".yellow());
                }
                "init" => {
                    println!("{}", "Init Command".bold());
                    println!("The init command makes a pre-filled abbreviations.json file on your PC for use by the other commands. This command has to be run in order to use the CLI.");
                }
                "search" => {
                    println!("{}", "Search Command".bold());
                    println!("This command searches an abbreviation from the abbreviations.json file and returns the meaning(s).");
                    println!("  Arguments:");
                    println!("  {}: The abbreviation to search. ({})", "abbreviation".bold(), "REQUIRED".red());
                }
                "list" => {
                    println!("{}", "List Command".bold());
                    println!("This command lists all the abbreviations in the abbreviations.json file.");
                }
                _ => {
                    println!("{}: \"{}\" is not a command! Type \"abbreviation help\" to get a list of commands.", "Error".red().bold(),cmd);
                }
            }
        }
        None => {
            println!("{} List of Commands", "abbreviation-rs".truecolor(183, 65, 14));
            println!("help");
            println!("init");
            println!("search");
        }
    }
}