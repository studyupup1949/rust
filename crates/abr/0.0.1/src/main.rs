use structopt::StructOpt;
use ::phf::{Map, phf_map};
use std::process::Command;

/*
    "ldpi" => Galaxy Y,
    "mdpi" => iphone,
    "hdpi" => Android One,
    "xhdpi" => Moto x,
    "xxhdpi" => Google Pixel,
    "xxxhdpi" => Google Pixel XL,
*/

static SIZE_MAP: Map<&'static str, &'static str> = phf_map! {
    "ldpi" => "240x320",
    "mdpi" => "320x480",
    "hdpi" => "480x854",
    "xhdpi" => "720x1280",
    "xxhdpi" => "1080x1920",
    "xxxhdpi" => "1440x2560",
};

static DENSITY_MAP: Map<&'static str, &'static str> = phf_map! {
    "ldpi" => "133",
    "mdpi" => "165",
    "hdpi" => "218",
    "xhdpi" => "312",
    "xxhdpi" => "441",
    "xxxhdpi" => "534",
};

/// Set screen size of a phone using adb.
#[derive(StructOpt)]
struct Cli {
    display: String
}

fn main() {
    let args = Cli::from_args();

    let cmd_diplay = args.display;

    let display_tuple = (
        SIZE_MAP.get::<str>(&cmd_diplay.to_string()),
        DENSITY_MAP.get::<str>(&cmd_diplay.to_string())
    );

    //set up size of display
    Command::new("adb")
        .arg("shell")
        .arg("wm")
        .arg("size")
        .arg(display_tuple.0.unwrap_or(&"reset"))
        .spawn()
        .expect("");

    //set up density of display
    Command::new("adb")
        .arg("shell")
        .arg("wm")
        .arg("density")
        .arg(display_tuple.1.unwrap_or(&"reset"))
        .spawn()
        .expect("");

}
