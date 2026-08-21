use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use chrono::{Timelike, Utc};

use super::cultist::CultistTier;
use super::nodes::{NodeId, NodeState};

/// 成就类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementCategory {
    /// 献祭（token 相关）
    Sacrifice,
    /// 崩溃（转生相关）
    Collapse,
    /// 教团（信徒相关）
    Cult,
    /// 建造（节点相关）
    Building,
    /// 隐藏成就
    Hidden,
}

/// 成就 ID（53 个成就）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementId {
    // === 献祭类 (10) ===
    FirstOffering,
    TokenTithe,
    WhisperHoarder,
    SilentFortune,
    AbyssalWealth,
    TokenFlood,
    MillionWhispers,
    BillionWhispers,
    TokenAddict,
    InfiniteStream,

    // === 崩溃类 (10) ===
    FirstCollapse,
    CycleBreaker,
    SerialCollapser,
    TruthSeeker,
    TruthHoarder,
    DimensionWalker,
    DimensionMaster,
    EternalReturner,
    AbyssVeteran,
    BeyondMadness,

    // === 教团类 (13) ===
    FirstRecruit,
    SmallCult,
    GrowingCongregation,
    CultLeader,
    MassCult,
    TierUnlocker,
    EliteCultist,
    CultistArmy,
    FullRoster,
    T10Recruiter,
    CultistLegion,
    DiverseCult,
    CultistOverlord,

    // === 建造类 (10) ===
    FirstNode,
    NodeCollector,
    Dim1Complete,
    Dim2Explorer,
    Dim2Complete,
    Dim3Explorer,
    Dim3Complete,
    NodeHoarder,
    MasterBuilder,
    Architect,

    // === 隐藏类 (10) ===
    NightOwl,
    SpeedRunner,
    ZeroSanity,
    PerfectBalance,
    Minimalist,
    SecretPath,
    AbyssGazer,
    ElderSign,
    Ouroboros,
    Transcendence,
}

/// 成就的静态数据
pub struct AchievementData {
    /// i18n 名称 key
    pub name_key: &'static str,
    /// i18n 描述 key
    pub desc_key: &'static str,
    /// 成就类别
    pub category: AchievementCategory,
    /// 奖励真理数量
    pub reward_truths: u32,
    /// 是否为隐藏成就
    pub is_hidden: bool,
}

