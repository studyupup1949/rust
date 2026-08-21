use clap::Parser;

fn main() {
    let args = ac_qu_ai_nt_cli_clap::Args::parse();

    ac_qu_ai_nt_cli_clap::main(args);
}
