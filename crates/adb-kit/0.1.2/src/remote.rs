use crate::device::ADB;
use crate::error::{ADBError, ADBResult};
use log::debug;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

impl ADB {
    /// 启用设备远程调试
    pub fn enable_remote_debugging(
        &self,
        device_id: &str,
        port: u16,
    ) -> ADBResult<String> {
        // 获取设备 IP 地址
        let ip_cmd = "ip addr show wlan0 | grep 'inet ' | cut -d' ' -f6 | cut -d/ -f1";
        let ip = self.shell(device_id, ip_cmd)?;
        let ip = ip.trim();

        if ip.is_empty() {
            return Err(ADBError::DeviceError(
                "无法获取设备 IP 地址".to_string(),
            ));
        }

        // 设置设备监听端口
        let cmd = format!("setprop service.adb.tcp.port {}", port);
        self.shell(device_id, &cmd)?;

        // 重启 ADB 服务
        self.shell(device_id, "stop adbd && start adbd")?;

        // 返回连接地址
        let conn_addr = format!("{}:{}", ip, port);
        debug!("远程调试已启用: {}", conn_addr);

        Ok(conn_addr)
    }

    /// 获取设备架构
    pub fn get_device_architecture(&self, device_id: &str) -> ADBResult<String> {
        let output = self.shell(device_id, "getprop ro.product.cpu.abi")?;
        let arch = output.trim();

        // 将 Android 架构名称映射到 Frida 服务器架构名称
        let frida_arch = match arch {
            "armeabi-v7a" | "armeabi" => "arm",
            "arm64-v8a" => "arm64",
            "x86" => "x86",
            "x86_64" => "x86_64",
            _ => {
                return Err(ADBError::DeviceError(format!(
                    "不支持的架构: {}",
                    arch
                )))
            }
        };

        debug!("设备架构: {}", frida_arch);
        Ok(frida_arch.to_string())
    }

    /// 在设备上启动 Frida 服务器
    pub fn start_frida_server(
        &self,
        device_id: &str,
        frida_server_path: &str,
        port: u16,
        server_name: Option<&str>,
        use_root: Option<bool>,
    ) -> ADBResult<()> {
        // 确定服务器名称
        let server_name = server_name.unwrap_or("frida-server");
        let use_root = use_root.unwrap_or(true);

        // 检查 Frida 服务器是否已在运行
        let ps_output = self
            .shell(device_id, &format!("ps -A | grep {}", server_name))?;
        if !ps_output.trim().is_empty() {
            debug!("{} 已经在设备 {} 上运行", server_name, device_id);

            // 检查是否在正确的端口上运行
            let netstat_output = self
                .shell(
                    device_id,
                    &format!("netstat -ano | grep LISTEN | grep :{}", port),
                )?;
            if !netstat_output.trim().is_empty() {
                debug!("{} 已在端口 {} 上监听", server_name, port);
                return Ok(());
            } else {
                // 如果在运行但端口不对，停止现有服务器
                debug!(
                    "停止现有 {} 以在正确的端口上重启",
                    server_name
                );
                self.stop_frida_server(device_id, Some(server_name))?;
            }
        }

        // 确定设备上的 Frida 服务器路径
        let device_frida_path = if PathBuf::from(frida_server_path).exists() {
            // 如果是本地路径，先确定设备架构
            let arch = self.get_device_architecture(device_id)?;

            // 根据架构选择正确的 frida-server 二进制文件
            let arch_specific_path = format!("{}-{}", frida_server_path, arch);
            let local_frida_path = if PathBuf::from(&arch_specific_path).exists() {
                arch_specific_path
            } else {
                frida_server_path.to_string()
            };

            // 推送到设备上并指定名称
            let device_path = format!("/data/local/tmp/{}", server_name);
            self.push(device_id, &local_frida_path, &device_path, None)?;

            // 设置可执行权限
            self.shell(device_id, &format!("chmod 755 {}", device_path))?;
            device_path
        } else {
            // 如果不是本地路径，假设它已经在设备上
            frida_server_path.to_string()
        };

        // 根据 use_root 参数确定启动命令
        let start_cmd = if use_root {
            format!("su -c '{} -l 0.0.0.0:{}'", device_frida_path, port)
        } else {
            format!("{} -l 0.0.0.0:{}", device_frida_path, port)
        };

        debug!("使用命令启动 {}: {}", server_name, start_cmd);
        self.shell_no_wait(device_id, &start_cmd)?;

        // 给服务器一些启动时间
        thread::sleep(Duration::from_secs(2));

        // 验证服务器是否在运行
        let verification_attempts = 3;
        for attempt in 1..=verification_attempts {
            let ps_output = self
                .shell(device_id, &format!("ps -A | grep {}", server_name))?;
            if !ps_output.trim().is_empty() {
                debug!(
                    "{} 成功启动 (尝试 {})",
                    server_name, attempt
                );
                return Ok(());
            }

            if attempt < verification_attempts {
                debug!(
                    "等待 {} 启动 (尝试 {}/{})",
                    server_name, attempt, verification_attempts
                );
                thread::sleep(Duration::from_secs(1));
            }
        }

        Err(ADBError::CommandError(format!(
            "无法启动 {}。请检查权限或服务器二进制文件。",
            server_name
        )))
    }

