fn main() {
    if let Err(e) = acro::get_args().and_then(acro::run) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
