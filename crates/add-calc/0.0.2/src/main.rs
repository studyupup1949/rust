use ext::{no_std::pipelines::{pipe::Pipe, tap::Tap}, standard::functions::{ext::StdAnyExt, fun::read_line}};
use regex::Regex;
use std::process;
fn main() {
    let r = Regex::new(r"([+-]?(?:\d+\.?\d*|\.\d+))").unwrap();
    loop {
        read_line()
            .trim_end()
            .tap(|s| if s.is_empty() { process::exit(0) })
            .pipe(|s| r.find_iter(s))
            .filter_map(|m| m.as_str().parse::<f64>().ok())
            .sum::<f64>()
            .echo();
    }
}