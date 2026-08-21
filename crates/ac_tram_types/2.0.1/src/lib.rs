use std::collections::HashMap;

use async_timing_util::unix_timestamp_ms;
use mungos::{
    derive::{MungosIndexed, StringObjectId},
    mongodb::bson::{doc, serde_helpers::hex_string_as_object_id},
};
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use typeshare::typeshare;

pub mod legacy;

#[typeshare(serialized_as = "number")]
pub type I64 = i64;

#[typeshare]
pub type PriceMap = HashMap<String, f64>;
#[typeshare]
pub type BalancesMap = HashMap<String, AssetBalance>; // (asset_name, AssetBalance)
#[typeshare]
pub type AccountBalancesMap = HashMap<AccountType, BalancesMap>; // (account_type, BalancesMap)
#[typeshare]
pub type PositionsMap = HashMap<String, Position>;
#[typeshare]
pub type AccountPositionsMap = HashMap<AccountType, PositionsMap>;
#[typeshare]
pub type LatencyMap = HashMap<String, I64>; // (region, lat in ms)
#[typeshare]
pub type ExchangeSymbolsMap = HashMap<String, String>; // (exchange, symbol)

pub type ExchangeKeysMap = HashMap<Exchange, ExchangeApiKeys>;
pub type InjDenomMap = HashMap<String, (String, u32)>; // (denom, (symbol, decimals))

#[typeshare]
#[derive(Debug, Deserialize, Serialize, Clone, MungosIndexed)]
pub struct InternalSymbol {
    #[serde(default)]
    #[unique_index]
    pub symbol: String,
    #[serde(default)]
    pub exchange: ExchangeSymbolsMap,
}

#[typeshare]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenOrder {
    pub sid: String,             // subaccount id order was made from
    pub order_id: String,        // order id
    pub created_at: I64,         // order created timestamp
    pub symbol: String,          // trading symbol
    pub ins_type: String,        // instrument type
    pub side: TradeSide,         // buy or sell
    pub price: f64,              // order price
    pub filled_qty: f64,         // filled quantity
    pub filled_notional: f64,    // filled quantity usd notional
    pub remaining_qty: f64,      // remaining quantity
    pub remaining_notional: f64, // remaining quantity usd notional
    pub status: OrderStatus,     // order status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lat: Option<I64>, // latency in ms
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tif: Option<OrderTif>, // order time in force
}

#[typeshare]
#[derive(Debug, Deserialize, Serialize, Clone, MungosIndexed)]
#[unique_doc_index(doc! { "tid": 1, "oid": 1 })]
pub struct Fill {
    #[index]
    pub ts: I64, // exchange time message sent (unix timestamp milliseconds) (E on binance) (indexed)

    #[index]
    pub sid: String, // subaccount id (indexed)

    #[index]
    pub oid: String, // order id (indexed)

    #[index]
    pub tid: String, // transaction id (unique indexed)

    #[index]
    pub stat: OrderStatus, // order status

    #[index]
    pub side: TradeSide, // "buy" or "sell" (indexed)

    #[index]
    pub sym: String, // market symbol (indexed)

    #[index]
    pub ins: String, // instrument type (indexed)

    #[index]
    #[serde(default)]
    pub m: bool, // is maker

    #[index]
    #[serde(default)]
    pub tif: OrderTif, // time in force

    pub p: f64,   // price
    pub q: f64,   // filled quantity
    pub qn: f64,  // quantity notional
    pub fee: f64, // fee notional
    #[serde(default)]
    pub lat: LatencyMap, // latency in ms, recv_ts - sent_ts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_ts: Option<I64>, // transaction timestamp
}

#[typeshare]
#[derive(Debug, Default, Deserialize, Serialize, Clone, MungosIndexed)]
pub struct Balances {
    #[index]
    pub sid: String, // strategy id

    #[index]
    pub ts: I64, // timestamp of balance snapshot

    #[index]
    pub failure: bool, // did the recording fail

    pub notional: f64, // all the notionals added (eg spot, perp, on chain, staked, reward)
    pub balances: AccountBalancesMap, // full breakdown of account balances split by account type. eg { "spot": { "USDT": { "total": ... } }, "perp": { "USDT": { "total": ... } } }
    pub pos_notional: f64,            // total value of all positions on account
    pub positions: AccountPositionsMap, // full breakdown of positions split by market type. eg { "perp": { "BTCUSDT": { "total": ... } }, "future": { "ETHUSDT": { "total": ... } },  }
}

#[typeshare]
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct AssetBalance {
    pub total: f64,
    pub locked: f64,
    pub notional: f64,
}

