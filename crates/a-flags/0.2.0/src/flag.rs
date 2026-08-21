use colored::Colorize;
use image::{Rgb, RgbImage};
use phf::phf_map;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::process::{Command as SysCommand, Stdio};
use std::str::FromStr;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flag {
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

impl Flag {
    pub fn as_str(&self) -> &'static str {
        match self {
            Flag::Rainbow => "rainbow",
            Flag::Lesbian => "lesbian",
            Flag::Gay => "gay",
            Flag::Bisexual => "bisexual",
            Flag::Transgender => "transgender",
            Flag::Asexual => "asexual",
            Flag::Pansexual => "pansexual",
            Flag::NonBinary => "nonbinary",
            Flag::GenderQueer => "genderqueer",
            Flag::Mlm => "mlm",
            Flag::Aromantic => "aromantic",
            Flag::Polysexual => "polysexual",
            Flag::Demiboy => "demiboy",
            Flag::Demigirl => "demigirl",
            Flag::Agender => "agender",
            Flag::Bigender => "bigender",
            Flag::Genderfluid => "genderfluid",
            Flag::Abrosexual => "abrosexual",
            Flag::Neutrois => "neutrois",
            Flag::Trigender => "trigender",
        }
    }
}

impl FromStr for Flag {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rainbow" => Ok(Flag::Rainbow),
            "lesbian" => Ok(Flag::Lesbian),
            "gay" => Ok(Flag::Gay),
            "bisexual" => Ok(Flag::Bisexual),
            "transgender" => Ok(Flag::Transgender),
            "asexual" => Ok(Flag::Asexual),
            "pansexual" => Ok(Flag::Pansexual),
            "nonbinary" => Ok(Flag::NonBinary),
            "genderqueer" => Ok(Flag::GenderQueer),
            "mlm" => Ok(Flag::Mlm),
            "aromantic" => Ok(Flag::Aromantic),
            "polysexual" => Ok(Flag::Polysexual),
            "demiboy" => Ok(Flag::Demiboy),
            "demigirl" => Ok(Flag::Demigirl),
            "agender" => Ok(Flag::Agender),
            "bigender" => Ok(Flag::Bigender),
            "genderfluid" => Ok(Flag::Genderfluid),
            "abrosexual" => Ok(Flag::Abrosexual),
            "neutrois" => Ok(Flag::Neutrois),
            "trigender" => Ok(Flag::Trigender),
            _ => Err(format!("Invalid flag name: {}", s)),
        }
    }
}

pub static FLAG_PALETTES: phf::Map<&'static str, &[&str]> = phf_map! {
    "rainbow" => &["#E40303", "#FF8C00", "#FFED00", "#008026", "#004CFF", "#732982"],
    "lesbian" => &["#D62E02", "#FD9855", "#FFFFFF", "#D161A2", "#A20160"],
    "gay" => &["#078D70", "#26CEAA", "#98E8C1", "#FFFFFF", "#7BADE2", "#5049CC", "#3D1A78"],
    "bisexual" => &["#D60270", "#D60270", "#9B4F96", "#0038A8", "#0038A8"],
    "transgender" => &["#55CDFC", "#F7A8B8", "#FFFFFF", "#F7A8B8", "#55CDFC"],
    "asexual" => &["#000000", "#A4A4A4", "#FFFFFF", "#800080"],
    "pansexual" => &["#FF218C", "#FFD800", "#21B1FF"],
    "nonbinary" => &["#FFF430", "#FFFFFF", "#9C59D1", "#000000"],
    "genderqueer" => &["#B57EDC", "#FFFFFF", "#4A8123"],
    "mlm" => &["#78D70D", "#3CB371", "#FFFFFF", "#3CB371", "#78D70D"],
    "aromantic" => &["#3DA542", "#A7D379", "#FFFFFF", "#A9A9A9", "#000000"],
    "polysexual" => &["#F61CB9", "#07D569", "#1C92F6"],
    "demiboy" => &["#7F7FFF", "#C4C4C4", "#FFFFFF", "#C4C4C4", "#7F7FFF"],
    "demigirl" => &["#FC9EBB", "#C4C4C4", "#FFFFFF", "#C4C4C4", "#FC9EBB"],
    "agender" => &["#000000", "#B9B9B9", "#FFFFFF", "#B7F684", "#FFFFFF", "#B9B9B9", "#000000"],
    "bigender" => &["#C479A2", "#F9CEDF", "#FFFFFF", "#F9CEDF", "#C479A2"],
    "genderfluid" => &["#FF75A2", "#FFFFFF", "#BD88F5", "#000000", "#333EBD"],
    "abrosexual" => &["#75EAB6", "#ADF784", "#FFFFFF", "#F78BB3", "#F12D6F"],
    "neutrois" => &["#FFFFFF", "#4DDC82", "#000000"],
    "trigender" => &["#F6AAB7", "#DAA8E9", "#A3E3F4", "#DAA8E9", "#F6AAB7"],
};

