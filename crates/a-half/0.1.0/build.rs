extern crate cc;

fn main() {
    cc::Build::new()
        .file("cbits/half.c")
        .compile("half");
}
