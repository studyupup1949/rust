use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::cultist::UnlockCondition;
use super::currency::CurrencyState;
use super::production;
use super::shop::ShopState;

/// 节点效果类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
    /// 产出加成
    ProductionBoost,
    /// 理智降低减缓
    SanReduction,
    /// 特殊效果
    Special,
    /// 独立效果（不依赖其他系统）
    Independent,
    /// 信徒相关加成
    CultistBoost,
    /// 价格折扣
    PriceDiscount,
    /// 自动化
    Automation,
}

/// 节点 ID，三个维度共 30 种节点
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeId {
    // === 维度 1（10 个）===
    OverclockedCooler,
    DarknetRelay,
    CacheAccelerator,
    OverclockArray,
    FirewallShard,
    SanityAnchor,
    IsolationSandbox,
    RecruitmentPost,
    CodeAltar,
    DataRecycler,

    // === 维度 2（12 个）===
    EntropyReducer,
    SynapticWeb,
    QuantumSuperposer,
    AbyssResonator,
    VoidCompiler,
    CoolantTransmutation,
    MindBackup,
    EntropyBarrier,
    MutationCatalyst,
    MindSplitter,
    TimeDilationField,
    AutoAltar,

    // === 维度 3（8 个）===
    DimensionalRift,
    ElderTouch,
    SingularityEngine,
    CausalityWeaver,
    EternalEcho,
    EyeOfTheAbyss,
    CausalityWeapon,
    SpacetimeFolder,
}

/// 节点的静态数据
pub struct NodeData {
    /// i18n 名称 key
    pub name_key: &'static str,
    /// i18n 描述 key
    pub desc_key: &'static str,
    /// 效果类型
    pub effect_type: EffectType,
    /// 基础价格（碎语）
    pub base_price: f64,
    /// 真理消耗（维度 3 节点）
    pub truth_cost: Option<u32>,
    /// 涨价系数
    pub price_growth: f64,
    /// 所属维度
    pub dimension: u32,
    /// 解锁条件
    pub unlock_condition: UnlockCondition,
    /// 最大数量限制（None 表示无限）
    pub max_count: Option<u32>,
}

