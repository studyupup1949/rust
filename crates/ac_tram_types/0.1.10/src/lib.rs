use std::collections::HashMap;

use bson::serde_helpers::hex_string_as_object_id;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub type ExchangeKeysMap = HashMap<Exchange, ExchangeApiKeys>;
pub type PriceMap = HashMap<String, f64>;
pub type BalancesMap = HashMap<String, AssetBalance>; // (asset_name, AssetBalance)
pub type AccountBalancesMap = HashMap<AccountType, BalancesMap>; // (account_type, BalancesMap)
pub type PositionsMap = HashMap<String, Position>;
pub type AccountPositionsMap = HashMap<AccountType, PositionsMap>;
pub type InjDenomMap = HashMap<String, (String, u32)>; // (denom, (symbol, decimals))
pub type LatencyMap = HashMap<String, i64>; // (region, lat in ms)

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenOrder {
    pub sid: String,             // subaccount id order was made from
    pub subaccount: String,      // subaccount order was made from
    pub team: String,            // team order is part of
    pub portfolio: String,       // portfolio order is part of
    pub exchange: Exchange,      // exchange order is on
    pub order_id: String,        // order id
    pub created_at: i64,         // order created timestamp
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
    pub tif: Option<OrderTif>, // order time in force
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Fill {
    pub ts: i64, // exchange time message sent (unix timestamp milliseconds) (E on binance) (indexed)
    pub sid: String, // subaccount id (indexed)
    pub oid: String, // order id (indexed)
    pub tid: String, // transaction id (unique indexed)
    pub stat: String, // order status
    pub side: TradeSide, // "buy" or "sell" (indexed)
    pub sym: String, // market symbol (indexed)
    pub ins: String, // instrument type (indexed)
    pub p: f64,  // price
    pub q: f64,  // filled quantity
    pub qn: f64, // quantity notional
    pub fee: f64, // fee notional
    #[serde(default)]
    pub lat: LatencyMap, // latency in ms, recv_ts - sent_ts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t_ts: Option<i64>, // transaction timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tif: Option<OrderTif>, // time in force
    #[serde(skip_serializing_if = "Option::is_none")]
    pub m: Option<bool>, // is maker
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Balances {
    #[serde(
        default,
        rename = "_id",
        with = "hex_string_as_object_id",
        skip_serializing_if = "String::is_empty"
    )]
    pub id: String,
    pub sid: String,                    // strategy id
    pub ts: i64,                        // timestamp of balance snapshot
    pub notional: f64, // all the notionals added (eg spot, perp, on chain, staked, reward)
    pub balances: AccountBalancesMap, // full breakdown of account balances split by account type. eg { "spot": { "USDT": { "total": ... } }, "perp": { "USDT": { "total": ... } } }
    pub pos_notional: f64,            // total value of all positions on account
    pub positions: AccountPositionsMap, // full breakdown of positions split by market type. eg { "perp": { "BTCUSDT": { "total": ... } }, "future": { "ETHUSDT": { "total": ... } },  }
    pub failure: bool,                  // did the recording fail
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct AssetBalance {
    pub total: f64,
    pub locked: f64,
    pub notional: f64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Position {
    pub total: f64,
    pub notional: f64,
    pub side: PositionSide,
    pub leverage: f64,
    pub maint_margin: f64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct BalanceTransfer {
    #[serde(
        default,
        rename = "_id",
        with = "hex_string_as_object_id",
        skip_serializing_if = "String::is_empty"
    )]
    pub id: String, // mongo _id
    pub sid: String,   // strategy id
    pub ts: i64,       // approx timestamp of transfer
    pub amount: f64, // amount to adjust PnL by (notional USD). can be positive (transfer in), or negative (transfer out)
    pub enabled: bool, // whether to use this transfer to adjust pnl

    // the fields below aren't functionally needed, but useful to keep track of
    #[serde(default)]
    pub other: String, // this is either source or destination account, depending on transfer in / out. Should be either the other sid or 'external'
    #[serde(default)]
    pub requester: String, // trader that requests the transfer
    #[serde(default)]
    pub sender: String, // user that executes the transfer
    #[serde(default)]
    pub approvers: Vec<String>, // users that approve the transfer (for multi sig transactions)
    #[serde(default)]
    pub symbol: String, // symbol transfered
    #[serde(default)]
    pub coin_amount: f64, // amount transferred notional
    #[serde(default)]
    pub link: Option<String>, // link to defi transaction for transfer
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Subaccount {
    #[serde(
        default,
        rename = "_id",
        with = "hex_string_as_object_id",
        skip_serializing_if = "String::is_empty"
    )]
    pub id: String,
    pub name: String,
    pub subaccount: String,
    pub team: String,
    pub portfolio: String,
    pub trader: String,
    pub exchange: Exchange,
    pub enabled: bool,
    #[serde(default)]
    pub client_funds: bool,
    #[serde(default)]
    pub symbols: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_staked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binance_spot: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binance_usdmf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bybit_spot: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bybit_usdt_perp: Option<bool>,
}

impl Default for Subaccount {
    fn default() -> Self {
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
            binance_spot: None,
            binance_usdmf: None,
            check_staked: None,
            bybit_spot: None,
            bybit_usdt_perp: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ExchangeApiKeys {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,
    pub key: String,
    pub secret: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiKeysConfig {
    pub exchange_keys: ExchangeKeysMap,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BinanceFee {
    pub vip: i64, // 0 - 9
    pub ins: BinanceInstrument,
    pub maker: f64,
    pub taker: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<BinanceUsdmfQuote>,
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
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
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum MarketType {
    Spot,
    LinearSwap,
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum TradeSide {
    #[serde(alias = "BUY")]
    Buy,
    #[serde(alias = "SELL")]
    Sell,
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum PositionSide {
    #[serde(alias = "LONG", alias = "buy", alias = "long")]
    Long,
    #[serde(alias = "SHORT", alias = "sell", alias = "short")]
    Short,
    #[serde(alias = "BOTH", alias = "net")]
    Both,
    Unknown,
}

impl Default for PositionSide {
    fn default() -> Self {
        PositionSide::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AggType {
    Pnl,
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum OrderStatus {
    #[serde(alias = "NEW")]
    New,
    Open,
    PartialFilled,
    Closed,
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum OrderTif {
    Day,
    #[serde(alias = "GTC")]
    Gtc,
    Gtd,
    #[serde(alias = "GTX")]
    Gtx,
    #[serde(alias = "IOC")]
    Ioc,
    #[serde(alias = "FOK")]
    Fok,
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AccountType {
    Spot,
    Perp,
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

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum BinanceInstrument {
    Spot,
    Usdmf,
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum BinanceUsdmfQuote {
    Usdt,
    Busd,
}
