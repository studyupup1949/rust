// 产出计算器 — 纯函数模块

use super::cultist::CultistTier;
use super::nodes::{NodeId, NodeState};
use super::san::SanState;
use super::shop::{ShopState, ShopUpgradeId};
use super::synergy::SynergyId;
use super::GameState;

/// 产出计算结果
#[derive(Debug, Clone)]
pub struct ProductionResult {
    /// 每个等级信徒的单个产出（已含所有加成）
    pub per_cultist: [f64; 10],
    /// 总信徒产出/秒
    pub total_cultist_production: f64,
    /// 节点独立产出/秒（虚空编译器、维度裂隙、奇点引擎）
    pub node_independent_production: f64,
    /// 总产出/秒（信徒 + 节点独立 + token 流入不计入此处）
    pub total_production_per_sec: f64,
    /// SAN 效率系数
    pub san_efficiency: f64,
    /// 全局加成百分比（来自商店升级）
    pub global_bonus_pct: f64,
    /// 维度加成百分比（来自节点效果）
    pub dimension_bonus_pct: f64,
    /// 协同加成百分比
    pub synergy_bonus_pct: f64,
}

/// 计算信徒招募价格（纯函数）
/// price = base_price × growth^owned × (1 - 0.05 × discount_level)
pub fn cultist_recruit_price(tier: CultistTier, owned: u32, shop: &ShopState) -> f64 {
    let data = tier.data();
    let discount_level = shop
        .levels
        .get(&ShopUpgradeId::CultistDiscount)
        .copied()
        .unwrap_or(0);
    let discount_factor = (1.0 - 0.05 * discount_level as f64).max(0.0);
    data.base_price * data.price_growth.powi(owned as i32) * discount_factor
}

/// 计算节点购买价格（纯函数）
/// price = base_price × growth^owned × (1 - 0.05 × discount_level)
pub fn node_purchase_price(node_id: NodeId, owned: u32, shop: &ShopState) -> f64 {
    let data = node_id.data();
    let discount_level = shop
        .levels
        .get(&ShopUpgradeId::NodeDiscount)
        .copied()
        .unwrap_or(0);
    let discount_factor = (1.0 - 0.05 * discount_level as f64).max(0.0);
    data.base_price * data.price_growth.powi(owned as i32) * discount_factor
}

/// 计算批量购买总价（纯函数）
/// total = base × (growth^owned × (growth^count - 1)) / (growth - 1) × discount
pub fn batch_price(base: f64, growth: f64, owned: u32, count: u32, discount: f64) -> f64 {
    if count == 0 {
        return 0.0;
    }
    if (growth - 1.0).abs() < f64::EPSILON {
        return base * count as f64 * discount;
    }
    base * (growth.powi(owned as i32) * (growth.powi(count as i32) - 1.0)) / (growth - 1.0)
        * discount
}

/// 计算最大可购买数量（纯函数）
/// max_n = floor(log((budget × (growth-1)) / (base × growth^owned × discount) + 1) / log(growth))
pub fn max_purchasable(base: f64, growth: f64, owned: u32, budget: f64, discount: f64) -> u32 {
    if budget <= 0.0 || base <= 0.0 || discount <= 0.0 {
        return 0;
    }
    if growth <= 1.0 {
        return (budget / (base * discount)) as u32;
    }
    let numerator = budget * (growth - 1.0);
    let denominator = base * growth.powi(owned as i32) * discount;
    let inner = numerator / denominator + 1.0;
    if inner <= 0.0 {
        return 0;
    }
    let result = inner.ln() / growth.ln();
    if result < 0.0 {
        0
    } else {
        result.floor() as u32
    }
}

/// 计算 SAN 效率系数（纯函数）
/// 根据 SAN 等级返回基础效率，考虑熵屏障和拥抱疯狂的修正
pub fn san_efficiency(san: &SanState, nodes: &NodeState, shop: &ShopState) -> f64 {
    let current = san.current;

    // 基础效率按 SAN 等级
    let base_eff = if current <= 0.0 {
        0.0 // Collapsed
    } else if current < 20.0 {
        0.50 // Insane (1-19)
    } else if current < 40.0 {
        0.70 // Terrified (20-39)
    } else if current < 60.0 {
        0.85 // Anxious (40-59)
    } else if current < 80.0 {
        0.95 // Uneasy (60-79)
    } else {
        1.0 // Lucid (80-100)
    };

    let mut efficiency: f64 = base_eff;

    // 熵屏障：SAN < 30 时产出惩罚减半
    let has_entropy_barrier = nodes
        .owned
        .get(&NodeId::EntropyBarrier)
        .copied()
        .unwrap_or(0)
        > 0;
    if has_entropy_barrier && current < 30.0 {
        efficiency = 1.0 - (1.0 - base_eff) * 0.5;
    }

    // 拥抱疯狂：SAN < 40 时额外 +0.30
    let has_embrace_madness = shop
        .levels
        .get(&ShopUpgradeId::EmbraceMadness)
        .copied()
        .unwrap_or(0)
        > 0;
    if has_embrace_madness && current < 40.0 {
        efficiency += 0.30;
    }

    efficiency.clamp(0.0, 2.0)
}

