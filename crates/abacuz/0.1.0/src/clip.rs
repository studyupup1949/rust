use clap::{App, Arg, ArgMatches};

pub fn parse<'a>() -> ArgMatches<'a> {
    App::new("Abacuz")
    .version(clap::crate_version!())
    .author("Éverton M. Vieira <everton.muvi@gmail.com>")
    .about("Abacuz ( Qinpel Server ) is a command that servers the graphical user and command interfaces of the Pointel platform. Is the backend of the Qinpel, the Quick Interface for Power Intelligence. It also provides easy access for the file system and database source.")
    .arg(
      Arg::with_name("debug")
        .short("d")
        .long("debug")
        .takes_value(false)
        .required(false)
        .help("On what host should I serve?"),
    )
    .arg(
      Arg::with_name("verbose")
        .short("v")
        .long("verbose")
        .takes_value(false)
        .required(false)
        .help("On what host should I serve?"),
    )
    .arg(
      Arg::with_name("host")
        .short("h")
        .long("host")
        .value_name("ADDRESS")
        .takes_value(true)
        .required(false)
        .help("On what host should I serve?"),
    )
    .arg(
      Arg::with_name("port")
        .short("p")
        .long("port")
        .value_name("NUMBER")
        .takes_value(true)
        .required(false)
        .help("On what port should I serve?"),
    )
    .arg(
      Arg::with_name("no-run")
        .short("x")
        .long("no-run")
        .takes_value(false)
        .required(false)
        .help("Should I exit without serving?"),
    )
    .get_matches()
}
