use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 游戏统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStats {
    /// 总转生次数
    pub total_rebirths: u32,
    /// 总 token 消耗量
    pub total_tokens_consumed: u64,
    /// 总碎语收入
    pub total_whispers_earned: f64,
    /// 当前轮开始时间
    pub current_cycle_start: DateTime<Utc>,
    /// 峰值产出速率
    pub peak_production_rate: f64,
    /// 总游戏时间（秒）
    pub total_playtime_seconds: u64,
    /// 本轮变异事件次数
    pub mutations_this_cycle: u32,
    /// 总合成次数
    pub total_fusions: u32,
    /// 本轮碎语收入
    pub whispers_this_cycle: f64,
    /// 本轮建造节点数
    pub nodes_this_cycle: u32,
    /// 已解锁的最高维度
    pub max_dimension_unlocked: u32,
    /// 上次存档时间（用于离线收益计算）
    pub last_save_time: DateTime<Utc>,
}

impl GameStats {
    pub fn new() -> Self {
        Self {
            total_rebirths: 0,
            total_tokens_consumed: 0,
            total_whispers_earned: 0.0,
            current_cycle_start: Utc::now(),
            peak_production_rate: 0.0,
            total_playtime_seconds: 0,
            mutations_this_cycle: 0,
            total_fusions: 0,
            whispers_this_cycle: 0.0,
            nodes_this_cycle: 0,
            max_dimension_unlocked: 1,
            last_save_time: Utc::now(),
        }
    }
}

impl Default for GameStats {
    fn default() -> Self {
        Self::new()
    }
}