    /// 停止在设备上运行的 Frida 服务器
    pub fn stop_frida_server(
        &self,
        device_id: &str,
        server_name: Option<&str>,
    ) -> ADBResult<()> {
        let server_name = server_name.unwrap_or("frida-server");

        // 尝试不使用 root 权限停止
        let output = self
            .shell(device_id, &format!("pkill {}", server_name))?;

        // 如果失败，尝试使用 root 权限
        if output.contains("Operation not permitted") {
            self.shell(device_id, &format!("su -c 'pkill {}'", server_name))?;
        }

        // 验证服务器是否已停止
        thread::sleep(Duration::from_secs(1));
        let ps_output = self
            .shell(device_id, &format!("ps -A | grep {}", server_name))?;
        if !ps_output.trim().is_empty() {
            return Err(ADBError::CommandError(format!(
                "无法停止 {}",
                server_name
            )));
        }

        debug!("{} 成功停止", server_name);
        Ok(())
    }

    /// 重启设备到正常模式
    pub fn reboot(&self, device_id: &str) -> ADBResult<()> {
        self.with_retry(|| {
            let mut cmd = std::process::Command::new(&self.config.path);
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            let output = cmd.arg("reboot")
                .output()
                .map_err(|e| ADBError::CommandError(format!("无法执行重启命令: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!("重启命令失败: {}", stderr)));
            }

            debug!("已发送重启命令到设备 {}", device_id);
            Ok(())
        })
    }

    /// 重启设备到恢复模式
    pub fn reboot_recovery(&self, device_id: &str) -> ADBResult<()> {
        self.with_retry(|| {
            let mut cmd = std::process::Command::new(&self.config.path);
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            let output = cmd.arg("reboot")
                .arg("recovery")
                .output()
                .map_err(|e| ADBError::CommandError(format!("无法执行重启到恢复模式命令: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!("重启到恢复模式命令失败: {}", stderr)));
            }

            debug!("已发送重启到恢复模式命令到设备 {}", device_id);
            Ok(())
        })
    }

    /// 重启设备到引导加载程序模式
    pub fn reboot_bootloader(&self, device_id: &str) -> ADBResult<()> {
        self.with_retry(|| {
            let mut cmd = std::process::Command::new(&self.config.path);
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            let output = cmd.arg("reboot")
                .arg("bootloader")
                .output()
                .map_err(|e| ADBError::CommandError(format!("无法执行重启到引导加载程序模式命令: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!("重启到引导加载程序模式命令失败: {}", stderr)));
            }

            debug!("已发送重启到引导加载程序模式命令到设备 {}", device_id);
            Ok(())
        })
    }
}