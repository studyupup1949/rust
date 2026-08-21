mod flag;

use crate::flag::*;

use clap::{Arg, Command};

fn main() {
    let matches = Command::new("LGBT Flags")
        .version("0.2")
        .about("Displays LGBT pride flags with colors and generates flag images")
        .arg(
            Arg::new("colorize")
                .short('c')
                .long("colorize")
                .help("Colorizes the output text using the specified flag colors")
                .num_args(1),
        )
        .arg(
            Arg::new("flag")
                .short('f')
                .long("flag")
                .help("The type of flag to display in the terminal")
                .value_parser([
                    "rainbow",
                    "lesbian",
                    "gay",
                    "bisexual",
                    "transgender",
                    "asexual",
                    "pansexual",
                    "nonbinary",
                    "genderqueer",
                    "mlm",
                    "aromantic",
                    "polysexual",
                    "demiboy",
                    "demigirl",
                    "agender",
                    "bigender",
                    "genderfluid",
                    "abrosexual",
                    "neutrois",
                    "trigender",
                ])
                .default_value("rainbow"),
        )
        .arg(
            Arg::new("image")
                .short('i')
                .long("image")
                .help("Generate an image of the specified flag")
                .value_parser([
                    "rainbow",
                    "lesbian",
                    "gay",
                    "bisexual",
                    "transgender",
                    "asexual",
                    "pansexual",
                    "nonbinary",
                    "genderqueer",
                    "mlm",
                    "aromantic",
                    "polysexual",
                    "demiboy",
                    "demigirl",
                    "agender",
                    "bigender",
                    "genderfluid",
                    "abrosexual",
                    "neutrois",
                    "trigender",
                ]),
        )
        .arg(
            Arg::new("width")
                .short('a')
                .long("width")
                .help("The width of the flag in the terminal")
                .default_value("20")
                .value_parser(clap::value_parser!(u32)),
        )
        .arg(
            Arg::new("height")
                .short('b')
                .long("height")
                .help("The height of the flag in the terminal")
                .default_value("10")
                .value_parser(clap::value_parser!(u32)),
        )
        .arg(
            Arg::new("command")
                .help("Command to execute and colorize")
                .required(false)
                .num_args(1..)
                .trailing_var_arg(true),
        )
        .get_matches();

    let flag_str = matches.get_one::<String>("flag").unwrap();
    let flag = flag_str.parse::<Flag>().unwrap_or(Flag::Rainbow);

    let width = *matches.get_one::<u32>("width").unwrap();
    let height = *matches.get_one::<u32>("height").unwrap();

    if let Some(flag_str) = matches.get_one::<String>("colorize") {
        let flag = flag_str.parse::<Flag>().unwrap_or(Flag::Rainbow);
        let mut command_args = matches
            .get_many::<String>("command")
            .map(|vals| vals.map(|s| s.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        if !command_args.is_empty() {
            let command = command_args.remove(0);
            run_and_colorize(flag, command, &command_args);
            return;
        }
    }

    if let Some(flag_str) = matches.get_one::<String>("image") {
        let flag = flag_str.parse::<Flag>().unwrap_or(Flag::Rainbow);
        let _ = generate_flag_image(flag, width * 10, height * 10);
    } else {
        display_flag_in_terminal(flag, width, height);
    }
}
