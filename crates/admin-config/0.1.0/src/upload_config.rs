use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    /// 允许的文件类型 (MIME types, 逗号分隔)
    pub allowed_types: String,
    /// 最大文件大小 (MB)
    pub max_file_size: u64,
    /// 上传目录
    pub upload_dir: String,
    /// 临时目录
    pub temp_dir: String,
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            allowed_types: "image/jpeg,image/png,image/gif,image/webp,application/pdf".to_string(),
            max_file_size: 20,
            upload_dir: "./uploads".to_string(),
            temp_dir: "./temp".to_string(),
        }
    }
}

impl UploadConfig {
    pub fn allowed_types_list(&self) -> Vec<String> {
        self.allowed_types.split(',').map(|s| s.trim().to_string()).collect()
    }

    pub fn max_file_size_bytes(&self) -> u64 {
        self.max_file_size * 1024 * 1024
    }
}
