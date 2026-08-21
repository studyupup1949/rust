use clap::{Arg, ArgMatches, Command};

pub fn parse() -> ArgMatches {
    Command::new("abacuz")
        .version(clap::crate_version!())
        .author("Éverton M. Vieira <everton.muvi@gmail.com>")
        .about("Abacuz is a command program that servers an easy web access for file storage with authentication and authorization.")
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .takes_value(false)
                .required(false)
                .help("On what host should I serve?"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .takes_value(false)
                .required(false)
                .help("On what host should I serve?"),
        )
        .arg(
            Arg::new("host")
                .short('h')
                .long("host")
                .value_name("ADDRESS")
                .takes_value(true)
                .required(false)
                .help("On what host should I serve?"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("NUMBER")
                .takes_value(true)
                .required(false)
                .help("On what port should I serve?"),
        )
        .get_matches()
}
