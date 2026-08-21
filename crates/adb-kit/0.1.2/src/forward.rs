use crate::device::ADB;
use crate::error::{ADBError, ADBResult};
use log::debug;
use std::process::Command;

impl ADB {
    /// 将本地端口转发到设备端口
    pub fn forward(
        &self,
        device_id: &str,
        local_port: u16,
        device_port: u16,
    ) -> ADBResult<()> {
        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);

            // 如果指定了设备 ID 则添加
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            let output = cmd
                .arg("forward")
                .arg(format!("tcp:{}", local_port))
                .arg(format!("tcp:{}", device_port))
                .output()
                .map_err(|e| ADBError::CommandError(format!("无法执行 ADB forward: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB forward 命令失败: {}",
                    stderr
                )));
            }

            debug!(
                "端口转发已设置: localhost:{} -> device:{}",
                local_port, device_port
            );
            Ok(())
        })
    }

    /// 移除端口转发
    pub fn remove_forward(&self, local_port: u16) -> ADBResult<()> {
        self.with_retry(|| {
            let output = Command::new(&self.config.path)
                .arg("forward")
                .arg("--remove")
                .arg(format!("tcp:{}", local_port))
                .output()
                .map_err(|e| {
                    ADBError::CommandError(format!("无法执行 ADB remove-forward: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB remove-forward 命令失败: {}",
                    stderr
                )));
            }

            debug!("已移除端口转发 localhost:{}", local_port);
            Ok(())
        })
    }

    /// 移除所有端口转发
    pub fn remove_all_forwards(&self) -> ADBResult<()> {
        self.with_retry(|| {
            let output = Command::new(&self.config.path)
                .arg("forward")
                .arg("--remove-all")
                .output()
                .map_err(|e| {
                    ADBError::CommandError(format!("无法执行 ADB remove-all-forwards: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB remove-all-forwards 命令失败: {}",
                    stderr
                )));
            }

            debug!("已移除所有端口转发");
            Ok(())
        })
    }

    /// 列出所有端口转发
    pub fn list_forwards(&self) -> ADBResult<String> {
        self.with_retry(|| {
            let output = Command::new(&self.config.path)
                .arg("forward")
                .arg("--list")
                .output()
                .map_err(|e| {
                    ADBError::CommandError(format!("无法执行 ADB list-forwards: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB list-forwards 命令失败: {}",
                    stderr
                )));
            }

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(stdout)
        })
    }

    /// 反向端口转发（设备到主机）
    pub fn reverse(
        &self,
        device_id: &str,
        remote_port: u16,
        local_port: u16,
    ) -> ADBResult<()> {
        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);

            // 如果指定了设备 ID 则添加
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            let output = cmd
                .arg("reverse")
                .arg(format!("tcp:{}", remote_port))
                .arg(format!("tcp:{}", local_port))
                .output()
                .map_err(|e| ADBError::CommandError(format!("无法执行 ADB reverse: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB reverse 命令失败: {}",
                    stderr
                )));
            }

            debug!(
                "反向端口转发已设置: device:{} -> localhost:{}",
                remote_port, local_port
            );
            Ok(())
        })
    }

    /// 移除反向端口转发
    pub fn remove_reverse(&self, device_id: &str, remote_port: u16) -> ADBResult<()> {
        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);

            // 如果指定了设备 ID 则添加
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            let output = cmd
                .arg("reverse")
                .arg("--remove")
                .arg(format!("tcp:{}", remote_port))
                .output()
                .map_err(|e| {
                    ADBError::CommandError(format!("无法执行 ADB remove-reverse: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB remove-reverse 命令失败: {}",
                    stderr
                )));
            }

            debug!("已移除反向端口转发 device:{}", remote_port);
            Ok(())
        })
    }

    /// 移除所有反向端口转发
    pub fn remove_all_reverses(&self, device_id: &str) -> ADBResult<()> {
        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);

            // 如果指定了设备 ID 则添加
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            let output = cmd
                .arg("reverse")
                .arg("--remove-all")
                .output()
                .map_err(|e| {
                    ADBError::CommandError(format!("无法执行 ADB remove-all-reverses: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB remove-all-reverses 命令失败: {}",
                    stderr
                )));
            }

            debug!("已移除设备上所有反向端口转发");
            Ok(())
        })
    }
}