impl AchievementId {
    /// 返回该成就的静态数据
    pub fn data(&self) -> AchievementData {
        match self {
            // === 献祭类 ===
            AchievementId::FirstOffering => AchievementData {
                name_key: "achievements.first_offering_name",
                desc_key: "achievements.first_offering_desc",
                category: AchievementCategory::Sacrifice,
                reward_truths: 0,
                is_hidden: false,
            },
            AchievementId::TokenTithe => AchievementData {
                name_key: "achievements.token_tithe_name",
                desc_key: "achievements.token_tithe_desc",
                category: AchievementCategory::Sacrifice,
                reward_truths: 1,
                is_hidden: false,
            },
            AchievementId::WhisperHoarder => AchievementData {
                name_key: "achievements.whisper_hoarder_name",
                desc_key: "achievements.whisper_hoarder_desc",
                category: AchievementCategory::Sacrifice,
                reward_truths: 1,
                is_hidden: false,
            },
            AchievementId::SilentFortune => AchievementData {
                name_key: "achievements.silent_fortune_name",
                desc_key: "achievements.silent_fortune_desc",
                category: AchievementCategory::Sacrifice,
                reward_truths: 2,
                is_hidden: false,
            },
            AchievementId::AbyssalWealth => AchievementData {
                name_key: "achievements.abyssal_wealth_name",
                desc_key: "achievements.abyssal_wealth_desc",
                category: AchievementCategory::Sacrifice,
                reward_truths: 3,
                is_hidden: false,
            },
            AchievementId::TokenFlood => AchievementData {
                name_key: "achievements.token_flood_name",
                desc_key: "achievements.token_flood_desc",
                category: AchievementCategory::Sacrifice,
                reward_truths: 3,
                is_hidden: false,
            },
            AchievementId::MillionWhispers => AchievementData {
                name_key: "achievements.million_whispers_name",
                desc_key: "achievements.million_whispers_desc",
                category: AchievementCategory::Sacrifice,
                reward_truths: 5,
                is_hidden: false,
            },
            AchievementId::BillionWhispers => AchievementData {
                name_key: "achievements.billion_whispers_name",
                desc_key: "achievements.billion_whispers_desc",
                category: AchievementCategory::Sacrifice,
                reward_truths: 10,
                is_hidden: false,
            },
            AchievementId::TokenAddict => AchievementData {
                name_key: "achievements.token_addict_name",
                desc_key: "achievements.token_addict_desc",
                category: AchievementCategory::Sacrifice,
                reward_truths: 5,
                is_hidden: false,
            },
            AchievementId::InfiniteStream => AchievementData {
                name_key: "achievements.infinite_stream_name",
                desc_key: "achievements.infinite_stream_desc",
                category: AchievementCategory::Sacrifice,
                reward_truths: 15,
                is_hidden: false,
            },
            // === 崩溃类 ===
            AchievementId::FirstCollapse => AchievementData {
                name_key: "achievements.first_collapse_name",
                desc_key: "achievements.first_collapse_desc",
                category: AchievementCategory::Collapse,
                reward_truths: 1,
                is_hidden: false,
            },
            AchievementId::CycleBreaker => AchievementData {
                name_key: "achievements.cycle_breaker_name",
                desc_key: "achievements.cycle_breaker_desc",
                category: AchievementCategory::Collapse,
                reward_truths: 2,
                is_hidden: false,
            },
            AchievementId::SerialCollapser => AchievementData {
                name_key: "achievements.serial_collapser_name",
                desc_key: "achievements.serial_collapser_desc",
                category: AchievementCategory::Collapse,
                reward_truths: 3,
                is_hidden: false,
            },
            AchievementId::TruthSeeker => AchievementData {
                name_key: "achievements.truth_seeker_name",
                desc_key: "achievements.truth_seeker_desc",
                category: AchievementCategory::Collapse,
                reward_truths: 2,
                is_hidden: false,
            },
            AchievementId::TruthHoarder => AchievementData {
                name_key: "achievements.truth_hoarder_name",
                desc_key: "achievements.truth_hoarder_desc",
                category: AchievementCategory::Collapse,
                reward_truths: 5,
                is_hidden: false,
            },
            AchievementId::DimensionWalker => AchievementData {
                name_key: "achievements.dimension_walker_name",
                desc_key: "achievements.dimension_walker_desc",
                category: AchievementCategory::Collapse,
                reward_truths: 3,
                is_hidden: false,
            },
            AchievementId::DimensionMaster => AchievementData {
                name_key: "achievements.dimension_master_name",
                desc_key: "achievements.dimension_master_desc",
                category: AchievementCategory::Collapse,
                reward_truths: 5,
                is_hidden: false,
            },
            AchievementId::EternalReturner => AchievementData {
                name_key: "achievements.eternal_returner_name",
                desc_key: "achievements.eternal_returner_desc",
                category: AchievementCategory::Collapse,
                reward_truths: 5,
                is_hidden: false,
            },
            AchievementId::AbyssVeteran => AchievementData {
                name_key: "achievements.abyss_veteran_name",
                desc_key: "achievements.abyss_veteran_desc",
                category: AchievementCategory::Collapse,
                reward_truths: 10,
                is_hidden: false,
            },
            AchievementId::BeyondMadness => AchievementData {
                name_key: "achievements.beyond_madness_name",
                desc_key: "achievements.beyond_madness_desc",
                category: AchievementCategory::Collapse,
                reward_truths: 15,
                is_hidden: false,
            },
            // === 教团类 ===
            AchievementId::FirstRecruit => AchievementData {
                name_key: "achievements.first_recruit_name",
                desc_key: "achievements.first_recruit_desc",
                category: AchievementCategory::Cult,
                reward_truths: 0,
                is_hidden: false,
            },
            AchievementId::SmallCult => AchievementData {
                name_key: "achievements.small_cult_name",
                desc_key: "achievements.small_cult_desc",
                category: AchievementCategory::Cult,
                reward_truths: 1,
                is_hidden: false,
            },
            AchievementId::GrowingCongregation => AchievementData {
                name_key: "achievements.growing_congregation_name",
                desc_key: "achievements.growing_congregation_desc",
                category: AchievementCategory::Cult,
                reward_truths: 2,
                is_hidden: false,
            },
            AchievementId::CultLeader => AchievementData {
                name_key: "achievements.cult_leader_name",
                desc_key: "achievements.cult_leader_desc",
                category: AchievementCategory::Cult,
                reward_truths: 3,
                is_hidden: false,
            },
            AchievementId::MassCult => AchievementData {
                name_key: "achievements.mass_cult_name",
                desc_key: "achievements.mass_cult_desc",
                category: AchievementCategory::Cult,
                reward_truths: 5,
                is_hidden: false,
            },
            AchievementId::TierUnlocker => AchievementData {
                name_key: "achievements.tier_unlocker_name",
                desc_key: "achievements.tier_unlocker_desc",
                category: AchievementCategory::Cult,
                reward_truths: 2,
                is_hidden: false,
            },
            AchievementId::EliteCultist => AchievementData {
                name_key: "achievements.elite_cultist_name",
                desc_key: "achievements.elite_cultist_desc",
                category: AchievementCategory::Cult,
                reward_truths: 3,
                is_hidden: false,
            },
            AchievementId::CultistArmy => AchievementData {
                name_key: "achievements.cultist_army_name",
                desc_key: "achievements.cultist_army_desc",
                category: AchievementCategory::Cult,
                reward_truths: 5,
                is_hidden: false,
            },
            AchievementId::FullRoster => AchievementData {
                name_key: "achievements.full_roster_name",
                desc_key: "achievements.full_roster_desc",
                category: AchievementCategory::Cult,
                reward_truths: 5,
                is_hidden: false,
            },
            AchievementId::T10Recruiter => AchievementData {
                name_key: "achievements.t10_recruiter_name",
                desc_key: "achievements.t10_recruiter_desc",
                category: AchievementCategory::Cult,
                reward_truths: 10,
                is_hidden: false,
            },
            AchievementId::CultistLegion => AchievementData {
                name_key: "achievements.cultist_legion_name",
                desc_key: "achievements.cultist_legion_desc",
                category: AchievementCategory::Cult,
                reward_truths: 8,
                is_hidden: false,
            },
            AchievementId::DiverseCult => AchievementData {
                name_key: "achievements.diverse_cult_name",
                desc_key: "achievements.diverse_cult_desc",
                category: AchievementCategory::Cult,
                reward_truths: 5,
                is_hidden: false,
            },
            AchievementId::CultistOverlord => AchievementData {
                name_key: "achievements.cultist_overlord_name",
                desc_key: "achievements.cultist_overlord_desc",
                category: AchievementCategory::Cult,
                reward_truths: 15,
                is_hidden: false,
            },
            // === 建造类 ===
            AchievementId::FirstNode => AchievementData {
                name_key: "achievements.first_node_name",
                desc_key: "achievements.first_node_desc",
                category: AchievementCategory::Building,
                reward_truths: 0,
                is_hidden: false,
            },
            AchievementId::NodeCollector => AchievementData {
                name_key: "achievements.node_collector_name",
                desc_key: "achievements.node_collector_desc",
                category: AchievementCategory::Building,
                reward_truths: 1,
                is_hidden: false,
            },
            AchievementId::Dim1Complete => AchievementData {
                name_key: "achievements.dim1_complete_name",
                desc_key: "achievements.dim1_complete_desc",
                category: AchievementCategory::Building,
                reward_truths: 3,
                is_hidden: false,
            },
            AchievementId::Dim2Explorer => AchievementData {
                name_key: "achievements.dim2_explorer_name",
                desc_key: "achievements.dim2_explorer_desc",
                category: AchievementCategory::Building,
                reward_truths: 2,
                is_hidden: false,
            },
            AchievementId::Dim2Complete => AchievementData {
                name_key: "achievements.dim2_complete_name",
                desc_key: "achievements.dim2_complete_desc",
                category: AchievementCategory::Building,
                reward_truths: 5,
                is_hidden: false,
            },
            AchievementId::Dim3Explorer => AchievementData {
                name_key: "achievements.dim3_explorer_name",
                desc_key: "achievements.dim3_explorer_desc",
                category: AchievementCategory::Building,
                reward_truths: 5,
                is_hidden: false,
            },
            AchievementId::Dim3Complete => AchievementData {
                name_key: "achievements.dim3_complete_name",
                desc_key: "achievements.dim3_complete_desc",
                category: AchievementCategory::Building,
                reward_truths: 15,
                is_hidden: false,
            },
            AchievementId::NodeHoarder => AchievementData {
                name_key: "achievements.node_hoarder_name",
                desc_key: "achievements.node_hoarder_desc",
                category: AchievementCategory::Building,
                reward_truths: 3,
                is_hidden: false,
            },
            AchievementId::MasterBuilder => AchievementData {
                name_key: "achievements.master_builder_name",
                desc_key: "achievements.master_builder_desc",
                category: AchievementCategory::Building,
                reward_truths: 8,
                is_hidden: false,
            },
            AchievementId::Architect => AchievementData {
                name_key: "achievements.architect_name",
                desc_key: "achievements.architect_desc",
                category: AchievementCategory::Building,
                reward_truths: 10,
                is_hidden: false,
            },
            // === 隐藏类 ===
            AchievementId::NightOwl => AchievementData {
                name_key: "achievements.night_owl_name",
                desc_key: "achievements.night_owl_desc",
                category: AchievementCategory::Hidden,
                reward_truths: 2,
                is_hidden: true,
            },
            AchievementId::SpeedRunner => AchievementData {
                name_key: "achievements.speed_runner_name",
                desc_key: "achievements.speed_runner_desc",
                category: AchievementCategory::Hidden,
                reward_truths: 3,
                is_hidden: true,
            },
            AchievementId::ZeroSanity => AchievementData {
                name_key: "achievements.zero_sanity_name",
                desc_key: "achievements.zero_sanity_desc",
                category: AchievementCategory::Hidden,
                reward_truths: 2,
                is_hidden: true,
            },
            AchievementId::PerfectBalance => AchievementData {
                name_key: "achievements.perfect_balance_name",
                desc_key: "achievements.perfect_balance_desc",
                category: AchievementCategory::Hidden,
                reward_truths: 3,
                is_hidden: true,
            },
            AchievementId::Minimalist => AchievementData {
                name_key: "achievements.minimalist_name",
                desc_key: "achievements.minimalist_desc",
                category: AchievementCategory::Hidden,
                reward_truths: 5,
                is_hidden: true,
            },
            AchievementId::SecretPath => AchievementData {
                name_key: "achievements.secret_path_name",
                desc_key: "achievements.secret_path_desc",
                category: AchievementCategory::Hidden,
                reward_truths: 5,
                is_hidden: true,
            },
            AchievementId::AbyssGazer => AchievementData {
                name_key: "achievements.abyss_gazer_name",
                desc_key: "achievements.abyss_gazer_desc",
                category: AchievementCategory::Hidden,
                reward_truths: 5,
                is_hidden: true,
            },
            AchievementId::ElderSign => AchievementData {
                name_key: "achievements.elder_sign_name",
                desc_key: "achievements.elder_sign_desc",
                category: AchievementCategory::Hidden,
                reward_truths: 8,
                is_hidden: true,
            },
            AchievementId::Ouroboros => AchievementData {
                name_key: "achievements.ouroboros_name",
                desc_key: "achievements.ouroboros_desc",
                category: AchievementCategory::Hidden,
                reward_truths: 10,
                is_hidden: true,
            },
            AchievementId::Transcendence => AchievementData {
                name_key: "achievements.transcendence_name",
                desc_key: "achievements.transcendence_desc",
                category: AchievementCategory::Hidden,
                reward_truths: 20,
                is_hidden: true,
            },
        }
    }

