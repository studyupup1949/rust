use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::currency::CurrencyState;

/// 商店路径
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShopPath {
    /// 力量之路
    Power,
    /// 知识之路
    Knowledge,
    /// 疯狂之路
    Madness,
    /// 超越之路
    Transcendence,
    /// 教团之路
    Cult,
}

/// 商店升级 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShopUpgradeId {
    // === 力量之路 (6) ===
    AbyssInfusion,
    SacrificeEfficiency,
    NodeDiscount,
    CultistDiscount,
    MassSacrifice,
    SynthesisMastery,

    // === 知识之路 (8) ===
    DimensionalSense,
    AbyssAnalytics,
    TemporalSense,
    AbyssWhisper,
    MutationMastery,
    AutoPurge,
    CultistAutonomy,
    Precognition,

    // === 疯狂之路 (7) ===
    EmbraceMadness,
    AbyssGambler,
    VoidResonance,
    CollapseAccelerator,
    NirvanaProtocol,
    AbyssPact,
    ChaosHarvest,

    // === 超越之路 (6) ===
    Dimension2Pass,
    Dimension3Pass,
    AbyssArchitect,
    ElderWhisperer,
    EternalArchive,
    Awakening,

    // === 教团之路 (6) ===
    ZealousPreaching,
    ShadowPromotion,
    CultExpansion,
    SoulHarvest,
    AbyssSummoning,
    EternalCult,
}

/// 商店升级的静态数据
pub struct ShopUpgradeData {
    /// i18n 名称 key
    pub name_key: &'static str,
    /// i18n 描述 key
    pub desc_key: &'static str,
    /// 所属路径
    pub path: ShopPath,
    /// 每级真理消耗（多级升级为 Vec，一次性购买为单元素）
    pub costs: Vec<u32>,
    /// 最大等级
    pub max_level: u32,
    /// 是否为一次性购买
    pub is_one_time: bool,
}

