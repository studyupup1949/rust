use std::collections::HashMap;

use bson::{oid::ObjectId, serde_helpers::hex_string_as_object_id, Document};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub mod traits;

pub type LatencyMap = HashMap<String, i64>;

/// BufferMsg is exchange / stream agnostic shape requiring little to no parsing from the stream json messages
/// incoming messages are converted to this shape, stored in mongo, and parsed / moved downstream as free CPU allows
/// meant to be very lightweight to maximize crawler side throughput
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BufferMsg {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub msg: String,
    pub recv_ts: i64,
}

/// BufferMsg is exchange / stream agnostic shape requiring little to no parsing from the stream json messages
/// incoming messages are converted to this shape, stored in mongo, and parsed / moved downstream as free CPU allows
/// meant to be very lightweight to maximize crawler side throughput
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BufferMsgNoId {
    pub msg: String,
    pub recv_ts: i64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BboMsg {
    pub ts: i64, // timestamp in ms (event ts)
    pub bp: f64, // bid price
    pub bq: f64, // bid quantity in base asset
    pub ap: f64, // ask price
    pub aq: f64, // ask quantity in base asset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub up_id: Option<i64>, // update id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_ts: Option<i64>, // transaction ts (on binance)
    #[serde(default)]
    pub lat: LatencyMap, // latency data
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CandlestickMsg {
    #[serde(skip_serializing_if = "Option::is_none", rename = "_id")]
    pub id: Option<ObjectId>,
    pub ts: i64,        // period open timestamp
    pub candle_id: i64, // used to identify candle to modify
    pub open: f64,      // period open price
    pub high: f64,      // period high price
    pub low: f64,       // period low price
    pub close: f64,     // period close price
    pub count: i64,     // transaction count
    pub b_vol: f64,     // volume in base asset
    pub q_vol: f64,     // volume in quote asset
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FundingRateMsg {
    pub ts: i64,  // timestamp in ms (ideally unique)
    pub mp: f64,  // mark price
    pub ip: f64,  // index price
    pub esp: f64, // estimated settle price
    #[serde(default)]
    pub ft: i64, // funding time (ms)
    pub fr: f64,  // funding rate
    pub nft: i64, // next funding time (ms)
    #[serde(default)]
    pub nfr: f64, // next funding rate
    #[serde(default)]
    pub lat: LatencyMap, // latency data
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct L2EventMsg {
    pub ts: i64, // timestamp (event ts on binance)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_ts: Option<i64>, // transaction timestamp (on binance)
    pub asks: Vec<[f64; 2]>, // [price_level, qty]
    pub bids: Vec<[f64; 2]>, // [price_level, qty]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq_id: Option<i64>, // sequence id, used for checking data integrity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_seq_id: Option<i64>, // previous sequence id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f_seq_id: Option<i64>, // first seq id (binance)
    #[serde(default)]
    pub lat: LatencyMap, // latency data
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct L2SnapshotMsg {
    pub ts: i64,                 // timestamp in ms
    pub bids: Vec<[f64; 2]>,     // full orderbook [price_level, qty]
    pub asks: Vec<[f64; 2]>,     // full orderbook [price_level, qty]
    pub bid_bins: Vec<[f64; 2]>, // .1% price level bin aggregation. [price_level_open, qty]
    pub ask_bins: Vec<[f64; 2]>, // .1% price level bin aggregation. [price_level_open, qty]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq_id: Option<i64>, // update id
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct L2TopKMsg {
    pub ts: i64, // timestamp in ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_ts: Option<i64>, // transaction timestamp (on binance)
    pub bids: Vec<[f64; 2]>, // [price_level, qty]
    pub asks: Vec<[f64; 2]>, // [price_level, qty]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seq_id: Option<i64>, // sequence id, used for checking data integrity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_seq_id: Option<i64>, // previous sequence id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f_seq_id: Option<i64>, // first seq id (binance)
    #[serde(default)]
    pub lat: LatencyMap, // latency data
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TradeMsg {
    pub ts: i64, // timestamp in ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_ts: Option<i64>, // transaction time (binance)
    pub side: TradeSide, // buy or sell
    pub price: f64, // trade price
    pub q_base: f64, // base quantity
    pub q_quote: f64, // quote quantity
    pub trd_id: String, // trade id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f_trd_id: Option<String>, // first trade id (binance aggTrade)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l_trd_id: Option<String>, // last trade id (binance aggTrade)
    #[serde(default)]
    pub lat: LatencyMap, // latency data
}

#[derive(
    Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TradeSide {
    #[default]
    #[serde(alias = "Buy")]
    Buy,
    #[serde(alias = "Sell")]
    Sell,
}

#[derive(
    Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CrawlerType {
    #[default]
    #[strum(to_string = "l2_event", serialize = "depth")]
    L2Event,
    #[strum(to_string = "l2_topk", serialize = "depth20")]
    L2Topk,
    #[strum(to_string = "trade", serialize = "aggTrade")]
    Trade,
    #[strum(to_string = "funding_rate", serialize = "markPrice")]
    FundingRate,
    #[strum(to_string = "bbo", serialize = "bookTicker")]
    Bbo,
    Ticker,
    L2Snapshot,
    Candlestick,
}

#[derive(
    Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Exchange {
    #[default]
    Binance,
    Huobi,
    Bybit,
    Okx,
}

#[derive(
    Copy, Clone, Serialize, Deserialize, Display, Debug, EnumString, PartialEq, Hash, Eq, Default,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum MarketType {
    #[default]
    Spot,
    LinearFuture,
    InverseFuture,
    LinearSwap,
    InverseSwap,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IntegrityGap {
    #[serde(
        default,
        rename = "_id",
        skip_serializing_if = "String::is_empty",
        with = "hex_string_as_object_id"
    )]
    pub id: String,
    pub start_ts: i64, // the ts of the last datapoint before the gap
    pub end_ts: i64,   // the ts of first data after the gap
    pub last_json_before: Document,
    pub first_json_after: Document,
    pub crawler_type: CrawlerType,
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub symbol: String,
}

#[derive(Copy, Clone, Serialize, Deserialize, Display, Debug, EnumString, PartialEq, Hash, Eq)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum WsMode {
    Async,
    Sync,
}

impl Default for WsMode {
    fn default() -> Self {
        Self::Async
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MigrationFailure {
    pub app_mode: String,
    pub start_ts: i64,
    pub end_ts: i64,
    pub error_ts: i64,
    pub db: String,
    pub collection: String,
    pub error: String,
}