impl NodeId {
    /// 返回该节点的静态数据
    pub fn data(&self) -> NodeData {
        match self {
            // === 维度 1 ===
            NodeId::OverclockedCooler => NodeData {
                name_key: "nodes.overclocked_cooler_name",
                desc_key: "nodes.overclocked_cooler_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 50.0,
                truth_cost: None,
                price_growth: 1.15,
                dimension: 1,
                unlock_condition: UnlockCondition::Always,
                max_count: None,
            },
            NodeId::DarknetRelay => NodeData {
                name_key: "nodes.darknet_relay_name",
                desc_key: "nodes.darknet_relay_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 200.0,
                truth_cost: None,
                price_growth: 1.18,
                dimension: 1,
                unlock_condition: UnlockCondition::TotalWhispers(100.0),
                max_count: None,
            },
            NodeId::CacheAccelerator => NodeData {
                name_key: "nodes.cache_accelerator_name",
                desc_key: "nodes.cache_accelerator_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 800.0,
                truth_cost: None,
                price_growth: 1.20,
                dimension: 1,
                unlock_condition: UnlockCondition::TotalWhispers(500.0),
                max_count: None,
            },
            NodeId::OverclockArray => NodeData {
                name_key: "nodes.overclock_array_name",
                desc_key: "nodes.overclock_array_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 3000.0,
                truth_cost: None,
                price_growth: 1.22,
                dimension: 1,
                unlock_condition: UnlockCondition::TotalWhispers(2000.0),
                max_count: None,
            },
            NodeId::FirewallShard => NodeData {
                name_key: "nodes.firewall_shard_name",
                desc_key: "nodes.firewall_shard_desc",
                effect_type: EffectType::SanReduction,
                base_price: 500.0,
                truth_cost: None,
                price_growth: 1.25,
                dimension: 1,
                unlock_condition: UnlockCondition::TotalWhispers(300.0),
                max_count: None,
            },
            NodeId::SanityAnchor => NodeData {
                name_key: "nodes.sanity_anchor_name",
                desc_key: "nodes.sanity_anchor_desc",
                effect_type: EffectType::SanReduction,
                base_price: 2000.0,
                truth_cost: None,
                price_growth: 1.28,
                dimension: 1,
                unlock_condition: UnlockCondition::TotalWhispers(1000.0),
                max_count: None,
            },
            NodeId::IsolationSandbox => NodeData {
                name_key: "nodes.isolation_sandbox_name",
                desc_key: "nodes.isolation_sandbox_desc",
                effect_type: EffectType::SanReduction,
                base_price: 5000.0,
                truth_cost: None,
                price_growth: 1.30,
                dimension: 1,
                unlock_condition: UnlockCondition::TotalWhispers(3000.0),
                max_count: None,
            },
            NodeId::RecruitmentPost => NodeData {
                name_key: "nodes.recruitment_post_name",
                desc_key: "nodes.recruitment_post_desc",
                effect_type: EffectType::CultistBoost,
                base_price: 1000.0,
                truth_cost: None,
                price_growth: 1.20,
                dimension: 1,
                unlock_condition: UnlockCondition::TotalWhispers(500.0),
                max_count: None,
            },
            NodeId::CodeAltar => NodeData {
                name_key: "nodes.code_altar_name",
                desc_key: "nodes.code_altar_desc",
                effect_type: EffectType::Special,
                base_price: 4000.0,
                truth_cost: None,
                price_growth: 1.25,
                dimension: 1,
                unlock_condition: UnlockCondition::TotalWhispers(2000.0),
                max_count: None,
            },
            NodeId::DataRecycler => NodeData {
                name_key: "nodes.data_recycler_name",
                desc_key: "nodes.data_recycler_desc",
                effect_type: EffectType::Independent,
                base_price: 1500.0,
                truth_cost: None,
                price_growth: 1.18,
                dimension: 1,
                unlock_condition: UnlockCondition::TotalWhispers(800.0),
                max_count: None,
            },
            // === 维度 2 ===
            NodeId::EntropyReducer => NodeData {
                name_key: "nodes.entropy_reducer_name",
                desc_key: "nodes.entropy_reducer_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 50000.0,
                truth_cost: None,
                price_growth: 1.20,
                dimension: 2,
                unlock_condition: UnlockCondition::Dimension(2),
                max_count: None,
            },
            NodeId::SynapticWeb => NodeData {
                name_key: "nodes.synaptic_web_name",
                desc_key: "nodes.synaptic_web_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 150000.0,
                truth_cost: None,
                price_growth: 1.22,
                dimension: 2,
                unlock_condition: UnlockCondition::Dimension(2),
                max_count: None,
            },
            NodeId::QuantumSuperposer => NodeData {
                name_key: "nodes.quantum_superposer_name",
                desc_key: "nodes.quantum_superposer_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 500000.0,
                truth_cost: None,
                price_growth: 1.25,
                dimension: 2,
                unlock_condition: UnlockCondition::DimensionAndRebirths(2, 1),
                max_count: None,
            },
            NodeId::AbyssResonator => NodeData {
                name_key: "nodes.abyss_resonator_name",
                desc_key: "nodes.abyss_resonator_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 1500000.0,
                truth_cost: None,
                price_growth: 1.28,
                dimension: 2,
                unlock_condition: UnlockCondition::DimensionAndRebirths(2, 2),
                max_count: None,
            },
            NodeId::VoidCompiler => NodeData {
                name_key: "nodes.void_compiler_name",
                desc_key: "nodes.void_compiler_desc",
                effect_type: EffectType::Special,
                base_price: 5000000.0,
                truth_cost: None,
                price_growth: 1.30,
                dimension: 2,
                unlock_condition: UnlockCondition::DimensionAndRebirths(2, 3),
                max_count: None,
            },
            NodeId::CoolantTransmutation => NodeData {
                name_key: "nodes.coolant_transmutation_name",
                desc_key: "nodes.coolant_transmutation_desc",
                effect_type: EffectType::SanReduction,
                base_price: 100000.0,
                truth_cost: None,
                price_growth: 1.25,
                dimension: 2,
                unlock_condition: UnlockCondition::Dimension(2),
                max_count: None,
            },
            NodeId::MindBackup => NodeData {
                name_key: "nodes.mind_backup_name",
                desc_key: "nodes.mind_backup_desc",
                effect_type: EffectType::SanReduction,
                base_price: 300000.0,
                truth_cost: None,
                price_growth: 1.28,
                dimension: 2,
                unlock_condition: UnlockCondition::DimensionAndRebirths(2, 1),
                max_count: None,
            },
            NodeId::EntropyBarrier => NodeData {
                name_key: "nodes.entropy_barrier_name",
                desc_key: "nodes.entropy_barrier_desc",
                effect_type: EffectType::SanReduction,
                base_price: 800000.0,
                truth_cost: None,
                price_growth: 1.30,
                dimension: 2,
                unlock_condition: UnlockCondition::DimensionAndRebirths(2, 2),
                max_count: None,
            },
            NodeId::MutationCatalyst => NodeData {
                name_key: "nodes.mutation_catalyst_name",
                desc_key: "nodes.mutation_catalyst_desc",
                effect_type: EffectType::Special,
                base_price: 200000.0,
                truth_cost: None,
                price_growth: 1.22,
                dimension: 2,
                unlock_condition: UnlockCondition::Dimension(2),
                max_count: None,
            },
            NodeId::MindSplitter => NodeData {
                name_key: "nodes.mind_splitter_name",
                desc_key: "nodes.mind_splitter_desc",
                effect_type: EffectType::Special,
                base_price: 1000000.0,
                truth_cost: None,
                price_growth: 1.28,
                dimension: 2,
                unlock_condition: UnlockCondition::DimensionAndRebirths(2, 2),
                max_count: None,
            },
            NodeId::TimeDilationField => NodeData {
                name_key: "nodes.time_dilation_field_name",
                desc_key: "nodes.time_dilation_field_desc",
                effect_type: EffectType::Independent,
                base_price: 3000000.0,
                truth_cost: None,
                price_growth: 1.30,
                dimension: 2,
                unlock_condition: UnlockCondition::DimensionAndRebirths(2, 3),
                max_count: None,
            },
            NodeId::AutoAltar => NodeData {
                name_key: "nodes.auto_altar_name",
                desc_key: "nodes.auto_altar_desc",
                effect_type: EffectType::Automation,
                base_price: 2000000.0,
                truth_cost: None,
                price_growth: 1.25,
                dimension: 2,
                unlock_condition: UnlockCondition::DimensionAndRebirths(2, 2),
                max_count: None,
            },
            // === 维度 3 ===
            NodeId::DimensionalRift => NodeData {
                name_key: "nodes.dimensional_rift_name",
                desc_key: "nodes.dimensional_rift_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 100000000.0,
                truth_cost: Some(10),
                price_growth: 1.30,
                dimension: 3,
                unlock_condition: UnlockCondition::Dimension(3),
                max_count: None,
            },
            NodeId::ElderTouch => NodeData {
                name_key: "nodes.elder_touch_name",
                desc_key: "nodes.elder_touch_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 500000000.0,
                truth_cost: Some(25),
                price_growth: 1.35,
                dimension: 3,
                unlock_condition: UnlockCondition::DimensionAndRebirths(3, 2),
                max_count: None,
            },
            NodeId::SingularityEngine => NodeData {
                name_key: "nodes.singularity_engine_name",
                desc_key: "nodes.singularity_engine_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 2000000000.0,
                truth_cost: Some(50),
                price_growth: 1.40,
                dimension: 3,
                unlock_condition: UnlockCondition::DimensionAndRebirths(3, 5),
                max_count: None,
            },
            NodeId::CausalityWeaver => NodeData {
                name_key: "nodes.causality_weaver_name",
                desc_key: "nodes.causality_weaver_desc",
                effect_type: EffectType::Special,
                base_price: 1000000000.0,
                truth_cost: Some(30),
                price_growth: 1.35,
                dimension: 3,
                unlock_condition: UnlockCondition::DimensionAndRebirths(3, 3),
                max_count: None,
            },
            NodeId::EternalEcho => NodeData {
                name_key: "nodes.eternal_echo_name",
                desc_key: "nodes.eternal_echo_desc",
                effect_type: EffectType::Special,
                base_price: 5000000000.0,
                truth_cost: Some(75),
                price_growth: 1.40,
                dimension: 3,
                unlock_condition: UnlockCondition::DimensionAndTruths(3, 30),
                max_count: None,
            },
            NodeId::EyeOfTheAbyss => NodeData {
                name_key: "nodes.eye_of_the_abyss_name",
                desc_key: "nodes.eye_of_the_abyss_desc",
                effect_type: EffectType::Special,
                base_price: 10000000000.0,
                truth_cost: Some(100),
                price_growth: 1.50,
                dimension: 3,
                unlock_condition: UnlockCondition::DimensionAndTruths(3, 50),
                max_count: Some(3),
            },
            NodeId::CausalityWeapon => NodeData {
                name_key: "nodes.causality_weapon_name",
                desc_key: "nodes.causality_weapon_desc",
                effect_type: EffectType::ProductionBoost,
                base_price: 25000000000.0,
                truth_cost: Some(150),
                price_growth: 1.50,
                dimension: 3,
                unlock_condition: UnlockCondition::DimensionAndTruths(3, 75),
                max_count: Some(2),
            },
            NodeId::SpacetimeFolder => NodeData {
                name_key: "nodes.spacetime_folder_name",
                desc_key: "nodes.spacetime_folder_desc",
                effect_type: EffectType::Independent,
                base_price: 100000000000.0,
                truth_cost: Some(200),
                price_growth: 1.50,
                dimension: 3,
                unlock_condition: UnlockCondition::DimensionAndTruths(3, 100),
                max_count: Some(1),
            },
        }
    }

