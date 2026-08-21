use acme_dns_client::{AcmeDnsClient, Credentials};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "acme-dns-cli")]
#[command(about = "Tiny CLI to test an acme-dns server")]
struct Cli {
    /// Base URL of the acme-dns API, e.g. https://auth.example.org/
    #[arg(long, env = "ACME_DNS_API_BASE")]
    api_base: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Call /register and print the resulting credentials as JSON.
    Register {
        /// CIDR networks allowed to call /update (comma-separated or repeated).
        #[arg(long, value_delimiter = ',')]
        allowfrom: Option<Vec<String>>,
    },

    /// Call /update using credentials from environment.
    ///
    /// Uses ACME_DNS_USERNAME, ACME_DNS_PASSWORD, ACME_DNS_SUBDOMAIN,
    /// ACME_DNS_FULLDOMAIN for credentials, and ACME_DNS_ALLOWFROM optional.
    Update {
        /// TXT value to set for the challenge.
        #[arg(long)]
        txt: String,
    },

    /// Call /health and print result.
    Health,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let client = AcmeDnsClient::new(&cli.api_base)?;

    match cli.command {
        Command::Register { allowfrom } => {
            let allow_ref = allowfrom.as_ref().map(|v| v.as_slice());
            let creds = client.register(allow_ref).await?;
            println!("{}", serde_json::to_string_pretty(&creds)?);
        }

        Command::Update { txt } => {
            let creds = Credentials::from_env()?;
            client.update_txt(&creds, &txt).await?;
            println!("update OK for {}", creds.fulldomain);
        }

        Command::Health => {
            client.health().await?;
            println!("health OK");
        }
    }

    Ok(())
}
