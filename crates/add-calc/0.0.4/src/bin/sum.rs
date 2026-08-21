use ext::{no_std::pipelines::{pipe::Pipe, tap::Tap}, standard::functions::{ext::StdAnyExt, fun::read_line}};
use regex_automata::nfa::thompson::pikevm::PikeVM;
use std::process;
fn main() {
    let re = PikeVM::new(r"([+-]?(?:\d+\.?\d*|\.\d+))").unwrap();
    let mut cache = re.create_cache();
    loop {
        read_line()
            .trim_end()
            .tap(|s| if s.is_empty() { process::exit(0) })
            .pipe(|s| re.find_iter(&mut cache, s).filter_map(|m| s[m.range()].parse::<f64>().ok()))
            .sum::<f64>()
            .echo();
    }
}