impl ShopUpgradeId {
    /// 返回该升级的静态数据
    pub fn data(&self) -> ShopUpgradeData {
        match self {
            // === 力量之路 ===
            ShopUpgradeId::AbyssInfusion => ShopUpgradeData {
                name_key: "shop.abyss_infusion_name",
                desc_key: "shop.abyss_infusion_desc",
                path: ShopPath::Power,
                costs: vec![1, 3, 5, 10, 20],
                max_level: 5,
                is_one_time: false,
            },
            ShopUpgradeId::SacrificeEfficiency => ShopUpgradeData {
                name_key: "shop.sacrifice_efficiency_name",
                desc_key: "shop.sacrifice_efficiency_desc",
                path: ShopPath::Power,
                costs: vec![2, 5, 10, 20],
                max_level: 4,
                is_one_time: false,
            },
            ShopUpgradeId::NodeDiscount => ShopUpgradeData {
                name_key: "shop.node_discount_name",
                desc_key: "shop.node_discount_desc",
                path: ShopPath::Power,
                costs: vec![3, 8, 15],
                max_level: 3,
                is_one_time: false,
            },
            ShopUpgradeId::CultistDiscount => ShopUpgradeData {
                name_key: "shop.cultist_discount_name",
                desc_key: "shop.cultist_discount_desc",
                path: ShopPath::Power,
                costs: vec![3, 8, 15],
                max_level: 3,
                is_one_time: false,
            },
            ShopUpgradeId::MassSacrifice => ShopUpgradeData {
                name_key: "shop.mass_sacrifice_name",
                desc_key: "shop.mass_sacrifice_desc",
                path: ShopPath::Power,
                costs: vec![10],
                max_level: 1,
                is_one_time: true,
            },
            ShopUpgradeId::SynthesisMastery => ShopUpgradeData {
                name_key: "shop.synthesis_mastery_name",
                desc_key: "shop.synthesis_mastery_desc",
                path: ShopPath::Power,
                costs: vec![25],
                max_level: 1,
                is_one_time: true,
            },
            // === 知识之路 ===
            ShopUpgradeId::DimensionalSense => ShopUpgradeData {
                name_key: "shop.dimensional_sense_name",
                desc_key: "shop.dimensional_sense_desc",
                path: ShopPath::Knowledge,
                costs: vec![1, 3, 5],
                max_level: 3,
                is_one_time: false,
            },
            ShopUpgradeId::AbyssAnalytics => ShopUpgradeData {
                name_key: "shop.abyss_analytics_name",
                desc_key: "shop.abyss_analytics_desc",
                path: ShopPath::Knowledge,
                costs: vec![2, 5, 10],
                max_level: 3,
                is_one_time: false,
            },
            ShopUpgradeId::TemporalSense => ShopUpgradeData {
                name_key: "shop.temporal_sense_name",
                desc_key: "shop.temporal_sense_desc",
                path: ShopPath::Knowledge,
                costs: vec![3, 8],
                max_level: 2,
                is_one_time: false,
            },
            ShopUpgradeId::AbyssWhisper => ShopUpgradeData {
                name_key: "shop.abyss_whisper_name",
                desc_key: "shop.abyss_whisper_desc",
                path: ShopPath::Knowledge,
                costs: vec![5, 12, 25],
                max_level: 3,
                is_one_time: false,
            },
            ShopUpgradeId::MutationMastery => ShopUpgradeData {
                name_key: "shop.mutation_mastery_name",
                desc_key: "shop.mutation_mastery_desc",
                path: ShopPath::Knowledge,
                costs: vec![8, 20],
                max_level: 2,
                is_one_time: false,
            },
            ShopUpgradeId::AutoPurge => ShopUpgradeData {
                name_key: "shop.auto_purge_name",
                desc_key: "shop.auto_purge_desc",
                path: ShopPath::Knowledge,
                costs: vec![15],
                max_level: 1,
                is_one_time: true,
            },
            ShopUpgradeId::CultistAutonomy => ShopUpgradeData {
                name_key: "shop.cultist_autonomy_name",
                desc_key: "shop.cultist_autonomy_desc",
                path: ShopPath::Knowledge,
                costs: vec![20],
                max_level: 1,
                is_one_time: true,
            },
            ShopUpgradeId::Precognition => ShopUpgradeData {
                name_key: "shop.precognition_name",
                desc_key: "shop.precognition_desc",
                path: ShopPath::Knowledge,
                costs: vec![30],
                max_level: 1,
                is_one_time: true,
            },
            // === 疯狂之路 ===
            ShopUpgradeId::EmbraceMadness => ShopUpgradeData {
                name_key: "shop.embrace_madness_name",
                desc_key: "shop.embrace_madness_desc",
                path: ShopPath::Madness,
                costs: vec![2, 5, 10, 20, 40],
                max_level: 5,
                is_one_time: false,
            },
            ShopUpgradeId::AbyssGambler => ShopUpgradeData {
                name_key: "shop.abyss_gambler_name",
                desc_key: "shop.abyss_gambler_desc",
                path: ShopPath::Madness,
                costs: vec![5, 15],
                max_level: 2,
                is_one_time: false,
            },
            ShopUpgradeId::VoidResonance => ShopUpgradeData {
                name_key: "shop.void_resonance_name",
                desc_key: "shop.void_resonance_desc",
                path: ShopPath::Madness,
                costs: vec![8, 20, 40],
                max_level: 3,
                is_one_time: false,
            },
            ShopUpgradeId::CollapseAccelerator => ShopUpgradeData {
                name_key: "shop.collapse_accelerator_name",
                desc_key: "shop.collapse_accelerator_desc",
                path: ShopPath::Madness,
                costs: vec![10, 25],
                max_level: 2,
                is_one_time: false,
            },
            ShopUpgradeId::NirvanaProtocol => ShopUpgradeData {
                name_key: "shop.nirvana_protocol_name",
                desc_key: "shop.nirvana_protocol_desc",
                path: ShopPath::Madness,
                costs: vec![20],
                max_level: 1,
                is_one_time: true,
            },
            ShopUpgradeId::AbyssPact => ShopUpgradeData {
                name_key: "shop.abyss_pact_name",
                desc_key: "shop.abyss_pact_desc",
                path: ShopPath::Madness,
                costs: vec![35],
                max_level: 1,
                is_one_time: true,
            },
            ShopUpgradeId::ChaosHarvest => ShopUpgradeData {
                name_key: "shop.chaos_harvest_name",
                desc_key: "shop.chaos_harvest_desc",
                path: ShopPath::Madness,
                costs: vec![50],
                max_level: 1,
                is_one_time: true,
            },
            // === 超越之路 ===
            ShopUpgradeId::Dimension2Pass => ShopUpgradeData {
                name_key: "shop.dimension2_pass_name",
                desc_key: "shop.dimension2_pass_desc",
                path: ShopPath::Transcendence,
                costs: vec![5],
                max_level: 1,
                is_one_time: true,
            },
            ShopUpgradeId::Dimension3Pass => ShopUpgradeData {
                name_key: "shop.dimension3_pass_name",
                desc_key: "shop.dimension3_pass_desc",
                path: ShopPath::Transcendence,
                costs: vec![25],
                max_level: 1,
                is_one_time: true,
            },
            ShopUpgradeId::AbyssArchitect => ShopUpgradeData {
                name_key: "shop.abyss_architect_name",
                desc_key: "shop.abyss_architect_desc",
                path: ShopPath::Transcendence,
                costs: vec![10, 25, 50],
                max_level: 3,
                is_one_time: false,
            },
            ShopUpgradeId::ElderWhisperer => ShopUpgradeData {
                name_key: "shop.elder_whisperer_name",
                desc_key: "shop.elder_whisperer_desc",
                path: ShopPath::Transcendence,
                costs: vec![15, 35],
                max_level: 2,
                is_one_time: false,
            },
            ShopUpgradeId::EternalArchive => ShopUpgradeData {
                name_key: "shop.eternal_archive_name",
                desc_key: "shop.eternal_archive_desc",
                path: ShopPath::Transcendence,
                costs: vec![40],
                max_level: 1,
                is_one_time: true,
            },
            ShopUpgradeId::Awakening => ShopUpgradeData {
                name_key: "shop.awakening_name",
                desc_key: "shop.awakening_desc",
                path: ShopPath::Transcendence,
                costs: vec![100],
                max_level: 1,
                is_one_time: true,
            },
            // === 教团之路 ===
            ShopUpgradeId::ZealousPreaching => ShopUpgradeData {
                name_key: "shop.zealous_preaching_name",
                desc_key: "shop.zealous_preaching_desc",
                path: ShopPath::Cult,
                costs: vec![2, 5, 10, 20],
                max_level: 4,
                is_one_time: false,
            },
            ShopUpgradeId::ShadowPromotion => ShopUpgradeData {
                name_key: "shop.shadow_promotion_name",
                desc_key: "shop.shadow_promotion_desc",
                path: ShopPath::Cult,
                costs: vec![3, 8, 15],
                max_level: 3,
                is_one_time: false,
            },
            ShopUpgradeId::CultExpansion => ShopUpgradeData {
                name_key: "shop.cult_expansion_name",
                desc_key: "shop.cult_expansion_desc",
                path: ShopPath::Cult,
                costs: vec![5, 12, 25],
                max_level: 3,
                is_one_time: false,
            },
            ShopUpgradeId::SoulHarvest => ShopUpgradeData {
                name_key: "shop.soul_harvest_name",
                desc_key: "shop.soul_harvest_desc",
                path: ShopPath::Cult,
                costs: vec![15],
                max_level: 1,
                is_one_time: true,
            },
            ShopUpgradeId::AbyssSummoning => ShopUpgradeData {
                name_key: "shop.abyss_summoning_name",
                desc_key: "shop.abyss_summoning_desc",
                path: ShopPath::Cult,
                costs: vec![30],
                max_level: 1,
                is_one_time: true,
            },
            ShopUpgradeId::EternalCult => ShopUpgradeData {
                name_key: "shop.eternal_cult_name",
                desc_key: "shop.eternal_cult_desc",
                path: ShopPath::Cult,
                costs: vec![50],
                max_level: 1,
                is_one_time: true,
            },
        }
    }

