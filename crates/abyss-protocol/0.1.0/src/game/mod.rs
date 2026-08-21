pub mod currency;
pub mod san;
pub mod cultist;
pub mod nodes;
pub mod shop;
pub mod achievements;
pub mod synergy;
pub mod save;
pub mod stats;
pub mod production;
pub mod rebirth;
pub mod mutation;
pub mod engine;

pub use currency::CurrencyState;
pub use san::{SanLevel, SanState};
pub use cultist::{CultistState, CultistTier, UnlockCondition};
pub use nodes::{NodeId, NodeState};
pub use shop::{ShopState, ShopUpgradeId};
pub use achievements::{AchievementId, AchievementState};
pub use synergy::{SynergyId, SynergyState};
pub use stats::GameStats;
pub use production::ProductionResult;
pub use rebirth::{RebirthSummary, RebirthError};
pub use mutation::MutationResult;
pub use engine::{TickResult, BatchMode};
pub use cultist::{RecruitError, FuseError};
pub use nodes::PurchaseError;
pub use san::RepairError;
pub use shop::ShopError;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::monitor::TokenEvent;

/// 游戏总状态，聚合所有子系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub currency: CurrencyState,
    pub san: SanState,
    pub cultists: CultistState,
    pub nodes: NodeState,
    pub shop: ShopState,
    pub achievements: AchievementState,
    pub synergies: SynergyState,
    pub stats: GameStats,
    pub current_cycle_start: DateTime<Utc>,
}

impl GameState {
    /// 创建新游戏状态
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            currency: CurrencyState::new(),
            san: SanState::new(),
            cultists: CultistState::new(),
            nodes: NodeState::new(),
            shop: ShopState::new(),
            achievements: AchievementState::new(),
            synergies: SynergyState::new(),
            stats: GameStats::new(),
            current_cycle_start: now,
        }
    }

    /// 处理 token 流入事件：基础 1:1 转化为 whispers
    pub fn process_token_event(&mut self, event: &TokenEvent) {
        let total = event.total_tokens() as f64;
        self.currency.whispers += total;
        self.stats.total_tokens_consumed += event.total_tokens();
        self.stats.total_whispers_earned += total;
    }

    /// 执行一次游戏 tick，调用 engine::tick() 完成完整逻辑
    pub fn tick<R: rand::Rng>(&mut self, dt: f64, rng: &mut R) -> TickResult {
        engine::tick(self, dt, rng)
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
