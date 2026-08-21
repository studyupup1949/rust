use std::collections::HashSet;
use serde::{Deserialize, Serialize};

use super::cultist::CultistState;
use super::nodes::{NodeId, NodeState};

/// 协同效应 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SynergyId {
    /// 僵尸网络蜂群
    BotnetSwarm,
    /// 硅基共振
    SiliconResonance,
    /// 量子叠加
    QuantumOverlap,
    /// 深渊交响
    AbyssSymphony,
    /// 信徒洪流
    CultistFlood,
    /// 理智前线
    SanityFrontline,
    /// 突变风暴
    MutationStorm,
    /// 终极融合
    UltimateFusion,
    /// 完美教团
    PerfectCult,
    /// 深渊建筑师
    AbyssArchitect,
}

/// 协同效应的静态数据
pub struct SynergyData {
    /// i18n 名称 key
    pub name_key: &'static str,
    /// i18n 触发条件描述 key
    pub condition_desc_key: &'static str,
    /// i18n 效果描述 key
    pub effect_desc_key: &'static str,
}

impl SynergyId {
    /// 返回该协同效应的静态数据
    pub fn data(&self) -> SynergyData {
        match self {
            SynergyId::BotnetSwarm => SynergyData {
                name_key: "synergies.botnet_swarm_name",
                condition_desc_key: "synergies.botnet_swarm_condition",
                effect_desc_key: "synergies.botnet_swarm_effect",
            },
            SynergyId::SiliconResonance => SynergyData {
                name_key: "synergies.silicon_resonance_name",
                condition_desc_key: "synergies.silicon_resonance_condition",
                effect_desc_key: "synergies.silicon_resonance_effect",
            },
            SynergyId::QuantumOverlap => SynergyData {
                name_key: "synergies.quantum_overlap_name",
                condition_desc_key: "synergies.quantum_overlap_condition",
                effect_desc_key: "synergies.quantum_overlap_effect",
            },
            SynergyId::AbyssSymphony => SynergyData {
                name_key: "synergies.abyss_symphony_name",
                condition_desc_key: "synergies.abyss_symphony_condition",
                effect_desc_key: "synergies.abyss_symphony_effect",
            },
            SynergyId::CultistFlood => SynergyData {
                name_key: "synergies.cultist_flood_name",
                condition_desc_key: "synergies.cultist_flood_condition",
                effect_desc_key: "synergies.cultist_flood_effect",
            },
            SynergyId::SanityFrontline => SynergyData {
                name_key: "synergies.sanity_frontline_name",
                condition_desc_key: "synergies.sanity_frontline_condition",
                effect_desc_key: "synergies.sanity_frontline_effect",
            },
            SynergyId::MutationStorm => SynergyData {
                name_key: "synergies.mutation_storm_name",
                condition_desc_key: "synergies.mutation_storm_condition",
                effect_desc_key: "synergies.mutation_storm_effect",
            },
            SynergyId::UltimateFusion => SynergyData {
                name_key: "synergies.ultimate_fusion_name",
                condition_desc_key: "synergies.ultimate_fusion_condition",
                effect_desc_key: "synergies.ultimate_fusion_effect",
            },
            SynergyId::PerfectCult => SynergyData {
                name_key: "synergies.perfect_cult_name",
                condition_desc_key: "synergies.perfect_cult_condition",
                effect_desc_key: "synergies.perfect_cult_effect",
            },
            SynergyId::AbyssArchitect => SynergyData {
                name_key: "synergies.abyss_architect_name",
                condition_desc_key: "synergies.abyss_architect_condition",
                effect_desc_key: "synergies.abyss_architect_effect",
            },
        }
    }

    /// 所有协同效应的列表
    pub const ALL: [SynergyId; 10] = [
        SynergyId::BotnetSwarm,
        SynergyId::SiliconResonance,
        SynergyId::QuantumOverlap,
        SynergyId::AbyssSymphony,
        SynergyId::CultistFlood,
        SynergyId::SanityFrontline,
        SynergyId::MutationStorm,
        SynergyId::UltimateFusion,
        SynergyId::PerfectCult,
        SynergyId::AbyssArchitect,
    ];
}

