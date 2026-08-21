use serde::{Deserialize, Serialize};

/// 货币状态：碎语（主要货币）和禁忌真理（高级货币）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyState {
    /// 碎语 - 主要货币，由 token 消耗产生
    pub whispers: f64,
    /// 禁忌真理 - 高级货币，通过转生获得
    pub forbidden_truths: u32,
}

impl CurrencyState {
    pub fn new() -> Self {
        Self {
            whispers: 0.0,
            forbidden_truths: 0,
        }
    }
}

impl Default for CurrencyState {
    fn default() -> Self {
        Self::new()
    }
}
