use crate::device::ADB;
use crate::error::{ADBError, ADBResult};
use log::{debug, warn, info};
use std::sync::{Arc};
use std::time::{Duration, Instant};

/// 资源管理器结构体
///
/// 负责跟踪和清理设备上的临时文件
pub struct ResourceManager {
    device_id: String,
    temp_files: Vec<String>,
    start_time: Instant,
    adb: Arc<ADB>,
}

impl ResourceManager {
    /// 创建新的资源管理器
    pub fn new(adb: Arc<ADB>, device_id: &str) -> Self {
        Self {
            device_id: device_id.to_string(),
            temp_files: Vec::new(),
            start_time: Instant::now(),
            adb,
        }
    }

    /// 添加临时文件到跟踪列表
    pub fn track_temp_file(&mut self, path: &str) {
        self.temp_files.push(path.to_string());
        debug!("添加临时文件到跟踪: {}", path);
    }

    /// 手动清理所有跟踪的临时文件
    pub fn cleanup(&mut self) -> ADBResult<()> {
        let mut errors = Vec::new();

        for file in &self.temp_files {
            match self.adb.shell(&self.device_id, &format!("rm -f {}", file)) {
                Ok(_) => debug!("已删除临时文件: {}", file),
                Err(e) => {
                    warn!("删除临时文件 {} 失败: {}", file, e);
                    errors.push(format!("文件 {}: {}", file, e));
                }
            }
        }

        // 清空跟踪列表
        self.temp_files.clear();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ADBError::FileError(format!(
                "清理临时文件时发生错误: {}",
                errors.join(", ")
            )))
        }
    }

    /// 获取操作持续时间
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

// 为 ResourceManager 实现 Drop 特性，在超出作用域时自动清理资源
impl Drop for ResourceManager {
    fn drop(&mut self) {
        if !self.temp_files.is_empty() {
            info!(
                "自动清理 {} 个设备上的临时文件 {}",
                self.device_id,
                self.temp_files.len()
            );

            // 尝试清理资源，但忽略错误（因为这是在 Drop 中）
            let _ = self.cleanup();
        }
    }
}

// 为 ADB 添加资源管理支持
impl ADB {
    /// 创建资源管理器
    pub fn create_resource_manager(&self, device_id: &str) -> ResourceManager {
        ResourceManager::new(Arc::new(self.clone()), device_id)
    }

    /// 使用资源管理器执行操作
    pub fn with_resources<F, T>(&self, device_id: &str, f: F) -> ADBResult<T>
    where
        F: FnOnce(&mut ResourceManager) -> ADBResult<T>,
    {
        let mut manager = self.create_resource_manager(device_id);
        let result = f(&mut manager);

        // 自动清理资源
        let _ = manager.cleanup();

        result
    }

    /// 优化的截图功能（使用资源管理器）
    pub fn take_screenshot_managed(
        &self,
        device_id: &str,
        output_path: &str,
    ) -> ADBResult<()> {
        self.with_resources(device_id, |resources| {
            // 创建设备上的临时文件路径
            let device_path = format!("/sdcard/screenshot_{}.png",
                                      chrono::Local::now().format("%Y%m%d_%H%M%S"));

            // 添加到资源跟踪
            resources.track_temp_file(&device_path);

            // 执行截图
            self.shell(device_id, &format!("screencap -p {}", device_path))?;

            // 下载到本地
            self.pull(device_id, &device_path, output_path, None)?;

            Ok(())
        })
    }

    /// 优化的屏幕录制功能（使用资源管理器）
    pub fn record_screen_managed(
        &self,
        device_id: &str,
        output_path: &str,
        duration_secs: u32,
        size: Option<&str>,
    ) -> ADBResult<()> {
        self.with_resources(device_id, |resources| {
            // 创建设备上的临时文件路径
            let device_path = format!("/sdcard/recording_{}.mp4",
                                      chrono::Local::now().format("%Y%m%d_%H%M%S"));

            // 添加到资源跟踪
            resources.track_temp_file(&device_path);

            // 构建录制命令
            let mut command = format!("screenrecord --time-limit {} ", duration_secs.min(180));

            if let Some(resolution) = size {
                command.push_str(&format!("--size {} ", resolution));
            }

            command.push_str(&device_path);

            // 执行录制（会阻塞直到录制完成）
            self.shell(device_id, &command)?;

            // 下载到本地
            self.pull(device_id, &device_path, output_path, None)?;

            Ok(())
        })
    }

    /// 使用临时文件执行操作
    pub fn with_temp_file<F, T>(&self, device_id: &str, prefix: &str, suffix: &str, f: F) -> ADBResult<T>
    where
        F: FnOnce(&str) -> ADBResult<T>,
    {
        // 生成唯一的临时文件名
        let temp_filename = format!(
            "/sdcard/{}_{}_{}{}",
            prefix,
            chrono::Local::now().format("%Y%m%d_%H%M%S"),
            rand::random::<u32>(),
            suffix
        );

        // 执行操作
        let result = f(&temp_filename);

        // 操作完成后删除临时文件
        let _ = self.shell(device_id, &format!("rm -f {}", temp_filename));

        result
    }
}