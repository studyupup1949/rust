extern crate cc;

fn main() {
    cc::Build::new().flag("-ffreestanding")
                    .file("src/abort.c")
                    .compile("abort");
}
