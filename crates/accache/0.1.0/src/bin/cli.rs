use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::str::FromStr;

use accache::format::{Entry, read_file, write_file};
use accache::{Accache, AccacheError, CommitmentConfig, Compression, Pubkey, RefreshPolicy};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::json;

#[derive(Parser, Debug)]
#[command(
    name = "accache",
    version,
    about = "Fetch, cache, and persist Solana account data"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Fetch one or more accounts from RPC and write them to a `.acc` file.
    Fetch(FetchArgs),
    /// Re-fetch all accounts in an existing `.acc` file from RPC.
    Refresh(RefreshArgs),
    /// List the contents of a `.acc` file as a table.
    List(FileArg),
    /// Show details for a single account inside a `.acc` file.
    Inspect(InspectArgs),
    /// Merge two or more `.acc` files; later files win on conflict.
    Merge(MergeArgs),
    /// Export a `.acc` file to JSON or test-validator-compatible per-account files.
    Export(ExportArgs),
}

#[derive(Args, Debug)]
struct FetchArgs {
    /// Account pubkeys (base58). Accepts space-separated and/or comma-separated lists.
    #[arg(required = true, value_delimiter = ',')]
    pubkeys: Vec<String>,
    /// RPC endpoint URL.
    #[arg(long)]
    rpc: String,
    /// Output `.acc` file path.
    #[arg(long, short)]
    out: PathBuf,
    /// Commitment level.
    #[arg(long, default_value = "confirmed")]
    commitment: CommitmentArg,
    /// Disable zstd compression (default is zstd level 3).
    #[arg(long)]
    no_compress: bool,
}

#[derive(Args, Debug)]
struct RefreshArgs {
    /// Existing `.acc` file to refresh.
    file: PathBuf,
    /// RPC endpoint URL.
    #[arg(long)]
    rpc: String,
    /// Output path. Defaults to overwriting the input file.
    #[arg(long, short)]
    out: Option<PathBuf>,
    #[arg(long, default_value = "confirmed")]
    commitment: CommitmentArg,
    #[arg(long)]
    no_compress: bool,
}

#[derive(Args, Debug)]
struct FileArg {
    file: PathBuf,
}

#[derive(Args, Debug)]
struct InspectArgs {
    file: PathBuf,
    pubkey: String,
    /// How to render account data.
    #[arg(long, default_value = "none")]
    data: DataFormat,
}

#[derive(Args, Debug)]
struct MergeArgs {
    /// Two or more `.acc` files. Later files override earlier ones on conflict.
    #[arg(required = true, num_args = 2..)]
    files: Vec<PathBuf>,
    #[arg(long, short)]
    out: PathBuf,
    #[arg(long)]
    no_compress: bool,
}

#[derive(Args, Debug)]
struct ExportArgs {
    file: PathBuf,
    /// Output format.
    #[arg(long, default_value = "json")]
    format: ExportFormat,
    /// Output path. For `json`, a single file. For `test-validator`, a directory.
    #[arg(long, short)]
    out: PathBuf,
}

#[derive(Clone, Debug, ValueEnum)]
enum CommitmentArg {
    Processed,
    Confirmed,
    Finalized,
}

