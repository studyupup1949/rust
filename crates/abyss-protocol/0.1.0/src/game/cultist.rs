use serde::{Deserialize, Serialize};

use super::currency::CurrencyState;
use super::nodes::{NodeId, NodeState};
use super::production;
use super::shop::{ShopState, ShopUpgradeId};

/// 信徒解锁条件
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnlockCondition {
    /// 始终可用
    Always,
    /// 累计碎语达到指定数量
    TotalWhispers(f64),
    /// 到达指定维度
    Dimension(u32),
    /// 到达指定维度且完成指定次数转生
    DimensionAndRebirths(u32, u32),
    /// 到达指定维度且拥有指定数量禁忌真理
    DimensionAndTruths(u32, u32),
}

/// 信徒等级 T1-T10
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CultistTier {
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
    T10,
}

/// 信徒等级的静态数据
pub struct CultistData {
    /// i18n 名称 key
    pub name_key: &'static str,
    /// 显示符号
    pub symbol: &'static str,
    /// 基础产出（碎语/秒）
    pub base_production: f64,
    /// 基础价格（碎语）
    pub base_price: f64,
    /// 涨价系数
    pub price_growth: f64,
    /// 解锁条件
    pub unlock_condition: UnlockCondition,
}

impl CultistTier {
    /// 返回该等级的静态数据
    pub fn data(&self) -> CultistData {
        match self {
            CultistTier::T1 => CultistData {
                name_key: "cultists.t1_name",
                symbol: "·",
                base_production: 0.1,
                base_price: 10.0,
                price_growth: 1.08,
                unlock_condition: UnlockCondition::Always,
            },
            CultistTier::T2 => CultistData {
                name_key: "cultists.t2_name",
                symbol: "☻",
                base_production: 0.8,
                base_price: 80.0,
                price_growth: 1.10,
                unlock_condition: UnlockCondition::Always,
            },
            CultistTier::T3 => CultistData {
                name_key: "cultists.t3_name",
                symbol: "◈",
                base_production: 5.0,
                base_price: 500.0,
                price_growth: 1.12,
                unlock_condition: UnlockCondition::TotalWhispers(1000.0),
            },
            CultistTier::T4 => CultistData {
                name_key: "cultists.t4_name",
                symbol: "⛧",
                base_production: 30.0,
                base_price: 3000.0,
                price_growth: 1.13,
                unlock_condition: UnlockCondition::TotalWhispers(10000.0),
            },
            CultistTier::T5 => CultistData {
                name_key: "cultists.t5_name",
                symbol: "Ψ",
                base_production: 200.0,
                base_price: 20000.0,
                price_growth: 1.14,
                unlock_condition: UnlockCondition::Dimension(2),
            },
            CultistTier::T6 => CultistData {
                name_key: "cultists.t6_name",
                symbol: "Ω",
                base_production: 1500.0,
                base_price: 150000.0,
                price_growth: 1.15,
                unlock_condition: UnlockCondition::DimensionAndRebirths(2, 1),
            },
            CultistTier::T7 => CultistData {
                name_key: "cultists.t7_name",
                symbol: "◉",
                base_production: 12000.0,
                base_price: 1200000.0,
                price_growth: 1.15,
                unlock_condition: UnlockCondition::DimensionAndRebirths(2, 3),
            },
            CultistTier::T8 => CultistData {
                name_key: "cultists.t8_name",
                symbol: "✦",
                base_production: 100000.0,
                base_price: 10000000.0,
                price_growth: 1.16,
                unlock_condition: UnlockCondition::Dimension(3),
            },
            CultistTier::T9 => CultistData {
                name_key: "cultists.t9_name",
                symbol: "꩜",
                base_production: 1000000.0,
                base_price: 100000000.0,
                price_growth: 1.17,
                unlock_condition: UnlockCondition::DimensionAndRebirths(3, 5),
            },
            CultistTier::T10 => CultistData {
                name_key: "cultists.t10_name",
                symbol: "⬟",
                base_production: 10000000.0,
                base_price: 1000000000.0,
                price_growth: 1.18,
                unlock_condition: UnlockCondition::DimensionAndTruths(3, 50),
            },
        }
    }

    /// 返回该等级在 counts 数组中的索引
    pub fn index(&self) -> usize {
        match self {
            CultistTier::T1 => 0,
            CultistTier::T2 => 1,
            CultistTier::T3 => 2,
            CultistTier::T4 => 3,
            CultistTier::T5 => 4,
            CultistTier::T6 => 5,
            CultistTier::T7 => 6,
            CultistTier::T8 => 7,
            CultistTier::T9 => 8,
            CultistTier::T10 => 9,
        }
    }

