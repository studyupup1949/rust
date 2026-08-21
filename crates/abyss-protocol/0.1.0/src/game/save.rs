use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::GameState;

/// 完整存档数据
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub game_state: GameState,
}

/// 存档管理器
pub struct SaveManager {
    save_dir: PathBuf,
    last_save_time: Instant,
    auto_save_interval: Duration,
}

impl SaveManager {
    /// 创建存档管理器，自动确定存档目录
    pub fn new() -> Result<Self> {
        let save_dir = Self::resolve_save_dir()?;
        Ok(Self {
            save_dir,
            last_save_time: Instant::now(),
            auto_save_interval: Duration::from_secs(60),
        })
    }

    /// 保存游戏状态（原子写入）
    pub fn save(&mut self, state: &GameState) -> Result<()> {
        // 确保目录存在
        fs::create_dir_all(&self.save_dir)?;

        let data = SaveData {
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: Utc::now(),
            game_state: state.clone(),
        };

        let json = serde_json::to_string_pretty(&data)?;

        let save_path = self.save_dir.join("save.json");
        let tmp_path = self.save_dir.join("save.json.tmp");

        // 原子写入：先写 tmp，再 rename
        fs::write(&tmp_path, &json)?;
        fs::rename(&tmp_path, &save_path)?;

        self.mark_saved();
        Ok(())
    }

    /// 加载存档，返回 None 表示无存档
    pub fn load(&self) -> Result<Option<GameState>> {
        let save_path = self.save_dir.join("save.json");

        if !save_path.exists() {
            return Ok(None);
        }

        let content = match fs::read_to_string(&save_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[WARN] Failed to read save file: {}", e);
                return Ok(None);
            }
        };

        let data: SaveData = match serde_json::from_str(&content) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("[WARN] Save file corrupted, starting new game: {}", e);
                return Ok(None);
            }
        };

        // 版本兼容性检查
        let current_version = env!("CARGO_PKG_VERSION");
        if data.version != current_version {
            eprintln!(
                "[WARN] Save version mismatch (save: {}, current: {}), starting new game",
                data.version, current_version
            );
            return Ok(None);
        }

        Ok(Some(data.game_state))
    }

    /// 检查是否需要自动存档
    pub fn should_auto_save(&self) -> bool {
        self.last_save_time.elapsed() >= self.auto_save_interval
    }

    /// 标记已存档（重置计时器）
    pub fn mark_saved(&mut self) {
        self.last_save_time = Instant::now();
    }

    /// 解析存档目录路径
    fn resolve_save_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
        Ok(config_dir.join("abyss-protocol"))
    }
}
