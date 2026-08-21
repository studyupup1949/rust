extern crate reqwest;
extern crate rand;
extern crate serde;
extern crate meval;

mod commands;
mod utils;
mod apis;
mod game;

use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: <command> [arguments...]");
        eprintln!("Commands:");
        eprintln!("  grep <pattern> <file>");
        eprintln!("  cat <file1> [file2...]");
        eprintln!("  ls <directory>");
        eprintln!("  diff <file1> <file2>");
        eprintln!("  time");
        eprintln!("  date");
        eprintln!("  calc");
        eprintln!("  password");
        eprintln!("  currencyconvert <amount> <from_currency> <to_currency>");
        eprintln!("  weather <location>");
        eprintln!("  shorten <long_url>");
        eprintln!("  define <word>");
        eprintln!("  reverse <string>");
        eprintln!("  analyze <data>");
        eprintln!("  xo");
        return Ok(());
    }

    let _result: Result<(), String> = match args[1].as_str() {
        "grep" => Ok(commands::grep(&args[2], &args[3])?),
        "cat" => Ok(commands::cat(&args[2..], None)?),
        "ls" => Ok(commands::ls(args.get(2).map_or(".", String::as_str))?),
        "password" => {
            utils::password_generator();
            Ok(())
        }
        "reverse" => {
            let reversed = utils::reverse_string(&args[2..].join(" "));
            println!("{}", reversed);
            Ok(())

        }
        "analyze" => {
            let analysis = utils::text_analyzer(&args[2..].join(" "));
            println!("{}", analysis);
            Ok(())
        }
        "calc" => {
            utils::calculator();
            Ok(())
        }
        "weather" => {
            if args.len() < 3 {
                eprintln!("Usage: weather <location>");
                Ok(())
            } else {
                if let Err(e) = apis::fetch_weather(&args[2]){
                    eprintln!("Error fetching weather: {}", e);
                }
                Ok(())
            }
        }
        "currencyconvert" => {
            if args.len() < 5 {
                eprintln!("Usage: currencyconvert <amount> <from_currency> <to_currency>");
                Ok(())
            } else {
                let amount: f64 = args[2].parse().unwrap_or(0.0);
                apis::currency_converter(amount, &args[3], &args[4]);
                Ok(())
            }
        }
        "shorten" => {
            if args.len() < 3 {
                eprintln!("Usage: shorten <long_url>");
                Ok(())
            } else {
                apis::url_shortener(&args[2]);
                Ok(())
            }
        }
        "define" => {
            if args.len() < 3 {
                eprintln!("Usage: define <word>");
                Ok(())
            } else {
               if let Err(e) = apis::fetch_definition(&args[2]){
                eprintln!("Error fetching definition: {}", e);
               } 
               Ok(())
            }
        }
        "xo" => {
            game::x_o_game();
            Ok(())
        }
        "diff" => {
            if args.len() < 4 {
                eprintln!("Usage: diff <file1> <file2>");
                Ok(())
            } else {
                if let Err(e) = commands::diff(&args[2], &args[3]) {
                    eprintln!("Error comparing files: {}", e);
                }
                Ok(())
            }
        }
        "time" => {
            let current_time = commands::time();
            println!("Current time: {}", current_time);
            Ok(())
        }
        "date" => {
            let current_date = commands::date();
            println!("Current date: {}", current_date);
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            Ok(())
        }
    };
    Ok(())
}
