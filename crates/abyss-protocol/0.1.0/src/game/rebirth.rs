// 转生系统 — 真理计算 + 重置逻辑

use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::GameState;
use super::nodes::NodeId;
use super::san::SanState;
use super::shop::ShopUpgradeId;

/// 转生结算数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebirthSummary {
    pub cycle_duration_secs: u64,
    pub whispers_harvested: f64,
    pub nodes_constructed: u32,
    pub peak_production: f64,
    pub san_repairs: u32,
    pub mutations_witnessed: u32,
    pub truths_gained: u32,
    pub total_truths_after: u32,
}

/// 转生错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum RebirthError {
    InsufficientWhispers,
}

/// 计算转生获得的禁忌真理（纯函数）
///
/// 被动转生 coefficient = 1.0，主动转生 = 0.7
/// 公式: floor(log2(total_whispers / 10,000)) × coefficient
/// + mind_splitter_count
/// × (1 + 0.5 × causality_weapon_count)
/// × 1.5 (if has_collapse_accelerator)
/// 最终 floor 为 u32
pub fn calculate_rebirth_truths(
    total_whispers: f64,
    is_passive: bool,
    mind_splitter_count: u32,
    causality_weapon_count: u32,
    has_collapse_accelerator: bool,
) -> u32 {
    if total_whispers < 10_000.0 {
        return 0;
    }

    let coefficient = if is_passive { 1.0 } else { 0.7 };
    let base = (total_whispers / 10_000.0).log2().floor() * coefficient;

    // 意识分裂器: 加上 mind_splitter_count
    let with_splitter = base + mind_splitter_count as f64;

    // 因果律武器: 乘以 (1 + 0.5 × count)
    let with_weapon = with_splitter * (1.0 + 0.5 * causality_weapon_count as f64);

    // 崩溃加速: 乘以 1.5
    let with_accel = if has_collapse_accelerator {
        with_weapon * 1.5
    } else {
        with_weapon
    };

    // 最终 floor 为 u32，确保非负
    with_accel.floor().max(0.0) as u32
}

/// 检查是否满足主动转生条件（累计碎语 ≥ 50,000）
pub fn can_active_rebirth(state: &GameState) -> bool {
    state.stats.total_whispers_earned >= 50_000.0
}

/// 执行转生重置逻辑
///
/// 1. 计算真理
/// 2. 构建 RebirthSummary
/// 3. 计算碎语保留（意识备份 / 永恒回响，取较大者）
/// 4. 计算信徒保留（涅槃协议 / 永恒教团）
/// 5. 重置：碎语→保留量、信徒→保留量、节点清零、SAN→上限、修复次数归零
/// 6. 增加真理、转生次数
/// 7. 重置周期统计、协同效应
pub fn execute_rebirth(state: &mut GameState, is_passive: bool) -> RebirthSummary {
    // --- 1. 计算真理 ---
    let mind_splitter_count = state.nodes.count(NodeId::MindSplitter);
    let causality_weapon_count = state.nodes.count(NodeId::CausalityWeapon);
    let has_collapse_accel = state.shop.has(ShopUpgradeId::CollapseAccelerator);

    let truths_gained = calculate_rebirth_truths(
        state.stats.total_whispers_earned,
        is_passive,
        mind_splitter_count,
        causality_weapon_count,
        has_collapse_accel,
    );

    // --- 2. 构建 RebirthSummary ---
    let now = Utc::now();
    let cycle_duration_secs = now
        .signed_duration_since(state.current_cycle_start)
        .num_seconds()
        .max(0) as u64;

    let summary = RebirthSummary {
        cycle_duration_secs,
        whispers_harvested: state.stats.whispers_this_cycle,
        nodes_constructed: state.stats.nodes_this_cycle,
        peak_production: state.stats.peak_production_rate,
        san_repairs: state.san.repair_count_this_cycle,
        mutations_witnessed: state.stats.mutations_this_cycle,
        truths_gained,
        total_truths_after: state.currency.forbidden_truths + truths_gained,
    };

    // --- 3. 计算碎语保留 ---
    let current_whispers = state.currency.whispers;

    // 意识备份 (MindBackup): 被动转生专属, 保留 2% × count (上限 20%)
    let mind_backup_count = state.nodes.count(NodeId::MindBackup);
    let mind_backup_pct = if is_passive {
        (0.02 * mind_backup_count as f64).min(0.20)
    } else {
        0.0
    };

    // 永恒回响 (EternalEcho): 被动/主动均可, 保留 3% × count (上限 30%)
    let eternal_echo_count = state.nodes.count(NodeId::EternalEcho);
    let eternal_echo_pct = (0.03 * eternal_echo_count as f64).min(0.30);

    // 取两者中较大的保留百分比
    let whisper_retention_pct = mind_backup_pct.max(eternal_echo_pct);
    let retained_whispers = current_whispers * whisper_retention_pct;

    // --- 4. 计算信徒保留 ---
    let mut retained_cultists = [0u32; 10];

    // 涅槃协议 (NirvanaProtocol): 被动转生专属, 保留所有等级 10% (floor)
    let has_nirvana = state.shop.has(ShopUpgradeId::NirvanaProtocol);
    if is_passive && has_nirvana {
        for i in 0..10 {
            retained_cultists[i] = retained_cultists[i].max(state.cultists.counts[i] / 10);
        }
    }

    // 永恒教团 (EternalCult): 被动/主动均可, 保留 T5+ 的 20% (floor)
    let has_eternal_cult = state.shop.has(ShopUpgradeId::EternalCult);
    if has_eternal_cult {
        // T5 index=4, T6=5, ..., T10=9
        for i in 4..10 {
            retained_cultists[i] = retained_cultists[i].max(state.cultists.counts[i] / 5);
        }
    }

    // --- 5. 重置 ---
    state.currency.whispers = retained_whispers;
    state.cultists.counts = retained_cultists;
    state.nodes.owned.clear();
    let max_san = SanState::max_san(&state.shop);
    state.san.current = max_san;
    state.san.repair_count_this_cycle = 0;

    // --- 6. 增加真理、转生次数 ---
    state.currency.forbidden_truths += truths_gained;
    state.stats.total_rebirths += 1;

    // --- 7. 重置周期统计 ---
    state.stats.mutations_this_cycle = 0;
    state.stats.whispers_this_cycle = 0.0;
    state.stats.nodes_this_cycle = 0;
    state.current_cycle_start = now;

    // 重置协同效应
    state.synergies.active.clear();

    summary
}
