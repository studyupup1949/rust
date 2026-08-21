use atty::Stream;

pub mod codons;
pub mod matrix;
pub mod writer;

// check if there is anything coming from stdin
pub fn is_stdin() -> bool {
    !atty::is(Stream::Stdin)
}
