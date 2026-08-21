fn main() {
    eprintln!(
        "Warning: 'createsuperuser' is deprecated. Use 'admin-cli createsuperuser' instead."
    );
    let args: Vec<String> = std::env::args().collect();
    let bin_path = args.first().map(|s| {
        let p = std::path::Path::new(s);
        p.parent().unwrap_or_else(|| std::path::Path::new(""))
            .join("admin-cli")
    }).unwrap_or_else(|| std::path::PathBuf::from("admin-cli"));

    let mut cmd = std::process::Command::new(&bin_path);
    cmd.arg("createsuperuser");
    for arg in &args[1..] {
        cmd.arg(arg);
    }

    match cmd.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("Error executing admin-cli: {}", e);
            std::process::exit(1);
        }
    }
}
