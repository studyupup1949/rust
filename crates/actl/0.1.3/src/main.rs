extern crate sta;
extern crate serde;
extern crate serde_json;
extern crate rand;

#[macro_use]
extern crate serde_derive;

use std::process;
use std::string::String;
use sta::disp::print;
use sta::arg;
use sta::conv::b_str;
use sta::stringutil;
use sta::appinfo::AppInfo;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub ethct_interface: String,
}

const APP_INFO: AppInfo = AppInfo {name: "actl", author: "Acrimon"};

pub mod global {
    pub mod messages {
        pub static GENERIC_SUB_NOARG: &'static str = "No arguments provided. Append the -h switch to display a list of valid operation parameters.";

        pub static HELP: &'static str = "List of valid operations:
    -h # Display a list of possible operations.
    -[OPERATION]h # Display a list of parameters for that operation.
    -P # Do package related stuff. Updating and cleaning cache and the similar stuff.
    -E # Connect via ethernet with the configured adaper.
    -S # System security related stuff.\n";

        pub static PCTLS_HELP: &'static str = "List of valid parameters:
    -[OPERATION]h # Display a list of possible subswitches.
    -y # Invoke trizen to do a full upgrade.
    -o # Optimize the pacman database.
    -k # Clean the package cache of everything except for installed packages.
    -u # Display orphan packages.\n";

        pub static SYSEC_HELP: &'static str = "List of valid parameters:
    -[OPERATION]h # Display a list of possible subswitches.
    -r # Check for rootkits.\n";
    }
}

pub mod parser {
    use sta::conv::b_str;

    pub fn unknown_param(param: char) -> String {
        let errorstring: String = [b_str("Encountered unexpected operation parameter \""), param.to_string(), b_str("\".")].concat();
        return errorstring;
    }
}

pub mod misc {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::Result;
    use std::process;
    use std::env;
    use std::path::PathBuf;
    use std::fs;
    use sta::disp::print;
    use sta::conv::b_str;
    use serde_json;
    use ::global::messages;
    use ::Config;
    use ::APP_INFO;

    pub fn test_no_arg_param(args: &String) {
        if args.len() < 3 {
            print(messages::GENERIC_SUB_NOARG);
            process::exit(1);
        }
    }

    pub fn config_name() -> PathBuf {
        let mut config_file_path = PathBuf::new();
        config_file_path.push(env::home_dir().unwrap());
        config_file_path.push([".config/", APP_INFO.name, "/config.json"].concat());
        return config_file_path;
    }

    pub fn loadconfig() -> Config {
        let config_file_name = config_name();
        let mut config_file = File::open(config_file_name).expect("File not found.");

        let mut serialized_config = String::new();
        config_file.read_to_string(&mut serialized_config)
            .expect("Something went wrong reading the file.");

        let config: Config = serde_json::from_str(&serialized_config).unwrap();
        return config;
    }

    pub fn loadconfig_raw() -> String {
        let config_file_name = config_name();
        let mut config_file = File::open(config_file_name).expect("File not found.");

        let mut serialized_config = String::new();
        config_file.read_to_string(&mut serialized_config)
            .expect("Something went wrong reading the file.");

        return serialized_config;
    }

    pub fn generate_config() -> Result<()> {
        let config_file_name = config_name();

        let config: Config = Config {ethct_interface: b_str("enp0s3")};
        let serialized_config = serde_json::to_string(&config).unwrap();

        let mut config_file = File::create(config_file_name)?;
        config_file.write_all(serialized_config.as_bytes())?;
        return Ok(());
    }

    pub fn make_template() {
        let mut app_path = PathBuf::new();
        app_path.push(env::home_dir().unwrap());
        app_path.push([".config/", APP_INFO.name].concat());
        let _ = fs::create_dir_all(app_path);
        let _ = generate_config();
    }
}

pub mod sysec {
    use std::process;
    use sta::disp::print;
    use rand;
    use rand::Rng;
    use ::misc;
    use ::global::messages;
    use ::parser;

    pub fn gen_pass(length: usize) -> String {
        let mut rng = rand::thread_rng();

        let pass = rng.gen_ascii_chars().take(length).collect::<String>();
        return pass;
    }

    pub fn run(args: &Vec<String>) {
        let _ = misc::test_no_arg_param(&args[1]);

        for param in args[1].chars() {
            match param {
                '-' => {},
                'S' => {},

                'h' => { print(messages::SYSEC_HELP); },

                'r' => {
                    let _ = process::Command::new("sudo").arg("rkhunter").arg("--check").arg("--sk").status();
                },

                'p' => {
                    if args.len() < 3 {
                        print("Missing length parameter.");
                        process::exit(1);
                    }

                    let length = args[2].parse::<usize>().unwrap();
                    print(gen_pass(length));
                },

                _ => { print(parser::unknown_param(param)); },
            }
        }
    }
}

pub mod ethct {
    use std::process;
    use ::misc;
    use ::Config;

    pub fn run() {
        let config: Config = misc::loadconfig();
        let _ = process::Command::new("sudo").arg("ip").arg("link").arg("set").arg(&config.ethct_interface).arg("up").status();
        let _ = process::Command::new("sudo").arg("dhcpcd").arg(&config.ethct_interface).status();
    }
}

pub mod pctls {
    use std::process;
    use sta::disp::{print, newline};
    use ::global::messages;
    use ::parser;
    use ::misc;

    pub fn run(args: &Vec<String>) {
        let _ = misc::test_no_arg_param(&args[1]);

        for param in args[1].chars() {
            match param {
                '-' => {},
                'P' => {},

                'h' => { print(messages::PCTLS_HELP); },

                'y' => {
                    let _ = process::Command::new("trizen").arg("-Syu").status();
                    newline();
                },

                'o' => {
                    let _ = process::Command::new("sudo").arg("pacman-optimize").status();
                },

                'k' => {
                    let _ = process::Command::new("trizen").arg("-Sc").status();
                    newline();
                },

                'u' => {
                    let _ = process::Command::new("trizen").arg("-Qdtq").status();
                    newline();
                },

                _ => { print(parser::unknown_param(param)); },
            }
        }
        return;
    }
}

fn main() {
    let args = arg::get_args();

    if args.len() < 2 {
        print("No arguments provided. Use the -h switch to display a list of valid operations.");
        process::exit(1);
    } else if args[1].len() < 2 {
        print("No arguments provided. Use the -h switch to display a list of valid operations.");
        process::exit(1);
    }

    if stringutil::get_char_pos(args[1].clone(), 0) != '-' {
        print("Arguments must be prefixed by a single hyphen. Use the -h parameter to display a list of valid operations.");
        process::exit(1);
    }

    let subopt = stringutil::get_char_pos(args[1].clone(), 1);
    match subopt {
        'h' => { print(global::messages::HELP); },
        'P' => { pctls::run(&args); },
        'E' => { ethct::run(); },
        'S' => { sysec::run(&args); },
        ':' => { misc::make_template(); },
        '_' => { print(misc::loadconfig_raw()); },
        _ => { print([b_str("Encountered unknown operation \""), subopt.to_string(), b_str("\".")].concat()); },
    }
}
