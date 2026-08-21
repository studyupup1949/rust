use acuity_index_substrate_api::{Bytes32, Index, Key, SubstrateKey};
use anyhow::Result;
use byte_unit::{Byte, UnitType};
use clap::{Parser, Subcommand};
use futures_util::StreamExt;
use subxt::utils::AccountId32;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// URL of Acuity Substrate Index node to connect to
    #[arg(short, long)]
    pub url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Indexer status.
    Status,
    /// Subscribe to indexer status.
    SubscribeStatus,
    /// Query how much disk space is used by the index
    SizeOnDisk,
    /// Query for event variants the chain supports
    GetVariants,
    /// Query for events with a key
    GetEvents {
        #[command(subcommand)]
        command: KeyCommands,
    },
    /// Query for events with a key
    SubscribeEvents {
        #[command(subcommand)]
        command: KeyCommands,
    },
}

#[derive(Debug, Subcommand)]
enum KeyCommands {
    /// AccountId
    AccountId {
        /// Key to search for
        #[arg(short, long)]
        key: AccountId32,
    },
    /// AccountIndex
    AccountIndex {
        /// Key to search for
        #[arg(short, long)]
        key: u32,
    },
    /// BountyIndex
    BountyIndex {
        /// Key to search for
        #[arg(short, long)]
        key: u32,
    },
    /// EraIndex
    EraIndex {
        /// Key to search for
        #[arg(short, long)]
        key: u32,
    },
    /// MessageId
    MessageId {
        /// Key to search for
        #[arg(short, long)]
        key: Bytes32,
    },
    /// PoolId
    PoolId {
        /// Key to search for
        #[arg(short, long)]
        key: u32,
    },
    /// PreimageHash
    PreimageHash {
        /// Key to search for
        #[arg(short, long)]
        key: Bytes32,
    },
    /// ProposalHash
    ProposalHash {
        /// Key to search for
        #[arg(short, long)]
        key: Bytes32,
    },
    /// ProposalIndex
    ProposalIndex {
        /// Key to search for
        #[arg(short, long)]
        key: u32,
    },
    /// RefIndex
    RefIndex {
        /// Key to search for
        #[arg(short, long)]
        key: u32,
    },
    /// RegistrarIndex
    RegistrarIndex {
        /// Key to search for
        #[arg(short, long)]
        key: u32,
    },
    /// SessionIndex
    SessionIndex {
        /// Key to search for
        #[arg(short, long)]
        key: u32,
    },
    /// TipHash
    TipHash {
        /// Key to search for
        #[arg(short, long)]
        key: Bytes32,
    },
    /// Variant
    Variant {
        /// Key type to search for
        #[arg(short, long)]
        pallet_id: u8,
        /// Key to search for
        #[arg(short, long)]
        variant_id: u8,
    },
}

impl Into<Key> for KeyCommands {
    fn into(self) -> Key {
        match self {
            KeyCommands::AccountId { key } => {
                Key::Substrate(SubstrateKey::AccountId(Bytes32(key.0)))
            }
            KeyCommands::AccountIndex { key } => Key::Substrate(SubstrateKey::AccountIndex(key)),
            KeyCommands::BountyIndex { key } => Key::Substrate(SubstrateKey::BountyIndex(key)),
            KeyCommands::EraIndex { key } => Key::Substrate(SubstrateKey::EraIndex(key)),
            KeyCommands::MessageId { key } => Key::Substrate(SubstrateKey::MessageId(key)),
            KeyCommands::PoolId { key } => Key::Substrate(SubstrateKey::PoolId(key)),
            KeyCommands::PreimageHash { key } => Key::Substrate(SubstrateKey::PreimageHash(key)),
            KeyCommands::ProposalHash { key } => Key::Substrate(SubstrateKey::ProposalHash(key)),
            KeyCommands::ProposalIndex { key } => Key::Substrate(SubstrateKey::ProposalIndex(key)),
            KeyCommands::RefIndex { key } => Key::Substrate(SubstrateKey::RefIndex(key)),
            KeyCommands::RegistrarIndex { key } => {
                Key::Substrate(SubstrateKey::RegistrarIndex(key))
            }
            KeyCommands::SessionIndex { key } => Key::Substrate(SubstrateKey::SessionIndex(key)),
            KeyCommands::TipHash { key } => Key::Substrate(SubstrateKey::TipHash(key)),
            KeyCommands::Variant {
                pallet_id,
                variant_id,
            } => Key::Variant(pallet_id, variant_id),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut index = Index::connect(cli.url).await?;

    match cli.command {
        Commands::Status => {
            let spans = index.status().await?;
            for span in spans {
                println!("{}", span);
            }
        }
        Commands::SubscribeStatus => {
            let mut stream = index.subscribe_status().await?;
            loop {
                tokio::select! {
                    biased;

                    _ = tokio::signal::ctrl_c() => {
                        drop(stream);
                        break;
                    }
                    Some(spans) = stream.next() => {
                        println!("Indexed spans:");
                        for span in spans? {
                            println!("{}", span);
                        }
                    }
                }
            }

            println!("unsubscribing");
            index.unsubscribe_status().await?;
            println!("unsubscribed");
        }
        Commands::SizeOnDisk => {
            let size = index.size_on_disk().await?;
            println!(
                "Size on disk: {}",
                Byte::from_u64(size).get_appropriate_unit(UnitType::Binary)
            );
        }
        Commands::GetVariants => {
            let variants = index.get_variants().await;
            println!("Variants: {:?}", variants);
        }
        Commands::GetEvents { command } => {
            let events = index.get_events(command.into()).await?;
            for event in events {
                println!("{}", event);
            }
        }
        Commands::SubscribeEvents { command } => {
            let key: Key = command.into();
            let mut event_stream = index.subscribe_events(key.clone()).await?;
            loop {
                tokio::select! {
                    biased;

                    _ = tokio::signal::ctrl_c() => {
                        drop(event_stream);
                        break;
                    }
                    Some(events) = event_stream.next() => {
                        for event in events? {
                            println!("{}", event);
                        }
                    }
                }
            }

            println!("unsubscribing");
            index.unsubscribe_events(key).await?;
            println!("unsubscribed");
        }
    };

    Ok(())
}