    /// 所有成就 ID 的列表
    pub const ALL: [AchievementId; 53] = [
        // 献祭类 (10)
        AchievementId::FirstOffering,
        AchievementId::TokenTithe,
        AchievementId::WhisperHoarder,
        AchievementId::SilentFortune,
        AchievementId::AbyssalWealth,
        AchievementId::TokenFlood,
        AchievementId::MillionWhispers,
        AchievementId::BillionWhispers,
        AchievementId::TokenAddict,
        AchievementId::InfiniteStream,
        // 崩溃类 (10)
        AchievementId::FirstCollapse,
        AchievementId::CycleBreaker,
        AchievementId::SerialCollapser,
        AchievementId::TruthSeeker,
        AchievementId::TruthHoarder,
        AchievementId::DimensionWalker,
        AchievementId::DimensionMaster,
        AchievementId::EternalReturner,
        AchievementId::AbyssVeteran,
        AchievementId::BeyondMadness,
        // 教团类 (13)
        AchievementId::FirstRecruit,
        AchievementId::SmallCult,
        AchievementId::GrowingCongregation,
        AchievementId::CultLeader,
        AchievementId::MassCult,
        AchievementId::TierUnlocker,
        AchievementId::EliteCultist,
        AchievementId::CultistArmy,
        AchievementId::FullRoster,
        AchievementId::T10Recruiter,
        AchievementId::CultistLegion,
        AchievementId::DiverseCult,
        AchievementId::CultistOverlord,
        // 建造类 (10)
        AchievementId::FirstNode,
        AchievementId::NodeCollector,
        AchievementId::Dim1Complete,
        AchievementId::Dim2Explorer,
        AchievementId::Dim2Complete,
        AchievementId::Dim3Explorer,
        AchievementId::Dim3Complete,
        AchievementId::NodeHoarder,
        AchievementId::MasterBuilder,
        AchievementId::Architect,
        // 隐藏类 (10)
        AchievementId::NightOwl,
        AchievementId::SpeedRunner,
        AchievementId::ZeroSanity,
        AchievementId::PerfectBalance,
        AchievementId::Minimalist,
        AchievementId::SecretPath,
        AchievementId::AbyssGazer,
        AchievementId::ElderSign,
        AchievementId::Ouroboros,
        AchievementId::Transcendence,
    ];
}

