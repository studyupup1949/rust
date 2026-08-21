use std::env;
use std::process::{Command, exit};
use std::path::PathBuf;

fn main() {
    // 1. Verify Node.js runtime is present on the path
    let node_check = Command::new("node").arg("--version").output();
    if node_check.is_err() {
        eprintln!("\n❌ Error: Node.js runtime not found on your system path.");
        eprintln!("Architecture-As-Memory (AAM) requires Node.js (version 18 or above) to run.");
        eprintln!("Please install Node.js from https://nodejs.org/ and try again.\n");
        exit(1);
    }

    // 2. Resolve the core JS entrypoint
    let exe_path = env::current_exe().expect("Failed to locate running executable path");
    let base_dir = exe_path.parent().expect("Failed to locate executable base directory");
    
    // Look in sibling js/bin/aam.js
    let mut js_path = PathBuf::from(base_dir);
    js_path.push("js");
    js_path.push("bin");
    js_path.push("aam.js");

    if !js_path.exists() {
        // Fallback check to CARGO_MANIFEST_DIR to allow running cargo run -- inside our workspace
        if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            let mut dev_path = PathBuf::from(manifest_dir);
            dev_path.push("js");
            dev_path.push("bin");
            dev_path.push("aam.js");
            if dev_path.exists() {
                js_path = dev_path;
            }
        }
    }

    if !js_path.exists() {
        eprintln!("\n❌ Error: AAM core JavaScript module could not be located.");
        eprintln!("Please ensure the 'js' directory is placed alongside the running binary.\n");
        exit(1);
    }

    // 3. Collect and forward command line arguments
    let args: Vec<String> = env::args().skip(1).collect();

    // 4. Run the subprocess
    let mut process = Command::new("node")
        .arg(js_path)
        .args(&args)
        .spawn()
        .expect("Failed to execute Node.js process for AAM");

    let status = process.wait().expect("AAM process interrupted unexpectedly");

    exit(status.code().unwrap_or(1));
}