    /// 所有升级的列表
    pub const ALL: [ShopUpgradeId; 33] = [
        // 力量之路
        ShopUpgradeId::AbyssInfusion,
        ShopUpgradeId::SacrificeEfficiency,
        ShopUpgradeId::NodeDiscount,
        ShopUpgradeId::CultistDiscount,
        ShopUpgradeId::MassSacrifice,
        ShopUpgradeId::SynthesisMastery,
        // 知识之路
        ShopUpgradeId::DimensionalSense,
        ShopUpgradeId::AbyssAnalytics,
        ShopUpgradeId::TemporalSense,
        ShopUpgradeId::AbyssWhisper,
        ShopUpgradeId::MutationMastery,
        ShopUpgradeId::AutoPurge,
        ShopUpgradeId::CultistAutonomy,
        ShopUpgradeId::Precognition,
        // 疯狂之路
        ShopUpgradeId::EmbraceMadness,
        ShopUpgradeId::AbyssGambler,
        ShopUpgradeId::VoidResonance,
        ShopUpgradeId::CollapseAccelerator,
        ShopUpgradeId::NirvanaProtocol,
        ShopUpgradeId::AbyssPact,
        ShopUpgradeId::ChaosHarvest,
        // 超越之路
        ShopUpgradeId::Dimension2Pass,
        ShopUpgradeId::Dimension3Pass,
        ShopUpgradeId::AbyssArchitect,
        ShopUpgradeId::ElderWhisperer,
        ShopUpgradeId::EternalArchive,
        ShopUpgradeId::Awakening,
        // 教团之路
        ShopUpgradeId::ZealousPreaching,
        ShopUpgradeId::ShadowPromotion,
        ShopUpgradeId::CultExpansion,
        ShopUpgradeId::SoulHarvest,
        ShopUpgradeId::AbyssSummoning,
        ShopUpgradeId::EternalCult,
    ];
}

