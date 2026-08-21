// 游戏引擎 — tick 循环编排

use rand::Rng;

use super::achievements::{AchievementContext, AchievementId};
use super::cultist::{CultistTier, CultistState};
use super::mutation::{self, MutationResult};
use super::nodes::NodeId;
use super::production::{self, ProductionResult};
use super::rebirth::{self, RebirthSummary};
use super::san::SanState;
use super::shop::ShopUpgradeId;
use super::synergy::SynergyId;
use super::GameState;

/// Tick 结果，包含本帧发生的事件
#[derive(Debug, Clone)]
pub struct TickResult {
    pub production: ProductionResult,
    pub mutation: Option<MutationResult>,
    pub new_achievements: Vec<AchievementId>,
    pub new_synergies: Vec<SynergyId>,
    pub lost_synergies: Vec<SynergyId>,
    pub dimension_unlocked: Option<u32>,
    pub rebirth_triggered: Option<RebirthSummary>,
    pub auto_recruits: Vec<(CultistTier, u32)>,
    pub auto_fusions: Vec<(CultistTier, u32)>,
}

/// 批量购买模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchMode {
    X1,
    X10,
    X100,
    XMax,
}

impl BatchMode {
    pub fn next(self) -> Self {
        match self {
            BatchMode::X1 => BatchMode::X10,
            BatchMode::X10 => BatchMode::X100,
            BatchMode::X100 => BatchMode::XMax,
            BatchMode::XMax => BatchMode::X1,
        }
    }

    pub fn count(self) -> Option<u32> {
        match self {
            BatchMode::X1 => Some(1),
            BatchMode::X10 => Some(10),
            BatchMode::X100 => Some(100),
            BatchMode::XMax => None,
        }
    }
}

/// 执行完整的 tick 循环
///
/// 按顺序执行：
/// 1. 计算总产出
/// 2. 累加碎语
/// 3. 更新 SAN 衰减
/// 4. 检测变异事件
/// 5. SAN 崩溃检查（降至 0 触发被动转生）
/// 6. 执行自动化（招募所/自动祭坛/自动净化/信徒自治）
/// 7. 检测协同效应
/// 8. 检测成就
/// 9. 检测维度解锁
/// 10. 更新统计数据
pub fn tick<R: Rng>(state: &mut GameState, dt: f64, rng: &mut R) -> TickResult {
    let mut auto_recruits: Vec<(CultistTier, u32)> = Vec::new();
    let mut auto_fusions: Vec<(CultistTier, u32)> = Vec::new();

    // === 1. 计算总产出 ===
    let prod = production::calculate_production(state);

    // === 2. 累加碎语 ===
    let whisper_gain = prod.total_production_per_sec * dt;
    state.currency.whispers += whisper_gain;
    state.stats.whispers_this_cycle += whisper_gain;
    state.stats.total_whispers_earned += whisper_gain;

    // === 3. 更新 SAN 衰减 ===
    let decay = SanState::decay_rate(
        prod.total_production_per_sec,
        &state.nodes,
        &state.shop,
        &state.synergies,
    );
    state.san.current = (state.san.current - decay * dt).max(0.0);

    // === 4. 检测变异事件 ===
    let mutation_result = mutation::check_mutation(
        state.stats.max_dimension_unlocked,
        prod.total_production_per_sec,
        dt,
        &state.nodes,
        &state.shop,
        &state.synergies,
        rng,
    );

    // 应用变异结果
    if let Some(ref m) = mutation_result {
        state.currency.whispers += m.whispers_gained;
        state.stats.whispers_this_cycle += m.whispers_gained;
        state.stats.total_whispers_earned += m.whispers_gained;
        state.san.current = (state.san.current - m.san_cost).max(0.0);
        state.stats.mutations_this_cycle += 1;
    }

    // === 5. SAN 崩溃检查 ===
    if state.san.current <= 0.0 {
        let summary = rebirth::execute_rebirth(state, true);
        return TickResult {
            production: prod,
            mutation: mutation_result,
            new_achievements: Vec::new(),
            new_synergies: Vec::new(),
            lost_synergies: Vec::new(),
            dimension_unlocked: None,
            rebirth_triggered: Some(summary),
            auto_recruits,
            auto_fusions,
        };
    }

    // === 6. 执行自动化 ===
    run_automation(state, dt, &mut auto_recruits, &mut auto_fusions);

    // === 7. 检测协同效应 ===
    let (new_synergies, lost_synergies) =
        state.synergies.check_all(&state.cultists, &state.nodes);

    // === 8. 检测成就 ===
    let cycle_duration_secs = chrono::Utc::now()
        .signed_duration_since(state.current_cycle_start)
        .num_seconds()
        .max(0) as u64;

    let ctx = AchievementContext {
        total_tokens: state.stats.total_tokens_consumed,
        total_whispers_earned: state.stats.total_whispers_earned,
        current_whispers: state.currency.whispers,
        total_rebirths: state.stats.total_rebirths,
        forbidden_truths: state.currency.forbidden_truths,
        max_dimension: state.stats.max_dimension_unlocked,
        cultists: &state.cultists,
        nodes: &state.nodes,
        total_playtime_secs: state.stats.total_playtime_seconds,
        peak_production: state.stats.peak_production_rate,
        san_current: state.san.current,
        cycle_duration_secs,
    };
    let new_achievements = state.achievements.check_all(&ctx);

    // 发放成就真理奖励
    for &ach_id in &new_achievements {
        let reward = ach_id.data().reward_truths;
        state.currency.forbidden_truths += reward;
    }

    // === 9. 检测维度解锁 ===
    let dimension_unlocked = check_dimension_unlock(state);

    // === 10. 更新统计数据 ===
    if prod.total_production_per_sec > state.stats.peak_production_rate {
        state.stats.peak_production_rate = prod.total_production_per_sec;
    }
    state.stats.total_playtime_seconds += dt.ceil() as u64;

    TickResult {
        production: prod,
        mutation: mutation_result,
        new_achievements,
        new_synergies,
        lost_synergies,
        dimension_unlocked,
        rebirth_triggered: None,
        auto_recruits,
        auto_fusions,
    }
}

