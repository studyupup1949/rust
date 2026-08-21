extern crate serde;
extern crate serde_json;
extern crate rand;
extern crate ignite;

#[macro_use]
extern crate serde_derive;

use ignite::str_util;
use ignite::argument;
use std::process;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub ethct_interface: String,
}

fn newline() {
    println!("");
}

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
    -r # Check for rootkits.
    -p # Generate a random string of alphanumeric characters.\n";
    }
}

pub mod parser {
    pub fn unknown_param(param: char) -> String {
        let errorstring: String = ["Encountered unexpected operation parameter \"".to_string(), param.to_string(), "\".".to_string()].concat();
        return errorstring;
    }
}

pub mod misc {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::Result;
    use std::process;
    use std::path::PathBuf;
    use std::fs;
    use serde_json;
    use ::global::messages;
    use ::Config;

    pub fn test_no_arg_param(args: &String) {
        if args.len() < 3 {
            println!("{}", messages::GENERIC_SUB_NOARG);
            process::exit(1);
        }
    }

    pub fn config_name() -> PathBuf {
        let mut config_file_path = PathBuf::new();
        config_file_path.push(dirs::home_dir().unwrap());
        config_file_path.push([".config/", "actl", "/config.json"].concat());
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

        let config: Config = Config {ethct_interface: "enp0s3".to_string()};
        let serialized_config = serde_json::to_string(&config).unwrap();

        let mut config_file = File::create(config_file_name)?;
        config_file.write_all(serialized_config.as_bytes())?;
        return Ok(());
    }

    pub fn make_template() {
        let mut app_path = PathBuf::new();
        app_path.push(dirs::home_dir().unwrap());
        app_path.push([".config/", "actl"].concat());
        let _ = fs::create_dir_all(app_path);
        let _ = generate_config();
    }
}

pub mod sysec {
    use rand;
    use rand::Rng;
    use ::misc;
    use ::global::messages;
    use ::parser;
    use rand::distributions::Alphanumeric;
    use std::process;

    pub fn gen_pass(length: usize) -> String {
        let mut rng = rand::thread_rng();

        let pass = rng.sample_iter(&Alphanumeric).take(length).collect::<String>();
        return pass;
    }

    pub fn run(args: &Vec<String>) {
        let _ = misc::test_no_arg_param(&args[1]);

        for param in args[1].chars() {
            match param {
                '-' => {},
                'S' => {},

                'h' => { println!("{}", messages::SYSEC_HELP); },

                'r' => {
                    let _ = process::Command::new("sudo").arg("rkhunter").arg("--check").arg("--sk").status();
                },

                'p' => {
                    if args.len() < 3 {
                        println!("Missing length parameter.");
                        process::exit(1);
                    }

                    let length = args[2].parse::<usize>().unwrap();
                    println!("{}", gen_pass(length));
                },

                _ => { println!("{}", parser::unknown_param(param)); },
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
    use ::global::messages;
    use ::parser;
    use ::misc;
    use ::newline;

    pub fn run(args: &Vec<String>) {
        let _ = misc::test_no_arg_param(&args[1]);

        for param in args[1].chars() {
            match param {
                '-' => {},
                'P' => {},

                'h' => { println!("{}", messages::PCTLS_HELP); },

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

                _ => { println!("{}", parser::unknown_param(param)); },
            }
        }
        return;
    }
}

fn main() {
    let args = argument::get_args();

    if args.len() < 2 {
        println!("No arguments provided. Use the -h switch to display a list of valid operations.");
        process::exit(1);
    } else if args[1].len() < 2 {
        println!("No arguments provided. Use the -h switch to display a list of valid operations.");
        process::exit(1);
    }

    if str_util::get_char(&args[1], 0) != '-' {
        println!("Arguments must be prefixed by a single hyphen. Use the -h parameter to display a list of valid operations.");
        process::exit(1);
    }

    let subopt = str_util::get_char(&args[1], 1);
    match subopt {
        'h' => { println!("{}", global::messages::HELP); },
        'P' => { pctls::run(&args); },
        'E' => { ethct::run(); },
        'S' => { sysec::run(&args); },
        ':' => { misc::make_template(); },
        '_' => { println!("{}", misc::loadconfig_raw()); },
        _ => { println!("{}", ["Encountered unknown operation \"".to_string(), subopt.to_string(), "\".".to_string()].concat()); },
    }
}
