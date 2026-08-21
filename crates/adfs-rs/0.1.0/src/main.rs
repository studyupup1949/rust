mod login;

use clap::{Args, Parser, Subcommand};
use login::login_command;

#[derive(Args)]
struct LoginArgs {
    #[arg(short, long)]
    username: String,
    #[arg(short, long)]
    target_role_arn: String,
    #[arg(short, long)]
    ad_role: String,
    #[arg(short, long)]
    role_session_name: String,
    password: String,
}

#[derive(Subcommand)]
enum AdfsSubcommand {
    Login(LoginArgs),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct AdfsCli {
    #[arg(short, long)]
    ad_url: String,

    #[arg(short, long, default_value = "/tmp/adfs-rs-creds")]
    temp_creds_file: String,

    #[clap(subcommand)]
    subcommand: AdfsSubcommand,
}

#[::tokio::main]
async fn main() {
    let cli_args = AdfsCli::parse();
    match cli_args.subcommand {
        AdfsSubcommand::Login(login_args) => {
            login_command(
                cli_args.ad_url,
                cli_args.temp_creds_file,
                login_args.username,
                login_args.password,
                login_args.target_role_arn,
                login_args.ad_role,
                login_args.role_session_name,
            )
            .await
        }
    };
}