/// 成就检测上下文（从 GameState 各字段快照，避免借用冲突）
pub struct AchievementContext<'a> {
    pub total_tokens: u64,
    pub total_whispers_earned: f64,
    pub current_whispers: f64,
    pub total_rebirths: u32,
    pub forbidden_truths: u32,
    pub max_dimension: u32,
    pub cultists: &'a super::cultist::CultistState,
    pub nodes: &'a NodeState,
    pub total_playtime_secs: u64,
    pub peak_production: f64,
    pub san_current: f64,
    pub cycle_duration_secs: u64,
}

/// 成就状态：已解锁的成就集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementState {
    pub unlocked: HashSet<AchievementId>,
}

impl AchievementState {
    pub fn new() -> Self {
        Self {
            unlocked: HashSet::new(),
        }
    }

    /// 检测所有未解锁成就，返回新解锁的成就 ID 列表。
    /// 调用方负责根据返回的 ID 调用 `id.data().reward_truths` 发放真理奖励。
    pub fn check_all(&mut self, ctx: &AchievementContext) -> Vec<AchievementId> {
        let all_ids = AchievementId::ALL;
        let mut newly_unlocked = Vec::new();

        for &id in &all_ids {
            if self.unlocked.contains(&id) {
                continue;
            }
            if Self::is_condition_met(id, ctx) {
                self.unlocked.insert(id);
                newly_unlocked.push(id);
            }
        }

        newly_unlocked
    }

