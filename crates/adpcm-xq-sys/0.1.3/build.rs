fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(feature = "nds") {
        cc::Build::new()
            .file("adpcm-xq-nds/adpcm-xq.c")
            .file("adpcm-xq-nds/adpcm-lib.c")
            .compile("adpcm-xq-nds");
    } else {
        cc::Build::new()
            .file("adpcm-xq/adpcm-xq.c")
            .file("adpcm-xq/adpcm-lib.c")
            .compile("adpcm-xq");
    }
    println!("cargo:rerun-if-changed=src/lib.c");
    Ok(())
}