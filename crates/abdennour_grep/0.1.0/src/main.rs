use abdennour_grep::run;
use abdennour_grep::Config;
use std::env;
use std::process;
fn main() {
    let conf = Config::new(env::args()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(1)
    });
    if let Err(err) = run(conf) {
        eprintln!("application error: {}", err);
        process::exit(1);
    }
}
