use actix_web_admin::cli;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let bin_name = args.first().map(|s| {
        std::path::Path::new(s)
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "admin-cli".to_string())
    }).unwrap_or_else(|| "admin-cli".to_string());

    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    let result = match command {
        "createsuperuser" | "create" | "add" => cli::create_superuser_interactive(&args[2..]).await,
        "deleteuser" | "delete" | "remove" => cli::delete_user(&args[2..]).await,
        "listusers" | "list" | "ls" => cli::list_users(&args[2..]).await,
        "help" | "--help" | "-h" => {
            cli::print_help(&bin_name);
            Ok(())
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            cli::print_help(&bin_name);
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
