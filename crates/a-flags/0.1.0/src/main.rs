use clap::{Arg, Command};
use colored::*;

#[derive(Debug, Clone, Copy)]
enum Flag {
    Rainbow,
    Lesbian,
    Gay,
    Bisexual,
    Transgender,
    Asexual,
    Pansexual,
    NonBinary,
    GenderQueer,
    Mlm,
    Aromantic,
    Polysexual,
    Demiboy,
    Demigirl,
    Agender,
    Bigender,
    Genderfluid,
    Abrosexual,
    Neutrois,
    Trigender,
}

fn get_flag_palette(flag: Flag) -> Vec<&'static str> {
    match flag {
        Flag::Rainbow => vec![
            "#E40303", "#FF8C00", "#FFED00", "#008026", "#004CFF", "#732982",
        ],
        Flag::Lesbian => vec!["#D62E02", "#FD9855", "#FFFFFF", "#D161A2", "#A20160"],
        Flag::Gay => vec![
            "#078D70", "#26CEAA", "#98E8C1", "#FFFFFF", "#7BADE2", "#5049CC", "#3D1A78",
        ],
        Flag::Bisexual => vec!["#D60270", "#D60270", "#9B4F96", "#0038A8", "#0038A8"],
        Flag::Transgender => vec!["#55CDFC", "#F7A8B8", "#FFFFFF", "#F7A8B8", "#55CDFC"],
        Flag::Asexual => vec!["#000000", "#A4A4A4", "#FFFFFF", "#800080"],
        Flag::Pansexual => vec!["#FF218C", "#FFD800", "#21B1FF"],
        Flag::NonBinary => vec!["#FFF430", "#FFFFFF", "#9C59D1", "#000000"],
        Flag::GenderQueer => vec!["#B57EDC", "#FFFFFF", "#4A8123"],
        Flag::Mlm => vec!["#78D70D", "#3CB371", "#FFFFFF", "#3CB371", "#78D70D"],
        Flag::Aromantic => vec!["#3DA542", "#A7D379", "#FFFFFF", "#A9A9A9", "#000000"],
        Flag::Polysexual => vec!["#F61CB9", "#07D569", "#1C92F6"],
        Flag::Demiboy => vec!["#7F7FFF", "#C4C4C4", "#FFFFFF", "#C4C4C4", "#7F7FFF"],
        Flag::Demigirl => vec!["#FC9EBB", "#C4C4C4", "#FFFFFF", "#C4C4C4", "#FC9EBB"],
        Flag::Agender => vec![
            "#000000", "#B9B9B9", "#FFFFFF", "#B7F684", "#FFFFFF", "#B9B9B9", "#000000",
        ],
        Flag::Bigender => vec!["#C479A2", "#F9CEDF", "#FFFFFF", "#F9CEDF", "#C479A2"],
        Flag::Genderfluid => vec!["#FF75A2", "#FFFFFF", "#BD88F5", "#000000", "#333EBD"],
        Flag::Abrosexual => vec!["#75EAB6", "#ADF784", "#FFFFFF", "#F78BB3", "#F12D6F"],
        Flag::Neutrois => vec!["#FFFFFF", "#4DDC82", "#000000"],
        Flag::Trigender => vec!["#F6AAB7", "#DAA8E9", "#A3E3F4", "#DAA8E9", "#F6AAB7"],
    }
}
fn main() {
    let matches = Command::new("LGBT Flags")
        .version("1.0")
        .about("Displays LGBT pride flags with colors")
        .arg(
            Arg::new("flag")
                .short('f')
                .long("flag")
                .help("The type of flag to display")
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
        .get_matches();

    let flag_str = matches.get_one::<String>("flag").unwrap();
    let flag = match flag_str.as_str() {
        "rainbow" => Flag::Rainbow,
        "lesbian" => Flag::Lesbian,
        "gay" => Flag::Gay,
        "bisexual" => Flag::Bisexual,
        "transgender" => Flag::Transgender,
        "asexual" => Flag::Asexual,
        "pansexual" => Flag::Pansexual,
        "nonbinary" => Flag::NonBinary,
        "genderqueer" => Flag::GenderQueer,
        "mlm" => Flag::Mlm,
        "aromantic" => Flag::Aromantic,
        "polysexual" => Flag::Polysexual,
        "demiboy" => Flag::Demiboy,
        "demigirl" => Flag::Demigirl,
        "agender" => Flag::Agender,
        "bigender" => Flag::Bigender,
        "genderfluid" => Flag::Genderfluid,
        "abrosexual" => Flag::Abrosexual,
        "neutrois" => Flag::Neutrois,
        "trigender" => Flag::Trigender,
        _ => Flag::Rainbow, // Default to Raibow flag
                            //
    };

    let palette = get_flag_palette(flag);
    for color in palette {
        println!(
            "{}",
            " ".repeat(50).on_truecolor(
                u8::from_str_radix(&color[1..3], 16).unwrap(),
                u8::from_str_radix(&color[3..5], 16).unwrap(),
                u8::from_str_radix(&color[5..7], 16).unwrap()
            )
        );
    }
}