/// 维度解锁判定
///
/// - 维度一：始终解锁（初始值）
/// - 维度二：累计碎语 ≥ 50,000 OR 转生次数 ≥ 1 OR 维度二通行证已购买
/// - 维度三：禁忌真理 ≥ 15 AND 转生次数 ≥ 3 OR 维度三通行证已购买
///
/// 返回新解锁的维度编号（如果有）
fn check_dimension_unlock(state: &mut GameState) -> Option<u32> {
    let current_max = state.stats.max_dimension_unlocked;

    // 维度二判定
    if current_max < 2 {
        let has_dim2_pass = state.shop.has(ShopUpgradeId::Dimension2Pass);
        let whispers_enough = state.stats.total_whispers_earned >= 50_000.0;
        let has_rebirth = state.stats.total_rebirths >= 1;

        if has_dim2_pass || whispers_enough || has_rebirth {
            state.stats.max_dimension_unlocked = 2;
            return Some(2);
        }
    }

    // 维度三判定
    if current_max < 3 && state.stats.max_dimension_unlocked >= 2 {
        let has_dim3_pass = state.shop.has(ShopUpgradeId::Dimension3Pass);
        let truths_enough = state.currency.forbidden_truths >= 15;
        let rebirths_enough = state.stats.total_rebirths >= 3;

        if has_dim3_pass || (truths_enough && rebirths_enough) {
            state.stats.max_dimension_unlocked = 3;
            return Some(3);
        }
    }

    None
}

