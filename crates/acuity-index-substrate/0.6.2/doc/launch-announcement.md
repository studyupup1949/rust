# Hybrid Indexer Launch Announcement

[Hybrid Index](https://github.com/hybrid-explorer/hybrid-indexer) is a Rust software library for indexing events in [Substrate](https://substrate.io/) blockchains. It stores the bare minimum in its database to identify events containing certain parameters, e.g. `AccountId`, and should be used in tandem with a light client or full node to actually retrieve event contents. This is why it is called "Hybrid".

Hybrid can either be used in a decentralized or centralized manner:

* decentralized - A dapp can run the indexer directly on the user's device. It can be configured to only index certain block ranges and event parameters to minimize resource usage such as time, bandwidth and storage space.

* centralized - The indexer can be run in a data center. Dapps can use it for chain queries that are not possible via light client or WSS connection to a full node.

Currently hybrid connects directly to a full node to index it. In a later version it will be able to index a chain via a light client. 

Development of this tool was funded by a [grant](https://github.com/w3f/Grants-Program/blob/master/applications/hybrid.md) from the [Web3 Foundation](https://web3.foundation/). The Web3 Foundation funds research and development teams building the technology stack of the decentralized web. It was established in Zug, Switzerland by Ethereum co-founder and former CTO Gavin Wood. Polkadot is the Foundation's flagship project.

## Architecture

![Hybrid Architecture](https://raw.githubusercontent.com/ethernomad/hybrid-diagram/main/hybrid.png)

A Hybrid indexer binary has to be built for each chain type in a similar manner to how a full node binary using the Substrate library has to be built. For example, [Polkadot Indexer](https://github.com/hybrid-explorer/polkadot-indexer) indexes events on chains supported by the polkadot binary (Polkadot, Kusama, Rococo and Westend).

It reads events in blocks using [subxt](https://github.com/paritytech/subxt) and indexes these events in a key-value database using the [sled](http://sled.rs/) library. This is considerably more efficient than storing the index in an SQL database.

Events that have identifying parameters will be indexed. For example the Transfer event in the Balances pallet is identifiable by the `AccountId` of both `from` and `to`.

Hybrid has built-in indexing macros for the following Substrate pallets: System, Preimage, Indices, Balances, Transaction Payment, Staking, Session, Democracy, Collective, Elections Phragmen, Treasury, Vesting, Identity, Proxy, Multisig, Fast Unstake, Election Provider Multi-phase, Tips, Bounties, Child Bounties, Bags List, Nomination Pools.

Hybrid currently supports indexing of the following event parameters: `AccountId`, `AccountIndex`, `AuctionIndex`, `BountyIndex`, `CandidateHash`, `EraIndex`, `MessageId`, `ParaId`, `PoolId`, `PreimageHash`, `ProposalHash`, `RefIndex`, `RegistrarIndex`, `SessionIndex`, `TipHash`.

Additionally, all events are indexed by event variant. This means that, for example, a list of all balance transfers for all accounts can be obtained. 

To index a block, first a query has to be made to determine the hash from the block number. Then a second query for the metadata version. Finally the block itself is downloaded. In order to ensure throughput is as high as possible, multiple blocks are indexed simultaneously to counteract the round-trip delay.

When a chain is going to potentially perform a runtime upgrade, the Hybrid Indexer for the chain will need a new release with any updated events. If an instance of the indexer is not updated before the runtime upgrade occurs, it can be restarted with the new version at the correct block number.

WSS queries are handled via the highly scalable [tokio_tungstenite](https://github.com/snapview/tokio-tungstenite) Rust library.

It is possible to subscribe to queries so that the dapp will be altered as soon as the event is emitted.

The database keys are constructed in such a way that events can be found using iterators starting at a specific block number. For example, for for the AccountId keyspace:

`AccountId/BlockNumber/EventIndex`

Database entries are key-only. No value is stored. The block number and event index are all that need to be returned for each event found. This reduces the size of the index database and increases decentralization. The frontend can query the chain in a decentralized manner to retrieve the event.

Read the [Tutorial](https://github.com/hybrid-explorer/hybrid-indexer/blob/main/doc/tutorial.md) to learn how to index your Substrate chain with Hybrid. 

## Hybrid Dapp

![Hybrid Dapp](https://raw.githubusercontent.com/hybrid-explorer/hybrid-indexer/main/doc/hybrid-dapp.png)

[Hybrid Dapp](https://github.com/hybrid-explorer/hybrid-dapp) is a rudimentary frontend to instances of Hybrid Indexer. Events can be searched by event parameters and event types. It can be configured to support multiple chains.

In the future, a hosted Substrate block explorer will be launched based on Hybrid Dapp.

## Web3 Foundation

Learn more about the Web3 Foundation by visiting their [website](https://web3.foundation/), and stay up to date with the latest developments by following them on [Medium](https://medium.com/web3foundation) or [Twitter](https://twitter.com/web3foundation).