    /// 所有节点的列表
    pub const ALL: [NodeId; 30] = [
        // 维度 1
        NodeId::OverclockedCooler,
        NodeId::DarknetRelay,
        NodeId::CacheAccelerator,
        NodeId::OverclockArray,
        NodeId::FirewallShard,
        NodeId::SanityAnchor,
        NodeId::IsolationSandbox,
        NodeId::RecruitmentPost,
        NodeId::CodeAltar,
        NodeId::DataRecycler,
        // 维度 2
        NodeId::EntropyReducer,
        NodeId::SynapticWeb,
        NodeId::QuantumSuperposer,
        NodeId::AbyssResonator,
        NodeId::VoidCompiler,
        NodeId::CoolantTransmutation,
        NodeId::MindBackup,
        NodeId::EntropyBarrier,
        NodeId::MutationCatalyst,
        NodeId::MindSplitter,
        NodeId::TimeDilationField,
        NodeId::AutoAltar,
        // 维度 3
        NodeId::DimensionalRift,
        NodeId::ElderTouch,
        NodeId::SingularityEngine,
        NodeId::CausalityWeaver,
        NodeId::EternalEcho,
        NodeId::EyeOfTheAbyss,
        NodeId::CausalityWeapon,
        NodeId::SpacetimeFolder,
    ];
}