/// Runs a system command and colorizes its output according to the selected flag's palette.
///
/// # Arguments
/// * `flag` - The flag whose palette will be used for colorizing the output.
/// * `command` - The command to run.
/// * `args` - The arguments for the command.
///
/// This function captures the command's standard output, processes each line, and prints it in color
/// corresponding to the flag's palette. The lines will be colored in a cycling fashion.

pub fn run_and_colorize(flag: Flag, command: &str, args: &[&str]) {
    let palette = get_flag_palette(flag);
    let mut color_index = 0;

    let mut child = SysCommand::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start command");

    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let reader = BufReader::new(stdout);

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                let color = palette[color_index % palette.len()];
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&color[1..3], 16),
                    u8::from_str_radix(&color[3..5], 16),
                    u8::from_str_radix(&color[5..7], 16),
                ) {
                    println!("{}", line.truecolor(r, g, b));
                } else {
                    eprintln!("Invalid color code: {}", color);
                }

                color_index += 1;
            }
            Err(err) => eprintln!("Error reading line: {}", err),
        }
    }
}

/// Retrieves the color palette for a specific flag.
///
/// # Arguments
/// * `flag` - The flag whose palette is to be fetched.
///
/// # Returns
/// The color palette for the specified flag.

pub fn get_flag_palette(flag: Flag) -> &'static [&'static str] {
    match FLAG_PALETTES.get(flag.as_str()) {
        Some(palette) => palette,
        None => &["#FFFFFF"],
    }
}

/// Generates an image representing a flag with the selected palette.
///
/// # Arguments
/// * `flag` - The flag whose image to generate.
/// * `width` - The width of the generated image.
/// * `height` - The height of the generated image.
///
/// # Returns
/// Returns a `Result`, indicating success or failure.

pub fn generate_flag_image(flag: Flag, width: u32, height: u32) -> Result<(), String> {
    let palette = get_flag_palette(flag);
    let stripe_height = height / palette.len() as u32;

    let mut img = RgbImage::new(width, height);

    for (i, &color) in palette.iter().enumerate() {
        let (r, g, b) = (
            u8::from_str_radix(&color[1..3], 16).unwrap(),
            u8::from_str_radix(&color[3..5], 16).unwrap(),
            u8::from_str_radix(&color[5..7], 16).unwrap(),
        );
        let rgb = Rgb([r, g, b]);

        for y in (i as u32 * stripe_height)..((i as u32 + 1) * stripe_height) {
            for x in 0..width {
                img.put_pixel(x, y, rgb);
            }
        }
    }

    fs::create_dir_all("flags").map_err(|e| e.to_string())?;
    let path = format!("flags/{}.png", flag.as_str());
    img.save(Path::new(&path)).map_err(|e| e.to_string())?;
    println!("✅ Flag image saved as {}", path);

    Ok(())
}

/// Displays a representation of the flag in the terminal using colored stripes.
///
/// # Arguments
/// * `flag` - The flag to display.
/// * `width` - The width of the display area.
/// * `height` - The height of the display area.
///
/// This function will display each stripe of the flag in its designated color in the terminal,
/// pausing briefly between each stripe.

pub fn display_flag_in_terminal(flag: Flag, width: u32, height: u32) {
    let palette = get_flag_palette(flag);

    let stripe_height = height / palette.len() as u32;

    println!();

    for &color in palette.iter() {
        let (r, g, b) = (
            u8::from_str_radix(&color[1..3], 16).unwrap(),
            u8::from_str_radix(&color[3..5], 16).unwrap(),
            u8::from_str_radix(&color[5..7], 16).unwrap(),
        );

        for _ in 0..stripe_height {
            println!("{}", " ".repeat(width as usize).on_truecolor(r, g, b));
        }

        thread::sleep(Duration::from_millis(200));
    }
}