/// 协同效应状态：当前激活的协同效应集合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynergyState {
    pub active: HashSet<SynergyId>,
}

impl SynergyState {
    pub fn new() -> Self {
        Self {
            active: HashSet::new(),
        }
    }

    /// 检测所有协同效应的激活状态
    /// 返回 (新激活的协同, 新失活的协同)
    pub fn check_all(
        &mut self,
        cultists: &CultistState,
        nodes: &NodeState,
    ) -> (Vec<SynergyId>, Vec<SynergyId>) {
        let mut newly_activated = Vec::new();
        let mut newly_deactivated = Vec::new();

        for &synergy_id in SynergyId::ALL.iter() {
            let condition_met = Self::check_condition(synergy_id, cultists, nodes, &self.active);
            let was_active = self.active.contains(&synergy_id);

            if condition_met && !was_active {
                self.active.insert(synergy_id);
                newly_activated.push(synergy_id);
            } else if !condition_met && was_active {
                self.active.remove(&synergy_id);
                newly_deactivated.push(synergy_id);
            }
        }

        (newly_activated, newly_deactivated)
    }

    /// 检查单个协同效应的条件是否满足
    fn check_condition(
        synergy_id: SynergyId,
        cultists: &CultistState,
        nodes: &NodeState,
        _active: &HashSet<SynergyId>,
    ) -> bool {
        match synergy_id {
            // 僵尸网络: T2 信徒 ≥20 + 暗网中继站 ≥3
            SynergyId::BotnetSwarm => {
                cultists.counts[1] >= 20
                    && nodes.count(NodeId::DarknetRelay) >= 3
            }
            // 硅基共鸣: T3 信徒 ≥10 + 超频阵列 ≥2
            SynergyId::SiliconResonance => {
                cultists.counts[2] >= 10
                    && nodes.count(NodeId::OverclockArray) >= 2
            }
            // 量子叠加: T5 信徒 ≥5 + 量子叠加器 ≥2
            SynergyId::QuantumOverlap => {
                cultists.counts[4] >= 5
                    && nodes.count(NodeId::QuantumSuperposer) >= 2
            }
            // 深渊交响: T6 信徒 ≥3 + 深渊共鸣腔 ≥2
            SynergyId::AbyssSymphony => {
                cultists.counts[5] >= 3
                    && nodes.count(NodeId::AbyssResonator) >= 2
            }
            // 信徒洪流: 信徒总数 ≥100 + 信徒招募所 ≥3
            SynergyId::CultistFlood => {
                cultists.total_count() >= 100
                    && nodes.count(NodeId::RecruitmentPost) >= 3
            }
            // 理智防线: 防火墙碎片 ≥5 + 冷却液血液化 ≥3
            SynergyId::SanityFrontline => {
                nodes.count(NodeId::FirewallShard) >= 5
                    && nodes.count(NodeId::CoolantTransmutation) >= 3
            }
            // 变异风暴: 变异催化剂 ≥5 + 隔离沙箱 ≥3
            SynergyId::MutationStorm => {
                nodes.count(NodeId::MutationCatalyst) >= 5
                    && nodes.count(NodeId::IsolationSandbox) >= 3
            }
            // 终极融合: 每种维度三产出节点各 ≥1
            // 维度三产出节点: DimensionalRift, ElderTouch, SingularityEngine, CausalityWeapon
            SynergyId::UltimateFusion => {
                nodes.count(NodeId::DimensionalRift) >= 1
                    && nodes.count(NodeId::ElderTouch) >= 1
                    && nodes.count(NodeId::SingularityEngine) >= 1
                    && nodes.count(NodeId::CausalityWeapon) >= 1
            }
            // 完美教团: 每种等级的信徒各 ≥1
            SynergyId::PerfectCult => {
                cultists.counts.iter().all(|&c| c >= 1)
            }
            // 深渊建筑师: 每种维度的节点各拥有 ≥1
            SynergyId::AbyssArchitect => {
                nodes.dimension_total(1) >= 1
                    && nodes.dimension_total(2) >= 1
                    && nodes.dimension_total(3) >= 1
            }
        }
    }
}

impl Default for SynergyState {
    fn default() -> Self {
        Self::new()
    }
}
