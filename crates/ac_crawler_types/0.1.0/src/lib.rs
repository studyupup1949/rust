use std::collections::HashMap;

use mungos::ObjectId;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub mod traits;

pub type LatencyMap = HashMap<String, i64>;

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
    pub ts: i64,  // timestamp in ms (unique)
    pub mp: f64,  // mark price
    pub ip: f64,  // index price
    pub esp: f64, // estimated settle price
    pub fr: f64,  // funding rate
    pub nft: i64, // next funding time
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
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TradeMsg {
    pub ts: i64,         // timestamp in ms
    pub side: TradeSide, // buy or sell
    pub price: f64,      // trade price
    pub q_base: f64,     // base quantity
    pub q_quote: f64,    // quote quantity
    pub trd_id: String,  // trade id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q_contract: Option<f64>, // Number of contracts, always None for Spot
    #[serde(default)]
    pub lat: LatencyMap, // latency data
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TradeSide {
    Buy,
    Sell,
}

impl Default for TradeSide {
    fn default() -> Self {
        TradeSide::Buy
    }
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CrawlerType {
    L2Events,
    L2Snapshot,
    L2Topk,
    Trades,
    Candlestick,
    Ticker,
    FundingRate,
    Bbo,
}

impl Default for CrawlerType {
    fn default() -> Self {
        CrawlerType::L2Events
    }
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Exchange {
    Binance,
    Huobi,
    Bybit,
}

impl Default for Exchange {
    fn default() -> Self {
        Exchange::Binance
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Display, Debug, EnumString, PartialEq, Hash, Eq)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum MarketType {
    Spot,
    LinearFuture,
    InverseFuture,
    LinearSwap,
    InverseSwap,
}

impl Default for MarketType {
    fn default() -> Self {
        MarketType::Spot
    }
}
