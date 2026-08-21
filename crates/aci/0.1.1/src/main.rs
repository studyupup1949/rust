#[tokio::main]
async fn main() {
    match aci::run_cli(std::env::args().skip(1)).await {
        Ok(output) => {
            if !output.stdout.is_empty() {
                print!("{}", output.stdout);
            }
            if !output.stderr.is_empty() {
                eprint!("{}", output.stderr);
            }
            if output.exit_code != 0 {
                std::process::exit(output.exit_code);
            }
        }
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "code": "ACI_ERROR",
                    "message": error.to_string(),
                }))
                .unwrap_or_else(|_| "{\"code\":\"ACI_ERROR\",\"message\":\"unknown\"}".to_string())
            );
            std::process::exit(2);
        }
    }
}