#[typeshare]
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Position {
    pub total: f64,
    pub notional: f64,
    pub side: PositionSide,
    pub leverage: f64,
    pub maint_margin: f64,
}

#[typeshare]
#[derive(Debug, Default, Deserialize, Serialize, Clone, MungosIndexed, StringObjectId)]
pub struct BalanceTransfer {
    #[serde(
        default,
        rename = "_id",
        with = "hex_string_as_object_id",
        skip_serializing_if = "String::is_empty"
    )]
    pub id: String, // mongo _id

    #[index]
    pub sid: String, // strategy id

    #[index]
    pub ts: I64, // approx timestamp of transfer

    #[index]
    pub enabled: bool, // whether to use this transfer to adjust pnl

    pub amount: f64, // amount to adjust PnL by (notional USD). can be positive (transfer in), or negative (transfer out)

    // the fields below aren't functionally needed, but useful to keep track of
    #[serde(default)]
    #[index]
    pub other: String, // this is either source or destination account, depending on transfer in / out. Should be either the other sid or 'external'

    #[serde(default)]
    #[index]
    pub requester: String, // trader that requests the transfer

    #[serde(default)]
    #[index]
    pub sender: String, // user that executes the transfer

    #[serde(default)]
    #[index]
    pub symbol: String, // symbol transfered

    #[serde(default)]
    pub approvers: Vec<String>, // users that approve the transfer (for multi sig transactions)

    #[serde(default)]
    pub coin_amount: f64, // amount transferred notional

    #[serde(default)]
    pub link: Option<String>, // link to defi transaction for transfer
}

#[typeshare]
#[derive(Debug, Deserialize, Serialize, Clone, MungosIndexed, StringObjectId)]
pub struct Subaccount {
    #[serde(
        default,
        rename = "_id",
        with = "hex_string_as_object_id",
        skip_serializing_if = "String::is_empty"
    )]
    pub id: String,

    #[index]
    pub name: String,

    #[index]
    pub subaccount: String,

    #[index]
    pub team: String,

    #[index]
    pub portfolio: String,

    #[index]
    pub trader: String,

    #[index]
    pub exchange: Exchange,

    #[index]
    pub enabled: bool,

    #[serde(default)]
    pub created_at: I64,
    #[serde(default)]
    pub updated_at: I64,

    #[serde(default)]
    #[index]
    pub client_funds: bool,

    #[serde(default)]
    pub symbols: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_staked: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub binance_spot: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub binance_spot_margin: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub binance_usdmf: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bybit_spot: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bybit_usdt_perp: Option<bool>,
}

impl Default for Subaccount {
    fn default() -> Self {
        let ts = unix_timestamp_ms() as i64;
        Subaccount {
            id: Default::default(),
            name: Default::default(),
            subaccount: Default::default(),
            team: Default::default(),
            portfolio: Default::default(),
            trader: Default::default(),
            exchange: Exchange::Binance,
            enabled: true,
            client_funds: false,
            symbols: Default::default(),
            created_at: ts,
            updated_at: ts,
            binance_spot: None,
            binance_spot_margin: None,
            binance_usdmf: None,
            check_staked: None,
            bybit_spot: None,
            bybit_usdt_perp: None,
        }
    }
}

#[typeshare]
#[derive(Debug, Deserialize, Serialize, Clone, Default, MungosIndexed)]
pub struct ExchangeApiKeys {
    #[unique_index]
    pub sid: String,
    pub key: String,
    pub secret: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
}

#[typeshare]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BinanceFee {
    pub vip: I64, // 0 - 9
    pub ins: BinanceInstrument,
    pub maker: f64,
    pub taker: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<BinanceUsdmfQuote>,
}

#[typeshare]
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Display,
    EnumString,
    PartialEq,
    Hash,
    Eq,
    Clone,
    Copy,
    MungosIndexed,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Exchange {
    Binance,
    Bybit,
    Injective,
    Okx,
    Huobi,
    Bitmart,
    Bitkub,
    Dydx,
    Kucoin,
    Bluefin,
}

#[typeshare]
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Display,
    EnumString,
    PartialEq,
    Hash,
    Eq,
    Clone,
    Copy,
    Default,
    MungosIndexed,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum MarketType {
    #[default]
    Spot,
    #[serde(alias = "usdmf", alias = "perp")]
    #[strum(serialize = "usdmf", serialize = "perp")]
    LinearSwap,
}

#[typeshare]
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Display,
    EnumString,
    PartialEq,
    Hash,
    Eq,
    Clone,
    Copy,
    MungosIndexed,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TradeSide {
    #[serde(alias = "BUY")]
    #[strum(to_string = "buy", serialize = "BUY")]
    Buy,
    #[serde(alias = "SELL")]
    #[strum(to_string = "sell", serialize = "SELL")]
    Sell,
}