/// 执行自动化逻辑
///
/// - 信徒招募所 (RecruitmentPost): 每 60 秒自动招募 1 个 T1（每个招募所）
/// - 自动祭坛 (AutoAltar): 每 30 秒自动招募最高可用等级信徒 1 个（每个祭坛）
/// - 自动净化 (AutoPurge): SAN < 25 时自动修复
/// - 信徒自治 (CultistAutonomy): 自动合成（简化：每 60 秒尝试合成一次最低可合成等级）
///
/// 使用 dt 累积方式简化实现：按概率近似（dt/interval 的概率触发）
fn run_automation(
    state: &mut GameState,
    dt: f64,
    auto_recruits: &mut Vec<(CultistTier, u32)>,
    auto_fusions: &mut Vec<(CultistTier, u32)>,
) {
    // --- 信徒招募所: 每 60 秒自动招募 1 个 T1 ---
    let recruitment_post_count = state.nodes.count(NodeId::RecruitmentPost);
    if recruitment_post_count > 0 {
        // 信徒洪流协同：招募所速度翻倍（间隔减半）
        let interval = if state.synergies.active.contains(&SynergyId::CultistFlood) {
            30.0
        } else {
            60.0
        };
        let recruits_per_tick = (recruitment_post_count as f64 * dt / interval).floor() as u32;
        if recruits_per_tick > 0 {
            state.cultists.counts[CultistTier::T1.index()] += recruits_per_tick;
            auto_recruits.push((CultistTier::T1, recruits_per_tick));
        }
    }

    // --- 自动祭坛: 每 30 秒自动招募最高可用等级信徒 1 个 ---
    let auto_altar_count = state.nodes.count(NodeId::AutoAltar);
    if auto_altar_count > 0 {
        let recruits_per_tick = (auto_altar_count as f64 * dt / 30.0).floor() as u32;
        if recruits_per_tick > 0 {
            // 找到最高可用等级（有足够碎语招募的最高等级）
            if let Some(tier) = find_highest_affordable_tier(state) {
                let price = production::cultist_recruit_price(
                    tier,
                    state.cultists.counts[tier.index()],
                    &state.shop,
                );
                let actual_recruits = recruits_per_tick.min(
                    (state.currency.whispers / price).floor() as u32
                );
                if actual_recruits > 0 {
                    // 逐个扣费（价格递增）
                    let mut recruited = 0u32;
                    for _ in 0..actual_recruits {
                        let p = production::cultist_recruit_price(
                            tier,
                            state.cultists.counts[tier.index()],
                            &state.shop,
                        );
                        if state.currency.whispers >= p {
                            state.currency.whispers -= p;
                            state.cultists.counts[tier.index()] += 1;
                            recruited += 1;
                        } else {
                            break;
                        }
                    }
                    if recruited > 0 {
                        auto_recruits.push((tier, recruited));
                    }
                }
            }
        }
    }

    // --- 自动净化: SAN < 25 时自动修复至 35 ---
    let has_auto_purge = state.shop.has(ShopUpgradeId::AutoPurge);
    if has_auto_purge && state.san.current < 25.0 {
        let max_san = SanState::max_san(&state.shop);
        let target = 35.0_f64.min(max_san);
        // 持续修复直到 SAN >= target 或碎语不足
        while state.san.current < target {
            let cost = state.san.repair_cost();
            if state.currency.whispers < cost {
                break;
            }
            let amount = state.san.repair_amount(&state.nodes);
            state.currency.whispers -= cost;
            state.san.current = (state.san.current + amount).min(max_san);
            state.san.repair_count_this_cycle += 1;
        }
    }

    // --- 信徒自治: 自动合成 ---
    let has_cultist_autonomy = state.shop.has(ShopUpgradeId::CultistAutonomy);
    if has_cultist_autonomy {
        let fusion_cost = CultistState::fusion_cost(&state.shop);
        // 尝试从低到高合成
        for tier in &CultistTier::ALL[..9] {
            // T1-T9
            if state.cultists.counts[tier.index()] >= fusion_cost {
                let next_tier = tier.next_tier().unwrap();
                state.cultists.counts[tier.index()] -= fusion_cost;
                state.cultists.counts[next_tier.index()] += 1;
                auto_fusions.push((*tier, 1));
                state.stats.total_fusions += 1;
                // 数据回收站返还
                let recycler_count = state.nodes.count(NodeId::DataRecycler);
                if recycler_count > 0 {
                    let data = tier.data();
                    let refund = production::batch_price(
                        data.base_price,
                        data.price_growth,
                        state.cultists.counts[tier.index()],
                        fusion_cost,
                        1.0,
                    ) * 0.20
                        * recycler_count as f64;
                    state.currency.whispers += refund;
                }
                break; // 每 tick 只合成一次
            }
        }
    }
}

/// 找到最高可用等级（有足够碎语招募的最高等级）
fn find_highest_affordable_tier(state: &GameState) -> Option<CultistTier> {
    for tier in CultistTier::ALL.iter().rev() {
        let price = production::cultist_recruit_price(
            *tier,
            state.cultists.counts[tier.index()],
            &state.shop,
        );
        if state.currency.whispers >= price {
            return Some(*tier);
        }
    }
    None
}

/// 计算离线收益
///
/// - 未购买时间感知：效率 0%（无离线收益）
/// - 已购买时间感知：基础效率 20%
/// - 时间膨胀场：每个 +10%
/// - 离线时间上限 8 小时（28800 秒）
pub fn calculate_offline_earnings(
    production_per_sec: f64,
    offline_seconds: f64,
    has_temporal_sense: bool,
    time_dilation_count: u32,
) -> f64 {
    if !has_temporal_sense {
        return 0.0;
    }

    let capped_seconds = offline_seconds.min(28800.0);
    let efficiency = 0.20 + 0.10 * time_dilation_count as f64;
    production_per_sec * capped_seconds * efficiency
}
