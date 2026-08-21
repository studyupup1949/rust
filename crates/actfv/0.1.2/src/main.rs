use std::env;
use std::process::exit;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        actfv::show_usage();
        exit(1);
    } else {
        let source_file_path = &args[1];
        let target_file_path = &args[2];
        let profile = &args[3];

        let source_map = actfv::parse_source(source_file_path).unwrap();
        let entries = actfv::get_entries_for_profile(source_map, profile)?;
        actfv::write_target(entries, target_file_path)?;
    }
    Ok(())
}
