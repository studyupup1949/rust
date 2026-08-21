use crate::device::ADB;
use crate::error::{ADBError, ADBResult};
use log::{debug, info, trace, warn};
use std::collections::HashMap;
use std::process::Command;
use std::str;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

// 缓存 Android 版本号
static ANDROID_VERSION_CACHE: Lazy<Mutex<HashMap<String, f32>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

// 缓存 PID 信息
static PID_CACHE: Lazy<Mutex<HashMap<String, (i32, Instant)>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

// 缓存超时时间（3秒）
const PID_CACHE_TIMEOUT: Duration = Duration::from_secs(3);

impl ADB {
    /// 使用指数退避策略重试操作
    pub fn with_retry<F, T>(&self, f: F) -> ADBResult<T>
    where
        F: Fn() -> ADBResult<T>,
    {
        crate::utils::retry_with_backoff(self.config.max_retries, self.config.retry_delay, f)
    }

    /// 带超时的操作执行
    pub fn with_timeout<F, T>(&self, f: F) -> ADBResult<T>
    where
        F: FnOnce() -> ADBResult<T> + Send + 'static,
        T: Send + 'static,
    {
        crate::utils::with_timeout(self.config.timeout, f)
    }

    /// 检查 ADB 是否可用并获取版本
    pub fn check_adb(&self) -> ADBResult<String> {
        self.with_retry(|| {
            let output = Command::new(&self.config.path)
                .arg("version")
                .output()
                .map_err(|e| ADBError::CommandError(format!("无法执行 ADB: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB 命令失败: {}",
                    stderr
                )));
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let version_line = stdout
                .lines()
                .next()
                .ok_or_else(|| ADBError::CommandError("无法解析 ADB 版本".to_string()))?;

            debug!("ADB 版本检查成功: {}", version_line);
            Ok(version_line.to_string())
        })
    }