    /// 所有等级的迭代
    pub const ALL: [CultistTier; 10] = [
        CultistTier::T1,
        CultistTier::T2,
        CultistTier::T3,
        CultistTier::T4,
        CultistTier::T5,
        CultistTier::T6,
        CultistTier::T7,
        CultistTier::T8,
        CultistTier::T9,
        CultistTier::T10,
    ];

    /// 返回下一个等级，T10 返回 None
    pub fn next_tier(self) -> Option<CultistTier> {
        match self {
            CultistTier::T1 => Some(CultistTier::T2),
            CultistTier::T2 => Some(CultistTier::T3),
            CultistTier::T3 => Some(CultistTier::T4),
            CultistTier::T4 => Some(CultistTier::T5),
            CultistTier::T5 => Some(CultistTier::T6),
            CultistTier::T6 => Some(CultistTier::T7),
            CultistTier::T7 => Some(CultistTier::T8),
            CultistTier::T8 => Some(CultistTier::T9),
            CultistTier::T9 => Some(CultistTier::T10),
            CultistTier::T10 => None,
        }
    }
}

/// 信徒状态：10 个等级各自的数量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultistState {
    pub counts: [u32; 10],
}

impl CultistState {
    pub fn new() -> Self {
        Self { counts: [0; 10] }
    }

    /// 所有等级信徒的总数
    pub fn total_count(&self) -> u32 {
        self.counts.iter().sum()
    }

    /// 招募信徒：检查余额、扣费、增加数量
    pub fn recruit(
        &mut self,
        tier: CultistTier,
        currency: &mut CurrencyState,
        shop: &ShopState,
    ) -> Result<(), RecruitError> {
        let price = production::cultist_recruit_price(tier, self.counts[tier.index()], shop);
        if currency.whispers < price {
            return Err(RecruitError::InsufficientWhispers {
                cost: price,
                balance: currency.whispers,
            });
        }
        currency.whispers -= price;
        self.counts[tier.index()] += 1;
        Ok(())
    }

    /// 合成信徒：检查数量、消耗低级、产出高级
    /// 返回合成返还的碎语（数据回收站效果）
    pub fn fuse(
        &mut self,
        tier: CultistTier,
        shop: &ShopState,
        nodes: &NodeState,
    ) -> Result<f64, FuseError> {
        // T10 不能再合成
        if tier == CultistTier::T10 {
            return Err(FuseError::AlreadyMaxTier);
        }

        let fusion_cost = Self::fusion_cost(shop);
        let available = self.counts[tier.index()];

        if available < fusion_cost {
            return Err(FuseError::InsufficientCultists {
                required: fusion_cost,
                available,
            });
        }

        // 扣除低级信徒
        self.counts[tier.index()] -= fusion_cost;

        // 增加高级信徒
        let next_tier = tier.next_tier().unwrap(); // safe: already checked not T10
        self.counts[next_tier.index()] += 1;

        // 计算数据回收站返还
        let data_recycler_count = nodes
            .owned
            .get(&NodeId::DataRecycler)
            .copied()
            .unwrap_or(0);

        let refund = if data_recycler_count > 0 {
            let data = tier.data();
            let count_after = self.counts[tier.index()];
            let batch = production::batch_price(
                data.base_price,
                data.price_growth,
                count_after,
                fusion_cost,
                1.0,
            );
            batch * 0.20 * data_recycler_count as f64
        } else {
            0.0
        };

        Ok(refund)
    }

    /// 获取合成所需数量（默认 5，合成精通降低为 4）
    pub fn fusion_cost(shop: &ShopState) -> u32 {
        let mastery_level = shop
            .levels
            .get(&ShopUpgradeId::SynthesisMastery)
            .copied()
            .unwrap_or(0);
        if mastery_level >= 1 {
            4
        } else {
            5
        }
    }
}

impl Default for CultistState {
    fn default() -> Self {
        Self::new()
    }
}

/// 信徒招募错误
#[derive(Debug, Clone, PartialEq)]
pub enum RecruitError {
    InsufficientWhispers { cost: f64, balance: f64 },
    TierLocked,
}

/// 信徒合成错误
#[derive(Debug, Clone, PartialEq)]
pub enum FuseError {
    InsufficientCultists { required: u32, available: u32 },
    AlreadyMaxTier,
}