/// 商店状态：每种升级的当前等级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopState {
    pub levels: HashMap<ShopUpgradeId, u32>,
}

impl ShopState {
    pub fn new() -> Self {
        Self {
            levels: HashMap::new(),
        }
    }

    /// 购买升级：检查真理余额/等级上限、扣费、升级
    pub fn purchase(
        &mut self,
        upgrade_id: ShopUpgradeId,
        currency: &mut CurrencyState,
    ) -> Result<(), ShopError> {
        let data = upgrade_id.data();
        let current_level = self.level(upgrade_id);

        // 检查是否已达最大等级
        if current_level >= data.max_level {
            return Err(ShopError::AlreadyMaxLevel);
        }

        // 获取当前等级对应的真理消耗
        let cost = data.costs[current_level as usize];

        // 检查真理余额
        if currency.forbidden_truths < cost {
            return Err(ShopError::InsufficientTruths {
                cost,
                balance: currency.forbidden_truths,
            });
        }

        // 扣除真理
        currency.forbidden_truths -= cost;

        // 升级等级 +1
        *self.levels.entry(upgrade_id).or_insert(0) += 1;

        Ok(())
    }

    /// 获取某升级的当前等级（未购买返回 0）
    pub fn level(&self, upgrade_id: ShopUpgradeId) -> u32 {
        self.levels.get(&upgrade_id).copied().unwrap_or(0)
    }

    /// 检查某升级是否已购买（等级 >= 1）
    pub fn has(&self, upgrade_id: ShopUpgradeId) -> bool {
        self.level(upgrade_id) >= 1
    }
}

impl Default for ShopState {
    fn default() -> Self {
        Self::new()
    }
}

/// 商店购买错误
#[derive(Debug, Clone, PartialEq)]
pub enum ShopError {
    InsufficientTruths { cost: u32, balance: u32 },
    AlreadyMaxLevel,
}