    /// 列出可用设备
    pub fn list_devices(&self) -> ADBResult<Vec<crate::device::ADBDevice>> {
        self.with_retry(|| {
            let output = Command::new(&self.config.path)
                .arg("devices")
                .arg("-l") // 长格式以获取更多详细信息
                .output()
                .map_err(|e| ADBError::CommandError(format!("无法执行 ADB: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB devices 命令失败: {}",
                    stderr
                )));
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut devices = Vec::new();

            trace!("ADB devices 输出: {}", stdout);

            // 跳过第一行(标题)
            for line in stdout.lines().skip(1) {
                if line.trim().is_empty() {
                    continue;
                }

                // 解析设备行
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let id = parts[0].to_string();
                    let status_str = parts[1];
                    let status = crate::device::DeviceStatus::from(status_str);

                    // 创建基础设备
                    let mut device = crate::device::ADBDevice::new(&id, status);

                    // 提取设备名称和其他属性
                    if parts.len() > 2 {
                        // 提取设备型号
                        if let Some(model_part) = parts.iter().find(|p| p.starts_with("model:")) {
                            let model = model_part.trim_start_matches("model:");
                            device = device.with_model(model);

                            // 使用型号作为设备名称
                            device = device.with_name(model);
                        }

                        // 提取产品信息
                        if let Some(product_part) = parts.iter().find(|p| p.starts_with("product:")) {
                            let product = product_part.trim_start_matches("product:");
                            device = device.with_product(product);
                        }

                        // 提取传输 ID
                        if let Some(transport_part) = parts.iter().find(|p| p.starts_with("transport_id:")) {
                            let transport = transport_part.trim_start_matches("transport_id:");
                            device = device.with_transport_id(transport);
                        }
                    }

                    // 如果名称还是默认的设备 ID，尝试获取更好的名称
                    if device.name == format!("Device {}", id) && device.is_online() {
                        if let Ok(model) = self.shell(&id, "getprop ro.product.model") {
                            let model = model.trim();
                            if !model.is_empty() {
                                device = device.with_name(model);
                            }
                        }
                    }

                    devices.push(device);
                }
            }

            info!("发现 {} 个 ADB 设备", devices.len());
            Ok(devices)
        })
    }

    /// 连接到远程设备
    pub fn connect(&self, ip: &str, port: u16) -> ADBResult<()> {
        self.with_retry(|| {
            let output = Command::new(&self.config.path)
                .arg("connect")
                .arg(format!("{}:{}", ip, port))
                .output()
                .map_err(|e| {
                    ADBError::CommandError(format!("无法连接到远程设备: {}", e))
                })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if !output.status.success() || stdout.contains("failed") || stdout.contains("unable") {
                let error_msg = if !stderr.is_empty() {
                    format!("ADB 连接失败: {}", stderr)
                } else {
                    format!("ADB 连接失败: {}", stdout)
                };
                return Err(ADBError::CommandError(error_msg));
            }

            info!("成功连接到远程设备 {}:{}", ip, port);
            Ok(())
        })
    }

    /// 断开与远程设备的连接
    pub fn disconnect(&self, ip: &str, port: Option<u16>) -> ADBResult<()> {
        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);
            cmd.arg("disconnect");

            if let Some(p) = port {
                cmd.arg(format!("{}:{}", ip, p));
            } else {
                cmd.arg(ip);
            }

            let output = cmd.output().map_err(|e| {
                ADBError::CommandError(format!("无法断开与远程设备的连接: {}", e))
            })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB 断开连接失败: {}",
                    stderr
                )));
            }

            info!("成功断开与远程设备 {} 的连接", ip);
            Ok(())
        })
    }

    /// 断开所有远程连接
    pub fn disconnect_all(&self) -> ADBResult<()> {
        self.with_retry(|| {
            let output = Command::new(&self.config.path)
                .arg("disconnect")
                .output()
                .map_err(|e| {
                    ADBError::DeviceError(format!("无法断开所有设备连接: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::DeviceError(format!(
                    "ADB 断开所有连接失败: {}",
                    stderr
                )));
            }

            debug!("成功断开与所有远程设备的连接");
            Ok(())
        })
    }

    /// 在设备上执行 shell 命令
    pub fn shell(&self, device_id: &str, command: &str) -> ADBResult<String> {
        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);

            // 添加设备 ID
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            let output = cmd.arg("shell").arg(command).output().map_err(|e| {
                ADBError::DeviceError(format!("无法执行 ADB shell: {}", e))
            })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if !output.status.success() {
                return Err(ADBError::DeviceError(format!(
                    "ADB shell 命令失败: {}",
                    stderr
                )));
            }

            if !stderr.is_empty() {
                warn!("ADB shell 命令产生了 stderr 输出: {}", stderr);
            }

            trace!("Shell 命令 '{}' 输出: {}", command, stdout);
            Ok(stdout)
        })
    }

    /// 执行 shell 命令但不等待完成
    pub fn shell_no_wait(&self, device_id: &str, command: &str) -> ADBResult<()> {
        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);

            // 添加设备 ID
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            // 启动进程但不等待
            let child = cmd.arg("shell").arg(command).spawn().map_err(|e| {
                ADBError::DeviceError(format!("无法执行 ADB shell: {}", e))
            })?;

            debug!("在设备 {} 上启动命令: {}", device_id, command);

            // 如果启用了连接池，可以在这里存储子进程
            if let Ok(mut pool) = self.connections.lock() {
                pool.insert(format!("{}:{}", device_id, command), std::sync::Arc::new(std::sync::Mutex::new(child)));
            }

            Ok(())
        })
    }

    /// 通过 IP 地址查找设备
    pub fn find_device_by_ip(&self, ip: &str) -> ADBResult<Option<String>> {
        // 获取已连接设备列表
        let devices = self.list_devices()?;

        // 查找包含该 IP 的设备
        for device in devices {
            if device.id.contains(ip) {
                return Ok(Some(device.id));
            }
        }

        // 未找到则返回 None
        Ok(None)
    }

    /// 获取设备属性
    pub fn get_prop(&self, device_id: &str, prop_name: &str) -> ADBResult<String> {
        let command = format!("getprop {}", prop_name);
        let output = self.shell(device_id, &command)?;
        Ok(output.trim().to_string())
    }

    /// 设置设备属性
    pub fn set_prop(&self, device_id: &str, prop_name: &str, prop_value: &str) -> ADBResult<()> {
        let command = format!("setprop {} {}", prop_name, prop_value);
        self.shell(device_id, &command)?;
        Ok(())
    }

    /// 获取设备所有属性
    pub fn get_all_props(&self, device_id: &str) -> ADBResult<HashMap<String, String>> {
        let output = self.shell(device_id, "getprop")?;
        Ok(crate::utils::parse_properties(&output))
    }

    /// 检查设备是否在线
    pub fn is_device_online(&self, device_id: &str) -> ADBResult<bool> {
        let devices = self.list_devices()?;

        for device in devices {
            if device.id == device_id {
                return Ok(device.is_online());
            }
        }

        Ok(false)
    }

    /// 重启 ADB 服务器
    pub fn restart_server(&self) -> ADBResult<()> {
        self.with_retry(|| {
            // 首先停止服务器
            let output = Command::new(&self.config.path)
                .arg("kill-server")
                .output()
                .map_err(|e| ADBError::CommandError(format!("无法停止 ADB 服务器: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("ADB kill-server 可能失败: {}", stderr);
                // 我们继续尝试启动服务器，即使停止可能失败
            }

            // 短暂延迟，确保服务器停止
            std::thread::sleep(Duration::from_millis(500));

            // 启动服务器
            let output = Command::new(&self.config.path)
                .arg("start-server")
                .output()
                .map_err(|e| ADBError::CommandError(format!("无法启动 ADB 服务器: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB start-server 失败: {}",
                    stderr
                )));
            }

            info!("成功重启 ADB 服务器");

            // 短暂延迟，确保服务器启动完成
            std::thread::sleep(Duration::from_millis(1000));

            Ok(())
        })
    }

    /// 执行任意 ADB 命令
    pub fn run_command(&self, args: &[&str]) -> ADBResult<String> {
        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);

            // 添加全局附加参数（如果有）
            if let Some(additional_args) = &self.config.additional_args {
                for arg in additional_args {
                    cmd.arg(arg);
                }
            }

            // 添加命令特定参数
            for arg in args {
                cmd.arg(arg);
            }

            let output = cmd.output().map_err(|e| {
                ADBError::CommandError(format!("无法执行 ADB 命令: {}", e))
            })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if !output.status.success() {
                let error_msg = if !stderr.is_empty() {
                    stderr
                } else {
                    stdout.clone()
                };

                return Err(ADBError::CommandError(format!(
                    "ADB 命令失败: {}",
                    error_msg
                )));
            }

            debug!("ADB 命令执行成功: {:?}", args);
            Ok(stdout)
        })
    }

    /// 等待设备连接
    pub fn wait_for_device(&self, device_id: &str, timeout_ms: Option<u64>) -> ADBResult<bool> {
        let timeout = timeout_ms.unwrap_or(30000); // 默认30秒
        let poll_interval = 500; // 500毫秒

        info!("等待设备 {} 连接...", device_id);

        let result = crate::utils::wait_with_polling(
            timeout,
            poll_interval,
            || self.is_device_online(device_id),
            Some(|elapsed| {
                if elapsed % 5000 == 0 { // 每5秒打印一次日志
                    debug!("等待设备 {} 连接，已等待 {}s...", device_id, elapsed / 1000);
                }
            })
        )?;

        if result {
            info!("设备 {} 已连接", device_id);
        } else {
            warn!("等待设备 {} 连接超时", device_id);
        }

        Ok(result)
    }

    /// 获取 ADB 服务器版本
    pub fn get_server_version(&self) -> ADBResult<u32> {
        let output = self.run_command(&["version"])?;

        // 尝试从输出中提取版本号
        let re = regex::Regex::new(r"Android Debug Bridge version (\d+)\.(\d+)\.(\d+)").unwrap();
        if let Some(caps) = re.captures(&output) {
            let major: u32 = caps[1].parse().unwrap_or(0);
            let minor: u32 = caps[2].parse().unwrap_or(0);
            let patch: u32 = caps[3].parse().unwrap_or(0);

            // 转换为一个整数表示
            let version = major * 10000 + minor * 100 + patch;
            return Ok(version);
        }

        Err(ADBError::CommandError("无法解析 ADB 服务器版本".to_string()))
    }

    /// 优化版的进程 ID 获取
    pub fn get_pid_optimized(&self, device_id: &str, package_name: &str) -> ADBResult<Option<i32>> {
        let cache_key = format!("{}:{}", device_id, package_name);

        // 检查缓存
        if let Ok(cache) = PID_CACHE.lock() {
            if let Some((pid, timestamp)) = cache.get(&cache_key) {
                if Instant::now().duration_since(*timestamp) < PID_CACHE_TIMEOUT {
                    trace!("使用缓存的 PID: {} -> {}", package_name, pid);
                    return Ok(Some(*pid));
                }
            }
        }

        // 在 Android 8+ 系统上，首选 pidof 命令
        let android_version = self.get_android_version(device_id)?;

        if android_version >= 8.0 {
            // 使用 pidof（Android 8+ 的首选方法）
            let command = format!("pidof {}", package_name);
            let output = self.shell(device_id, &command)?;

            if !output.trim().is_empty() {
                if let Ok(pid) = output.trim().parse::<i32>() {
                    // 更新缓存
                    if let Ok(mut cache) = PID_CACHE.lock() {
                        cache.insert(cache_key, (pid, Instant::now()));
                    }
                    return Ok(Some(pid));
                }
            }
        }

        // 尝试使用 ps 命令（更通用的方法）
        let ps_command = if android_version >= 7.0 {
            // Android 7+ 系统使用不同的 ps 格式
            format!("ps -A | grep {} | grep -v grep", package_name)
        } else {
            // 较旧的 Android 版本使用传统 ps 格式
            format!("ps | grep {} | grep -v grep", package_name)
        };

        let output = self.shell(device_id, &ps_command)?;

        if !output.trim().is_empty() {
            let lines = output.lines().collect::<Vec<&str>>();

            for line in lines {
                let parts: Vec<&str> = line.split_whitespace().collect();

                // 确定 PID 位置（根据 Android 版本有所不同）
                let pid_index = if android_version >= 7.0 { 1 } else { 2 };

                if parts.len() > pid_index {
                    if let Ok(pid) = std::str::FromStr::from_str(parts[pid_index]) {
                        // 更新缓存
                        if let Ok(mut cache) = PID_CACHE.lock() {
                            cache.insert(cache_key, (pid, Instant::now()));
                        }
                        return Ok(Some(pid));
                    }
                }
            }
        }

        // 最后的尝试 - 使用 dumpsys
        let dumpsys_command = format!("dumpsys activity services | grep -i {}", package_name);
        let output = self.shell(device_id, &dumpsys_command)?;

        if !output.trim().is_empty() {
            let re = regex::Regex::new(r"pid=(\d+)")?;

            if let Some(caps) = re.captures(&output) {
                if let Some(pid_match) = caps.get(1) {
                    if let Ok(pid) = std::str::FromStr::from_str(pid_match.as_str()) {
                        // 更新缓存
                        if let Ok(mut cache) = PID_CACHE.lock() {
                            cache.insert(cache_key, (pid, Instant::now()));
                        }
                        return Ok(Some(pid));
                    }
                }
            }
        }

        debug!("无法找到包 {} 的 PID", package_name);
        Ok(None)
    }

    /// 检查包是否运行的优化版本
    pub fn is_package_running_optimized(
        &self,
        device_id: &str,
        package_name: &str,
    ) -> ADBResult<(bool, Option<i32>)> {
        // 直接使用优化版的 PID 获取方法
        if let Ok(Some(pid)) = self.get_pid_optimized(device_id, package_name) {
            return Ok((true, Some(pid)));
        }

        // 检查前台应用
        let current_app_cmd = "dumpsys window windows | grep -E 'mCurrentFocus|mFocusedApp'";
        let current_app = self.shell(device_id, current_app_cmd)?;

        if current_app.contains(package_name) {
            debug!("通过前台应用检查确认 {} 正在运行", package_name);
            return Ok((true, None));
        }

        // 最后一次尝试 - 检查服务
        let service_cmd = format!("dumpsys activity services | grep -i {}", package_name);
        let service_output = self.shell(device_id, &service_cmd)?;

        if !service_output.trim().is_empty() {
            debug!("通过服务检查确认 {} 正在运行", package_name);
            return Ok((true, None));
        }

        Ok((false, None))
    }

    /// 获取设备的 Android 版本
    fn get_android_version(&self, device_id: &str) -> ADBResult<f32> {
        // 先检查缓存
        if let Ok(cache) = ANDROID_VERSION_CACHE.lock() {
            if let Some(version) = cache.get(device_id) {
                return Ok(*version);
            }
        }

        // 如果缓存中没有，则查询设备
        let output = self.shell(device_id, "getprop ro.build.version.release")?;
        let version_str = output.trim();

        // 解析版本号
        let version = match version_str.split('.').next() {
            Some(major) => {
                if let Ok(major_num) = major.parse::<f32>() {
                    major_num
                } else {
                    // 默认返回一个保守的版本号
                    warn!("无法解析 Android 版本 '{}', 默认使用 5.0", version_str);
                    5.0
                }
            },
            None => {
                warn!("无法获取 Android 版本, 默认使用 5.0");
                5.0
            }
        };

        // 更新缓存
        if let Ok(mut cache) = ANDROID_VERSION_CACHE.lock() {
            cache.insert(device_id.to_string(), version);
        }

        Ok(version)
    }
}