    /// 判断单个成就的达成条件
    fn is_condition_met(id: AchievementId, ctx: &AchievementContext) -> bool {
        let total_cultists = ctx.cultists.total_count();
        let counts = &ctx.cultists.counts;

        match id {
            // === 献祭类 (10) ===
            AchievementId::FirstOffering => ctx.total_tokens >= 1,
            AchievementId::TokenTithe => ctx.total_tokens >= 100,
            AchievementId::WhisperHoarder => ctx.current_whispers >= 10_000.0,
            AchievementId::SilentFortune => ctx.current_whispers >= 100_000.0,
            AchievementId::AbyssalWealth => ctx.current_whispers >= 1_000_000.0,
            AchievementId::TokenFlood => ctx.total_tokens >= 10_000,
            AchievementId::MillionWhispers => ctx.total_whispers_earned >= 1_000_000.0,
            AchievementId::BillionWhispers => ctx.total_whispers_earned >= 1_000_000_000.0,
            AchievementId::TokenAddict => ctx.total_tokens >= 100_000,
            AchievementId::InfiniteStream => ctx.total_tokens >= 1_000_000,

            // === 崩溃类 (10) ===
            AchievementId::FirstCollapse => ctx.total_rebirths >= 1,
            AchievementId::CycleBreaker => ctx.total_rebirths >= 3,
            AchievementId::SerialCollapser => ctx.total_rebirths >= 10,
            AchievementId::TruthSeeker => ctx.forbidden_truths >= 10,
            AchievementId::TruthHoarder => ctx.forbidden_truths >= 50,
            AchievementId::DimensionWalker => ctx.max_dimension >= 2,
            AchievementId::DimensionMaster => ctx.max_dimension >= 3,
            AchievementId::EternalReturner => ctx.total_rebirths >= 25,
            AchievementId::AbyssVeteran => ctx.total_rebirths >= 50,
            AchievementId::BeyondMadness => ctx.total_rebirths >= 100,

            // === 教团类 (13) ===
            AchievementId::FirstRecruit => total_cultists >= 1,
            AchievementId::SmallCult => total_cultists >= 10,
            AchievementId::GrowingCongregation => total_cultists >= 50,
            AchievementId::CultLeader => total_cultists >= 100,
            AchievementId::MassCult => total_cultists >= 500,
            AchievementId::TierUnlocker => counts[CultistTier::T5.index()] >= 1,
            AchievementId::EliteCultist => counts[CultistTier::T8.index()] >= 1,
            AchievementId::CultistArmy => total_cultists >= 1000,
            AchievementId::FullRoster => CultistTier::ALL.iter().all(|t| counts[t.index()] >= 1),
            AchievementId::T10Recruiter => counts[CultistTier::T10.index()] >= 1,
            AchievementId::CultistLegion => total_cultists >= 5000,
            AchievementId::DiverseCult => {
                // 所有拥有信徒的等级都至少有 10 个
                let tiers_with_any = CultistTier::ALL.iter().filter(|t| counts[t.index()] > 0).count();
                tiers_with_any > 0
                    && CultistTier::ALL.iter().all(|t| counts[t.index()] == 0 || counts[t.index()] >= 10)
            }
            AchievementId::CultistOverlord => total_cultists >= 10000,

            // === 建造类 (10) ===
            AchievementId::FirstNode => Self::total_nodes(ctx.nodes) >= 1,
            AchievementId::NodeCollector => Self::total_nodes(ctx.nodes) >= 10,
            AchievementId::Dim1Complete => Self::all_dim_types_owned(ctx.nodes, 1),
            AchievementId::Dim2Explorer => ctx.nodes.dimension_total(2) >= 1,
            AchievementId::Dim2Complete => Self::all_dim_types_owned(ctx.nodes, 2),
            AchievementId::Dim3Explorer => ctx.nodes.dimension_total(3) >= 1,
            AchievementId::Dim3Complete => Self::all_dim_types_owned(ctx.nodes, 3),
            AchievementId::NodeHoarder => Self::total_nodes(ctx.nodes) >= 50,
            AchievementId::MasterBuilder => Self::total_nodes(ctx.nodes) >= 100,
            AchievementId::Architect => {
                Self::all_dim_types_owned(ctx.nodes, 1)
                    && Self::all_dim_types_owned(ctx.nodes, 2)
                    && Self::all_dim_types_owned(ctx.nodes, 3)
            }

            // === 隐藏类 (10) ===
            AchievementId::NightOwl => {
                let hour = Utc::now().hour();
                hour <= 3 // 0:00 - 3:59
            }
            AchievementId::SpeedRunner => {
                // 在 10 分钟内完成转生
                ctx.total_rebirths >= 1 && ctx.cycle_duration_secs < 600
            }
            AchievementId::ZeroSanity => ctx.san_current <= 0.0,
            AchievementId::PerfectBalance => (ctx.san_current - 50.0).abs() < 0.5,
            AchievementId::Minimalist => {
                // 只有 T1 信徒（且至少有 1 个）且完成过转生
                ctx.total_rebirths >= 1
                    && counts[CultistTier::T1.index()] >= 1
                    && counts[CultistTier::T2.index()..].iter().all(|&c| c == 0)
            }
            AchievementId::SecretPath => {
                // 特殊条件：拥有所有维度一节点各 ≥5 且 0 个信徒
                total_cultists == 0
                    && NodeId::ALL.iter()
                        .filter(|n| n.data().dimension == 1)
                        .all(|n| ctx.nodes.count(*n) >= 5)
            }
            AchievementId::AbyssGazer => ctx.total_playtime_secs >= 3600,
            AchievementId::ElderSign => ctx.forbidden_truths >= 100,
            AchievementId::Ouroboros => {
                ctx.total_rebirths >= 10 && ctx.total_whispers_earned >= 1_000_000.0
            }
            AchievementId::Transcendence => {
                ctx.max_dimension >= 3
                    && ctx.forbidden_truths >= 200
                    && ctx.total_rebirths >= 50
            }
        }
    }

    /// 计算所有节点的总拥有数量
    fn total_nodes(nodes: &NodeState) -> u32 {
        NodeId::ALL.iter().map(|id| nodes.count(*id)).sum()
    }

    /// 检查某维度的所有节点类型是否都至少拥有 1 个
    fn all_dim_types_owned(nodes: &NodeState, dimension: u32) -> bool {
        NodeId::ALL
            .iter()
            .filter(|id| id.data().dimension == dimension)
            .all(|id| nodes.count(*id) >= 1)
    }
}

impl Default for AchievementState {
    fn default() -> Self {
        Self::new()
    }
}
