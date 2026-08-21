// command that returns help info

pub fn help(cmd: Option<&str>) {
    match cmd {
        Some(cmd) => {
            match cmd {
                // ! COMMANDS GO HERE
                "help" => {
                    println!("Help Command");
                    println!("The help command provides a list of commands or specific information on a single command.");
                    println!("  Arguments:");
                    println!("  command: The command to provide information on. (OPTIONAL)");
                }
                "init" => {
                    println!("Init Command");
                    println!("The init command makes a pre-filled abbreviations.json file on your PC for use by the other commands. This command has to be run in order to use the CLI.");
                }
                "search" => {
                    println!("Search Command");
                    println!("This command searches an abbreviation from the abbreviation.json file and returns the meaning(s).");
                    println!("  Arguments:");
                    println!("  abbreviation: The abbreviation to search. (REQUIRED)");
                }
                _ => {
                    println!("\"{}\" is not a command! Type \"abbreviation help\" to get a list of commands.", cmd);
                }
            }
        }
        None => {
            println!("abbreviation-rs List of Commands");
            println!("help");
            println!("init");
            println!("search");
        }
    }
}