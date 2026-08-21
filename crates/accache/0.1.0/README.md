# accache

Fetch, cache, and persist Solana account data for in-process tests.

`accache` is the missing piece between an RPC and a [`mollusk-svm`](https://crates.io/crates/mollusk-svm) / [`litesvm`](https://crates.io/crates/litesvm) test. Pull accounts from RPC once, persist them to a small compressed `.acc` file, and load them back instantly in every subsequent test run — no network, no flake, no `getMultipleAccounts` quota.

- Hybrid sources: RPC, file(s), or both (file fall-through to RPC on miss)
- `RefreshPolicy::{Offline, OnMiss, Always, Ttl}`
- Compact, versioned `.acc` format (zstd-compressed, atomic writes)
- Sync API at the crate root; async mirror under `accache::nonblocking`
- CLI (`accache fetch / refresh / list / inspect / merge / export`)

## Install

Library:

```toml
[dev-dependencies]
accache = "0.1"
```

Default features are `rpc` (blocking RPC client) and `cli`. Disable defaults if you only want offline file loading:

```toml
[dev-dependencies]
accache = { version = "0.1", default-features = false }
```

| Feature        | Pulls in                                           |
|----------------|----------------------------------------------------|
| `rpc` *(default)* | `solana-rpc-client` (blocking)                  |
| `nonblocking`  | `tokio` + `solana-rpc-client::nonblocking`         |
| `cli` *(default)* | `clap`, `env_logger`, `hex` (for the binary)    |

CLI:

```bash
cargo install accache
```

## Library

### Fetch from RPC, cache to disk

```rust
use accache::{Accache, RefreshPolicy};

let pubkeys = [
    solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
    solana_pubkey::pubkey!("So11111111111111111111111111111111111111112"),
];

let acc = Accache::builder()
    .with_rpc("https://api.mainnet-beta.solana.com")
    .outfile("tests/fixtures/tokens.acc")
    .build()?;

let _ = acc.get_multiple(&pubkeys)?;
acc.flush()?; // writes the .acc file
```

### Load from a fixture, hand to mollusk-svm

```rust
use accache::{Accache, Account, Pubkey, RefreshPolicy};

let acc = Accache::builder()
    .with_files(["tests/fixtures/tokens.acc"])
    .refresh(RefreshPolicy::Offline)
    .build()?;

// `KeyedAccount` -> `(Pubkey, Account)` is what mollusk's accounts slice expects.
let accounts: Vec<(Pubkey, Account)> = acc
    .get_multiple(&pubkeys)?
    .into_iter()
    .map(Into::into)
    .collect();

let mollusk = mollusk_svm::Mollusk::new(&program_id, "target/deploy/program");
mollusk.process_and_validate_instruction(&ix, &accounts, &[Check::success()]);
```

### Hybrid: file fixtures + RPC fall-through

Pre-loaded accounts come from disk; misses fall through to RPC and the cache is re-persisted on `flush()`:

```rust
let acc = Accache::builder()
    .with_files(["tests/fixtures/seed.acc"])
    .with_rpc("https://api.mainnet-beta.solana.com")
    .outfile("tests/fixtures/seed.acc")          // overwrite-in-place
    .auto_persist(true)                          // write after every mutation
    .build()?;
```

### Refresh policies

| Policy            | When `get(key)` hits RPC                               |
|-------------------|--------------------------------------------------------|
| `Offline`         | Never. Misses return `AccacheError::Offline`.          |
| `OnMiss` *(default)* | When `key` is not cached.                            |
| `Always`          | Every call; cache acts only as a write-through buffer. |
| `Ttl(Duration)`   | When the cached entry is older than `Duration`.        |

### Async

Identical surface, async fns, behind the `nonblocking` feature:

```rust
use accache::nonblocking::Accache;

let acc = Accache::builder()
    .with_rpc("https://api.mainnet-beta.solana.com")
    .build_nonblocking()?;
let keyed = acc.get(&pk).await?;
```

### `KeyedAccount` ↔ mollusk

```rust
let keyed: KeyedAccount = /* ... */;

let one: (Pubkey, Account) = keyed.as_tuple();             // borrow
let many: Vec<(Pubkey, Account)> = vec![keyed].into_iter()
    .map(Into::into).collect();                            // owned
```

## CLI

```bash
accache fetch <PUBKEY>... --rpc <URL> --out <FILE> [--commitment processed|confirmed|finalized] [--no-compress]
accache refresh <FILE> --rpc <URL> [--out <FILE>]
accache list <FILE>
accache inspect <FILE> <PUBKEY> [--data none|hex|base64]
accache merge <FILE>... --out <FILE>
accache export <FILE> --format json|test-validator --out <PATH>
```

Pubkeys for `fetch` accept space-separated, comma-separated, or a mix:

```bash
accache fetch \
    TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA,So11111111111111111111111111111111111111112 \
    --rpc https://api.mainnet-beta.solana.com \
    --out tests/fixtures/tokens.acc

accache list tests/fixtures/tokens.acc
# key                                          owner                                          lamports   data_len exec
# So11111111111111111111111111111111111111112  ...                                              1000000   82       false
# ...

accache inspect tests/fixtures/tokens.acc \
    TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA --data hex
```

`export --format test-validator` writes one JSON file per account in the schema accepted by `solana-test-validator --account <PUBKEY> <FILE>` — useful for handing fixtures to a local validator instead of an in-process SVM.

## `.acc` format

```
+----------------------------------+
| Magic       "ACC1"   (4 bytes)   |
| Version     u16 (LE) = 1         |
| Flags       u16 (LE)             |  bit 0: zstd-compressed payload
| Count       u32 (LE)             |
| Reserved    [u8; 4]              |
+----------------------------------+
| Payload (optionally zstd-framed) |
|   bincode(Vec<Entry>)            |
+----------------------------------+
```

Each `Entry` mirrors `solana_account::Account` plus an optional `fetched_at_unix_ms` timestamp consumed by `RefreshPolicy::Ttl`. Writes are atomic (tempfile + rename) so a crashed write never corrupts an existing fixture.

## License

MIT OR Apache-2.0
