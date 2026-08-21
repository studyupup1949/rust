use crate::device::ADB;
use crate::error::{ADBResult};
use log::debug;

impl ADB {
    /// 从设备截图
    pub fn take_screenshot(
        &self,
        device_id: &str,
        output_path: &str,
    ) -> ADBResult<()> {
        // 在设备上截图并保存到临时文件
        let device_path = "/sdcard/screenshot.png";
        self.shell(device_id, &format!("screencap -p {}", device_path))?;

        // 下载截图到本地
        self.pull(device_id, device_path, output_path, None)?;

        // 清理设备上的临时文件
        self.shell(device_id, &format!("rm {}", device_path))?;

        Ok(())
    }

    /// 录制设备屏幕
    ///
    /// # 参数
    /// * `device_id` - 设备 ID
    /// * `output_path` - 本地输出路径
    /// * `duration_secs` - 录制时长（秒），最大 180 秒
    /// * `size` - 可选的分辨率，格式 "widthxheight"
    pub fn record_screen(
        &self,
        device_id: &str,
        output_path: &str,
        duration_secs: u32,
        size: Option<&str>,
    ) -> ADBResult<()> {
        // 设备上的临时文件路径
        let device_path = "/sdcard/screen_record.mp4";

        // 构建命令
        let mut command = format!("screenrecord --time-limit {} ", duration_secs.min(180)); // 最大 180 秒

        // 添加可选的分辨率参数
        if let Some(resolution) = size {
            command.push_str(&format!("--size {} ", resolution));
        }

        // 添加输出路径
        command.push_str(device_path);

        // 执行录制命令（这将阻塞直到录制完成）
        self.shell(device_id, &command)?;

        // 下载录制文件到本地
        self.pull(device_id, device_path, output_path, None)?;

        // 清理设备上的临时文件
        self.shell(device_id, &format!("rm {}", device_path))?;

        debug!("屏幕录制已保存到 {}", output_path);
        Ok(())
    }

    /// 从设备捕获日志
    pub fn capture_logs(
        &self,
        device_id: &str,
        tag: Option<&str>,
        priority: &str,
    ) -> ADBResult<String> {
        let tag_filter = tag.map_or(String::new(), |t| format!(" {}", t));
        self.shell(
            device_id,
            &format!("logcat -d{} *:{}", tag_filter, priority),
        )
    }

    /// 实时查看日志（返回立即执行的命令）
    pub fn watch_logs(
        &self,
        device_id: &str,
        tag: Option<&str>,
        priority: &str,
    ) -> ADBResult<()> {
        let tag_filter = tag.map_or(String::new(), |t| format!(" {}", t));
        let command = format!("logcat{} *:{}", tag_filter, priority);

        // 启动不等待的 shell 命令
        self.shell_no_wait(device_id, &command)
    }

    /// 清除日志
    pub fn clear_logs(&self, device_id: &str) -> ADBResult<()> {
        self.shell(device_id, "logcat -c")?;
        Ok(())
    }
}