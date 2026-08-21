use ad_event_log_parser::*;
use anyhow::{Context, Result};
use csv::WriterBuilder;
use std::env;
use std::fs::OpenOptions;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Ad Event Log Parser CLI - Usage:");
        println!("  cargo run -- <log_data>      Parses log data provided as argument.");
        println!("  cargo run -- --help          Gives help info.");
        println!("  cargo run -- --credits       Shows project credits.");
        return Ok(());
    }

    match args[1].as_str() {
        "--help" => {
            println!("Ad Event Log Parser CLI - Usage:");
            println!("  cargo run -- <log_data>      Parses log data provided as argument.");
            println!("  cargo run -- --help          Gives help info.");
            println!("  cargo run -- --credits       Shows project credits.");
        }
        "--credits" => {
            println!("Ad Event Log Parser by Olha Horban");
            println!("License: MIT");
        }
        _ => {
            let log_data = args[1..].join("\n");

            let file_path = "parsed_events.csv";
            let file_exists = Path::new(file_path).exists();

            let output_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)
                .with_context(|| "Failed to open or create output file")?;

            let mut wtr = WriterBuilder::new()
                .has_headers(!file_exists)
                .from_writer(output_file);

            if !file_exists {
                wtr.write_record(["date", "time", "uuid", "ip", "price"])
                    .with_context(|| "Failed to write CSV headers")?;
            }

            for line in log_data.lines() {
                match parse_ad_event(line) {
                    Ok(parsed_event) => {
                        let event: AdEventLog = parsed_event;
                        wtr.write_record(&[
                            event.date,
                            event.time,
                            event.uuid,
                            event.ip,
                            event.price.to_string(),
                        ])
                        .with_context(|| "Failed to write parsed event to CSV")?;
                    }
                    Err(e) => eprintln!("Error parsing log line: {}", e),
                }
            }

            wtr.flush().with_context(|| "Failed to flush CSV writer")?;
        }
    }

    Ok(())
}
