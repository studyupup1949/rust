//! Deprecation stub for the renamed `adl2sidm` CLI.

fn main() {
    eprintln!("`adl2sidm` has been renamed to `adl2rsdm`.");
    eprintln!("Install and run the new tool instead:");
    eprintln!("    cargo install adl2rsdm");
    eprintln!("    adl2rsdm <args>");
    std::process::exit(1);
}