#[typeshare]
#[derive(
    Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PositionSide {
    #[serde(alias = "LONG", alias = "buy", alias = "BUY")]
    #[strum(
        to_string = "long",
        serialize = "LONG",
        serialize = "buy",
        serialize = "BUY"
    )]
    Long,
    #[serde(alias = "SHORT", alias = "sell", alias = "SELL")]
    #[strum(
        to_string = "short",
        serialize = "SHORT",
        serialize = "sell",
        serialize = "SELL"
    )]
    Short,
    #[serde(alias = "BOTH", alias = "net")]
    #[strum(to_string = "both", serialize = "BOTH", serialize = "net")]
    Both,
    #[default]
    Unknown,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AggType {
    Pnl,
}

#[typeshare]
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Display,
    EnumString,
    PartialEq,
    Hash,
    Eq,
    Clone,
    Copy,
    MungosIndexed,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    #[serde(alias = "submitted", alias = "PENDING")]
    #[strum(
        to_string = "SUBMITTED",
        serialize = "submitted",
        serialize = "PENDING"
    )]
    Submitted,
    #[serde(alias = "live", alias = "created", alias = "open", alias = "OPEN")]
    #[strum(
        to_string = "NEW",
        serialize = "live",
        serialize = "created",
        serialize = "open",
        serialize = "OPEN"
    )]
    New,
    #[serde(
        alias = "partially_filled",
        alias = "partial-filled",
        alias = "PARTIAL_FILLED"
    )]
    #[strum(
        to_string = "PARTIALLY_FILLED",
        serialize = "partially_filled",
        serialize = "partial-filled",
        serialize = "PARTIAL_FILLED"
    )]
    PartiallyFilled,
    #[serde(alias = "filled", alias = "FILLED")]
    #[strum(to_string = "FILLED", serialize = "filled")]
    Filled,
    #[serde(alias = "CANCELLING")]
    #[strum(to_string = "PENDING_CANCEL", serialize = "CANCELLING")]
    PendingCancel,
    #[serde(alias = "canceled", alias = "CANCELLED")]
    #[strum(
        to_string = "CANCELED",
        serialize = "canceled",
        serialize = "CANCELLED"
    )]
    Canceled,
    #[serde(alias = "partial-canceled")]
    #[strum(to_string = "PARTIALLY_CANCELED", serialize = "partial-canceled")]
    PartiallyCanceled,
    #[serde(alias = "rejected", alias = "REJECTED")]
    #[strum(to_string = "REJECTED", serialize = "rejected")]
    Rejected,
    #[serde(alias = "EXPIRED_IN_MATCH", alias = "EXPIRED")]
    #[strum(to_string = "EXPIRED", serialize = "EXPIRED_IN_MATCH")]
    Expired,
    StandByPending,
    StandBy,
}

#[typeshare]
#[derive(
    Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum OrderTif {
    #[default]
    Unknown,
    #[serde(alias = "GTT")]
    #[strum(to_string = "gtt", serialize = "GTT")]
    Gtt,
    #[serde(alias = "GTC")]
    #[strum(to_string = "gtc", serialize = "GTC")]
    Gtc,
    #[serde(alias = "GTX")]
    #[strum(to_string = "gtx", serialize = "GTX")]
    Gtx,
    #[serde(alias = "IOC")]
    #[strum(to_string = "ioc", serialize = "IOC")]
    Ioc,
    #[serde(alias = "FOK")]
    #[strum(to_string = "fok", serialize = "FOK")]
    Fok,
    #[serde(alias = "limit-fok")]
    #[strum(to_string = "limit_fok", serialize = "limit-fok")]
    LimitFok,
    #[serde(alias = "stop-limit-fok")]
    #[strum(to_string = "stop_limit_fok", serialize = "stop-limit-fok")]
    StopLimitFok,
    PostOnly,
    Market,
    Limit,
    OptimalLimitIoc,
    #[serde(alias = "limit-maker")]
    #[strum(to_string = "limit_maker", serialize = "limit-maker")]
    LimitMaker,
    #[serde(alias = "stop-limit")]
    #[strum(to_string = "stop_limit", serialize = "stop-limit")]
    StopLimit,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AccountType {
    Spot,
    SpotMargin,
    Perp,
    Margin,
    Future,
    Trading,
    Frozen,
    Inj,
    Bank,
    Eth,
    Staked,
    Reward,
    Funding,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum BinanceInstrument {
    Spot,
    Usdmf,
}

#[typeshare]
#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum BinanceUsdmfQuote {
    Usdt,
    Busd,
}
