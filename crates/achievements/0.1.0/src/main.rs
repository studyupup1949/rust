use clap::{command, Parser, Subcommand};

use achievements::{
    config::{self, Day},
    days_since, Interval,
};

#[derive(Subcommand)]
enum Command {
    /// Displays the achivements
    Achievements,
    /// Adds a day to the config
    Add,
    /// Removes a day from the config
    Remove,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

fn main() {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Achievements) {
        Command::Achievements => {
            display_achievements();
        }
        _ => unimplemented!("Other commands not implemented yet"),
    }
}

fn display_achievements() {
    let config = config::read().expect("Failed to read config");

    for Day { label, date: day } in config.days.iter() {
        let days = days_since(*day);
        let achievement = Interval::from_days(days);
        println!("{}: {}", label, achievement);
    }
}
