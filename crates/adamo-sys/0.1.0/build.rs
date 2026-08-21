// build.rs — locate the precompiled libadamo for the host target and emit
// the link flags the Rust compiler needs.
//
// Only aarch64-unknown-linux-gnu (Jetson / Ubuntu 22.04 arm64) is
// supported today. Other targets fail fast with a clear message.
//
// Resolution order:
//   1. $ADAMO_LIB_DIR (escape hatch for local dev and offline builds).
//   2. prebuilt/$TARGET/ inside this crate (the shipped artifact).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const SUPPORTED_TARGETS: &[&str] = &["aarch64-unknown-linux-gnu"];

fn main() {
    println!("cargo:rerun-if-env-changed=ADAMO_LIB_DIR");
    println!("cargo:rerun-if-changed=build.rs");

    // docs.rs can't link against a real libadamo. Short-circuit so doc
    // builds succeed without the binaries present.
    if env::var_os("DOCS_RS").is_some() {
        return;
    }

    let target = env::var("TARGET").expect("cargo sets TARGET");
    if !SUPPORTED_TARGETS.contains(&target.as_str()) && env::var_os("ADAMO_LIB_DIR").is_none() {
        panic!(
            "adamo-sys: target `{target}` is not currently supported. \
             Supported targets: {}. \
             Set ADAMO_LIB_DIR to override if you have a libadamo built for your host.",
            SUPPORTED_TARGETS.join(", ")
        );
    }

    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let lib_dir = resolve_lib_dir(&crate_dir, &target);
    println!("cargo:rerun-if-changed={}", lib_dir.display());

    let lib_path = pick_library(&lib_dir).unwrap_or_else(|| {
        panic!(
            "adamo-sys: no libadamo.so found in {}. Set ADAMO_LIB_DIR to override.",
            lib_dir.display()
        )
    });

    // Stage the library into OUT_DIR so the link-search path is stable
    // regardless of where the source libadamo actually lives.
    let staged = out_dir.join(lib_path.file_name().unwrap());
    fs::copy(&lib_path, &staged).expect("copy libadamo to OUT_DIR");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=dylib=adamo");
    // Absolute rpath so `cargo run` / `cargo test` work without
    // LD_LIBRARY_PATH. `rustc-link-arg` applies to any bin/example/test
    // that ends up depending on this crate. Downstream distributable
    // binaries should override with $ORIGIN (ship libadamo.so alongside
    // the executable).
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", out_dir.display());

    // Surface the staged library path to downstream crates via
    // DEP_ADAMO_LIB_DIR.
    println!("cargo:lib_dir={}", out_dir.display());
}

fn resolve_lib_dir(crate_dir: &Path, target: &str) -> PathBuf {
    if let Ok(custom) = env::var("ADAMO_LIB_DIR") {
        return PathBuf::from(custom);
    }
    crate_dir.join("prebuilt").join(target)
}

fn pick_library(dir: &Path) -> Option<PathBuf> {
    // Jetson / Linux: libadamo.so. macOS dev override via ADAMO_LIB_DIR
    // may land on libadamo.dylib — accept both.
    for name in ["libadamo.so", "libadamo.dylib"] {
        let p = dir.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
