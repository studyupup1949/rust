use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCodeConfig {
    /// 验证码长度
    pub length: usize,
    /// 过期时间(秒)
    pub ttl: u64,
    /// 发送间隔(秒)
    pub send_interval: u64,
}

impl Default for VerificationCodeConfig {
    fn default() -> Self {
        Self {
            length: 6,
            ttl: 300,
            send_interval: 60,
        }
    }
}
