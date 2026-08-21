// prints version, simple command.
use colored::Colorize;

pub fn version() {
    let version = env!("CARGO_PKG_VERSION");
    println!("{} version v{}", "abbreviation-rs".truecolor(183, 65 ,14), version);
}