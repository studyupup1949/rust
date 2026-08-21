use std::collections::HashMap;

use async_timing_util::unix_timestamp_ms;
use mungos::mongodb::bson::serde_helpers::hex_string_as_object_id;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub type I64 = i64;

pub type PriceMap = HashMap<String, f64>;
pub type BalancesMap = HashMap<String, AssetBalance>; // (asset_name, AssetBalance)
pub type AccountBalancesMap = HashMap<AccountType, BalancesMap>; // (account_type, BalancesMap)
pub type PositionsMap = HashMap<String, Position>;
pub type AccountPositionsMap = HashMap<AccountType, PositionsMap>;
pub type LatencyMap = HashMap<String, I64>; // (region, lat in ms)
pub type ExchangeSymbolsMap = HashMap<String, String>; // (exchange, symbol)

pub type ExchangeKeysMap = HashMap<Exchange, ExchangeApiKeys>;
pub type InjDenomMap = HashMap<String, (String, u32)>; // (denom, (symbol, decimals))

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InternalSymbol {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub exchange: ExchangeSymbolsMap,
}

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

#[derive(Debug, Deserialize, Serialize)]
pub struct Fill {
    pub ts: I64, // exchange time message sent (unix timestamp milliseconds) (E on binance) (indexed)
    pub sid: String, // subaccount id (indexed)
    pub oid: String, // order id (indexed)
    pub tid: String, // transaction id (unique indexed)
    pub stat: OrderStatus, // order status
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
    pub t_ts: Option<I64>, // transaction timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tif: Option<OrderTif>, // time in force
    #[serde(skip_serializing_if = "Option::is_none")]
    pub m: Option<bool>, // is maker
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Balances {
    pub sid: String,                    // strategy id
    pub ts: I64,                        // timestamp of balance snapshot
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
    pub ts: I64,       // approx timestamp of transfer
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
    pub created_at: I64,
    #[serde(default)]
    pub updated_at: I64,
    #[serde(default)]
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
    pub vip: I64, // 0 - 9
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
    Bluefin,
}

#[derive(
    Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy, Default,
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

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
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

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AggType {
    Pnl,
}

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
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

#[derive(Serialize, Deserialize, Debug, Display, EnumString, PartialEq, Hash, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AccountType {
    Spot,
    SpotMargin,
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

impl TryFrom<Subaccount> for crate::Subaccount {
    type Error = anyhow::Error;

    fn try_from(value: Subaccount) -> Result<Self, Self::Error> {
        let subaccount = crate::Subaccount {
            id: value.id,
            name: value.name,
            subaccount: value.subaccount,
            team: value.team,
            portfolio: value.portfolio,
            trader: value.trader,
            exchange: value.exchange.to_string().parse()?,
            enabled: value.enabled,
            created_at: value.created_at,
            updated_at: value.updated_at,
            client_funds: value.client_funds,
            symbols: value.symbols,
            check_staked: value.check_staked,
            binance_spot: value.binance_spot,
            binance_spot_margin: value.binance_spot_margin,
            binance_usdmf: value.binance_usdmf,
            bybit_spot: value.bybit_spot,
            bybit_usdt_perp: value.bybit_usdt_perp,
        };
        Ok(subaccount)
    }
}

impl TryFrom<Balances> for crate::Balances {
    type Error = anyhow::Error;
    fn try_from(value: Balances) -> Result<Self, Self::Error> {
        let balances = Self {
            sid: value.sid,
            ts: value.ts,
            failure: value.failure,
            notional: value.notional,
            pos_notional: value.pos_notional,
            balances: value
                .balances
                .into_iter()
                .map(|(acct, bal)| {
                    (
                        acct.to_string().parse().unwrap(),
                        bal.into_iter().map(|(a, b)| (a, b.into())).collect(),
                    )
                })
                .collect(),
            positions: value
                .positions
                .into_iter()
                .map(|(acct, pos)| {
                    (
                        acct.to_string().parse().unwrap(),
                        pos.into_iter().map(|(s, p)| (s, p.into())).collect(),
                    )
                })
                .collect(),
        };
        Ok(balances)
    }
}

impl From<AssetBalance> for crate::AssetBalance {
    fn from(value: AssetBalance) -> Self {
        Self {
            total: value.total,
            locked: value.locked,
            notional: value.notional,
        }
    }
}

impl From<Position> for crate::Position {
    fn from(value: Position) -> Self {
        Self {
            total: value.total,
            notional: value.notional,
            side: value.side.to_string().parse().unwrap(),
            leverage: value.leverage,
            maint_margin: value.maint_margin,
        }
    }
}

impl TryFrom<Fill> for crate::Fill {
    type Error = anyhow::Error;
    fn try_from(value: Fill) -> Result<Self, Self::Error> {
        let fill = Self {
            ts: value.ts,
            sid: value.sid,
            oid: value.oid,
            tid: value.tid,
            stat: value.stat.to_string().parse()?,
            side: value.side.to_string().parse()?,
            sym: value.sym,
            ins: value.ins,
            m: value.m.unwrap_or_default(),
            tif: value.tif.unwrap_or_default().to_string().parse()?,
            p: value.p,
            q: value.q,
            qn: value.qn,
            fee: value.fee,
            lat: value.lat,
            t_ts: value.t_ts,
        };
        Ok(fill)
    }
}

impl TryFrom<BalanceTransfer> for crate::BalanceTransfer {
    type Error = anyhow::Error;
    fn try_from(value: BalanceTransfer) -> Result<Self, Self::Error> {
        let transfer = Self {
            id: value.id,
            sid: value.sid,
            ts: value.ts,
            enabled: value.enabled,
            amount: value.amount,
            other: value.other,
            requester: value.requester,
            sender: value.sender,
            symbol: value.symbol,
            approvers: value.approvers,
            coin_amount: value.coin_amount,
            link: value.link,
        };
        Ok(transfer)
    }
}

impl TryFrom<InternalSymbol> for crate::InternalSymbol {
    type Error = anyhow::Error;
    fn try_from(value: InternalSymbol) -> Result<Self, Self::Error> {
        let symbols = Self {
            symbol: value.symbol,
            exchange: value.exchange,
        };
        Ok(symbols)
    }
}
