use std::{env, path::PathBuf};

use rustc_version::{version_meta, Channel};

fn main() {
    println!("cargo:rerun-if-changed=src/ascii.h");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::Builder::default()
        .header("src/ascii.h")
        .blocklist_type("wchar_t")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("ascii.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=src/ascii.c");
    cc::Build::new().file("src/ascii.c").compile("ascii");

    if version_meta().unwrap().channel == Channel::Nightly {
        println!("cargo:rustc-cfg=feature=\"nightly\"");
    }
}
