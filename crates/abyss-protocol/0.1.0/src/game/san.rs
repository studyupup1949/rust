use serde::{Deserialize, Serialize};

use super::currency::CurrencyState;
use super::nodes::{NodeId, NodeState};
use super::shop::{ShopState, ShopUpgradeId};
use super::synergy::{SynergyId, SynergyState};

/// SAN 等级，根据当前理智值划分
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SanLevel {
    /// 80-100: 清醒
    Lucid,
    /// 60-79: 不安
    Uneasy,
    /// 40-59: 焦虑
    Anxious,
    /// 20-39: 恐惧
    Terrified,
    /// 1-19: 疯狂
    Insane,
    /// 0: 崩溃
    Collapsed,
}

impl SanLevel {
    /// 返回该等级对应的 i18n key
    pub fn i18n_key(&self) -> &'static str {
        match self {
            SanLevel::Lucid => "san_states.lucid",
            SanLevel::Uneasy => "san_states.uneasy",
            SanLevel::Anxious => "san_states.anxious",
            SanLevel::Terrified => "san_states.terrified",
            SanLevel::Insane => "san_states.insane",
            SanLevel::Collapsed => "san_states.collapsed",
        }
    }
}

/// 理智状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanState {
    /// 当前理智值 (0.0 - 100.0)
    pub current: f64,
    /// 本轮修复次数
    pub repair_count_this_cycle: u32,
}

impl SanState {
    pub fn new() -> Self {
        Self {
            current: 100.0,
            repair_count_this_cycle: 0,
        }
    }

    /// 根据当前理智值返回 SAN 等级
    pub fn level(&self) -> SanLevel {
        if self.current <= 0.0 {
            SanLevel::Collapsed
        } else if self.current < 20.0 {
            SanLevel::Insane
        } else if self.current < 40.0 {
            SanLevel::Terrified
        } else if self.current < 60.0 {
            SanLevel::Anxious
        } else if self.current < 80.0 {
            SanLevel::Uneasy
        } else {
            SanLevel::Lucid
        }
    }

    /// 计算修复成本：base × 1.5^repair_count_this_cycle (base = 100.0)
    pub fn repair_cost(&self) -> f64 {
        100.0 * 1.5_f64.powi(self.repair_count_this_cycle as i32)
    }

    /// 计算修复量：20 + 5 × 理智锚点数量
    pub fn repair_amount(&self, nodes: &NodeState) -> f64 {
        let anchor_count = nodes.count(NodeId::SanityAnchor) as f64;
        20.0 + 5.0 * anchor_count
    }

    /// 获取 SAN 上限：默认 100，深渊契约降至 80
    pub fn max_san(shop: &ShopState) -> f64 {
        if shop.has(ShopUpgradeId::AbyssPact) {
            80.0
        } else {
            100.0
        }
    }

    /// 计算 SAN 衰减速率
    /// base: 0.005 × sqrt(production_per_sec)
    /// FirewallShard: ×(1.0 - 0.05 × count)
    /// CoolantTransmutation: ×(1.0 - 0.08 × count)
    /// SanityFrontline synergy: ×0.5
    /// NirvanaProtocol shop upgrade: ×0.7
    /// CollapseAccelerator shop upgrade: ×1.5
    /// Clamp minimum to 0.0
    pub fn decay_rate(
        production_per_sec: f64,
        nodes: &NodeState,
        shop: &ShopState,
        synergies: &SynergyState,
    ) -> f64 {
        let base = 0.005 * production_per_sec.sqrt();

        let fw_count = nodes.count(NodeId::FirewallShard) as f64;
        let ct_count = nodes.count(NodeId::CoolantTransmutation) as f64;

        let mut rate = base;
        rate *= (1.0 - 0.05 * fw_count).max(0.0);
        rate *= (1.0 - 0.08 * ct_count).max(0.0);

        if synergies.active.contains(&SynergyId::SanityFrontline) {
            rate *= 0.5;
        }

        if shop.has(ShopUpgradeId::NirvanaProtocol) {
            rate *= 0.7;
        }

        if shop.has(ShopUpgradeId::CollapseAccelerator) {
            rate *= 1.5;
        }

        rate.max(0.0)
    }

    /// 修复 SAN：检查余额/上限、扣费、恢复 SAN
    /// 返回实际恢复的 SAN 量
    pub fn repair(
        &mut self,
        currency: &mut CurrencyState,
        nodes: &NodeState,
        shop: &ShopState,
    ) -> Result<f64, RepairError> {
        let max = Self::max_san(shop);

        // 检查 SAN 是否已满
        if self.current >= max {
            return Err(RepairError::SanAlreadyFull);
        }

        // 计算修复成本
        let cost = self.repair_cost();

        // 检查碎语余额
        if currency.whispers < cost {
            return Err(RepairError::InsufficientWhispers {
                cost,
                balance: currency.whispers,
            });
        }

        // 扣除碎语
        currency.whispers -= cost;

        // 计算修复量并应用（不超过上限）
        let amount = self.repair_amount(nodes);
        let old_san = self.current;
        self.current = (self.current + amount).min(max);
        let actual_healed = self.current - old_san;

        // 修复次数 +1
        self.repair_count_this_cycle += 1;

        Ok(actual_healed)
    }
}

impl Default for SanState {
    fn default() -> Self {
        Self::new()
    }
}

/// SAN 修复错误
#[derive(Debug, Clone, PartialEq)]
pub enum RepairError {
    InsufficientWhispers { cost: f64, balance: f64 },
    SanAlreadyFull,
}
