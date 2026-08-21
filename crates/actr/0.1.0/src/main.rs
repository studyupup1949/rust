use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        println!("actr - Actor RTC CLI");
        println!("\nUsage:");
        println!("  actr [OPTIONS]");
        println!("\nOptions:");
        println!("  -h, --help     Print help information");
        println!("  -V, --version  Print version information");
        return;
    }

    if args.len() > 1 && (args[1] == "--version" || args[1] == "-V") {
        println!("actr {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    println!("Welcome to Actor RTC CLI!");
    println!("Use 'actr --help' for more information.");
}