impl From<CommitmentArg> for CommitmentConfig {
    fn from(c: CommitmentArg) -> Self {
        match c {
            CommitmentArg::Processed => CommitmentConfig::processed(),
            CommitmentArg::Confirmed => CommitmentConfig::confirmed(),
            CommitmentArg::Finalized => CommitmentConfig::finalized(),
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
enum DataFormat {
    None,
    Hex,
    Base64,
}

#[derive(Clone, Debug, ValueEnum)]
enum ExportFormat {
    Json,
    TestValidator,
}

fn main() -> ExitCode {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .try_init();
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            match e {
                AccacheError::Io(_) | AccacheError::InvalidFile { .. } => ExitCode::from(2),
                _ => ExitCode::from(1),
            }
        }
    }
}

fn run(cli: Cli) -> accache::Result<()> {
    match cli.command {
        Cmd::Fetch(a) => cmd_fetch(a),
        Cmd::Refresh(a) => cmd_refresh(a),
        Cmd::List(a) => cmd_list(&a.file),
        Cmd::Inspect(a) => cmd_inspect(a),
        Cmd::Merge(a) => cmd_merge(a),
        Cmd::Export(a) => cmd_export(a),
    }
}

fn parse_pubkey(s: &str) -> accache::Result<Pubkey> {
    Pubkey::from_str(s).map_err(|e| AccacheError::InvalidPubkey(format!("{s}: {e}")))
}

fn compression_from(no_compress: bool) -> Compression {
    if no_compress {
        Compression::None
    } else {
        Compression::default()
    }
}

fn cmd_fetch(args: FetchArgs) -> accache::Result<()> {
    let pubkeys: Vec<Pubkey> = args
        .pubkeys
        .iter()
        .map(|s| parse_pubkey(s))
        .collect::<accache::Result<_>>()?;
    let acc = Accache::builder()
        .with_rpc(args.rpc)
        .commitment(args.commitment.into())
        .compression(compression_from(args.no_compress))
        .refresh(RefreshPolicy::Always)
        .build()?;
    let _ = acc.get_multiple(&pubkeys)?;
    acc.write_to(&args.out)?;
    println!(
        "wrote {} account(s) to {}",
        pubkeys.len(),
        args.out.display()
    );
    Ok(())
}

fn cmd_refresh(args: RefreshArgs) -> accache::Result<()> {
    let acc = Accache::builder()
        .with_files([&args.file])
        .with_rpc(args.rpc)
        .commitment(args.commitment.into())
        .compression(compression_from(args.no_compress))
        .build()?;
    acc.refresh_all()?;
    let target = args.out.as_deref().unwrap_or(&args.file);
    acc.write_to(target)?;
    println!("refreshed {} account(s) → {}", acc.len(), target.display());
    Ok(())
}

fn cmd_list(file: &Path) -> accache::Result<()> {
    let (header, entries) = read_file(file)?;
    println!(
        "{:<44} {:<44} {:>15} {:>10} {:>5}",
        "key", "owner", "lamports", "data_len", "exec"
    );
    for e in &entries {
        let pk = e.pubkey();
        let owner = Pubkey::new_from_array(e.owner);
        println!(
            "{:<44} {:<44} {:>15} {:>10} {:>5}",
            pk.to_string(),
            owner.to_string(),
            e.lamports,
            e.data.len(),
            e.executable,
        );
    }
    println!(
        "\n{} entries · version {} · {}",
        header.count,
        header.version,
        if header.is_zstd() {
            "zstd"
        } else {
            "uncompressed"
        }
    );
    Ok(())
}

fn cmd_inspect(args: InspectArgs) -> accache::Result<()> {
    let target = parse_pubkey(&args.pubkey)?;
    let (_, entries) = read_file(&args.file)?;
    let entry = entries
        .into_iter()
        .find(|e| e.pubkey() == target)
        .ok_or(AccacheError::NotFound(target))?;
    println!("key:        {}", entry.pubkey());
    println!("owner:      {}", Pubkey::new_from_array(entry.owner));
    println!("lamports:   {}", entry.lamports);
    println!("executable: {}", entry.executable);
    println!("rent_epoch: {}", entry.rent_epoch);
    println!("data_len:   {}", entry.data.len());
    if let Some(t) = entry.fetched_at_unix_ms {
        println!("fetched_at: {} ms (unix)", t);
    }
    match args.data {
        DataFormat::None => {}
        DataFormat::Hex => println!("data:       {}", hex::encode(&entry.data)),
        DataFormat::Base64 => println!("data:       {}", base64_encode(&entry.data)),
    }
    Ok(())
}

fn cmd_merge(args: MergeArgs) -> accache::Result<()> {
    let mut by_key: indexmap::IndexMap<[u8; 32], Entry> = indexmap::IndexMap::new();
    for path in &args.files {
        let (_, entries) = read_file(path)?;
        for e in entries {
            by_key.insert(e.key, e);
        }
    }
    let merged: Vec<Entry> = by_key.into_values().collect();
    let count = merged.len();
    write_file(&args.out, &merged, compression_from(args.no_compress))?;
    println!(
        "merged {} unique account(s) → {}",
        count,
        args.out.display()
    );
    Ok(())
}

fn cmd_export(args: ExportArgs) -> accache::Result<()> {
    let (_, entries) = read_file(&args.file)?;
    match args.format {
        ExportFormat::Json => {
            let arr: Vec<_> = entries.iter().map(entry_to_json).collect();
            std::fs::create_dir_all(args.out.parent().unwrap_or(Path::new(".")))?;
            std::fs::write(&args.out, serde_json::to_vec_pretty(&arr)?)?;
            println!("wrote JSON dump → {}", args.out.display());
        }
        ExportFormat::TestValidator => {
            std::fs::create_dir_all(&args.out)?;
            for e in &entries {
                let pk = e.pubkey();
                let body = test_validator_json(e);
                let path = args.out.join(format!("{pk}.json"));
                std::fs::write(&path, serde_json::to_vec_pretty(&body)?)?;
            }
            println!(
                "wrote {} test-validator file(s) → {}",
                entries.len(),
                args.out.display()
            );
        }
    }
    Ok(())
}

fn entry_to_json(e: &Entry) -> serde_json::Value {
    json!({
        "pubkey": e.pubkey().to_string(),
        "lamports": e.lamports,
        "owner": Pubkey::new_from_array(e.owner).to_string(),
        "executable": e.executable,
        "rent_epoch": e.rent_epoch,
        "data_base64": base64_encode(&e.data),
        "fetched_at_unix_ms": e.fetched_at_unix_ms,
    })
}

/// Match the schema used by `solana account --output json`, which is the format
/// accepted by `solana-test-validator --account <PUBKEY> <FILE>`.
fn test_validator_json(e: &Entry) -> serde_json::Value {
    json!({
        "pubkey": e.pubkey().to_string(),
        "account": {
            "lamports": e.lamports,
            "data": [base64_encode(&e.data), "base64"],
            "owner": Pubkey::new_from_array(e.owner).to_string(),
            "executable": e.executable,
            "rentEpoch": e.rent_epoch,
            "space": e.data.len(),
        }
    })
}

fn base64_encode(bytes: &[u8]) -> String {
    // Tiny, dep-free base64 encoder.
    const ALPHA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    let mut chunks = bytes.chunks_exact(3);
    for c in chunks.by_ref() {
        let n = ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32);
        out.push(ALPHA[(n >> 18 & 0x3F) as usize] as char);
        out.push(ALPHA[(n >> 12 & 0x3F) as usize] as char);
        out.push(ALPHA[(n >> 6 & 0x3F) as usize] as char);
        out.push(ALPHA[(n & 0x3F) as usize] as char);
    }
    let rem = chunks.remainder();
    match rem.len() {
        0 => {}
        1 => {
            let n = (rem[0] as u32) << 16;
            out.push(ALPHA[(n >> 18 & 0x3F) as usize] as char);
            out.push(ALPHA[(n >> 12 & 0x3F) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let n = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
            out.push(ALPHA[(n >> 18 & 0x3F) as usize] as char);
            out.push(ALPHA[(n >> 12 & 0x3F) as usize] as char);
            out.push(ALPHA[(n >> 6 & 0x3F) as usize] as char);
            out.push('=');
        }
        _ => unreachable!(),
    }
    out
}
