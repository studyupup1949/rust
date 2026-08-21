fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(feature = "nds") {
        cc::Build::new()
            .file("adpcm-xq-nds/adpcm-dns.c")
            .file("adpcm-xq-nds/adpcm-xq.c")
            .file("adpcm-xq-nds/adpcm-lib.c")
            .compile("adpcm-xq-nds");

        // Rerun if any of the NDS source files change
        println!("cargo:rerun-if-changed=adpcm-xq-nds/adpcm-dns.c");
        println!("cargo:rerun-if-changed=adpcm-xq-nds/adpcm-xq.c");
        println!("cargo:rerun-if-changed=adpcm-xq-nds/adpcm-lib.c");
    } else {
        cc::Build::new()
            .file("adpcm-xq/adpcm-dns.c")
            .file("adpcm-xq/adpcm-xq.c")
            .file("adpcm-xq/adpcm-lib.c")
            .compile("adpcm-xq");

        // Rerun if any of the standard source files change
        println!("cargo:rerun-if-changed=adpcm-xq/adpcm-dns.c");
        println!("cargo:rerun-if-changed=adpcm-xq/adpcm-xq.c");
        println!("cargo:rerun-if-changed=adpcm-xq/adpcm-lib.c");
    }
    Ok(())
}