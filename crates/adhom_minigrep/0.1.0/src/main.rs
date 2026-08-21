use std::env;
use std::process;

use adhom_minigrep;

fn main() {
    // let args: Vec<String> = env::args().collect();
    let config = adhom_minigrep::Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });
    if let Err(e) = adhom_minigrep::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
