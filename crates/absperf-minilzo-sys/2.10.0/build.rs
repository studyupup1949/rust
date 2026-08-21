fn main() {
    cc::Build::new()
        .file("ext/minilzo-2.10/minilzo.c")
        .compile("minilzo");
}