/// 节点状态：每种节点的拥有数量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub owned: HashMap<NodeId, u32>,
}

impl NodeState {
    pub fn new() -> Self {
        Self {
            owned: HashMap::new(),
        }
    }

    /// 获取某节点的拥有数量
    pub fn count(&self, node_id: NodeId) -> u32 {
        self.owned.get(&node_id).copied().unwrap_or(0)
    }

    /// 获取某维度的所有节点总数
    pub fn dimension_total(&self, dimension: u32) -> u32 {
        NodeId::ALL
            .iter()
            .filter(|id| id.data().dimension == dimension)
            .map(|id| self.count(*id))
            .sum()
    }

    /// 购买节点：检查余额/上限/真理、扣费、增加数量
    ///
    /// 注意：解锁条件（NodeLocked）的检查由调用方（engine 层）负责，
    /// 因为 NodeState 没有访问完整 GameState 的能力。
    pub fn purchase(
        &mut self,
        node_id: NodeId,
        currency: &mut CurrencyState,
        shop: &ShopState,
    ) -> Result<(), PurchaseError> {
        let data = node_id.data();
        let current_count = self.count(node_id);

        // 检查最大数量限制
        if let Some(max) = data.max_count {
            if current_count >= max {
                return Err(PurchaseError::MaxCountReached);
            }
        }

        // 计算碎语价格
        let price = production::node_purchase_price(node_id, current_count, shop);

        // 检查碎语余额
        if currency.whispers < price {
            return Err(PurchaseError::InsufficientWhispers {
                cost: price,
                balance: currency.whispers,
            });
        }

        // 检查真理消耗（维度三节点）
        if let Some(truth_cost) = data.truth_cost {
            if currency.forbidden_truths < truth_cost {
                return Err(PurchaseError::InsufficientTruths {
                    cost: truth_cost,
                    balance: currency.forbidden_truths,
                });
            }
        }

        // 扣除碎语
        currency.whispers -= price;

        // 扣除真理（如果需要）
        if let Some(truth_cost) = data.truth_cost {
            currency.forbidden_truths -= truth_cost;
        }

        // 增加拥有数量
        *self.owned.entry(node_id).or_insert(0) += 1;

        Ok(())
    }
}

impl Default for NodeState {
    fn default() -> Self {
        Self::new()
    }
}

/// 节点购买错误
#[derive(Debug, Clone, PartialEq)]
pub enum PurchaseError {
    InsufficientWhispers { cost: f64, balance: f64 },
    InsufficientTruths { cost: u32, balance: u32 },
    MaxCountReached,
    NodeLocked,
}

