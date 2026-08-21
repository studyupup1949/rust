// 变异事件 — 概率判定 + 收益/代价

use rand::Rng;
use super::nodes::{NodeId, NodeState};
use super::shop::{ShopState, ShopUpgradeId};
use super::synergy::{SynergyId, SynergyState};

/// 变异事件结果
#[derive(Debug, Clone)]
pub struct MutationResult {
    /// 获得的碎语
    pub whispers_gained: f64,
    /// 扣除的 SAN
    pub san_cost: f64,
}

/// 检查并执行变异事件
///
/// - 维度二未解锁时不触发
/// - 基础概率 2%/秒，变异催化剂 +5%/个
/// - 收益：5-20 秒产量的碎语（随机范围）
/// - 代价：2-8 点 SAN（隔离沙箱减少，变异风暴/变异掌控免疫）
pub fn check_mutation<R: Rng>(
    max_dimension: u32,
    production_per_sec: f64,
    dt: f64,
    nodes: &NodeState,
    shop: &ShopState,
    synergies: &SynergyState,
    rng: &mut R,
) -> Option<MutationResult> {
    // 维度二未解锁时不触发
    if max_dimension < 2 {
        return None;
    }

    // 计算触发概率
    let catalyst_count = nodes.count(NodeId::MutationCatalyst) as f64;
    let probability = ((0.02 + 0.05 * catalyst_count) * dt).clamp(0.0, 1.0);

    // 概率判定
    if rng.gen::<f64>() >= probability {
        return None;
    }

    // 计算碎语收益：5-20 秒产量
    let multiplier = rng.gen_range(5.0..=20.0);
    let whispers_gained = production_per_sec * multiplier;

    // 计算 SAN 代价
    let base_san_cost = rng.gen_range(2.0..=8.0);

    // 变异风暴协同激活 或 变异掌控升级已购买 → SAN 免疫
    let mutation_storm_active = synergies.active.contains(&SynergyId::MutationStorm);
    let mutation_mastery_level = shop.level(ShopUpgradeId::MutationMastery);

    let san_cost = if mutation_storm_active || mutation_mastery_level > 0 {
        0.0
    } else {
        // 隔离沙箱减少 SAN 扣除，每个 -1，最低为 0
        let sandbox_count = nodes.count(NodeId::IsolationSandbox) as f64;
        (base_san_cost - sandbox_count).max(0.0)
    };

    Some(MutationResult {
        whispers_gained,
        san_cost,
    })
}
