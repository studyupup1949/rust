# Grant 2 Milestone 4 Testing Guide

To run the unit tests:

```
git clone https://github.com/hybrid-explorer/hybrid-indexer
cd hybrid-indexer
git checkout milestone-2.4
cargo test

git clone https://github.com/hybrid-explorer/hybrid-api-rs
cd hybrid-api-rs
git checkout milestone-2.4
cargo test
```


Build the polkadot-indexer docker image (this will take 20+ mins):

```
git clone https://github.com/hybrid-explorer/polkadot-indexer/
cd polkadot-indexer
git checkout milestone-2.4
docker build .
```

Enter the docker image (replace [image_hash] with the hash outputted from the build):

```
docker run -p 8172:8172 -it [image_hash] /bin/bash
```

Run the indexer from within docker:

```
./target/release/polkadot-indexer -i --queue-depth 1
```

Due to rate limiting, indexing public endpoints must have a depth queue of 1 and is therefore much slower than indexing a local node. Indexing a local node has been observed at 1,500 blocks per second with a higher queue depth.

In a separate terminal, install hybrid-cli:

```
cargo +nightly install hybrid-cli
```

## Deliverable 1,2 - Status subscription, unsubscribing

Run the following command:

```
hybrid --url ws://127.0.0.1:8172 subscribe-status
```

Observe that when a new head block is indexed a list of indexed spans of blocks is outputted. Press ctrl+c to stop. It will unsubscribe before exiting.

Run the following command:

```
hybrid --url ws://127.0.0.1:8172 subscribe-events variant -p 0 -v 0
```

Observe that when a new block is indexed the block number and event index of any ExtrinsicSuccess events is outputted. Press ctrl+c to stop. It will unsubscribe before exiting.

## Deliverable 3 - Report each index size

Run the following command:

```
hybrid --url ws://127.0.0.1:8172 size-on-disk
```

Observe that the size on disk is outputted.

## Deliverable 4 - Rust API

The source code can be found here: https://github.com/hybrid-explorer/hybrid-api-rs/tree/milestone-2.4

The documentation can be found here: https://docs.rs/hybrid-api/latest/hybrid_api/

The Hybrid CLI source code shows how to use it: https://github.com/hybrid-explorer/hybrid-cli/blob/master/src/main.rs
