use clap::{command, Parser, Subcommand};

use achievements::{
    config::{self, Day},
    days_since, Interval,
};

#[derive(Subcommand)]
enum Command {
    /// Displays the achivements
    Achievements,
    /// List days in the config
    List,
    /// Adds a day to the config
    Add { label: String },
    /// Removes a day from the config
    Remove { label: String },
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
        Command::Achievements => display_achievements(),
        Command::List => list_days(),
        Command::Add { label } => add_day(label),
        Command::Remove { label } => remove_day(label),
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

fn list_days() {
    let config = config::read().expect("Failed to read config");
    for day in config.days.iter() {
        println!("'{}': {}", day.label, day.date);
    }
}

fn add_day(_label: String) {
    let _config = config::read().expect("Failed to read config");
    // TODO: Parse datetime somehow
    // TODO: Call config.set_date(label, date)

    println!(
        "TODO: Not implemented at this time, edit ~/.config/achievements/config.json manually"
    );
}

fn remove_day(label: String) {
    let mut config = config::read().expect("Failed to read config");
    config.remove_day(&label);
    config::write(&config).expect("Failed to write config");

    println!("Day with label '{label}' removed from config");
}
