use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use crate::AccountType;

pub type BalancesMap = HashMap<String, AssetBalance>; // (asset_name, AssetBalance)
pub type PositionsMap = HashMap<String, Position>; // (symbol_name, Position)
pub type StakedPositionsMap = HashMap<String, StakedPosition>; // (asset_name, StakedPosition)

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Balances {
    pub sid: String, // strategy id
    pub ts: i64,
    pub notional: f64,
    pub balances: BalancesMap,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos_notional: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positions: Option<PositionsMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staked_notional: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staked_bal: Option<StakedPositionsMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_chain_notional: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_chain_bal: Option<BalancesMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward_notional: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward_bal: Option<BalancesMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AssetBalance {
    pub locked: f64,
    pub total: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notional: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spot_borrow: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_without_borrow: Option<f64>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Position {
    pub amount: f64,
    pub notional: f64,
    pub side: PositionSide,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct StakedPosition {
    pub amount: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notional: Option<f64>,
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

impl From<Balances> for crate::Balances {
    fn from(value: Balances) -> crate::Balances {
        let mut balances: crate::AccountBalancesMap = HashMap::new();
        for (asset, balance) in value.balances {
            if asset.contains("SPOT-") {
                balances
                    .entry(AccountType::Spot)
                    .or_default()
                    .insert(asset.replace("SPOT-", ""), balance.into());
            } else if asset.contains("PERP-") {
                balances
                    .entry(AccountType::Perp)
                    .or_default()
                    .insert(asset.replace("PERP-", ""), balance.into());
            } else if asset.contains("REWARD-") {
                balances
                    .entry(AccountType::Reward)
                    .or_default()
                    .insert(asset.replace("REWARD-", ""), balance.into());
            } else if asset.contains("STAKED-") {
                balances
                    .entry(AccountType::Staked)
                    .or_default()
                    .insert(asset.replace("STAKED-", ""), balance.into());
            } else if asset.contains("TRADE-") {
                balances
                    .entry(AccountType::Trading)
                    .or_default()
                    .insert(asset.replace("TRADE-", ""), balance.into());
            } else if asset.contains("FROZEN") {
                balances
                    .entry(AccountType::Frozen)
                    .or_default()
                    .insert(asset.replace("FROZEN-", ""), balance.into());
            } else if asset.contains("USDM-") {
                balances
                    .entry(AccountType::Perp)
                    .or_default()
                    .insert(asset.replace("USDM-", ""), balance.into());
            } else {
                balances
                    .entry(AccountType::Trading)
                    .or_default()
                    .insert(asset, balance.into());
            }
        }
        let mut perp_positions: crate::PositionsMap = HashMap::new();
        for (symbol, position) in value.positions.unwrap_or_default() {
            perp_positions.insert(symbol, position.into());
        }
        let positions = [(AccountType::Perp, perp_positions)].into_iter().collect();
        crate::Balances {
            sid: value.sid,
            ts: value.ts,
            notional: value.notional + value.staked_notional.unwrap_or_default(),
            failure: value.failure.unwrap_or_default(),
            pos_notional: value.pos_notional.unwrap_or_default(),
            balances,
            positions,
        }
    }
}

impl From<AssetBalance> for crate::AssetBalance {
    fn from(value: AssetBalance) -> crate::AssetBalance {
        crate::AssetBalance {
            total: value.total,
            locked: value.locked,
            notional: value.notional.unwrap_or_default(),
        }
    }
}

impl From<StakedPosition> for crate::AssetBalance {
    fn from(value: StakedPosition) -> crate::AssetBalance {
        crate::AssetBalance {
            total: value.amount,
            locked: value.amount,
            notional: value.notional.unwrap_or_default(),
        }
    }
}

impl From<Position> for crate::Position {
    fn from(value: Position) -> crate::Position {
        crate::Position {
            total: value.amount,
            notional: value.notional,
            side: value
                .side
                .to_string()
                .parse()
                .expect("failed to convert position side"),
            leverage: 0.0,
            maint_margin: 0.0,
        }
    }
}

impl From<PositionSide> for crate::PositionSide {
    fn from(value: PositionSide) -> crate::PositionSide {
        value.to_string().parse().unwrap_or_else(|e| {
            panic!("failed to parse v0 PositionSide {value} into PositionSide | {e:#?}")
        })
    }
}
