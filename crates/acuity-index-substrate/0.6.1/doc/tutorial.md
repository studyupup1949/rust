# Tutorial

To learn how to build an indexer for a Substrate chain with Hybrid Indexer it is best to examine [Polkadot Indexer](https://github.com/hybrid-explorer/polkadot-indexer/).

Follow the subxt [instructions](https://github.com/paritytech/subxt#downloading-metadata-from-a-substrate-node) to download the metadata from the chain to be indexed:

```
subxt metadata --url <URL> > metadata.scale
```

It's generally a good idea to have the metadata in a separate workspace from the indexer. This avoids lengthly rebuilds during development.

'Cargo.toml'
```toml
[workspace]
resolver = "2"

members = [
  "metadata",
  "indexer",
]
```

'metadata/Cargo.toml'
```toml
[package]
name = "metadata"
version = "0.1.0"
edition = "2021"

[dependencies]
subxt = "0.32"
```

`metadata/src/lib/rs`
```rust
#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod metadata {}
```

Copy and modify the boilerplate `Cargo.toml` and `main.rs` from polkadot-indexer.

Create a struct that implements the subxt [Config](https://docs.rs/subxt/latest/subxt/config/trait.Config.html) trait. For example:

```rust
pub enum MyChainConfig {}

impl Config for MyChainConfig {
    type Hash = H256;
    type AccountId = AccountId32;
    type Address = MultiAddress<Self::AccountId, u32>;
    type Signature = MultiSignature;
    type Hasher = BlakeTwo256;
    type Header = SubstrateHeader<u32, BlakeTwo256>;
    type ExtrinsicParams = SubstrateExtrinsicParams<Self>;
}
```

Each chain to be indexed by the indexer implements the [RuntimeIndexer](https://docs.rs/hybrid-indexer/0.4.0/hybrid_indexer/shared/trait.RuntimeIndexer.html), [IndexKey](https://docs.rs/hybrid-indexer/0.4.0/hybrid_indexer/shared/trait.IndexKey.html) and [IndexTrees](https://docs.rs/hybrid-indexer/0.4.0/hybrid_indexer/shared/trait.IndexTrees.html) traits. For example, look at [PolkadotIndexer](https://github.com/hybrid-explorer/polkadot-indexer/blob/main/indexer/src/polkadot.rs#L46), [ChainKey](https://github.com/hybrid-explorer/polkadot-indexer/blob/main/indexer/src/main.rs#L62) and [ChainTrees](https://github.com/hybrid-explorer/polkadot-indexer/blob/54f5cdaf225e65cbcd0d5d962b68e92f5997b806/indexer/src/main.rs#L37).

Every event to be indexed is passed to `process_event()`. It needs to determine which pallet the event is from and use the correct macro to index it. Macros for Substrate pallets are provided by hybrid-indexer. Additional pallet macros can be provided.

```rust
#[derive(Clone, Debug)]
pub struct MyChainTrees {
    pub my_index: Tree,
}

impl IndexTrees for MyChainTrees {
    fn open(db: &Db) -> Result<Self, sled::Error> {
        Ok(MyChainTrees {
            my_index: db.open_tree(b"my_index")?,
        })
    }

    fn flush(&self) -> Result<(), sled::Error> {
        self.my_index.flush()?;
        Ok(())
    }
}
```

```rust
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
#[serde(tag = "type", content = "value")]
pub enum MyChainKey {
    MyKey(u32),
}

impl IndexKey for MyChainKey {
    type ChainTrees = MyChainTrees;

    fn write_db_key(
        &self,
        trees: &ChainTrees,
        block_number: u32,
        event_index: u16,
    ) -> Result<(), sled::Error> {
        let block_number = block_number.into();
        let event_index = event_index.into();
        match self {
            MyChainKey::MyKey(my_key) => {
                let key = U32Key {
                    key: (*my_key).into(),
                    block_number,
                    event_index,
                };
                trees.my_index.insert(key.as_bytes(), &[])?
            }
        };
        Ok(())
    }

    fn get_key_events(&self, trees: &ChainTrees) -> Vec<Event> {
        match self {
            MyChainKey::MyKey(my_key) => {
                get_events_u32(&trees.my_index, *my_key)
            }
        }
    }
}
```

```rust
pub struct PolkadotIndexer;

impl hybrid_indexer::shared::RuntimeIndexer for MyChainIndexer {
    type RuntimeConfig = MyChainConfig;
    type ChainKey = MyChainKey;

    fn get_name() -> &'static str {
        "mychain"
    }

    fn get_genesis_hash() -> <Self::RuntimeConfig as subxt::Config>::Hash {
        hex!["91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3"].into()
    }

    fn get_versions() -> &'static [u32] {
        &[0]
    }

    fn get_default_url() -> &'static str {
        "wss://rpc.mychain.io:443"
    }

    fn process_event(
        indexer: &hybrid_indexer::substrate::Indexer<Self>,
        block_number: u32,
        event_index: u32,
        event: subxt::events::EventDetails<Self::RuntimeConfig>,
    ) -> Result<(), subxt::Error> {
        match event.as_root_event::<Event>()? {
            // Substrate pallets.
            Event::Balances(event) => {
                index_balances_event![BalancesEvent, event, indexer, block_number, event_index]
            }
            // MyChain pallets.
            Event::MyPallet(event) => {
                index_mypallet_event![MyPalletEvent, event, indexer, block_number, event_index]
            }
            _ => {}
        };
        Ok(())
    }
```

Custom pallet indexer macros look something like this:

```rust
#[macro_export]
macro_rules! index_mypallet_event {
    ($event_enum: ty, $event: ident, $indexer: ident, $block_number: ident, $event_index: ident) => {
        match $event {
            <$event_enum>::MyEvent { who, my_key.. } => {
                $indexer.index_event(
                    Key::Substrate(SubstrateKey::AccountId(Bytes32(who.0))),
                    $block_number,
                    $event_index,
                )?;
                $indexer.index_event(
                    Key::Chain(MyChainKey::MyKey(my_key)),
                    $block_number,
                    $event_index,
                )?;
                2
            }
        }
    };
}
```

Examine the [API documentation](https://github.com/hybrid-explorer/hybrid-indexer/blob/main/doc/api.md) to help determine how to query the indexer.
