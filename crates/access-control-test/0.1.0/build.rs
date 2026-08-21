use std::{
    env,
    process::Command,
};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo:rerun-if-changed=Forc.toml");
    println!("cargo:rerun-if-changed=src/main.sw");
    println!("cargo:rerun-if-changed=../../libs/src/access_control.sw");

    let status = Command::new("forc")
        .args(["build", "--path", &manifest_dir])
        .status()
        .expect("failed to run forc build for access_control_test");

    assert!(
        status.success(),
        "forc build failed for access_control_test"
    );
}