/// 计算总产出（纯函数）
/// 汇总信徒产出 + 节点独立产出 + 各类加成
pub fn calculate_production(state: &GameState) -> ProductionResult {
    // === 全局加成（商店升级）===
    let mut global_bonus_pct = 0.0;

    // 深渊灌注: +0.08 per level
    let abyss_infusion_level = state
        .shop
        .levels
        .get(&ShopUpgradeId::AbyssInfusion)
        .copied()
        .unwrap_or(0);
    global_bonus_pct += 0.08 * abyss_infusion_level as f64;

    // 觉醒: ×2 (add 1.0)
    let has_awakening = state
        .shop
        .levels
        .get(&ShopUpgradeId::Awakening)
        .copied()
        .unwrap_or(0)
        > 0;
    if has_awakening {
        global_bonus_pct += 1.0;
    }

    // === 维度加成（节点效果）===
    let mut dimension_bonus_pct = 0.0;

    // OverclockedCooler: +0.08 per owned
    let overclocked_cooler = state
        .nodes
        .owned
        .get(&NodeId::OverclockedCooler)
        .copied()
        .unwrap_or(0);
    dimension_bonus_pct += 0.08 * overclocked_cooler as f64;

    // OverclockArray: +0.05 per owned
    let overclock_array = state
        .nodes
        .owned
        .get(&NodeId::OverclockArray)
        .copied()
        .unwrap_or(0);
    dimension_bonus_pct += 0.05 * overclock_array as f64;

    // EntropyReducer: +0.12 per owned
    let entropy_reducer = state
        .nodes
        .owned
        .get(&NodeId::EntropyReducer)
        .copied()
        .unwrap_or(0);
    dimension_bonus_pct += 0.12 * entropy_reducer as f64;

    // AbyssResonator: +0.10 per owned
    let abyss_resonator = state
        .nodes
        .owned
        .get(&NodeId::AbyssResonator)
        .copied()
        .unwrap_or(0);
    dimension_bonus_pct += 0.10 * abyss_resonator as f64;

    // CausalityWeaver: +0.20 per owned
    let causality_weaver = state
        .nodes
        .owned
        .get(&NodeId::CausalityWeaver)
        .copied()
        .unwrap_or(0);
    dimension_bonus_pct += 0.20 * causality_weaver as f64;

    // EyeOfTheAbyss: ×1.5 per owned (add 0.5 per owned)
    let eye_of_abyss = state
        .nodes
        .owned
        .get(&NodeId::EyeOfTheAbyss)
        .copied()
        .unwrap_or(0);
    dimension_bonus_pct += 0.50 * eye_of_abyss as f64;

    // === 协同加成 ===
    let mut synergy_bonus_pct = 0.0;
    let active = &state.synergies.active;

    if active.contains(&SynergyId::BotnetSwarm) {
        synergy_bonus_pct += 0.30;
    }
    if active.contains(&SynergyId::SiliconResonance) {
        synergy_bonus_pct += 0.20;
    }
    if active.contains(&SynergyId::QuantumOverlap) {
        synergy_bonus_pct += 0.20;
    }
    if active.contains(&SynergyId::AbyssSymphony) {
        synergy_bonus_pct += 0.50;
    }
    if active.contains(&SynergyId::CultistFlood) {
        synergy_bonus_pct += 0.30;
    }
    if active.contains(&SynergyId::UltimateFusion) {
        synergy_bonus_pct += 0.50;
    }
    if active.contains(&SynergyId::PerfectCult) {
        synergy_bonus_pct += 1.00;
    }
    if active.contains(&SynergyId::AbyssArchitect) {
        synergy_bonus_pct += 0.15;
    }

    // === SAN 效率 ===
    let san_eff = san_efficiency(&state.san, &state.nodes, &state.shop);

    // === 每等级信徒产出 ===
    let mut per_cultist = [0.0f64; 10];
    let multiplier = (1.0 + global_bonus_pct) * (1.0 + dimension_bonus_pct) * (1.0 + synergy_bonus_pct) * san_eff;

    for tier in CultistTier::ALL.iter() {
        let data = tier.data();
        per_cultist[tier.index()] = data.base_production * multiplier;
    }

    // === 总信徒产出 ===
    let mut total_cultist_production = 0.0;
    for tier in CultistTier::ALL.iter() {
        let count = state.cultists.counts[tier.index()];
        total_cultist_production += count as f64 * per_cultist[tier.index()];
    }

    // === 节点独立产出 ===
    let mut node_independent_production = 0.0;

    // VoidCompiler: 3000.0 per owned
    let void_compiler = state
        .nodes
        .owned
        .get(&NodeId::VoidCompiler)
        .copied()
        .unwrap_or(0);
    node_independent_production += 3000.0 * void_compiler as f64;

    // DimensionalRift: 50000.0 per owned
    let dimensional_rift = state
        .nodes
        .owned
        .get(&NodeId::DimensionalRift)
        .copied()
        .unwrap_or(0);
    node_independent_production += 50000.0 * dimensional_rift as f64;

    // SingularityEngine: 500000.0 per owned
    let singularity_engine = state
        .nodes
        .owned
        .get(&NodeId::SingularityEngine)
        .copied()
        .unwrap_or(0);
    node_independent_production += 500000.0 * singularity_engine as f64;

    // === 总产出 ===
    let total_production_per_sec = total_cultist_production + node_independent_production;

    ProductionResult {
        per_cultist,
        total_cultist_production,
        node_independent_production,
        total_production_per_sec,
        san_efficiency: san_eff,
        global_bonus_pct,
        dimension_bonus_pct,
        synergy_bonus_pct,
    }
}
